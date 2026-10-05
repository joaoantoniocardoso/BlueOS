//! Constant-family detection for IDL enum emission (D-05).

use alloc::collections::{BTreeMap, BTreeSet};

use crate::msg_ast::{Constant, DataType, Field, FieldCase, Message};

/// Constants that share a prefix and type, bound to one message field (D-05).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ConstantFamily {
    pub(crate) field_name: String,
    pub(crate) constant_prefix: String,
    pub(crate) constants: Vec<Constant>,
}

#[repr(u8)]
enum DatatypeSortKey {
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
    GlobalMessage = 12,
}

impl From<&DataType> for DatatypeSortKey {
    fn from(datatype: &DataType) -> Self {
        match datatype {
            DataType::String => Self::String,
            DataType::Bool => Self::Bool,
            DataType::U8 => Self::U8,
            DataType::U16 => Self::U16,
            DataType::U32 => Self::U32,
            DataType::U64 => Self::U64,
            DataType::I8 => Self::I8,
            DataType::I16 => Self::I16,
            DataType::I32 => Self::I32,
            DataType::I64 => Self::I64,
            DataType::F32 => Self::F32,
            DataType::F64 => Self::F64,
            DataType::GlobalMessage { .. } => Self::GlobalMessage,
        }
    }
}

pub(crate) fn constant_families(message: &Message) -> Vec<ConstantFamily> {
    let mut families = field_prefix_families(message);
    let mut claimed_constants = claimed_constant_names(&families);
    families.extend(grouped_constant_families(message, &mut claimed_constants));
    families
}

pub(crate) fn freestanding_constants(message: &Message) -> Vec<Constant> {
    let families = constant_families(message);
    let claimed = families
        .iter()
        .flat_map(|family| {
            family
                .constants
                .iter()
                .map(|constant| constant.name.as_str())
        })
        .collect::<BTreeSet<_>>();
    message
        .constants()
        .iter()
        .filter(|constant| !claimed.contains(constant.name.as_str()))
        .cloned()
        .collect()
}

fn field_prefix_families(message: &Message) -> Vec<ConstantFamily> {
    let mut families = Vec::new();
    let mut claimed_constants = BTreeSet::new();
    for field in scalar_or_const_fields(message) {
        let field_prefix = format!("{}_", field.name().to_uppercase());
        let family_constants: Vec<Constant> = message
            .constants()
            .iter()
            .filter(|constant| {
                constant.name.starts_with(&field_prefix)
                    && field.datatype() == constant.datatype
                    && !claimed_constants.contains(constant.name.as_str())
            })
            .cloned()
            .collect();
        if family_constants.len() < 2 {
            continue;
        }
        let family = ConstantFamily {
            field_name: field.name().to_string(),
            constant_prefix: field_prefix,
            constants: sorted_constants(family_constants),
        };
        claimed_constants.extend(constant_names(&family.constants));
        families.push(family);
    }
    families
}

fn grouped_constant_families(
    message: &Message,
    claimed_constants: &mut BTreeSet<String>,
) -> Vec<ConstantFamily> {
    let constants = message.constants();
    let mut groups: BTreeMap<(u8, String), Vec<usize>> = BTreeMap::new();
    for (index, constant) in constants.iter().enumerate() {
        if claimed_constants.contains(constant.name.as_str()) {
            continue;
        }
        let Some(prefix) = constant_name_prefix(&constant.name) else {
            continue;
        };
        groups
            .entry((datatype_key(&constant.datatype), prefix))
            .or_default()
            .push(index);
    }
    let mut grouped = groups
        .into_iter()
        .filter(|(_, indices)| indices.len() >= 2)
        .map(|((datatype_key, prefix), indices)| {
            let stem = prefix.trim_end_matches('_').to_string();
            let family_constants: Vec<Constant> = indices
                .iter()
                .map(|&index| constants[index].clone())
                .collect();
            (datatype_key, stem, prefix, family_constants)
        })
        .collect::<Vec<_>>();
    grouped.sort_by_key(|group| core::cmp::Reverse(group.1.len()));
    let fields: Vec<_> = scalar_or_const_fields(message).collect();
    let mut families = Vec::new();
    for (datatype_key, stem, prefix, family_constants) in grouped {
        let Some(field) = fields.iter().find(|field| {
            field_key_matches(field, datatype_key) && field_name_matches_stem(field, stem.as_str())
        }) else {
            continue;
        };
        if family_constants
            .iter()
            .any(|constant| claimed_constants.contains(constant.name.as_str()))
        {
            continue;
        }
        let family = ConstantFamily {
            field_name: field.name().to_string(),
            constant_prefix: prefix,
            constants: sorted_constants(family_constants),
        };
        claimed_constants.extend(constant_names(&family.constants));
        families.push(family);
    }
    families
}

