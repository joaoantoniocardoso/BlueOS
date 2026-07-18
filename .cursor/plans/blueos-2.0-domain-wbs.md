# BlueOS 2.0 — Domain WBS (M4 input)

**Status:** M4 workshop input — **not a boundary decision.** The tree is encoded in the catalog registry; this report only materializes it for human review.

**Regenerate:** `cargo run -q --bin wbs` (text) or `cargo run -q --bin wbs -- --json` (tooling).

---

## Subject-axis tree (Domain → Aggregate → Feature)

Six domains in registry order (`catalog/src/domain.rs` → `DOMAINS`):

| Domain | Aggregates | Rationale (abbrev.) |
|---|---|---|
| `vehicle` | autopilot, mavlink, recording | Flight controller, MAVLink routing, vehicle-side recording |
| `peripherals` | camera, serial_bridge, sonar, gps_nmea | Attached sensors and serial bridges beyond the flight stack |
| `onboard_computer` | host_control, files_kv, storage, shell_access | Companion-computer host control, files, storage, shell |
| `network` | wired_network, wireless_network, identity_discovery, net_diagnostics | Connectivity, LAN identity, link diagnostics |
| `blueos_platform` | versioning, extensions, web_ingress, message_bus | OS lifecycle, extensions, HTTP ingress, inter-service messaging |
| `presentation` | branding_ui | Web UI branding and frontend shell |

Under each aggregate, **143 features** (capabilities + frontend page features from `FeatureCatalog`) are listed sorted by capability id. Counts at the top of the report: 6 domains, 20 aggregates, 143 features.

---

## WebIngress vs MessageBus (PlatformInfra split)

`blueos_platform` formerly grouped HTTP/nginx concerns with Zenoh/pub-sub under a single **PlatformInfra** aggregate. The registry now splits them:

- **`web_ingress`** — nginx reverse proxy, static frontend delivery, HTTP routing to services.
- **`message_bus`** — Zenoh daemon and inter-service pub/sub messaging.

Both remain in `blueos_platform` because they are platform-wide infrastructure, but the split lets M4 discuss HTTP edge vs messaging bus as separate deployable surfaces.

---

## How to use in M4

1. Run `cargo run -q --bin wbs` and walk the tree domain by domain.
2. Use aggregate feature lists to see which capabilities cluster today — compare with service-split lenses in `blueos-2.0-split-approaches.md` and feature groupings (`cargo run -q --bin groupings` when wired).
3. Treat disagreements between subject-axis (domain WBS) and coupling/journey lenses as **discussion prompts**, not automatic re-homes.
