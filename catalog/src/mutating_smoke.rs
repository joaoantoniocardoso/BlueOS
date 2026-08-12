use std::fmt;

use crate::catalog::Catalog;
use crate::id::JourneyId;
use crate::journey::{derive_automatable, Automatable, HttpMethod, UserJourney};
use crate::runner::http_steps;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmokeRepair {
    None,
    HttpRoundTrip,
    FilesystemReplace,
    TmuxServiceRestart,
    ContainerRestart,
    HostReboot,
    ExternalProxy,
    /// Runner-local NetworkManager AP/station (`catalog/harness/wifi`).
    HostWifiRf,
    ManualDocumented,
}

impl fmt::Display for SmokeRepair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            SmokeRepair::None => "none",
            SmokeRepair::HttpRoundTrip => "http_round_trip",
            SmokeRepair::FilesystemReplace => "filesystem_replace",
            SmokeRepair::TmuxServiceRestart => "tmux_service_restart",
            SmokeRepair::ContainerRestart => "container_restart",
            SmokeRepair::HostReboot => "host_reboot",
            SmokeRepair::ExternalProxy => "external_proxy",
            SmokeRepair::HostWifiRf => "host_wifi_rf",
            SmokeRepair::ManualDocumented => "manual_documented",
        };
        write!(f, "{label}")
    }
}

pub struct MutatingSmokeEntry {
    pub journey_id: JourneyId,
    pub setup: SmokeRepair,
    pub restore: SmokeRepair,
    pub notes: &'static str,
}

