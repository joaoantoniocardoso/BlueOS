//! Wire read/write facts for ROS scalar primitives (one table, no parallel matches).

#![expect(
    clippy::arbitrary_source_item_ordering,
    reason = "wire tables stay beside the struct types they populate"
)]

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::msg_ast::DataType;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub(crate) enum ScalarPrimitive {
    String = 0,
    Bool = 1,
    U8 = 2,
    U16 = 3,
    U32 = 4,
    U64 = 5,
    I8 = 6,
    I16 = 7,
    I32 = 8,
    I64 = 9,
    F32 = 10,
    F64 = 11,
}

struct ScalarWireSpec {
    rust_type: &'static str,
    read_method: &'static str,
    write_method: &'static str,
    string_write: bool,
}

const SCALAR_WIRE: [ScalarWireSpec; 12] = [
    ScalarWireSpec {
        rust_type: "String",
        read_method: "read_string",
        write_method: "write_string",
        string_write: true,
    },
    ScalarWireSpec {
        rust_type: "bool",
        read_method: "read_bool",
        write_method: "write_bool",
        string_write: false,
    },
    ScalarWireSpec {
        rust_type: "u8",
        read_method: "read_u8",
        write_method: "write_u8",
        string_write: false,
    },
    ScalarWireSpec {
        rust_type: "u16",
        read_method: "read_u16",
        write_method: "write_u16",
        string_write: false,
    },
    ScalarWireSpec {
        rust_type: "u32",
        read_method: "read_u32",
        write_method: "write_u32",
        string_write: false,
    },
    ScalarWireSpec {
        rust_type: "u64",
        read_method: "read_u64",
        write_method: "write_u64",
        string_write: false,
    },
    ScalarWireSpec {
        rust_type: "i8",
        read_method: "read_i8",
        write_method: "write_i8",
        string_write: false,
    },
    ScalarWireSpec {
        rust_type: "i16",
        read_method: "read_i16",
        write_method: "write_i16",
        string_write: false,
    },
    ScalarWireSpec {
        rust_type: "i32",
        read_method: "read_i32",
        write_method: "write_i32",
        string_write: false,
    },
    ScalarWireSpec {
        rust_type: "i64",
        read_method: "read_i64",
        write_method: "write_i64",
        string_write: false,
    },
    ScalarWireSpec {
        rust_type: "f32",
        read_method: "read_f32",
        write_method: "write_f32",
        string_write: false,
    },
    ScalarWireSpec {
        rust_type: "f64",
        read_method: "read_f64",
        write_method: "write_f64",
        string_write: false,
    },
];

struct DatatypeScalarEntry {
    primitive: ScalarPrimitive,
    matches: fn(&DataType) -> bool,
}

const DATATYPE_SCALAR: [DatatypeScalarEntry; 12] = [
    DatatypeScalarEntry {
        primitive: ScalarPrimitive::String,
        matches: |datatype| matches!(datatype, DataType::String),
    },
    DatatypeScalarEntry {
        primitive: ScalarPrimitive::Bool,
        matches: |datatype| matches!(datatype, DataType::Bool),
    },
    DatatypeScalarEntry {
        primitive: ScalarPrimitive::U8,
        matches: |datatype| matches!(datatype, DataType::U8),
    },
    DatatypeScalarEntry {
        primitive: ScalarPrimitive::U16,
        matches: |datatype| matches!(datatype, DataType::U16),
    },
    DatatypeScalarEntry {
        primitive: ScalarPrimitive::U32,
        matches: |datatype| matches!(datatype, DataType::U32),
    },
    DatatypeScalarEntry {
        primitive: ScalarPrimitive::U64,
        matches: |datatype| matches!(datatype, DataType::U64),
    },
    DatatypeScalarEntry {
        primitive: ScalarPrimitive::I8,
        matches: |datatype| matches!(datatype, DataType::I8),
    },
    DatatypeScalarEntry {
        primitive: ScalarPrimitive::I16,
        matches: |datatype| matches!(datatype, DataType::I16),
    },
    DatatypeScalarEntry {
        primitive: ScalarPrimitive::I32,
        matches: |datatype| matches!(datatype, DataType::I32),
    },
    DatatypeScalarEntry {
        primitive: ScalarPrimitive::I64,
        matches: |datatype| matches!(datatype, DataType::I64),
    },
    DatatypeScalarEntry {
        primitive: ScalarPrimitive::F32,
        matches: |datatype| matches!(datatype, DataType::F32),
    },
    DatatypeScalarEntry {
        primitive: ScalarPrimitive::F64,
        matches: |datatype| matches!(datatype, DataType::F64),
    },
];

impl ScalarPrimitive {
    fn wire_spec(self) -> &'static ScalarWireSpec {
        &SCALAR_WIRE[self as u8 as usize]
    }

    pub(crate) fn rust_type_token(self) -> TokenStream {
        let rust_type = self.wire_spec().rust_type;
        if rust_type == "String" {
            quote! { String }
        } else {
            let ident = format_ident!("{}", rust_type);
            quote! { #ident }
        }
    }

    pub(crate) fn read_call(self) -> TokenStream {
        let method = format_ident!("{}", self.wire_spec().read_method);
        quote! { reader.#method() }
    }

    pub(crate) fn write_call(self, value: TokenStream) -> TokenStream {
        let method = format_ident!("{}", self.wire_spec().write_method);
        if self.wire_spec().string_write {
            quote! { writer.#method(#value.as_str())?; }
        } else {
            quote! { writer.#method(#value)?; }
        }
    }
}

pub(crate) fn scalar_primitive(datatype: &DataType) -> Option<ScalarPrimitive> {
    if matches!(datatype, DataType::GlobalMessage { .. }) {
        return None;
    }
    DATATYPE_SCALAR
        .iter()
        .find(|entry| (entry.matches)(datatype))
        .map(|entry| entry.primitive)
}
