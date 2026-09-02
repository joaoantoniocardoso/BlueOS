# Cross-check: catalog journey presence (1.4.5 since 1.4.3)

Date: 2026-08-25
Repo: BlueOS-docker-2.0-model
Method: git merge-base --is-ancestor INTRO TAG; git cat-file -e TAG:PATH
Rules: catalog/crates/catalog-git/src/feature_presence.rs (ancestor OR path)

## Verdict

Journey presence: PASS

All sampled journeys match presence.json claims. Git evidence confirms the single
added journey (level_horizon). No mismatches in samples.

## A) Added journey(s) in presence.json "added"

### level_horizon

- intro_commit: 06490f90ed01f7038c693549b6dd15d8bf4b8ca3
- source_path: core/frontend/src/components/vehiclesetup/configuration/compass/LevelHorizonCalibration.vue
- git log -1: 06490f90e Frontend: create LevelHorizonCalibration

| Check | 1.4.3 | 1.4.5 |
|-------|-------|-------|
| merge-base --is-ancestor INTRO TAG | exit=1 (not ancestor) | exit=1 (not ancestor) |
| cat-file -e TAG:PATH | exit=128 (missing) | exit=0 (exists) |

presence.json claims:
- present_1_4_3=false, why_1_4_3=absent  -> MATCH
- present_1_4_5=true, why_1_4_5=path     -> MATCH (path-only; intro not in 1.4.5 history)
- present_1_4_4=true (path exists on 1.4.4, exit=0) -> MATCH

Note: Cherry-pick/backport landed the file without intro_commit as ancestor of tag SHAs.

## B) Sample re-checks (3 present both, 3 absent on 1.4.5)

### Present on both 1.4.3 and 1.4.5

| journey | ancestor 1.4.3 | ancestor 1.4.5 | path 1.4.3 | path 1.4.5 | presence.json |
|---------|----------------|----------------|------------|------------|---------------|
| access_blueos_web_interface | 0 | 0 | 0 | 0 | true/true, ancestor+path |
| connect_to_wifi_network | 0 | 0 | 0 | 0 | true/true, ancestor+path |
| calibrate_compass | 0 | 0 | 0 | 0 | true/true, ancestor+path |

All MATCH.

### Absent on 1.4.5

| journey | ancestor 1.4.3 | ancestor 1.4.5 | path 1.4.3 | path 1.4.5 | presence.json |
|---------|----------------|----------------|------------|------------|---------------|
| browse_video_recordings | 1 | 1 | 128 | 128 | false/false, absent |
| change_ui_theme_color | 1 | 1 | 128 | 128 | false/false, absent |
| inspect_zenoh_network | 1 | 1 | 128 | 128 | false/false, absent |

All MATCH. Mismatches: none.

## C) Capabilities / features vs journeys

### Does catalog track capability intro tags separately from journeys?

No. Independent per-capability tag presence does not exist.

Evidence:
- catalog/feature_traces.json schema: top keys include journeys, commits, PRs;
  journey entries carry intro_commit, present_in_tags, first_tag. No capabilities array.
- catalog/feature_presence_map.json: journeys only (version_index keyed by tag).
- catalog-derive availability_for_capability() (requirement.rs:1504) merges
  availability from journeys that reference the capability. Capabilities inherit
  journey presence; they are not tagged independently.
- DeclaredCapability (catalog-derive/feature.rs) has id, aggregate, origin, rationale
  only; no intro_commit or present_in_tags fields.

### Extra non-journey features on 1.4.5 not on 1.4.3?

No (for catalog-tracked features).

- Only added journey: level_horizon (maps to CapabilityId::LevelHorizon in
  capability_registry.rs, owner VehicleSetup page).
- LevelHorizon capability presence is fully explained by the added journey.
- feature_presence_map.json is stale w.r.t. tag 1.4.5 (1.4.5 absent from
  version_index; generated before 1.4.5 release). extras/presence.json uses
  live git checks via compute_presence.py and is authoritative for this cross-check.

## D) 1.4.4 vs 1.4.4-beta.23 same commit

```
git rev-parse 1.4.4 1.4.4-beta.23
168e82a7a71212b674a7576d854a88b363cd7879
168e82a7a71212b674a7576d854a88b363cd7879
```

CONFIRMED: identical SHA. Matches presence.json tag_shas and sets_equal=true.

## Summary counts (presence.json)

- total journeys: 100
- present 1.4.3: 80
- present 1.4.5: 81
- added: 1 (level_horizon)
- removed: 0