pub const MUTATING_SMOKE_ENTRIES: &[MutatingSmokeEntry] = &[
    MutatingSmokeEntry {
        journey_id: JourneyId::AutoconnectToSavedWifiNetwork,
        setup: SmokeRepair::HostWifiRf,
        restore: SmokeRepair::HostWifiRf,
        notes: "host AP up; connect+save; AP down→status idle; AP up→autoconnect without POST /connect",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::ChangeUiThemeColor,
        setup: SmokeRepair::None,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "GET /theme snapshot before PUT; restore after mutate",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::ResetUiThemeColor,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "non-default theme before DELETE /theme/custom; restore snapshot after",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::RunLanSpeedTest,
        setup: SmokeRepair::None,
        restore: SmokeRepair::None,
        notes: "POST /lan_speed_test is idempotent side-effect-free",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::RemoveCustomLogo,
        setup: SmokeRepair::None,
        restore: SmokeRepair::None,
        notes: "DELETE /logo/custom is idempotent when no custom logo",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::RemoveCustomVehicleImage,
        setup: SmokeRepair::None,
        restore: SmokeRepair::None,
        notes: "DELETE /vehicle_image/custom is idempotent when no custom image",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::AcquireDynamicIpAddress,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "GET interface address snapshot; POST /dynamic_ip mutate; restore prior addresses; HostReboot if runner loses connectivity",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::AddCustomManifest,
        setup: SmokeRepair::None,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "POST /manifest/ create; DELETE manifest entry to restore",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::AssignStaticIpAddress,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "GET interface config snapshot; POST /address mutate; DELETE /address restore; HostReboot if stranded off-LAN",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::ChangeBoard,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "GET board state snapshot; POST /board mutate; POST restore prior board; TmuxServiceRestart ardupilot_manager if needed",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::ChangeMdnsHostname,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "GET /hostname; POST hostname=smoke-catalog; POST hostname=blueos restore",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::ConfigureHostDns,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "GET /host_dns snapshot; POST mutate; POST restore prior nameservers",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::ConfigureHotspotCredentials,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "GET hotspot credentials snapshot; POST /hotspot_credentials mutate; POST restore",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::ConfigureInstalledExtension,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "GET /extension/{id} snapshot; PUT/restart/disable mutate; restore config and re-enable if disabled",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::ConnectToWifiNetwork,
        setup: SmokeRepair::HostWifiRf,
        restore: SmokeRepair::HostWifiRf,
        notes: "Runner host-ap.sh up wpa2; POST /connect to BlueOS-Hotspot; disconnect+remove; host-ap down",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::ConnectToHiddenWifiNetwork,
        setup: SmokeRepair::HostWifiRf,
        restore: SmokeRepair::HostWifiRf,
        notes: "host AP up; POST /connect?hidden=true; disconnect+remove; host-ap down",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::ForceWifiNetworkPassword,
        setup: SmokeRepair::HostWifiRf,
        restore: SmokeRepair::HostWifiRf,
        notes: "host AP up; connect+save; POST /connect with new password; disconnect+remove; host-ap down",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::ReconnectToSavedWifiNetwork,
        setup: SmokeRepair::HostWifiRf,
        restore: SmokeRepair::HostWifiRf,
        notes: "host AP up; connect+save; disconnect; POST /connect empty password; cleanup; host-ap down",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::RejectInvalidWifiCredentials,
        setup: SmokeRepair::HostWifiRf,
        restore: SmokeRepair::HostWifiRf,
        notes: "host AP up; POST /connect wrong password expects failure; host-ap down",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::Delete3dModelOverride,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "setup uploads test .glb; DELETE /models/{name} mutate; re-upload snapshot or DELETE cleanup",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::DeleteLocalBlueosVersion,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::ExternalProxy,
        notes: "setup pulls spare tag; DELETE /version/delete mutate; re-pull deleted tag via caching proxy if needed",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::DeleteVideoRecording,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::FilesystemReplace,
        notes: "setup seeds test MP4; DELETE /recorder/files/{filename} mutate; restore file from snapshot",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::DetectWifiApLoss,
        setup: SmokeRepair::HostWifiRf,
        restore: SmokeRepair::HostWifiRf,
        notes: "host AP up; connect; host-ap down; poll GET /status until SSID clears",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::DisableOnboardDhcpServer,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "setup POST /dhcp when absent; DELETE /dhcp mutate; POST /dhcp restore from snapshot",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::DockerRegistryLogin,
        setup: SmokeRepair::None,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "POST /docker/login mutate; GET /docker/accounts snapshot; logout or re-login prior account",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::EditExtensionDevVersion,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "GET installed extension tag snapshot; PUT /extension/{id}/{tag} mutate; PUT restore prior tag",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::EnableLegacyCameraSupport,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "GET camera_legacy state snapshot; POST mutate; POST restore prior flag; HostReboot to apply",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::EnableOnboardDhcpServer,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "snapshot interface+DHCP state; POST /dhcp mutate; DELETE /dhcp restore",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::ForgetSavedWifiNetwork,
        setup: SmokeRepair::HostWifiRf,
        restore: SmokeRepair::HostWifiRf,
        notes: "host AP up; connect+save BlueOS-Hotspot; POST /remove; host-ap down",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::FreeDiskSpace,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::FilesystemReplace,
        notes: "setup copies disposable test file; DELETE /disk/paths mutate; restore file from filesystem snapshot",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::InstallCustomExtension,
        setup: SmokeRepair::None,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "POST /extension/ install; DELETE uninstall to restore",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::InstallExtension,
        setup: SmokeRepair::None,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "POST /extension/{id}/{tag}/install; DELETE uninstall to restore",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::ModifyBagDatabase,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "GET /get/* snapshot before POST /overwrite; restore snapshot after mutate",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::PullBlueosVersionWithoutSwitch,
        setup: SmokeRepair::ExternalProxy,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "pull via caching proxy; DELETE /version/delete pulled tag to restore disk",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::RebootOnboardComputer,
        setup: SmokeRepair::None,
        restore: SmokeRepair::HostReboot,
        notes: "POST /shutdown reboot deferred in automated smoke; runner waits for HTTP recovery when enabled",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::RemoveCameraStream,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "setup POST /streams create stream; DELETE /delete_stream mutate; recreate from snapshot",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::RemoveConfiguredNmeaSocket,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "setup POST /socks; DELETE /socks mutate; recreate sock from snapshot",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::RemoveSerialBridge,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "setup POST /bridges; DELETE /bridges mutate; recreate bridge from snapshot",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::RenameVehicle,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "GET /vehicle_name snapshot before POST; restore after mutate",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::ResetBlueosSettings,
        setup: SmokeRepair::FilesystemReplace,
        restore: SmokeRepair::ContainerRestart,
        notes: "backup /root/.config service trees before POST /settings/reset; restore files and docker restart blueos-core",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::RestartAutopilot,
        setup: SmokeRepair::None,
        restore: SmokeRepair::TmuxServiceRestart,
        notes: "POST /restart is idempotent when healthy; TmuxServiceRestart ardupilot_manager if process wedged",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::RestoreDefaultFirmware,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "GET /firmware_info snapshot; POST restore_default_firmware mutate; re-flash prior via install_firmware_from_url if needed",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::RunHostCommand,
        setup: SmokeRepair::None,
        restore: SmokeRepair::None,
        notes: "smoke uses no-op host command (e.g. true); destructive commands need per-command restore",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::RunSitlSimulation,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "snapshot board+SITL frame; POST /board and /sitl_frame mutate; POST restore prior board/frame",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::SetNetworkInterfacePriority,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "GET interface priority snapshot; POST /set_interfaces_priority mutate; POST restore ordering",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::StartAutopilot,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "snapshot running state; POST /start mutate; POST /stop restore if was stopped",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::StopAutopilot,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "snapshot running state; POST /stop mutate; POST /start restore if was running",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::SwitchLocalBlueosVersion,
        setup: SmokeRepair::FilesystemReplace,
        restore: SmokeRepair::ContainerRestart,
        notes: "docker tag :master→smoke-catalog-switch (copies DUT digest, not a pin); POST switch; restore tag :master (intended digest sha256:ae50d2d1d5935db0d764e2f14837039ff498d78abf22eb3c8f3fd68aa3fe4b76); DELETE smoke-catalog-switch",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::SyncSystemTime,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "GET system time snapshot; POST /set_time mutate; POST restore prior timestamp",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::ToggleHotspot,
        setup: SmokeRepair::HostWifiRf,
        restore: SmokeRepair::HostWifiRf,
        notes: "set E2E hotspot credentials; POST enable=true; host-station join+lease; disable; restore credentials",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::ToggleSmartHotspot,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "GET smart_hotspot setting snapshot; POST /smart_hotspot mutate; POST restore",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::UninstallExtension,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "setup POST install extension; DELETE uninstall mutate; POST install restore from snapshot",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::UpdateBlueosVersion,
        setup: SmokeRepair::ExternalProxy,
        restore: SmokeRepair::ExternalProxy,
        notes: "cache Docker Hub via proxy; GET /version/current snapshot; pull+switch mutate; switch back and ContainerRestart",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::UpdateBootstrapImage,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::ExternalProxy,
        notes: "GET /bootstrap/current snapshot; pull+POST bootstrap mutate; restore prior bootstrap tag; HostReboot if runner changed",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::UpdateFirmwareOnline,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "GET /firmware_info snapshot; POST install_firmware_from_url mutate; restore prior via install_firmware_from_url or restore_default_firmware",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::UpdateRaspberryEepromBootloader,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HostReboot,
        notes: "GET /raspi/eeprom_update snapshot; POST mutate; HostReboot; play-Pi reimage acceptable if EEPROM cannot revert",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::Upload3dModelOverride,
        setup: SmokeRepair::None,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "POST /models upload mutate; DELETE model restore",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::UploadCustomFirmware,
        setup: SmokeRepair::HttpRoundTrip,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "GET /firmware_info snapshot; POST install_firmware_from_file mutate; restore prior firmware via install_firmware_from_url or restore_default_firmware",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::UploadCustomLogo,
        setup: SmokeRepair::None,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "POST /branding/logo mutate; DELETE /branding/logo restore (multipart via catalog/fixtures)",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::UploadCustomVehicleImage,
        setup: SmokeRepair::None,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "POST /branding/vehicle-image mutate; DELETE restore (multipart via catalog/fixtures)",
    },
    MutatingSmokeEntry {
        journey_id: JourneyId::VehicleFirstBoot,
        setup: SmokeRepair::ExternalProxy,
        restore: SmokeRepair::HttpRoundTrip,
        notes: "proxy firmware URL; POST install_firmware_from_url mutate; restore_default_firmware or re-flash captured firmware_info",
    },
];

