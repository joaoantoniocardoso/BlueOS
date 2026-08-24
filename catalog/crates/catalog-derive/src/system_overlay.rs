use catalog_kernel::aggregate::Aggregate;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::provenance::Provenance;

pub struct SystemOverlayEntry {
    pub suffix: &'static str,
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
        aggregate: Aggregate::WebIngress,
        statement: concat!(
            "After power-on the operator can reach the vehicle web interface ",
            "without manual service recovery"
        ),
        criteria: &[
            concat!(
                "boot_timeline recorded HTTP 200 on port 80 at phase2 on 2 of 3 monitored hosts, each ",
                "only after one automatic reboot (catalog/boot-timeline-192.168.0.124/summary.json:55 at ",
                "73.4s, catalog/boot-timeline-192.168.0.177/summary.json:65 at 121.9s)"
            ),
            concat!(
                "the third host reached no HTTP response and no reboot before the monitor gave up ",
                "(catalog/boot-timeline-192.168.0.87/summary.json:16 max_wait)"
            ),
            "content/usage/installation.md documents first-boot expansion may take minutes",
        ],
        provenance: Provenance::asserted(concat!(
            "Operators and field techs expect a powered vehicle to present the BlueOS UI ",
            "without ssh intervention; boot_timeline captures this on real hardware",
        )),
        journey_ids: &[JourneyId::DiscoverBlueosOnNetwork],
        traces: &[
            "content/usage/installation.md:59",
            "catalog/boot-timeline-192.168.0.124/summary.json:55",
            "catalog/boot-timeline-192.168.0.177/summary.json:65",
            "catalog/boot-timeline-192.168.0.87/summary.json:16",
        ],
    },
    SystemOverlayEntry {
        suffix: "local_network_without_internet",
        aggregate: Aggregate::WiredNetwork,
        statement: concat!(
            "Core vehicle configuration remains available on the local network when ",
            "wide-area internet is unavailable"
        ),
        criteria: &[
            concat!(
                "journey:DiscoverBlueosOnNetwork declares two wired-connection preconditions and no ",
                "Network::Online (catalog/crates/catalog-data/src/journeys/beacon.rs:110-121)"
            ),
            concat!(
                "RenameVehicle (catalog/crates/catalog-data/src/journeys/beacon.rs:36) and InspectDiskUsage ",
                "(catalog/crates/catalog-data/src/journeys/disk_usage.rs:55) declare empty preconditions, so neither ",
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
            "catalog/crates/catalog-data/src/journeys/beacon.rs:36",
            "catalog/crates/catalog-data/src/journeys/beacon.rs:110",
            "catalog/crates/catalog-data/src/journeys/disk_usage.rs:55",
        ],
    },
    SystemOverlayEntry {
        suffix: "storage_pressure_visibility",
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
        aggregate: Aggregate::HostControl,
        statement: concat!(
            "Essential platform services are supervised and restarted when they exit ",
            "unexpectedly"
        ),
        criteria: &[
            concat!(
                "core/run-service.sh:142-154 restarts the service command in a while-true loop ",
                "after unexpected exit"
            ),
            "core/start-blueos-core:200 invokes run-service for each launched service",
            concat!(
                "journey:RebootOnboardComputer returns the platform to service without operator ",
                "intervention, so every service it lost is relaunched by that supervisor"
            ),
        ],
        provenance: Provenance::asserted(concat!(
            "BlueOS 1.x restarts essential services via the run-service while-true loop, ",
            "not via tmux itself",
        )),
        journey_ids: &[JourneyId::RebootOnboardComputer],
        traces: &["core/run-service.sh:142", "core/start-blueos-core:200"],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    use crate::requirement::{find_contamination, overlay_availability_from_ids};
    use crate::requirements_report::overlay_trace_failures;
    use catalog_core::catalog::Catalog;
    use catalog_kernel::version::availability_is_valid;

    const REVIEWED_TAGS: &[&str] = &["1.0.0", "1.4.0", "1.4-dev", "master"];

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

    #[test]
    fn overlay_entries_have_at_least_one_criterion() {
        for entry in SYSTEM_OVERLAY_ENTRIES {
            assert!(
                !entry.criteria.is_empty(),
                "overlay entry states no acceptance criterion: {}",
                entry.suffix
            );
        }
    }

    #[test]
    fn overlay_entries_are_present_on_at_least_one_tag() {
        let catalog = Catalog::bootstrap();
        for entry in SYSTEM_OVERLAY_ENTRIES {
            let availability = overlay_availability_from_ids(&catalog, entry.journey_ids);
            availability_is_valid(&availability).unwrap_or_else(|err| {
                panic!(
                    "overlay entry {} has invalid availability: {err}",
                    entry.suffix
                )
            });
            assert!(
                REVIEWED_TAGS
                    .iter()
                    .any(|tag| availability.present_on_dut(tag)),
                "overlay entry {} is present on no release tag, so it renders in no output",
                entry.suffix
            );
        }
    }

    #[test]
    fn overlay_traces_resolve_and_are_committed() {
        let failures = overlay_trace_failures();
        assert!(
            failures.is_empty(),
            "overlay traces must all resolve and be committed: {failures:#?}"
        );
    }
}
