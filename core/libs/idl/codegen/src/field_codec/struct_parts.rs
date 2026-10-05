//! Named steps for one message struct's proc-macro tokens.

use alloc::collections::BTreeMap;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    collect::MessageRecord,
    constant_family::freestanding_constants,
    msg_ast::{FieldCase, Message},
    rust_tokens::{
        constant_families_by_field, generate_enum_definition_tokens, generate_enum_impl_tokens,
    },
};

use super::wire::{const_value_tokens, decode_field_tokens, encode_field_tokens, rust_type_tokens};

pub(super) struct StructTokenParts {
    pub(super) enum_definitions: Vec<TokenStream>,
    pub(super) enum_impls: Vec<TokenStream>,
    pub(super) constants_mod: Option<TokenStream>,
    pub(super) fields: Vec<TokenStream>,
    pub(super) encode_fields: Vec<TokenStream>,
    pub(super) decode_assignments: TokenStream,
    pub(super) skip_placeholder: TokenStream,
    pub(super) write_placeholder: TokenStream,
    pub(super) field_count: usize,
}

pub(super) fn struct_token_parts(record: &MessageRecord) -> StructTokenParts {
    let message = &record.message;
    let families = constant_families_by_field(message);
    let enum_definitions = families
        .values()
        .map(|family| generate_enum_definition_tokens(&record.name, family))
        .collect();
    let enum_impls = families
        .values()
        .map(|family| generate_enum_impl_tokens(&record.name, family))
        .collect();
    let constants_mod = constants_mod_tokens(message, &record.name);
    let fields = struct_field_tokens(message, &families, &record.name);
    let encode_fields = encode_field_tokens(message, &families, &record.name);
    let decode_tokens = decode_field_tokens(message, &families, &record.name);
    let (skip_placeholder, write_placeholder) = empty_struct_placeholders(message);
    StructTokenParts {
        enum_definitions,
        enum_impls,
        constants_mod,
        fields,
        encode_fields,
        decode_assignments: decode_tokens.assignments,
        skip_placeholder,
        write_placeholder,
        field_count: message.fields().len(),
    }
}

fn constants_mod_tokens(message: &Message, record_name: &str) -> Option<TokenStream> {
    let mut constants = Vec::new();
    for constant in freestanding_constants(message) {
        let const_name = format_ident!("{}", constant.name);
        let const_tokens = const_value_tokens(&constant.value, &constant.datatype);
        constants.push(quote! {
            pub const #const_name: #const_tokens;
        });
    }
    if constants.is_empty() {
        return None;
    }
    let constants_name = format_ident!("constants_{}", record_name.to_case(Case::Snake));
    Some(quote! {
        pub mod #constants_name {
            #(#constants)*
        }
    })
}

fn struct_field_tokens(
    message: &Message,
    families: &BTreeMap<String, crate::constant_family::ConstantFamily>,
    record_name: &str,
) -> Vec<TokenStream> {
    message
        .fields()
        .iter()
        .map(|field| {
            let field_name = format_ident!("{}", crate::schema::rust_field_name(field));
            let field_type = rust_type_tokens(field, families, record_name);
            let serde_with = if matches!(field.case(), FieldCase::Array(_)) {
                quote! { #[serde(with = "serde_arrays")] }
            } else {
                quote! {}
            };
            quote! {
                #serde_with
                pub #field_name: #field_type,
            }
        })
        .collect()
}

fn empty_struct_placeholders(message: &Message) -> (TokenStream, TokenStream) {
    if message.fields().is_empty() {
        (
            quote! {
                if !reader.is_exhausted() {
                    reader.read_u8()?;
                }
            },
            quote! { writer.write_u8(0)?; },
        )
    } else {
        (quote! {}, quote! {})
    }
}
