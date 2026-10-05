//! Proc-macro token generation for CDR structs and constant enums.

use alloc::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{collect::MessageRecord, schema::schema_text};

use super::struct_parts::{StructTokenParts, struct_token_parts};

pub(crate) fn generate_struct_tokens(
    record: &MessageRecord,
    records: &BTreeMap<String, MessageRecord>,
) -> Result<TokenStream, crate::error::CodegenError> {
    let parts = struct_token_parts(record);
    let struct_name = format_ident!("{}", record.name);
    let schema_name = record.schema_name.as_str();
    let schema_text = schema_text(
        &record.schema_name,
        &record.source,
        &record.dependencies,
        records,
    )?;
    let type_hash = record.type_hash.as_str();
    Ok(assemble_struct_tokens(
        &struct_name,
        schema_name,
        &schema_text,
        type_hash,
        parts,
    ))
}

fn assemble_struct_tokens(
    struct_name: &proc_macro2::Ident,
    schema_name: &str,
    schema_text: &str,
    type_hash: &str,
    parts: StructTokenParts,
) -> TokenStream {
    let StructTokenParts {
        enum_definitions,
        enum_impls,
        constants_mod,
        fields,
        encode_fields,
        decode_assignments,
        skip_placeholder,
        write_placeholder,
        field_count,
    } = parts;
    quote! {
        #(#enum_definitions)*
        #constants_mod
        #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
        pub struct #struct_name {
            #(#fields)*
        }

        #(#enum_impls)*

        impl CdrStruct for #struct_name {
            fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
                #skip_placeholder
                Ok(Self {
                    #decode_assignments
                })
            }

            fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
                #write_placeholder
                #(#encode_fields)*
                Ok(())
            }
        }

        impl Message for #struct_name {
            const SCHEMA: &'static str = #schema_text;
            const SCHEMA_NAME: &'static str = #schema_name;
            const TYPE_HASH: &'static str = #type_hash;
        }

        impl #struct_name {
            pub const KNOWN_FIELD_COUNT: usize = #field_count;
        }
    }
}
