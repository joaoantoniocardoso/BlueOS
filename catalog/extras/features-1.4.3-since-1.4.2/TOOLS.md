# BlueOS 1.4.3 since 1.4.2 -- shipped binaries

## Scope

Tags: 1.4.2 (`0a0043f3f`) -> 1.4.3 (`c38aa1eef`). Binary API, CLI, and protocol
are the product surface. `blueos_ui: true` means Vue/nginx also wires the
capability (may also appear in INVENTORY.md / BEHAVIOR.md).

**Status** (see INVENTORY.md legend): on | opt-out (pirate) | pirate-only |
no UI | not launched | off.

## Pin table

| binary | pin 1.4.2 | pin 1.4.3 | delta |
|--------|-----------|-----------|-------|
| mavlink-camera-manager | t3.19.2 | t3.19.2 | unchanged |
| mavlink-server | 0.3.1 | 0.3.1 | unchanged |
| linux2rest | v0.6.2 | v0.6.2 | unchanged |
| zenoh | 1.0.0 | 1.0.0 | unchanged |
| mavlink2rest | t0.11.23 | t0.11.23 | unchanged |
| filebrowser | v2.30.0 | v2.30.0 | unchanged |
| ttyd | 1.6.3 | 1.6.3 | unchanged |
| mavp2p | v1.1.1 | v1.1.1 | unchanged |
| mavlink-router | v4 | v4 | unchanged |
| bridges | 0.10.3 | 0.10.3 | unchanged |
| machineid | 0.2.3 | 0.2.3 | unchanged |
| logviewer | v1.0.1 | v1.0.1 | unchanged |

## New functions

None. No binary pin change; no new binary functions.

## Unchanged product facts (same as 1.4.2)

- zenohd is launched (`ZENOH_BACKEND_FS_ROOT=$TOOLS_PATH/zenoh zenohd -c $TOOLS_PATH/zenoh/blueos-zenoh.json5`).
- MCM t3.19.2 has no `--zenoh`, `--recorder`, or `--enable-realtime-threads` clap; start-blueos-core does not pass them. MCM Zenoh video is not launched (and cannot be at this pin). Not a new function.
- mavlink-server 0.3.1 is pinned but is not in start-blueos-core; mavlink2rest (t0.11.23) is. Not a new function.

## Unchanged binaries

All 12 pins identical. start-blueos-core and core/tools/*/bootstrap.sh have empty git diff.

## Firmware

No firmware pin change in this window.
