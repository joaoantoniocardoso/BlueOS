//! TypeScript client source for a Service.

use alloc::collections::BTreeSet;
use core::fmt::Write as _;

use convert_case::{Case, Casing};

use super::super::{Endpoint, TYPESCRIPT_GENERATED_NOTICE};

pub(in super::super) fn typescript_client_source(service: &str, endpoints: &[Endpoint]) -> String {
    let mut key_helpers = BTreeSet::new();
    for endpoint in endpoints {
        key_helpers.insert(endpoint.kind.typescript_key_helper());
    }
    let mut source = format!("{TYPESCRIPT_GENERATED_NOTICE}\n");
    if !endpoints.is_empty() {
        source.push_str("import type * as Idl from '@blueos-idl/messages'\n\n");
    }
    if !key_helpers.is_empty() {
        let helpers = key_helpers.into_iter().collect::<Vec<_>>().join(",\n  ");
        _ = write!(
            source,
            "import {{\n  {helpers},\n}} from '../keys'\n",
            helpers = helpers
        );
    }
    _ = write!(
        source,
        "\n/** The name of the `{service}` Service, in each of its keys: `blueos/v1/{service}/...`. */\nexport const NAME = \
         '{service}'\n"
    );
    for endpoint in endpoints {
        let name = &endpoint.name;
        let export_name = typescript_export_name(name);
        let key_helper = endpoint.kind.typescript_key_helper();
        let kind = endpoint.kind_name();
        let key = endpoint.key(service);
        let interface = &endpoint.interface;
        _ = write!(
            source,
            "\n/** {kind} `{name}` at `{key}`. */\nexport const {export_name} = {{\n  kind: '{kind}' as const,\n  name: \
             '{name}',\n  key: {key_helper}(NAME, '{name}'),\n  type: '{}' as const,\n",
            interface.schema_name()
        );
        let parts = endpoint.kind.typescript_schema_parts();
        for (field, part) in parts {
            _ = writeln!(
                source,
                "  {field}: '{}' as const,",
                interface.part_schema_name(part)
            );
        }
        source.push_str("}\n");
        for (_, part) in parts {
            let type_name = if part.is_empty() {
                typescript_type_name(name)
            } else {
                format!("{export_name}{part}")
            };
            _ = writeln!(
                source,
                "export type {type_name} = Idl.{}",
                interface.typescript_type(part)
            );
        }
    }
    source
}

fn typescript_export_name(name: &str) -> String {
    name.to_owned()
}

fn typescript_type_name(name: &str) -> String {
    name.to_case(Case::UpperCamel)
}
