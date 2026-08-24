# Wire surface enumeration (P0a)

Serde wire strings that hit **committed** catalog artifacts. Rust struct/type names
never appear on the wire (C1). This document is the P0b/P0c/P0d input; it does not
rename anything.

Sources walked: `catalog/src/**` (`Deserialize`, `rename_all`, `include_str!`,
`precondition_label`, `requirements_json`, `generate_feature_presence` scraper).

## Committed artifacts (8 files)

| Path | Gate / consumer |
|------|-----------------|
| `catalog/requirements-baselines/1.4-dev.json` | `requirements --diff`, ratchet (1.4-dev tag) |
| `catalog/requirements-baselines/1.4.4-beta.21.json` | `requirements --diff` |
| `catalog/requirements-baselines/1.5.0-beta.40.json` | `requirements --diff` |
| `catalog/api-contracts/baseline.json` | `api_contracts --check` (`gate.sh`) |
| `catalog/extras/qa-harness-improve/ratchet_baseline.json` | `harness_ratchet` (`gate.sh`) |
| `catalog/feature_traces.json` | `include_str!` in `feature_trace.rs` (compile-time parse) |
| `catalog/feature_presence_map.json` | `enrich_feature_traces` input; `journey_presence.rs` source |
| `catalog/e2e/pages.json` | Playwright `page_load.spec.ts` (`frontend_smoke` output) |

`include_str!` in `catalog/src`: only `catalog/feature_traces.json`.

## Artifact top-level schemas

### requirements-baselines/*.json (Serialize + Deserialize)

Produced by `requirements_json()` / loaded by `load_requirements_baseline()`.

| JSON key | Rust type / source |
|----------|-------------------|
| `version_tag` | `RequirementsJson::version_tag` |
| `total` | `RequirementsJson::total` |
| `unknown_statement_count` | `RequirementsJson::unknown_statement_count` |
| `unknown_criteria_count` | `RequirementsJson::unknown_criteria_count` |
| `contamination_findings` | `RequirementsJson::contamination_findings` (count only; `ContaminationSource` enum not serialized here) |
| `by_kind` | `BTreeMap<String, usize>` keys from `format!("{:?}", RequirementKind).to_ascii_lowercase()` |
| `requirements[]` | `Vec<RequirementJsonRow>` |

`RequirementJsonRow` fields (all snake_case field names = Rust field names):

| Field | Wire notes |
|-------|------------|
| `id` | `RequirementId` string; constraint rows embed `precondition_label` (C3) |
| `kind` | `RequirementKind` (`rename_all = "snake_case"`) |
| `domain` | `Domain` (per-variant `#[serde(rename)]` already pinned) |
| `statement` | `RequirementStatement` internally tagged `status` + `rename_all` |
| `criteria` | `RequirementCriteria` internally tagged `status` + `rename_all` |
| `feature_id` | `FeatureId.0` via `as_str()` (snake_case; explicit `CapabilityId` renames) |
| `journey_id` | `JourneyId::to_string()` / `as_str()` (snake_case `#[serde(rename)]` on `JourneyId`) |
| `function_id` | `FunctionId::as_str()` |
| `verifying_journeys` | `Vec<String>` of `JourneyId::as_str()` |
| `availability_present` | `bool` |

`RequirementKind` wire values: `system`, `functional`, `interface`, `performance`,
`robustness`, `constraint`.

`RequirementStatement` / `RequirementCriteria` wire shapes:

```json
{"status": "known", "text": "..."}
{"status": "unknown", "reason": "..."}
{"status": "known", "items": [{"text": "..."}]}
```

`Domain` wire values: `vehicle`, `peripherals`, `onboard_computer`, `network`,
`blueos_platform`, `presentation`.

### api-contracts/baseline.json (Deserialize)

| JSON key | Rust type |
|----------|-----------|
| `routes[]` | `ApiContractSnapshot::routes` -> `ApiRouteContract` |

`ApiRouteContract` fields (no `rename_all`; field names = wire names):
`service`, `method`, `path`, `version`, `status_code`, `response_model`, `file`, `line`.

