//! Internal AST for codegen, adapted from `roslibrust_codegen` parse output.

use std::collections::BTreeSet;

use roslibrust_codegen::{ArrayType, ConstantInfo, FieldInfo, RosLiteral};

/// The fields and constants of one `.msg` file, in source order.
#[derive(Clone, Debug)]
pub struct Message {
    fields: Vec<Field>,
    constants: Vec<Constant>,
}

/// One field of a [`Message`]: its name, whether it is a scalar, a sequence or an array, and its type.
#[derive(Clone, Debug)]
pub struct Field {
    name: String,
    case: FieldCase,
    datatype: DataType,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Constant {
    pub name: String,
    pub datatype: DataType,
    pub value: ConstantValue,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataType {
    String,
    Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    GlobalMessage { package: String, name: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldCase {
    Scalar,
    Vector,
    Array(usize),
    Const(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConstantValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    String(String),
}

impl Message {
    pub fn from_ros_fields(fields: &[FieldInfo], constants: &[ConstantInfo]) -> Self {
        let fields = fields.iter().map(field_from_ros).collect::<Vec<_>>();
        let constants = constants.iter().map(constant_from_ros).collect::<Vec<_>>();
        Self { fields, constants }
    }

    #[cfg(test)]
    pub(crate) fn from_parts(fields: Vec<Field>, constants: Vec<Constant>) -> Self {
        Self { fields, constants }
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub fn constants(&self) -> &[Constant] {
        &self.constants
    }

    pub fn dependencies(&self) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        for field in &self.fields {
            if let DataType::GlobalMessage { package, name } = &field.datatype {
                names.insert(format!("{package}/msg/{name}"));
            }
        }
        names
    }
}

impl Field {
    #[cfg(test)]
    pub(crate) fn scalar_uint8(name: &str) -> Self {
        Self {
            name: name.to_string(),
            case: FieldCase::Scalar,
            datatype: DataType::U8,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn case(&self) -> FieldCase {
        self.case.clone()
    }

    pub fn datatype(&self) -> DataType {
        self.datatype.clone()
    }
}

pub fn field_signature(message: &Message) -> String {
    let mut parts = Vec::new();
    for field in message.fields() {
        let field_name = if field.name() == "type" {
            "type_"
        } else {
            field.name()
        };
        parts.push(format!("{field_name}:{}", ros_field_type_name(field)));
    }
    parts.join(";")
}

fn ros_field_type_name(field: &Field) -> String {
    let base = match &field.datatype {
        DataType::String => "string".to_string(),
        DataType::Bool => "bool".to_string(),
        DataType::U8 => "uint8".to_string(),
        DataType::U16 => "uint16".to_string(),
        DataType::U32 => "uint32".to_string(),
        DataType::U64 => "uint64".to_string(),
        DataType::I8 => "int8".to_string(),
        DataType::I16 => "int16".to_string(),
        DataType::I32 => "int32".to_string(),
        DataType::I64 => "int64".to_string(),
        DataType::F32 => "float32".to_string(),
        DataType::F64 => "float64".to_string(),
        DataType::GlobalMessage { package, name } => format!("{package}/{name}"),
    };
    match field.case {
        FieldCase::Vector => format!("{base}[]"),
        FieldCase::Array(size) => format!("{base}[{size}]"),
        FieldCase::Scalar | FieldCase::Const(_) => base,
    }
}

fn field_from_ros(field: &FieldInfo) -> Field {
    let name = if field.field_name == "type" {
        "type_".to_string()
    } else {
        field.field_name.clone()
    };
    let case = match field.field_type.array_info {
        ArrayType::NotArray => FieldCase::Scalar,
        ArrayType::FixedLength(size) => FieldCase::Array(size),
        ArrayType::Unbounded | ArrayType::Bounded(_) => FieldCase::Vector,
    };
    Field {
        name,
        case,
        datatype: datatype_from_field(field),
    }
}

fn constant_from_ros(constant: &ConstantInfo) -> Constant {
    Constant {
        name: constant.constant_name.clone(),
        datatype: scalar_datatype_from_type_name(&constant.constant_type),
        value: literal_to_value(&constant.constant_type, &constant.constant_value),
    }
}

fn datatype_from_field(field: &FieldInfo) -> DataType {
    if field.field_type.is_primitive() {
        scalar_datatype_from_type_name(&field.field_type.field_type)
    } else {
        let package = field
            .field_type
            .package_name
            .clone()
            .unwrap_or_else(|| field.field_type.source_package.clone());
        DataType::GlobalMessage {
            package,
            name: field.field_type.field_type.clone(),
        }
    }
}

fn scalar_datatype_from_type_name(type_name: &str) -> DataType {
    match type_name {
        "string" => DataType::String,
        "bool" => DataType::Bool,
        "uint8" | "byte" => DataType::U8,
        "uint16" => DataType::U16,
        "uint32" => DataType::U32,
        "uint64" => DataType::U64,
        "int8" | "char" => DataType::I8,
        "int16" => DataType::I16,
        "int32" => DataType::I32,
        "int64" => DataType::I64,
        "float32" => DataType::F32,
        "float64" => DataType::F64,
        other => panic!("unsupported ROS type {other}"),
    }
}

fn literal_to_value(type_name: &str, literal: &RosLiteral) -> ConstantValue {
    let text = literal.inner.trim();
    match type_name {
        "string" => ConstantValue::String(text.to_string()),
        "bool" => ConstantValue::U8(if text == "true" { 1 } else { 0 }),
        "uint8" | "byte" => ConstantValue::U8(text.parse().expect("uint8 constant")),
        "uint16" => ConstantValue::U16(text.parse().expect("uint16 constant")),
        "uint32" => ConstantValue::U32(text.parse().expect("uint32 constant")),
        "uint64" => ConstantValue::U64(text.parse().expect("uint64 constant")),
        "int8" | "char" => ConstantValue::I8(text.parse().expect("int8 constant")),
        "int16" => ConstantValue::I16(text.parse().expect("int16 constant")),
        "int32" => ConstantValue::I32(text.parse().expect("int32 constant")),
        "int64" => ConstantValue::I64(text.parse().expect("int64 constant")),
        "float32" => ConstantValue::F32(text.parse().expect("float32 constant")),
        "float64" => ConstantValue::F64(text.parse().expect("float64 constant")),
        other => panic!("unsupported constant type {other}"),
    }
}
