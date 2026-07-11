use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PrivilegeLevel {
    Root,
    System,
    Regular,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DangerousOperation {
    Reboot,
    Upgrade,
    SettingsReset,
    VehicleArm,
    FirmwareFlash,
    Other(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UserConfirmation {
    Required,
    NotRequired,
}