`ApiCoverageSource` (`journey`, `page`, `slo`, `negative_probe`) is **not** in this
baseline; it only appears in ephemeral `ApiCoverageReport` used by the ratchet.

### extras/qa-harness-improve/ratchet_baseline.json (Deserialize)

Flat object -> `HarnessRatchetCounts` (13 usize fields, snake_case field names):

`unknown_blast_radius`, `unknown_body_kind`, `unprobed_failure_modes`,
`mutating_smoke_missing_effect_read`, `client_orchestrated_missing_ui_plan`,
`open_harness_gap`, `unverified_observed_evidence`, `extractor_coverage_lapses`,
`unknown_requirement_statements`, `unknown_requirement_criteria`,
`requirement_contamination_findings`, `unmapped_api_routes`, `orphan_api_hits`.

### feature_traces.json (Deserialize, include_str!)

| JSON key | Rust type |
|----------|-----------|
| `schema_version` | `FeatureTraces::schema_version` |
| `repo` | `FeatureTraces::repo` |
| `source_presence` | `FeatureTraces::source_presence` |
| `tool` | `FeatureTraces::tool` |
| `generated_at` | `FeatureTraces::generated_at` |
| `commits` | `HashMap<String, TraceCommit>` |
| `pull_requests` | `HashMap<String, TracePullRequest>` |
| `issues` | `HashMap<String, TraceIssue>` |
| `intro_clusters` | `HashMap<String, IntroCluster>` |
| `journeys` | `Vec<TraceJourneyRef>` |

Nested serde types (all `Deserialize` in `feature_trace.rs`):

- `TraceCommit`: `sha`, `short`, `subject`, `author_name`, `author_email`,
  `authored_at`, `body`, `url`, `files_changed`, `files_changed_truncated`
- `TracePullRequest`: `number`, `title`, `url`, `state`, `author`, `merged_at`,
  `closed_at`, `base_ref`, `head_ref`, `labels`, `body`, `body_truncated`,
  `commit_shas`, `commit_headlines`, `files_changed`, `files_changed_truncated`,
  `merge_commit_sha`
- `TraceIssue`: `number`, `title`, `url`, `state`, `author`, `created_at`,
  `closed_at`, `labels`, `missing`
- `IntroCluster`: `intro_commit`, `landing_prs`, `squash_merge`, `merge_method`,
  `intro_sha_in_pr_commits`, `merge_commit_sha`, `by_journey`
- `by_journey` keys: **PascalCase `JourneyId` variant names** (C2), e.g.
  `InspectZenohNetwork` -> `JourneyDiscovery`
- `JourneyDiscovery`: `discovery_paths`, `follow_up_prs`, `backport_prs`, `issues`
- `ClusterIssueRef`: `number`, `sources`
- `IssueSource`: `kind`, `pr`
- `IssueSourceKind` (`rename_all = "snake_case"`): `closing`, `body`, `commit`,
  `timeline`, `search` (committed file uses `closing`, `body`, `timeline`)
- `TraceJourneyRef`: `journey` (**PascalCase**), `module`, `method`, `intro_commit`,
  `first_tag`, `present_on_master`, `present_on_1_4_dev`, `present_in_tags`

### feature_presence_map.json (Serialize only in tools; no `Deserialize` in lib)

| JSON key | Rust source (`tools/feature_presence.rs`) |
|----------|-------------------------------------------|
| `schema_version` | `FeaturePresenceMap::schema_version` |
| `note` | static string |
| `journeys[]` | `JourneyPresence` |
| `version_index` | `BTreeMap<String, Vec<String>>` (tag -> PascalCase journey ids) |

`JourneyPresence` fields: `journey` (**PascalCase**, scraped from
`id: JourneyId::Variant` via `id:\s*JourneyId::(\w+)`), `module`, `intro_commit`,
`intro_commit_short`, `method`, `present_in_tags`, `present_on_master`,
`present_on_1_4_dev`, `first_tag`, `tag_count`.

### e2e/pages.json (Serialize only)

