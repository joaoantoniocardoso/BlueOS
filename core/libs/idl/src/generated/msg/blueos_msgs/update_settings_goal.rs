#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateSettingsGoal {
    pub envelope: crate::msg::blueos_msgs::SettingsEnvelope,
}
impl CdrStruct for UpdateSettingsGoal {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            envelope: if reader.is_exhausted() {
                <crate::msg::blueos_msgs::SettingsEnvelope>::default()
            } else {
                <crate::msg::blueos_msgs::SettingsEnvelope>::cdr_decode_fields(reader)?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        <crate::msg::blueos_msgs::SettingsEnvelope>::cdr_encode_fields(&self.envelope, writer)?;
        Ok(())
    }
}
impl Message for UpdateSettingsGoal {
    const SCHEMA: &'static str = "# blueos_msgs/action/UpdateSettings\n# The Job type every Service serves on blueos/v1/<service>/command/UpdateSettings (D-11): replaces its settings\n# document. An instant Job type, so its ack is its final status. A Service without settings rejects it.\n\nblueos_msgs/SettingsEnvelope envelope\n================================================================================\nMSG: blueos_msgs/SettingField\n# blueos_msgs/msg/SettingField\n# Restart hint for one settings field (D-11).\n\nstring path\nbool restart_required\n================================================================================\nMSG: blueos_msgs/SettingsEnvelope\n# blueos_msgs/msg/SettingsEnvelope\n# JSON settings document plus per-field restart flags (D-11).\n\nstring document_json\nblueos_msgs/SettingField[] fields";
    const SCHEMA_NAME: &'static str = "blueos_msgs/action/UpdateSettings_Goal";
    const TYPE_HASH: &'static str =
        "41f1ec195a4488f589c5fe977571ed0d9e92b31603c3e1e095f3bcbc39b626f3";
}
impl UpdateSettingsGoal {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
