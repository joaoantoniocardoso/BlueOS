# Phase 2 — Precision design freeze

Source: `PRECISION_INVENTORY.md`. Frozen for Phase 3 implementation.

## 1. Hint source of truth

`feature_presence.rs` stays the single table of truth: `OVERRIDES` (journey-level)
and `MODULE_DEFAULT_PATH`/`MODULE_S` (module-level). `feature_trace_enrich.rs`
only *reads* these tables — never duplicates hint data.

Precedence (highest wins), matching "presence resolves intro commit the same
way traces resolve discovery paths":

1. **Journey-specific** — any `OVERRIDES` row(s) keyed by the journey id
   (`Override::Path` and/or `Override::Pickaxe`'s path). If present, these are
   the discovery paths — full stop. Broad module-token candidate matching is
   *not* consulted, so a wrong incidental match (e.g. `store/mavlink.ts` for a
   camera journey) can never outrank an explicit override.
2. **Module default** — `MODULE_DEFAULT_PATH` entry for the module, used only
   when no journey-specific row exists.
3. **MODULE_S fallback** — `core/start-blueos-core` + `require_token` gate,
   used only when neither (1) nor (2) exists (safety net for future journeys,
   not the steady state after this design).
4. `nginx.conf` special-case (module `nginx`) is unconditional and additive —
   unaffected by (1)-(3).

## 2. Table shape — reuse, no new table

No `JOURNEY_DISCOVERY_HINTS`. Reuse `OVERRIDES: &[(&str, Override)]` and allow
**multiple rows per journey id** (already legal — the type is a flat slice;
only the lookup was `.find()`-first). Multi-file journeys (e.g.
`ViewSystemInformation`, `ChangeBoard`, `ViewCameraStreams`) get two rows with
the same key instead of a new `Paths([...])` variant.

Concrete enricher API changes (`feature_trace_enrich.rs`):

- **New** `fn journey_override_paths(journey_id: &str) -> Vec<String>` —
  filters `OVERRIDES` for *all* rows matching `journey_id` (both `Path` and
  `Pickaxe`'s path field), replacing the single-match `if let` inside today's
  `module_hint_paths`.
- **`module_hint_paths(journey_id, module)`** keeps its signature, calls
  `journey_override_paths` first (for token widening) then appends
  `MODULE_DEFAULT_PATH`/nginx as today — used for `hint_word_tokens` only.
- **Call site in `enrich_group`** (~line 1323): before calling
  `discovery_paths`, check `journey_override_paths(journey_id)`. If non-empty,
  set `paths = dedup_sorted(widen_with_feature_dirs(journey_override_paths))`
  and skip `discovery_paths` entirely for that journey. Else fall through to
  the existing `discovery_paths(&candidate_paths, Some(module), &hints)` call.
- `resolve_commit` in `feature_presence.rs` is untouched — first-match `.find()`
  stays correct for intro-commit dating (multi-row journeys just mean the
  first declared row is the canonical "first added" file, unchanged behavior
  for existing single-row entries).

## 3. MODULE_S strategy

Add `OVERRIDES` rows (no `MODULE_DEFAULT_PATH` change — these binaries have no
`core/services/<module>` tree, a shared default would recreate the
too-broad problem):

| Journey | Module | New override(s) |
|---|---|---|
| `AccessWebTerminal` | ttyd | `Path("core/frontend/src/views/TerminalView.vue")` |
| `ViewSystemInformation` | linux2rest | `Path(".../views/SystemInformationView.vue")`, `Path(".../store/system-information.ts")` |
| `InspectMavlinkMessagesInBrowser` | mavlink2rest | `Path(".../views/MavlinkInspectorView.vue")`; optional `Pickaxe(<mavlink2rest term>, ".../views/MavlinkInspectorView.vue")` |
| `ViewCameraStreams` | mavlink_camera_manager | `Path(".../store/video.ts")`, `Path(".../components/video-manager/VideoManager.vue")` |
| `ConfigureCameraStream` | mavlink_camera_manager | `Path(".../video-manager/VideoStreamCreationDialog.vue")` |
| `RemoveCameraStream` | mavlink_camera_manager | `Path(".../video-manager/VideoStream.vue")` |
| `ConfigureUvcDeviceControls` | mavlink_camera_manager | `Path(".../video-manager/VideoControlsDialog.vue")` |
| `AccessBlueosWebInterface` | nginx | unchanged (already `core/tools/nginx/nginx.conf` via special-case) |

Effect per §1 precedence: all camera/mavlink2rest/linux2rest/ttyd journeys now
resolve via rule (1), never reaching `core/start-blueos-core` — this is what
fixes the `store/mavlink.ts` mis-resolution (that path only matched because
the generic `mavlink` module token leaked into candidate filtering; rule (1)
bypasses candidate filtering entirely).

`require_token`/`module_s_token` (`feature_trace_enrich.rs`) stay unchanged —
they remain the fallback for any *future* MODULE_S journey added without an
override. Not exercised by the 8 journeys above once overridden.

## 4. Shared-legit (no action, documented only)

These sibling pairs/groups keep one shared hint on purpose — QA must exclude
them from the "must diverge" metric:

- commander: `RebootOnboardComputer` / `ShutdownOnboardComputer`
- versionchooser: `UpdateBlueosVersion` / `SwitchLocalBlueosVersion` / `PullBlueosVersionWithoutSwitch`
- kraken: `InstallExtension` / `UninstallExtension`; `EditExtensionDevVersion` / `ConfigureInstalledExtension`
- ardupilot_manager: `StartAutopilot` / `StopAutopilot` / `RestartAutopilot`

## 5. QA metrics (`PRECISION_QA.md`, Phase 4)

- **Goldens**: unchanged list from `PLAN.md`/`ORCHESTRATOR_PROMPT.xml` — must
  still pass byte-for-byte (or documented-diff) after re-enrich.
- **Hub precision**: for each sampled hub journey (one per cluster in
  inventory), % of `follow_up_prs` whose `files_changed` intersect the
  journey's discovery paths (`discovery_path_covers`) — target ≥90%.
- **Sibling separation**: for each *separable* pair (excludes §4), compute
  `|follow_ups_A ∩ follow_ups_B| / |follow_ups_A ∪ follow_ups_B|` before/after;
  must drop vs. the pre-Phase-3 baseline captured in `PRECISION_INVENTORY.md`.
- **MODULE_S checks**: `AccessWebTerminal.follow_up_prs.len() > 0` if
  `TerminalView.vue` has post-intro history; all 4 camera journeys'
  discovery paths must not contain `store/mavlink.ts`.

## 6. Non-goals

- No `schema_version` bump (stays on current schema).
- No changes to `is_incidental_follow_up` / noise-path filtering
  (`discovery_path_noise_re`) — keep as-is.
- No graph/Graphify tooling.
- No Python — Rust only, per existing toolchain.

## 7. Implementation order

1. `feature_presence.rs`: add the `OVERRIDES` rows from §3 (and the
   inventory's non-MODULE_S rows: helper, commander, wifi, cable_guy, beacon,
   bridget, nmea_injector, ardupilot_manager, frontend_parameters — excluding
   §4 pairs).
2. `feature_trace_enrich.rs`: add `journey_override_paths`, wire the call-site
   precedence change in `enrich_group` (§2).
3. Unit tests: sibling non-pollution (separable pairs' discovery paths are
   disjoint) + MODULE_S camera journeys never yield `store/mavlink.ts`.
4. Run full `enrich` over goldens + sampled hubs.
5. Write `PRECISION_QA.md` (script under `catalog/src/tools/` or manual
   checklist) scoring §5 metrics; loop to Phase 5 on any miss.
