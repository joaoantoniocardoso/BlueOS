# NEXT11 audit

## N4 — Shared-legit accept-shared policy

**Decision (per `NEXT11_DESIGN.md` §N4):** no classifier. `IMPROVE_AUDIT.md` T3 already proved,
via real `git log -S<term>` pickaxe (not title/body heuristics), that 3 of the 4 investigated
groups are **impossible** to separate — their handlers are always edited together historically.
These 3 groups are formally modeled as pinned `SharedCluster`s: a `sibling_ratio(...) >= 0.99`
test per pair, inverse of the N1/N6 `< 0.40` separation gate. A pin catches accidental future
divergence (e.g. an `Override::Path` added to only one sibling) rather than re-deriving the
classification each time.

### Groups pinned

| Group | Pair(s) | T3 evidence | Measured ratio (current `feature_traces.json`) |
|---|---|---|---|
| Commander reboot/shutdown | `RebootOnboardComputer` / `ShutdownOnboardComputer` | One handler `POST /shutdown` (`core/services/commander/main.py:88-98`) keyed by `ShutdownType` enum; `-S'REBOOT'` vs `-S'POWEROFF'` over `core/services/commander`: 1 commit each, same commit | **1.0000** |
| Versionchooser trio | `UpdateBlueosVersion` / `SwitchLocalBlueosVersion`, `UpdateBlueosVersion` / `PullBlueosVersionWithoutSwitch`, `SwitchLocalBlueosVersion` / `PullBlueosVersionWithoutSwitch` | Distinct routes `set_version`/`pull_version` exist but `-S'set_version'` vs `-S'pull_version'` over `core/services/versionchooser` + `VersionChooser.vue`: 3 commits each, same 3 commits; `UpdateBlueosVersion` is literally the union of the other two | **1.0000** (all 3 pairs) |
| Kraken install/uninstall | `InstallExtension` / `UninstallExtension` | Distinct routes `POST /install`/`POST /uninstall` (`core/services/kraken/api/v1/routers/extension.py:20-29`) but `-S'install_extension'` vs `-S'uninstall_extension'`: 10 commits each, identical set | **1.0000** |

All 5 pairs measured **exactly** 1.0 against the current `feature_traces.json`
(`generated_at 2026-07-19T16:27:31-03:00`) — no threshold relaxation needed; the `>= 0.99` gate in
`feature_trace.rs` has headroom baked in only as a float-equality safety margin, not because any
pair is close to the edge.

### Excluded: `StartAutopilot`/`StopAutopilot`/`RestartAutopilot`

Per design non-goal, this group is **not** in the accept-shared list. T3's group 4 wired a
`Pickaxe` override for these three journeys (`feature_presence.rs` `OVERRIDES`), which measurably
separated their `follow_up_prs` once re-enriched. Re-measured now: `StartAutopilot`/`StopAutopilot`
= **0.5556** (T3 recorded 0.6 pre-full-re-enrich; both well above the old 1.0 pin and below the
`< 0.40` N6 separation gate — it belongs to N6/T3's improved-pickaxe track, not N4's pinned-shared
track). No test added here for N4; asserting `== 1.0` for this pair would be wrong and would mask
the T3 fix if it regressed.

### Tests added (`catalog/src/feature_trace.rs`)

- `commander_reboot_shutdown_is_pinned_shared_cluster` — `sibling_ratio >= 0.99`
- `versionchooser_trio_is_pinned_shared_cluster` — all 3 trio pairs, `sibling_ratio >= 0.99`
- `kraken_install_uninstall_is_pinned_shared_cluster` — `sibling_ratio >= 0.99`

`cargo test -p blueos-catalog --lib` — 172/172 passing (168 baseline + 4 new; the versionchooser
test asserts 3 pairs in one `#[test]`). `cargo fmt --check` clean.
