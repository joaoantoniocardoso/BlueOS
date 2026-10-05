//! Proc-macro token generation for CDR structs and constant enums.

mod scalar_wire;

use alloc::collections::BTreeMap;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::constant_family::{ConstantFamily, constant_families};
use crate::msg_ast::{Constant, ConstantValue, DataType, Message};

pub(crate) use scalar_wire::scalar_primitive;

pub(crate) struct DecodeFieldTokens {
    pub(crate) assignments: TokenStream,
}

pub(crate) fn constant_families_by_field(message: &Message) -> BTreeMap<String, ConstantFamily> {
    constant_families(message)
        .into_iter()
        .map(|family| (family.field_name.clone(), family))
        .collect()
}

pub(crate) fn enum_ident_for_field(message_name: &str, field_name: &str) -> proc_macro2::Ident {
    format_ident!("{}{}", message_name, field_name.to_case(Case::Pascal))
}

pub(crate) fn constant_raw_literal(constant: &Constant) -> TokenStream {
    constant_value_literal(&constant.value)
}

pub(crate) fn constant_value_literal(value: &ConstantValue) -> TokenStream {
    if let ConstantValue::String(value) = value {
        return quote! { #value };
    }
    numeric_constant_literal(value)
}

fn numeric_constant_literal(value: &ConstantValue) -> TokenStream {
    integer_constant_literal(value)
        .or_else(|| float_constant_literal(value))
        .unwrap_or_else(|| quote! { "" })
}

fn integer_constant_literal(value: &ConstantValue) -> Option<TokenStream> {
    match value {
        ConstantValue::U8(value) => Some(quote! { #value }),
        ConstantValue::U16(value) => Some(quote! { #value }),
        ConstantValue::U32(value) => Some(quote! { #value }),
        ConstantValue::U64(value) => Some(quote! { #value }),
        ConstantValue::I8(value) => Some(quote! { #value }),
        ConstantValue::I16(value) => Some(quote! { #value }),
        ConstantValue::I32(value) => Some(quote! { #value }),
        ConstantValue::I64(value) => Some(quote! { #value }),
        ConstantValue::F32(_) | ConstantValue::F64(_) | ConstantValue::String(_) => None,
    }
}

fn float_constant_literal(value: &ConstantValue) -> Option<TokenStream> {
    match value {
        ConstantValue::F32(value) => Some(quote! { #value }),
        ConstantValue::F64(value) => Some(quote! { #value }),
        _ => None,
    }
}

pub(crate) fn enum_variant_ident(
    constant: &Constant,
    family: &ConstantFamily,
) -> proc_macro2::Ident {
    let suffix = constant
        .name
        .strip_prefix(&family.constant_prefix)
        .unwrap_or(constant.name.as_str());
    let pascal = suffix.to_case(Case::Pascal);
    let variant_name = if pascal == "Unknown" {
        format!("{}{}", family.field_name.to_case(Case::Pascal), pascal)
    } else {
        pascal
    };
    format_ident!("{}", variant_name)
}

fn global_message_type_tokens(package: &str, name: &str) -> TokenStream {
    let package = format_ident!("{}", package);
    let name = format_ident!("{}", name);
    quote! { crate::msg::#package::#name }
}

pub(crate) fn scalar_rust_type_tokens(datatype: &DataType) -> TokenStream {
    if let Some(primitive) = scalar_primitive(datatype) {
        return primitive.rust_type_token();
    }
    let DataType::GlobalMessage { package, name } = datatype else {
        return quote! { () };
    };
    global_message_type_tokens(package, name)
}

pub(crate) fn generate_enum_definition_tokens(
    message_name: &str,
    family: &ConstantFamily,
) -> TokenStream {
    let enum_name = enum_ident_for_field(message_name, &family.field_name);
    let raw_type = scalar_rust_type_tokens(&family.constants[0].datatype);
    let mut sorted_constants = family.constants.clone();
    sorted_constants.sort_by_key(constant_discriminant);
    let mut variant_tokens = Vec::new();
    for (index, constant) in sorted_constants.iter().enumerate() {
        let variant = enum_variant_ident(constant, family);
        if index == 0 {
            variant_tokens.push(quote! { #[default] #variant, });
        } else {
            variant_tokens.push(quote! { #variant, });
        }
    }
    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        pub enum #enum_name {
            #(#variant_tokens)*
            Unknown(#raw_type),
        }
    }
}

pub(crate) fn generate_enum_impl_tokens(
    message_name: &str,
    family: &ConstantFamily,
) -> TokenStream {
    let enum_name = enum_ident_for_field(message_name, &family.field_name);
    let raw_type = scalar_rust_type_tokens(&family.constants[0].datatype);
    let mut from_arms = Vec::new();
    let mut as_arms = Vec::new();
    let mut sorted_constants = family.constants.clone();
    sorted_constants.sort_by_key(constant_discriminant);
    for constant in &sorted_constants {
        let variant = enum_variant_ident(constant, family);
        let raw = constant_raw_literal(constant);
        from_arms.push(quote! { #raw => Self::#variant, });
        as_arms.push(quote! { Self::#variant => #raw, });
    }
    from_arms.push(quote! { raw => Self::Unknown(raw), });
    as_arms.push(quote! { Self::Unknown(raw) => raw, });
    quote! {
        impl serde::Serialize for #enum_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                <#raw_type>::serialize(&self.as_raw(), serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for #enum_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Ok(Self::from_raw(<#raw_type>::deserialize(deserializer)?))
            }
        }

        impl #enum_name {
            pub fn from_raw(raw: #raw_type) -> Self {
                match raw {
                    #(#from_arms)*
                }
            }

            pub fn as_raw(self) -> #raw_type {
                match self {
                    #(#as_arms)*
                }
            }
        }
    }
}

pub(crate) fn constant_discriminant(constant: &Constant) -> u64 {
    match &constant.value {
        ConstantValue::U8(value) => *value as u64,
        ConstantValue::U16(value) => *value as u64,
        ConstantValue::U32(value) => *value as u64,
        ConstantValue::U64(value) => *value,
        ConstantValue::I8(value) => *value as u64,
        ConstantValue::I16(value) => *value as u64,
        ConstantValue::I32(value) => *value as u64,
        ConstantValue::I64(value) => *value as u64,
        ConstantValue::F32(value) => value.to_bits() as u64,
        ConstantValue::F64(value) => value.to_bits(),
        ConstantValue::String(_) => 0,
    }
}

pub(crate) fn read_result_tokens(datatype: &DataType) -> TokenStream {
    if let Some(primitive) = scalar_primitive(datatype) {
        return primitive.read_call();
    }
    global_message_decode_tokens(datatype)
}

pub(crate) fn read_primitive_tokens(datatype: &DataType) -> TokenStream {
    let read = read_result_tokens(datatype);
    quote! { #read? }
}

pub(crate) fn write_primitive_tokens(datatype: &DataType, value: TokenStream) -> TokenStream {
    if let Some(primitive) = scalar_primitive(datatype) {
        return primitive.write_call(value);
    }
    global_message_encode_tokens(datatype, value)
}

fn global_message_decode_tokens(datatype: &DataType) -> TokenStream {
    let DataType::GlobalMessage { package, name } = datatype else {
        return quote! { () };
    };
    let message = global_message_type_tokens(package, name);
    quote! { <#message>::cdr_decode_fields(reader) }
}

fn global_message_encode_tokens(datatype: &DataType, value: TokenStream) -> TokenStream {
    let DataType::GlobalMessage { package, name } = datatype else {
        return quote! { () };
    };
    let message = global_message_type_tokens(package, name);
    quote! { <#message>::cdr_encode_fields(&#value, writer)?; }
}
