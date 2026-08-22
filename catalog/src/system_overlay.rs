use crate::capability::Aggregate;
use crate::domain::Domain;
use crate::id::JourneyId;
use crate::provenance::Provenance;

pub struct SystemOverlayEntry {
    pub suffix: &'static str,
    pub domain: Domain,
    pub aggregate: Aggregate,
    pub statement: &'static str,
    pub criteria: &'static [&'static str],
    pub provenance: Provenance,
    pub journey_ids: &'static [JourneyId],
    pub traces: &'static [&'static str],
}

pub const SYSTEM_OVERLAY_ENTRIES: &[SystemOverlayEntry] = &[
    SystemOverlayEntry {
        suffix: "boot_to_usable_web_ui",
        domain: Domain::Presentation,
        aggregate: Aggregate::WebIngress,
        statement: concat!(
            "After power-on the operator can reach the vehicle web interface ",
            "without manual service recovery"
        ),
        criteria: &[
            "boot_timeline monitor records nginx HTTP 200 on port 80 after first boot",
            "content/usage/installation.md documents first-boot expansion may take minutes",
        ],
        provenance: Provenance::asserted(concat!(
            "Operators and field techs expect a powered vehicle to present the BlueOS UI ",
            "without ssh intervention; boot_timeline captures this on real hardware",
        )),
        journey_ids: &[JourneyId::DiscoverBlueosOnNetwork],
        traces: &["content/usage/installation.md:59"],
    },
    SystemOverlayEntry {
        suffix: "local_network_without_internet",
        domain: Domain::Network,
        aggregate: Aggregate::WiredNetwork,
        statement: concat!(
            "Core vehicle configuration remains available on the local network when ",
            "wide-area internet is unavailable"
        ),
        criteria: &[
            "journey:DiscoverBlueosOnNetwork uses wired Ethernet precondition only",
            concat!(
                "RenameVehicle (catalog/src/journeys/beacon.rs:36) and InspectDiskUsage ",
                "(catalog/src/journeys/disk_usage.rs:55) declare empty preconditions, so neither ",
                "requires Network::Online"
            ),
        ],
        provenance: Provenance::asserted(concat!(
            "ROV operators routinely configure vehicles on bench Ethernet before internet is ",
            "available; getting-started documents wired access",
        )),
        journey_ids: &[
            JourneyId::DiscoverBlueosOnNetwork,
            JourneyId::RenameVehicle,
            JourneyId::InspectDiskUsage,
        ],
        traces: &[
            "content/usage/getting-started/index.md:29",
            "catalog/src/journeys/beacon.rs:36",
            "catalog/src/journeys/disk_usage.rs:55",
        ],
    },
    SystemOverlayEntry {
        suffix: "storage_pressure_visibility",
        domain: Domain::OnboardComputer,
        aggregate: Aggregate::Storage,
        statement: concat!(
            "The operator can inspect storage consumption before userdata exhaustion ",
            "blocks operations"
        ),
        criteria: &[
            "journey:InspectDiskUsage returns a du-backed usage tree for a selected path",
            "journey:FreeDiskSpace removes files to reclaim space",
        ],
        provenance: Provenance::asserted(concat!(
            "Full userdata disks brick log and recording workflows; disk_usage journeys are ",
            "the operator-facing contract",
        )),
        journey_ids: &[JourneyId::InspectDiskUsage, JourneyId::FreeDiskSpace],
        traces: &["core/services/disk_usage/main.py:251"],
    },
    SystemOverlayEntry {
        suffix: "essential_service_supervision",
        domain: Domain::BlueOsPlatform,
        aggregate: Aggregate::HostControl,
        statement: concat!(
            "Essential platform services are supervised and restarted when they exit ",
            "unexpectedly"
        ),
        criteria: &[
            concat!(
                "core/run-service.sh:141-154 restarts the service command in a while-true loop ",
                "after unexpected exit"
            ),
            "core/start-blueos-core:200 invokes run-service for each launched service",
        ],
        provenance: Provenance::asserted(concat!(
            "BlueOS 1.x restarts essential services via the run-service while-true loop, ",
            "not via tmux itself",
        )),
        journey_ids: &[],
        traces: &["core/run-service.sh:141"],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    use crate::requirement::find_contamination;

    #[test]
    fn overlay_entry_count_within_cap() {
        assert!(
            SYSTEM_OVERLAY_ENTRIES.len() <= 12,
            "overlay must have at most 12 entries"
        );
        assert!(
            !SYSTEM_OVERLAY_ENTRIES.is_empty(),
            "overlay must not be empty"
        );
    }

    #[test]
    fn overlay_statements_pass_contamination_lint() {
        for entry in SYSTEM_OVERLAY_ENTRIES {
            assert!(
                find_contamination(entry.statement).is_none(),
                "overlay statement contaminated: {}",
                entry.suffix
            );
        }
    }

    #[test]
    fn overlay_citation_kind_is_asserted() {
        for entry in SYSTEM_OVERLAY_ENTRIES {
            assert!(
                matches!(entry.provenance, Provenance::Asserted { .. }),
                "overlay must stay Asserted-only so provenance_walk can exclude it: {}",
                entry.suffix
            );
        }
    }
}
