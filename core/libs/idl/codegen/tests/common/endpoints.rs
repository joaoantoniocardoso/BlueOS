//! Workspace harness and schema names for endpoint generator tests.

#![expect(
    dead_code,
    reason = "shared endpoint test fixtures; each integration test binary uses a subset"
)]

extern crate alloc;

use alloc::collections::BTreeSet;
use std::{env, fs, path::PathBuf, process};

use blueos_idl_codegen::{
    endpoints::{ManifestError, generate},
    message_schema_names,
};

pub(crate) const SET_LEVEL: &str = "blueos_example_msgs/action/SetLevel";
pub(crate) const LEVEL: &str = "blueos_example_msgs/srv/Level";
pub(crate) const PUMP: &str = "blueos_example_msgs/msg/PumpState";

/// A throwaway workspace directory, removed when dropped.
pub(crate) struct Workspace {
    pub(crate) root: PathBuf,
}

impl Drop for Workspace {
    fn drop(&mut self) {
        _ = fs::remove_dir_all(&self.root);
    }
}

impl Workspace {
    pub(crate) fn new(name: &str) -> Self {
        let root = env::temp_dir().join(format!("blueos-endpoints-{}-{name}", process::id()));
        _ = fs::remove_dir_all(&root);
        Self { root }
    }

    pub(crate) fn write(&self, path: &str, contents: &str) {
        let path = self.root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
}

pub(crate) fn rejection(manifest: &str) -> ManifestError {
    generate(manifest, "blueos_test_api", &messages()).unwrap_err()
}

/// The schema names `message_schema_names` lists for the three interface types above: one per part.
pub(crate) fn messages() -> BTreeSet<String> {
    [
        "blueos_example_msgs/action/SetLevel_Goal",
        "blueos_example_msgs/action/SetLevel_Result",
        "blueos_example_msgs/action/SetLevel_Feedback",
        "blueos_example_msgs/srv/Level_Request",
        "blueos_example_msgs/srv/Level_Response",
        PUMP,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

pub(crate) fn example_message_schema_names() -> BTreeSet<String> {
    let core_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    message_schema_names(&core_dir.join("libs/idl/interfaces")).expect("messages")
}

pub(crate) fn core_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}