`Vec<FrontendSmokeTarget>`: `name`, `page_id`, `path` (+ optional `landmarks`,
`skip_reason` omitted when empty).

`page_id` values come from `PageId::as_str()` (explicit `#[serde(rename)]` per
variant, e.g. `main`, `autopilot`, `zenoh_inspector`).

## Rust type -> wire strings -> artifact

| Rust type | JSON keys / variant strings | Artifact(s) |
|-----------|----------------------------|-------------|
| `RequirementsJson` | top-level keys above | requirements-baselines/*.json |
| `RequirementJsonRow` | row fields above | requirements-baselines/*.json |
| `RequirementKind` | `system`, `functional`, `interface`, `performance`, `robustness`, `constraint` | requirements-baselines/*.json (`kind`, `by_kind` keys) |
| `RequirementStatement` | `status`: `known`/`unknown`; `text`/`reason` | requirements-baselines/*.json |
| `RequirementCriteria` | `status`: `known`/`unknown`; `items`/`reason`; `AcceptanceCriterion.text` | requirements-baselines/*.json |
| `Domain` | `vehicle`, `peripherals`, ... (already `#[serde(rename)]`) | requirements-baselines/*.json |
| `JourneyId` | snake_case in requirements (`vehicle_first_boot`, ...); **PascalCase** in traces/presence | requirements-baselines; feature_traces; feature_presence_map |
| `FeatureId` / `CapabilityId` | snake_case capability strings | requirements-baselines (`feature_id`) |
| `FunctionId` | snake_case + optional hash suffix | requirements-baselines (`function_id`) |
| `precondition_label` output | embedded in `id` for `kind=constraint` (C3) | requirements-baselines/*.json |
| `ApiContractSnapshot` | `routes` | api-contracts/baseline.json |
| `ApiRouteContract` | route object fields | api-contracts/baseline.json |
| `HarnessRatchetCounts` | 13 counter field names | ratchet_baseline.json |
| `FeatureTraces` + 9 nested types | see feature_traces section | feature_traces.json |
| `IssueSourceKind` | `closing`, `body`, `commit`, `timeline`, `search` | feature_traces.json |
| `JourneyPresence` / `FeaturePresenceMap` | presence fields; PascalCase `journey` | feature_presence_map.json |
| `FrontendSmokeTarget` | `name`, `page_id`, `path` | e2e/pages.json |
| `PageId` | per-variant rename strings | e2e/pages.json (`page_id`) |

### Secondary (Serialize into regenerated output, not in table artifacts)

`ContaminationSource` (`feature`/`journey`/`overlay` tags), `ApiCoverageSource`,
`FeatureAvailability`, journey precondition enums, provenance wrappers, export
schema types: ~90 additional `rename_all` types in `catalog/src` with no committed
JSON consumer in the P0 walk set.

## P0b: types that MUST get explicit `#[serde(rename)]`

`rename_all` enums whose variant strings appear in committed artifacts (Domain
already pinned):

1. `RequirementKind` (`requirement.rs`)
2. `RequirementStatement` (`requirement.rs`) -- variant renames on `known`/`unknown`
3. `RequirementCriteria` (`requirement.rs`) -- variant renames on `known`/`unknown`
4. `IssueSourceKind` (`feature_trace.rs`)

All **20** `Deserialize` types reachable from committed artifacts must keep field
names stable (P0b pins enum variants; struct fields already match snake_case JSON):

`RequirementsJson`, `RequirementJsonRow`, `RequirementKind`, `RequirementStatement`,
`AcceptanceCriterion`, `RequirementCriteria`, `Domain`, `HarnessRatchetCounts`,
`ApiContractSnapshot`, `ApiRouteContract`, `FeatureTraces`, `TraceCommit`,
`TracePullRequest`, `TraceIssue`, `IssueSourceKind`, `IssueSource`, `ClusterIssueRef`,
`JourneyDiscovery`, `IntroCluster`, `TraceJourneyRef`.

Types with explicit per-variant rename already (P0b no-op for variants):
`Domain`, `JourneyId`, `ServiceId`, `CapabilityId`, `PageId`, `Aggregate`.

## C3: `precondition_label` / `{:?}` requirement IDs

**Confirmed.** `coverage.rs::precondition_label` formats several `Precondition`
arms with `{:?}` on Rust enums. Those strings are sanitized into constraint
requirement primary keys via `constraint_requirement_id()` -> `REQ/.../CON/<label>`.

36 constraint IDs in `1.4-dev` contain `::` (all 36 constraints).

| `precondition_label` arm | Example ID fragment |
|--------------------------|---------------------|
| `Data(data)` -> `Data::{data:?}` | `REQ/blueos_platform/extensions/CON/Data::ExtensionInstalled` |
| `Network(n)` -> `Network::{n:?}` | `REQ/vehicle/autopilot/CON/Network::Online` |
| `Hardware(h)` -> `Hardware::{h:?}` | `REQ/peripherals/gps_nmea/CON/Hardware::ExternalNmeaGps` |
| `Software(s)` -> `Software::{s:?}` | `REQ/.../CON/Software::PirateMode` |
| `NetworkResource(r)` -> `NetworkResource::{r:?}` | `REQ/.../CON/NetworkResource::HotspotCapable` |
| `Other(label)` | `REQ/.../CON/Other::BlueOS is newly installed...` |
| `HardwarePresent(label)` | `REQ/.../CON/HardwarePresent::Ping family sonar device` |
| `ServiceState` | `ServiceState::<service.as_str>(<state>)` (not Debug) |
| `ConfigClean` | `ConfigClean::<path>` (not Debug) |

P0c replaces `{:?}` with explicit `as_str()` / labels so IDs stay byte-identical
after P0b enum pinning.

## C2: PascalCase `JourneyId` keys

**Confirmed.** `JourneyId` uses snake_case `#[serde(rename)]` for requirements and
catalog export, but feature provenance uses **PascalCase variant names** scraped
from journey source (`tools/feature_presence.rs`: `id:\s*JourneyId::(\w+)`).

| Location | Example key / value |
|----------|---------------------|
| `feature_traces.json` `journeys[].journey` | `InspectZenohNetwork` |
| `feature_traces.json` `intro_clusters.*.by_journey` | `ConnectToWifiNetwork` |
| `feature_presence_map.json` `journeys[].journey` | `AccessBlueosWebInterface` |
| `feature_presence_map.json` `version_index.*[]` | PascalCase journey strings |
| `feature_trace.rs` tests | hardcoded `"InspectZenohNetwork"`, etc. |

Requirements baselines use snake_case journey references (`vehicle_first_boot`), not
PascalCase.

## Byte-identical after P0b/P0c (MUST NOT change unintentionally)

- All three `catalog/requirements-baselines/*.json` files (gate: `requirements --diff`)
- `catalog/api-contracts/baseline.json` (`api_contracts --check`)
- `catalog/extras/qa-harness-improve/ratchet_baseline.json` (`harness_ratchet`)
- `catalog/e2e/pages.json` (unless page set/routes change functionally)

`feature_traces.json` and `feature_presence_map.json` stay byte-identical through
P0b/P0c; P0d is the only phase that intentionally re-keys them.

## Allowed to change in P0d

- `feature_traces.json`: re-key `journeys[].journey` and `intro_clusters.*.by_journey`
  from PascalCase variant names to `JourneyId` rename strings (snake_case).
- `feature_presence_map.json`: same for `journeys[].journey` and `version_index` values.
- `tools/feature_presence.rs` scraper and `feature_trace.rs` test literals that
  hardcode PascalCase journey strings.
- Review line-by-line; no other committed artifact may drift in P0d.

## Not in P0 wire surface

- `catalog/src/journey_matrix.rs` `RawReport`/`RawJourney` (`Deserialize` from QA
  report JSON under `extras/qa-*`, not committed baselines).
- `catalog/src/runner.rs` `VersionCurrentJson` (live DUT HTTP, not committed).
- `catalog/src/bin/boot_timeline.rs` types (boot timeline summaries, separate path).