pub fn mutating_smoke_journey_ids() -> impl Iterator<Item = JourneyId> + Clone {
    MUTATING_SMOKE_ENTRIES.iter().map(|entry| entry.journey_id)
}

pub fn is_mutating_smoke_journey(journey_id: JourneyId) -> bool {
    MUTATING_SMOKE_ENTRIES
        .iter()
        .any(|entry| entry.journey_id == journey_id)
}

pub fn journey_has_mutating_http_route(journey: &UserJourney) -> bool {
    http_steps(journey)
        .iter()
        .any(|step| !matches!(step.route.method, HttpMethod::Get))
}

pub fn is_tier2_mutating_eligible(journey: &UserJourney) -> bool {
    if journey.id == JourneyId::ShutdownOnboardComputer {
        return false;
    }
    if derive_automatable(journey) != Automatable::Http {
        return false;
    }
    journey_has_mutating_http_route(journey) || crate::wifi_rf::is_rf_status_journey(journey.id)
}

pub fn is_tier2_mutating_hard_excluded(journey_id: JourneyId) -> bool {
    journey_id == JourneyId::ShutdownOnboardComputer
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tier2MutatingCoverage {
    pub eligible_count: usize,
    pub allowlisted_count: usize,
    pub missing: Vec<JourneyId>,
    pub excluded: Vec<JourneyId>,
}

pub fn tier2_mutating_coverage(catalog: &Catalog) -> Tier2MutatingCoverage {
    let allowlisted: std::collections::HashSet<JourneyId> = mutating_smoke_journey_ids().collect();
    let mut eligible_count = 0;
    let mut missing = Vec::new();
    let mut excluded = Vec::new();

    for journey in catalog.journeys() {
        if is_tier2_mutating_hard_excluded(journey.id) {
            if journey_has_mutating_http_route(journey)
                && derive_automatable(journey) == Automatable::Http
            {
                excluded.push(journey.id);
            }
            continue;
        }
        if !is_tier2_mutating_eligible(journey) {
            continue;
        }
        eligible_count += 1;
        if !allowlisted.contains(&journey.id) {
            missing.push(journey.id);
        }
    }

    missing.sort_by_key(|id| id.as_str());
    excluded.sort_by_key(|id| id.as_str());

    Tier2MutatingCoverage {
        eligible_count,
        allowlisted_count: allowlisted.len(),
        missing,
        excluded,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::journey::derive_automatable;
    use crate::runner::http_steps;

    #[test]
    fn mutating_smoke_entries_are_http_with_mutating_steps() {
        let catalog = Catalog::bootstrap();
        let journeys: std::collections::HashMap<_, _> = catalog
            .journeys()
            .iter()
            .map(|journey| (journey.id, journey))
            .collect();
        for entry in MUTATING_SMOKE_ENTRIES {
            let journey = journeys
                .get(&entry.journey_id)
                .unwrap_or_else(|| panic!("missing journey {}", entry.journey_id));
            assert_eq!(derive_automatable(journey), Automatable::Http);
            assert!(
                journey_has_mutating_http_route(journey)
                    || crate::wifi_rf::is_rf_status_journey(entry.journey_id),
                "{} has no mutating HTTP route steps",
                entry.journey_id
            );
        }
    }

    #[test]
    fn tier2_mutating_coverage_is_complete() {
        let catalog = Catalog::bootstrap();
        let coverage = tier2_mutating_coverage(&catalog);
        assert_eq!(coverage.allowlisted_count, MUTATING_SMOKE_ENTRIES.len());
        assert_eq!(
            coverage.allowlisted_count, coverage.eligible_count,
            "allowlist should cover every Tier-2-eligible journey"
        );
        assert!(
            coverage.missing.is_empty(),
            "missing Tier-2 mutating journeys: {:?}",
            coverage.missing
        );
    }

    #[test]
    fn shutdown_onboard_computer_is_excluded_not_missing() {
        let catalog = Catalog::bootstrap();
        let coverage = tier2_mutating_coverage(&catalog);
        assert!(coverage
            .excluded
            .contains(&JourneyId::ShutdownOnboardComputer));
        assert!(!coverage
            .missing
            .contains(&JourneyId::ShutdownOnboardComputer));
    }

    #[test]
    fn concrete_mutating_steps_in_allowlist_are_grounded() {
        let catalog = Catalog::bootstrap();
        let mut ungrounded = Vec::new();
        for journey in catalog.journeys() {
            if !is_mutating_smoke_journey(journey.id) {
                continue;
            }
            for step in http_steps(journey) {
                if matches!(step.route.method, crate::journey::HttpMethod::Get) {
                    continue;
                }
                if (step.route.path.contains('{') || step.route.path.contains('*'))
                    && crate::runner::mutating_smoke_path_bind(journey.id, step.route.path)
                        .is_none()
                {
                    continue;
                }
                if step.expected_status.is_none() {
                    ungrounded.push((journey.id, step.step_index, step.route.path));
                }
            }
        }
        assert!(
            ungrounded.is_empty(),
            "ungrounded concrete mutating steps: {ungrounded:?}"
        );
    }

    #[test]
    fn mutating_smoke_journey_ids_matches_entries() {
        let ids: Vec<_> = mutating_smoke_journey_ids().collect();
        assert_eq!(ids.len(), MUTATING_SMOKE_ENTRIES.len());
        for entry in MUTATING_SMOKE_ENTRIES {
            assert!(ids.contains(&entry.journey_id));
        }
    }
}
