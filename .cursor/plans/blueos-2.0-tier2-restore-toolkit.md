# Tier-2 mutating smoke — restore toolkit

Play Pi (`192.168.0.177`) is owned test hardware. Reimage is acceptable. Tier-2 grows by **declared restore**, not by avoiding mutation.

## 100% coverage gate (typed, not yet enforced)

A journey is **Tier-2 eligible** when:

- `derive_automatable(journey) == Automatable::Http`, and
- it has at least one known non-GET `RouteRef` step (concrete path, or templated — templated journeys still count but need path binding before live smoke).

**Hard-excluded** from the gate (not a coverage failure): `JourneyId::ShutdownOnboardComputer` only (power-off with no automated wake). Reboot, bag overwrite, and firmware journeys remain eligible.

**100%** means every eligible journey has a `MutatingSmokeEntry` in `MUTATING_SMOKE_ENTRIES` (`catalog/src/mutating_smoke.rs`). Report:

```bash
cargo run -q --bin tier2_coverage
```

Fields: `eligible_count`, `allowlisted_count`, `missing` (eligible but not allowlisted), `excluded` (shutdown). The test `tier2_mutating_coverage_is_complete` asserts `missing.is_empty()` — every eligible journey must have a `MutatingSmokeEntry` with `setup`, `restore`, and `notes`.

## Rule

Every `--mutating-smoke` allowlist entry must name how state returns to a known-good baseline after the mutate step(s). Prefer the lightest restore that works.

**Preconditions are setup, not only skip gates.** The same repair toolkit may **provision** missing preconditions before a journey runs (install an extension, seed a recording, enable pirate/advanced, select a board, pull a local image, start a sock/stream, etc.). Skip only when provisioning is impossible or out of scope for that entry. Typed `FixtureInventory` remains the declaration of “what this run provides”; runners may populate it by repairing the Pi rather than requiring the human to have pre-staged everything.

## Allowed restore / setup mechanisms (preferred order)

1. **HTTP round-trip** — GET snapshot → mutate → PUT/POST restore; or create → destroy pair (theme, branding, NMEA sock, manifest).
2. **Filesystem** — download/copy config from container or host, mutate via API, re-upload/replace files under `/root/.config/blueos`, `/usr/blueos/userdata`, etc.
3. **Service tmux restart** — attach service session, Ctrl-C, Up, Enter (reload process without full container bounce).
4. **Container restart** — `docker restart blueos-core` (or equivalent); wait for nginx + health.
5. **Host reboot** — reboot Pi; wait until BlueOS HTTP is back; resume smoke.
6. **External dependency proxy** — cache/serve Docker Hub and similar so version/extension pulls do not require live upstream. When provisioning git-based sources on the Pi/container, prefer **`git clone`** (fresh tree) over **`git fetch`** into an existing checkout — cleaner, reproducible, avoids dirty/partial fetch state.

## Hard exclude

- **Shutdown / power-off** with no automated wake path.

## Allowlist entry shape

`MutatingSmokeEntry` in `catalog/src/mutating_smoke.rs`:

```rust
pub struct MutatingSmokeEntry {
    pub journey_id: JourneyId,
    pub setup: SmokeRepair,      // may provision preconditions
    pub restore: SmokeRepair,
    pub notes: &'static str,
}
```

`SmokeRepair` variants: `None`, `HttpRoundTrip`, `FilesystemReplace`, `TmuxServiceRestart`, `ContainerRestart`, `HostReboot`, `ExternalProxy`, `ManualDocumented`.

Live runner restore automation is incremental — entries declare the target repair; `journey_http --mutating-smoke` still exercises only the current allowlist.

## Provisioning progress (2026-07-18)

HttpRoundTrip setup now clears these former live skips: `remove_configured_nmea_socket`, `remove_serial_bridge`, `delete_3d_model_override` (path bind `/models/{name}` → `smoke-catalog.glb`), `disable_onboard_dhcp_server`. Cable_guy teardowns: delete smoke static IP `192.168.0.178`, `DELETE /dhcp` after enable/disable, restore host DNS.

**Firmware + extensions (policy update):** flashing, install/uninstall/edit/restart/disable are allowlisted on the play Pi. Fixture `smoke-firmware.bin` (Navigator ArduSub). Store smoke uses `williangalvani.example1`; lifecycle restart/disable uses `blueos.major_tom`; custom install uses `alpine` as `smoke.catalog`.

**Core image switch (policy update):** `switch_local_blueos_version` is allowlisted. Setup `docker tag`s `bluerobotics/blueos-core:master` → `:smoke-catalog-switch` (same image, different tag), POST `/version/current` to the duplicate, wait for recovery, then restore to `:master`. Does not change the running image content—only exercises the switch path. `UpdateBlueosVersion` POST `/version/current` and bootstrap image replace remain deferred.

**mDNS hostname:** allowlisted; POST `hostname=smoke-catalog`, restore `hostname=blueos`.

**Wi-Fi connect (deferred — harness plan):** full suite will run from a Raspberry Pi 3 (tester/harness) against a Pi 4 (DUT), CI-triggered. Plan:
1. Pi3 creates a controlled hotspot; Pi4 (BlueOS) connects to it (`connect_to_wifi_network`).
2. Pi3 joins BlueOS emergency/smart hotspot (inverse direction).
Until that harness exists, `connect_to_wifi_network` stays skipped (fake SSID association hangs).

Remaining deferred: bootstrap image replace, UpdateBlueosVersion apply-switch, wifi connect (harness above).

## Not a RouteRef commit gate

Tier-1 GET `--smoke` remains the blocking gate. Tier-2 stays optional deeper coverage.
