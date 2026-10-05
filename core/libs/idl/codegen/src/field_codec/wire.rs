//! CDR encode/decode proc-macro tokens for message fields.

use alloc::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    constant_family::ConstantFamily,
    msg_ast::{ConstantValue, DataType, Field, FieldCase, Message},
    rust_tokens::{
        DecodeFieldTokens, enum_ident_for_field, read_primitive_tokens, read_result_tokens,
        scalar_rust_type_tokens, write_primitive_tokens,
    },
    schema::rust_field_name,
};

pub(crate) fn decode_field_tokens(
    message: &Message,
    families: &BTreeMap<String, ConstantFamily>,
    message_name: &str,
) -> DecodeFieldTokens {
    let mut assignments = Vec::new();
    for field in message.fields() {
        let field_name = format_ident!("{}", rust_field_name(field));
        let read = read_field_tokens(field, &field_name, families, message_name);
        assignments.push(quote! {
            #field_name: #read,
        });
    }
    DecodeFieldTokens {
        assignments: quote! { #(#assignments)* },
    }
}

pub(crate) fn read_field_tokens(
    field: &Field,
    _field_name: &proc_macro2::Ident,
    families: &BTreeMap<String, ConstantFamily>,
    message_name: &str,
) -> TokenStream {
    match field.case() {
        FieldCase::Vector if is_byte_sequence(field, families) => quote! {
            reader.read_or_default(|reader| {
                let length = reader.read_bounded_sequence_length()?;
                Ok(reader.read_bytes(length as usize)?.to_vec())
            })?
        },
        FieldCase::Vector => {
            let element = read_scalar_or_message_inner(field, families, message_name);
            quote! {
                reader.read_or_default(|reader| {
                    let length = reader.read_bounded_sequence_length()?;
                    let mut values = Vec::with_capacity(length as usize);
                    for _index in 0..length {
                        values.push(#element);
                    }
                    Ok(values)
                })?
            }
        }
        FieldCase::Array(size) => {
            let element = read_scalar_or_message_inner(field, families, message_name);
            quote! {
                reader.read_or_default(|reader| {
                    let mut values = [Default::default(); #size];
                    for value in &mut values {
                        *value = #element;
                    }
                    Ok(values)
                })?
            }
        }
        _ => read_scalar_or_message(field, families, message_name, false),
    }
}

pub(crate) fn read_scalar_or_message(
    field: &Field,
    families: &BTreeMap<String, ConstantFamily>,
    message_name: &str,
    _nested: bool,
) -> TokenStream {
    let read = if families.contains_key(field.name()) {
        let read = read_scalar_or_message_inner(field, families, message_name);
        quote! { Ok(#read) }
    } else {
        read_result_tokens(&field.datatype())
    };
    quote! { reader.read_or_default(|reader| #read)? }
}

pub(crate) fn read_scalar_or_message_inner(
    field: &Field,
    families: &BTreeMap<String, ConstantFamily>,
    message_name: &str,
) -> TokenStream {
    let read = read_primitive_tokens(&field.datatype());
    if let Some(family) = families.get(field.name()) {
        let enum_name = enum_ident_for_field(message_name, &family.field_name);
        return quote! { #enum_name::from_raw(#read) };
    }
    read
}

pub(crate) fn encode_field_tokens(
    message: &Message,
    families: &BTreeMap<String, ConstantFamily>,
    message_name: &str,
) -> Vec<TokenStream> {
    let mut tokens = Vec::new();
    for field in message.fields() {
        let field_name = format_ident!("{}", rust_field_name(field));
        tokens.push(write_field_tokens(
            field,
            quote! { self.#field_name },
            families,
            message_name,
        ));
    }
    tokens
}

pub(crate) fn write_field_tokens(
    field: &Field,
    value: TokenStream,
    families: &BTreeMap<String, ConstantFamily>,
    message_name: &str,
) -> TokenStream {
    match field.case() {
        FieldCase::Vector if is_byte_sequence(field, families) => quote! {
            writer.write_u32(#value.len() as u32)?;
            writer.write_bytes(&#value)?;
        },
        FieldCase::Vector => {
            let element_write = write_vector_element(field, families, message_name);
            quote! {
                writer.write_u32(#value.len() as u32)?;
                for element in #value.iter() {
                    #element_write
                }
            }
        }
        FieldCase::Array(_) => {
            let element_write = write_vector_element(field, families, message_name);
            quote! {
                for element in #value.iter() {
                    #element_write
                }
            }
        }
        FieldCase::Scalar | FieldCase::Const(_) => {
            write_scalar_or_message(field, value, families, message_name)
        }
    }
}

/// Whether `field` is a `uint8[]` of plain bytes, which the codec copies whole instead of byte by byte.
fn is_byte_sequence(field: &Field, families: &BTreeMap<String, ConstantFamily>) -> bool {
    matches!(field.datatype(), DataType::U8) && !families.contains_key(field.name())
}

pub(crate) fn write_vector_element(
    field: &Field,
    families: &BTreeMap<String, ConstantFamily>,
    message_name: &str,
) -> TokenStream {
    match field.datatype() {
        DataType::GlobalMessage { package, name } => {
            let package = format_ident!("{}", package);
            let name = format_ident!("{}", name);
            quote! {
                <crate::msg::#package::#name>::cdr_encode_fields(element, writer)?;
            }
        }
        DataType::String => quote! { writer.write_string(element.as_str())?; },
        _ => write_scalar_or_message(field, quote! { *element }, families, message_name),
    }
}

pub(crate) fn write_scalar_or_message(
    field: &Field,
    value: TokenStream,
    families: &BTreeMap<String, ConstantFamily>,
    _message_name: &str,
) -> TokenStream {
    if families.contains_key(field.name()) {
        return write_primitive_tokens(&field.datatype(), quote! { #value.as_raw() });
    }
    write_primitive_tokens(&field.datatype(), value)
}

pub(crate) fn rust_type_tokens(
    field: &Field,
    families: &BTreeMap<String, ConstantFamily>,
    message_name: &str,
) -> TokenStream {
    if let Some(family) = families.get(field.name()) {
        let enum_name = enum_ident_for_field(message_name, &family.field_name);
        return quote! { #enum_name };
    }
    let datatype = field.datatype();
    let base = scalar_rust_type_tokens(&datatype);
    match field.case() {
        FieldCase::Vector => quote! { Vec<#base> },
        FieldCase::Array(size) => quote! { [#base; #size] },
        FieldCase::Scalar | FieldCase::Const(_) => base,
    }
}

pub(crate) fn const_value_tokens(value: &ConstantValue, datatype: &DataType) -> TokenStream {
    typed_constant_literal(datatype, value)
}

fn typed_constant_literal(datatype: &DataType, value: &ConstantValue) -> TokenStream {
    let literal = crate::rust_tokens::constant_value_literal(value);
    match scalar_primitive_for_const(datatype) {
        Some(kind) => quote! { #kind = #literal },
        None if matches!(datatype, DataType::String) => quote! { &'static str = #literal },
        None => quote! { () },
    }
}

fn scalar_primitive_for_const(datatype: &DataType) -> Option<TokenStream> {
    match datatype {
        DataType::U8 => Some(quote! { u8 }),
        DataType::U16 => Some(quote! { u16 }),
        DataType::U32 => Some(quote! { u32 }),
        DataType::U64 => Some(quote! { u64 }),
        DataType::I8 => Some(quote! { i8 }),
        DataType::I16 => Some(quote! { i16 }),
        DataType::I32 => Some(quote! { i32 }),
        DataType::I64 => Some(quote! { i64 }),
        DataType::F32 => Some(quote! { f32 }),
        DataType::F64 => Some(quote! { f64 }),
        _ => None,
    }
}
