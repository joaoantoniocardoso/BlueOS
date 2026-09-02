# BlueOS 1.4.3 since 1.4.2 -- merged inventory

## Scope

Tags: 1.4.2 (`0a0043f3f`) -> 1.4.3 (`c38aa1eef`). Two commits (CI pimod
pin; ArduPilot-Parameter-Repository gitlink a2cf1db -> cada4b2).
FUNCTION = distinct job absent on 1.4.2.

**Status** (how 1.4.3 ships it):

| status | meaning |
|--------|---------|
| on | Enabled by default; no extra flag |
| opt-out (pirate) | On by default; Pirate Mode Extra configuration can disable |
| pirate-only | BlueOS UI only when Pirate Mode is on |
| no UI | Running or reachable (binary/API/nginx), no Vue page |
| API only | REST exists; no Networking/settings screen |
| not launched | In the binary, BlueOS does not start or configure it |
| off | Explicitly disabled in this release |

## New functions

None. Catalog journeys/functions unchanged (80 journeys, 87 functions on both tags).

### BlueOS

None.

### mavlink-camera-manager (t3.19.2)

None. Pin unchanged.

### mavlink-server (0.3.1)

None. Pin unchanged. Binary is not launched (same as 1.4.2).

### linux2rest (v0.6.2)

None. Pin unchanged.

## Behavioral changes (1)

Grouped before/after list: BEHAVIOR.md and inventory/BEHAVIOR.json.

### vehicle (1)

ardupilot_parameter_metadata.

## Shipped binaries

Pin table and per-binary notes: TOOLS.md and inventory/TOOLS.json.

All 12 binary pins unchanged: MCM t3.19.2, mavlink-server 0.3.1,
linux2rest v0.6.2, zenohd 1.0.0, mavlink2rest t0.11.23, filebrowser
v2.30.0, ttyd 1.6.3, mavp2p v1.1.1, mavlink-router v4, bridges 0.10.3,
machineid 0.2.3, logviewer v1.0.1.

**Unchanged product facts** (same as 1.4.2, not new functions): zenohd
is launched; MCM t3.19.2 has no `--zenoh`/`--recorder`/`--enable-realtime-threads`
clap, so MCM Zenoh video is not launched; mavlink-server 0.3.1 is pinned
but not in start-blueos-core (mavlink2rest is).

## Notable enhancements

None beyond the behavioral change above.

## Not included

**1.5-only catalog functions** (20 absent on 1.4.3): disk usage browser,
video recorder/extractor, zenoh inspector, branding/theme customization,
bootstrap image update, commander reset_blueos_settings, and related journeys.
See FUNCTIONS.md / functions.json.

**Not a drop / not new:** zenohd still runs; MCM Zenoh video cannot run
at this pin (no `--zenoh` clap). Unchanged vs 1.4.2.

**Resolved as non-functions:** ci_pimod_version (CI-only, not_product).

## Disputes resolved

2 items in MERGED.json disputed_resolved: ardupilot_parameter_metadata
(behavioral; same parameter-editor job; 4.6/4.5 metadata only, no 4.7
folders), ci_pimod_version (not_product). No remaining open disputes.