fn field_key_matches(field: &Field, expected_key: u8) -> bool {
    datatype_key(&field.datatype()) == expected_key
}

fn field_name_matches_stem(field: &Field, stem: &str) -> bool {
    field_matches_constant_stem(&field.name().to_uppercase(), stem)
}

fn constant_names(constants: &[Constant]) -> Vec<String> {
    constants
        .iter()
        .map(|constant| constant.name.clone())
        .collect()
}

fn datatype_key(datatype: &DataType) -> u8 {
    DatatypeSortKey::from(datatype) as u8
}

fn scalar_or_const_fields(message: &Message) -> impl Iterator<Item = &Field> {
    message.fields().iter().filter(|field| {
        matches!(field.case(), FieldCase::Scalar | FieldCase::Const(_))
            && !matches!(
                field.datatype(),
                DataType::String | DataType::Bool | DataType::GlobalMessage { .. }
            )
    })
}

fn sorted_constants(mut constants: Vec<Constant>) -> Vec<Constant> {
    constants.sort_by(|left, right| left.name.cmp(&right.name));
    constants
}

fn claimed_constant_names(families: &[ConstantFamily]) -> BTreeSet<String> {
    families
        .iter()
        .flat_map(|family| constant_names(&family.constants))
        .collect()
}

fn constant_name_prefix(name: &str) -> Option<String> {
    let underscore = name.rfind('_')?;
    if underscore == 0 {
        return None;
    }
    Some(format!("{}_", &name[..underscore]))
}

fn field_matches_constant_stem(field_upper: &str, stem: &str) -> bool {
    if field_upper == stem {
        return true;
    }
    field_upper.starts_with(stem)
        && field_upper
            .strip_prefix(stem)
            .is_some_and(|suffix| suffix.starts_with('_'))
}

#[cfg(test)]
mod tests {
    use super::{ConstantFamily, constant_families, field_matches_constant_stem};

    fn uint8_constant(name: &str, value: u8) -> crate::msg_ast::Constant {
        crate::msg_ast::Constant {
            name: name.to_string(),
            datatype: crate::msg_ast::DataType::U8,
            value: crate::msg_ast::ConstantValue::U8(value),
        }
    }

    #[test]
    fn field_stem_matches_status_and_self_test_phase() {
        assert!(field_matches_constant_stem("STATUS", "STATUS"));
        assert!(field_matches_constant_stem("SELF_TEST_PHASE", "SELF_TEST"));
        assert!(!field_matches_constant_stem("LEVEL", "SELF_TEST"));
    }

    #[test]
    fn constant_families_bind_status_and_self_test_prefixes() {
        let message = crate::msg_ast::Message::from_parts(
            vec![
                crate::msg_ast::Field::scalar_uint8("level"),
                crate::msg_ast::Field::scalar_uint8("max_level"),
                crate::msg_ast::Field::scalar_uint8("self_test_phase"),
            ],
            vec![
                uint8_constant("SELF_TEST_IDLE", 0),
                uint8_constant("SELF_TEST_RUNNING", 1),
            ],
        );
        let families = constant_families(&message);
        assert_eq!(families.len(), 1);
        assert_eq!(families[0].field_name, "self_test_phase");
        assert_eq!(families[0].constant_prefix, "SELF_TEST_");
    }

    #[test]
    fn constant_families_bind_state_prefix_with_needs_repair() {
        let message = crate::msg_ast::Message::from_parts(
            vec![crate::msg_ast::Field::scalar_uint8("state")],
            vec![
                uint8_constant("STATE_RECORDING", 0),
                uint8_constant("STATE_READY", 1),
                uint8_constant("STATE_NEEDS_REPAIR", 2),
                uint8_constant("STATE_REPAIRING", 3),
            ],
        );
        let families = constant_families(&message);
        assert_eq!(families.len(), 1);
        assert_eq!(families[0].constants.len(), 4);
    }

    #[test]
    fn constant_families_use_field_uppercase_prefix() {
        let message = crate::msg_ast::Message::from_parts(
            vec![crate::msg_ast::Field::scalar_uint8("status")],
            vec![
                uint8_constant("STATUS_QUEUED", 0),
                uint8_constant("STATUS_RUNNING", 1),
            ],
        );
        let families = constant_families(&message);
        assert_eq!(
            families,
            vec![ConstantFamily {
                field_name: "status".to_string(),
                constant_prefix: "STATUS_".to_string(),
                constants: vec![
                    uint8_constant("STATUS_QUEUED", 0),
                    uint8_constant("STATUS_RUNNING", 1),
                ],
            }]
        );
    }
}
