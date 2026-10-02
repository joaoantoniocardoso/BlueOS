//! Constant-family detection for IDL enum emission (D-05).

use alloc::collections::{BTreeMap, BTreeSet};

use crate::msg_ast::{Constant, DataType, FieldCase, Message};

/// Constants that share a prefix and type, bound to one message field (D-05).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ConstantFamily {
    pub(crate) field_name: String,
    pub(crate) constant_prefix: String,
    pub(crate) constants: Vec<Constant>,
}

pub(crate) fn constant_families(message: &Message) -> Vec<ConstantFamily> {
    let mut families = Vec::new();
    let mut claimed_constants = BTreeSet::new();
    for field in message.fields() {
        if !matches!(field.case(), FieldCase::Scalar | FieldCase::Const(_)) {
            continue;
        }
        if matches!(
            field.datatype(),
            DataType::String | DataType::Bool | DataType::GlobalMessage { .. }
        ) {
            continue;
        }
        let field_prefix = format!("{}_", field.name().to_uppercase());
        let mut family_constants: Vec<Constant> = message
            .constants()
            .iter()
            .filter(|constant| {
                constant.name.starts_with(&field_prefix)
                    && field.datatype() == constant.datatype
                    && !claimed_constants.contains(&constant.name)
            })
            .cloned()
            .collect();
        if family_constants.len() < 2 {
            continue;
        }
        family_constants.sort_by(|left, right| left.name.cmp(&right.name));
        for constant in &family_constants {
            claimed_constants.insert(constant.name.clone());
        }
        families.push(ConstantFamily {
            field_name: field.name().to_string(),
            constant_prefix: field_prefix,
            constants: family_constants,
        });
    }
    let mut groups = BTreeMap::new();
    for constant in message.constants() {
        if claimed_constants.contains(&constant.name) {
            continue;
        }
        let Some(prefix) = constant_name_prefix(&constant.name) else {
            continue;
        };
        groups
            .entry((constant.datatype.clone(), prefix))
            .or_insert_with(Vec::new)
            .push(constant.clone());
    }
    let mut grouped = groups
        .into_iter()
        .filter(|(_, constants)| !constants.is_empty())
        .map(|((datatype, prefix), constants)| {
            let stem = prefix.trim_end_matches('_');
            (datatype, stem.to_string(), prefix, constants)
        })
        .collect::<Vec<_>>();
    grouped.sort_by_key(|group| core::cmp::Reverse(group.1.len()));
    for field in message.fields() {
        if !matches!(field.case(), FieldCase::Scalar | FieldCase::Const(_)) {
            continue;
        }
        if matches!(
            field.datatype(),
            DataType::String | DataType::Bool | DataType::GlobalMessage { .. }
        ) {
            continue;
        }
        let field_upper = field.name().to_uppercase();
        for (datatype, stem, prefix, constants) in &grouped {
            if field.datatype() != *datatype {
                continue;
            }
            if !field_matches_constant_stem(&field_upper, stem) {
                continue;
            }
            if constants
                .iter()
                .any(|constant| claimed_constants.contains(&constant.name))
            {
                continue;
            }
            let mut family_constants = constants.clone();
            family_constants.sort_by(|left, right| left.name.cmp(&right.name));
            for constant in &family_constants {
                claimed_constants.insert(constant.name.clone());
            }
            families.push(ConstantFamily {
                field_name: field.name().to_string(),
                constant_prefix: prefix.clone(),
                constants: family_constants,
            });
            break;
        }
    }
    families
}

pub(crate) fn freestanding_constants(message: &Message) -> Vec<Constant> {
    let claimed = constant_families(message)
        .iter()
        .flat_map(|family| {
            family
                .constants
                .iter()
                .map(|constant| constant.name.clone())
        })
        .collect::<BTreeSet<_>>();
    message
        .constants()
        .iter()
        .filter(|constant| !claimed.contains(&constant.name))
        .cloned()
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
