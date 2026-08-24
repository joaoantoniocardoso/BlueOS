# SysML v2 ontology alignment -- plan

Mapping reference: `MAPPING.md` (every row cited to the pinned spec)
Spec: `catalog/spec/sysml-v2-release` submodule @ tag **2026-05** (`de1070ae`)
Baselines: `gate.sh` green; 478 tests; 388 requirements on 1.4-dev

## North star

Reverse engineering stays the job. The catalog keeps deriving the model from
`core/**` and re-proving it against DUTs; Rust stays the source of truth. This
rework only changes *vocabulary and shape* so that:

1. the model reads correctly to anyone fluent in SysML v2 / ISO 19514-1,
2. a `.sysml` exporter becomes a mechanical translation rather than a design exercise,
3. switching source of truth to SysON for BlueOS 2.0 stays a live option, not a rewrite.

**Hard invariant, every phase:** the extract -> verify -> gate loop stays green.
`gate.sh` is the thing SysML v2 cannot replace, so no phase may weaken it. Any
intentional change to a committed baseline is a named deliverable with a recorded
diff, never incidental fallout.

## Discovered constraints (these drive the ordering)

| # | Constraint | Consequence |
|---|---|---|
| C1 | serde never emits a struct's Rust type name | Type renames are JSON-free. `ObservedFacts`, `ServiceDefinition`, `UserJourney`, `Edge`, `Catalog` etc. cost nothing in baselines. |
| C2 | `feature_traces.json` (4.3 MB) is `include_str!`'d and keyed on PascalCase `JourneyId` variant names, with those literals hardcoded in tests | A `JourneyId` variant rename is a **build failure**, not a diff. Variants are frozen. |
| C3 | `precondition_label` formats with `{:?}` | Rust variant names are baked into requirement **primary keys** in both `requirements-baselines/*.json` (`.../CON/Data::ExtensionInstalled`). |
| C4 | **110** types crate-wide use `rename_all` rather than per-variant `rename` | Their JSON strings track Rust spelling. Only the subset reachable from a committed artifact matters; the rest serialize into output that is regenerated every run. Known-reachable: `RequirementKind`, `RequirementStatement`, `RequirementCriteria`, `ContaminationSource` (requirements baselines), `ApiCoverageSource` (api-contracts), `IssueSourceKind` (feature_traces). P0 starts by enumerating this set exactly. |
| C5 | **25** types derive `Deserialize`, across 6 files (`feature_trace.rs` 10, `api_contract.rs` 7, `requirement.rs` 4, `requirements_report.rs` 2, `harness_ratchet.rs` 1, `domain.rs` 1) | For these, a **field** rename is a parse failure, not a diff. Highest risk: `FeatureTraces` and its 9 companions (`include_str!`'d, so failures are compile-time), `RequirementJsonRow`, `RequirementsJson`, `HarnessRatchetCounts`, `ApiRouteContract`. |
| C6 | `ServiceId` / `CapabilityId` / `PageId` / `Aggregate` / `Domain` carry explicit per-variant `#[serde(rename)]` | Variant renames are JSON-safe as long as the rename strings are untouched. Deliberate; documented at `src/id.rs:11-13`. |
| C7 | `catalog/observed/` contains only `.gitkeep` | `blueos-catalog-two-layer.mdc:16` describes it as a generated export. Doc bug to fix in Phase 4. |

C1 plus C3/C4 give the ordering: **freeze the wire format first, then rename freely.**

## Rename tiers

**Tier A -- real collisions with KerML/SysML v2.** These make an export misleading.
In scope.

| Current | New | Hits / files |
|---|---|---|
| `Feature`, `FeatureId`, `FeatureCatalog` | deleted; collapse into `CapabilityId` / `CapabilityDef` | 18/10, 65/4, 36/7 |
| `FeatureLens`, `FeatureCommunity`, `FeaturePairAgreement`, `FeatureSplitConsensus`, `JourneyView` | `CapabilityLens`, `CapabilityCommunity`, `CapabilityPairAgreement`, `CapabilitySplitConsensus`, `CapabilityJourneyView` | 7/2, 7/2, 4/2, 4/2 |
| `FeatureAvailability` | `Availability` | 274/17 (largest in tier) |
| `FeatureTraces` | `IntroTraces` | 14/3 |
| `Port(pub u16)` | `TcpPort` | 9/4 |
| `Function`, `FunctionId`, `FunctionIo`, `FunctionCatalog`, `FUNCTION_COUNT` | `Action`, `ActionId`, `ActionParameter`, `ActionCatalog`, `ACTION_COUNT` | 9/3, 12/3, 10/2, 26/5 |
| `RequirementKind` | `RequirementClass` | 43/4 |
| `RequirementKind::Constraint` | removed; reclassified (Phase 3) | |
| `ServiceDefinition` | `ServiceJudgment` | 98/33 |
| `Interface` (enum in `src/interface.rs`) | `PortKind` | 239/33 (includes `RequirementKind::Interface`, count separately) |
| `HardwareRequirement`, `SoftwareRequirement`, `DataRequirement` | `HardwareAssumption`, `SoftwareAssumption`, `DataAssumption` | |
| `Requirement.functional_children` | `subrequirements` | library's own name |
| `Requirement.verifying_journeys` | `requirement_verifications` | library's own name |
| `Automatable` | `VerificationMethod` w/ `{ Inspect, Analyze, Demo, Test }` | standard enum |
| runner step result, `journey_matrix::CellState` | `Verdict { Pass, Fail, Inconclusive, Error }` | standard enum |

**Tier B -- non-standard but not colliding.** Deferred by default; `minimal-diff`
says do not churn these without a functional reason.

| Current | Would become | Hits / files | Recommendation |
|---|---|---|---|
| `UserJourney` | `UseCase` | 366/50 | Do it: cheap for the clarity, and it is the type the exporter emits as `use case def`. |
| `JourneyId` | `UseCaseId` | 1277/81 | **Skip.** No collision, variants are frozen by C2, and "journey id" reads fine. Accept the mixed naming; `MAPPING.md` records the correspondence. |
| `Edge` | `Connection` | 27/14 | Do it: 14 files, and `connect` is the export target. |
| `StateMachine` | `StateDef` | 11/5 | Skip. `state def` is the export target; the Rust name is already unambiguous. |
| `Aggregate` | `Subsystem` | 248/11 | Skip. It stays a package label; renaming buys nothing. |

## Phase status

- [x] **P0** Freeze the wire format
- [x] **P1** Collapse Feature into Capability
- [x] **P2** Tier A renames
- [x] **P3** Structural corrections
- [ ] **P4** Docs (in progress)
- [ ] **P5** Exporter

## Phases

| Phase | Work | Deliverable | QA gate |
|---|---|---|---|
| **P0** Freeze the wire format | **P0a:** enumerate exactly which of the 110 `rename_all` types (C4) and 25 `Deserialize` types (C5) are reachable from a committed artifact -- walk out from `requirements-baselines/*.json`, `api-contracts/baseline.json`, `ratchet_baseline.json`, `feature_traces.json`, `feature_presence_map.json`, `e2e/pages.json`. Record the set in `WIRE_SURFACE.md`. **P0b:** pin that set with explicit per-variant `#[serde(rename)]`. **P0c:** replace `{:?}` in `precondition_label` with explicit `as_str()` (C3). **P0d:** re-key `feature_traces.json` and `feature_presence_map.json` on rename strings instead of PascalCase variants; update the `id:\s*JourneyId::(\w+)` scraper and the hardcoded PascalCase test literals (C2). | `WIRE_SURFACE.md` plus a snapshot of every generated artifact, committed as the pre-rework reference | `gate.sh` green. Both requirements baselines **byte-identical** after P0b/P0c. The P0d re-key is the only intended diff, reviewed line by line. |
| **P1** Collapse Feature into Capability | Delete `Feature`/`FeatureId`/`FeatureCatalog`. `CapabilityId` becomes the requirement subject. `Feature.rationale` -> stays asserted on the owning service/page card (it is already sourced from there). `Feature.aggregate`/`origin` -> already in `CapabilityDef`/`FrontendCapabilityDef`. Rewrite the `REQ/{domain}/{aggregate}/SYS/{suffix}` ID grammar. Keep the clustering analysis, renamed per Tier A. | Regenerated `requirements-baselines/*.json` with a recorded `requirements --diff` | `gate.sh` green. Requirement count unchanged (388) except for documented ID-string changes. Diff reviewed and archived in this directory. |
| **P2** Tier A renames | One commit per module per `commit-granularity`. Mechanical; no behaviour change. Adopt `Verdict` and `VerificationMethod`. | | `gate.sh` green after **every** commit. All baselines byte-identical (P0 guarantees this). |
| **P3** Structural corrections | (a) Reclassify `RequirementKind::Constraint`: the 36 constraint requirements become `assume constraint` members of the requirements they gate. (b) Split use case from verification case: `UseCase` is the use case; report JSON becomes `VerificationRecord` rows carrying a `Verdict`. (c) Resolve `Action.children` (declared, never populated) and `ActionParameter` (always `Unknown`) -- populate or delete. (d) Decide `Availability` representation: `variation`/`variant` vs metadata. | Regenerated baselines with recorded diffs; a short rationale note per change | `gate.sh` green. Requirement total drops by the 36 reclassified constraints, and that number is asserted in a test so it cannot drift silently. |
| **P4** Docs | Rewrite the 5 `.mdc` rules that name renamed types: `blueos-catalog-two-layer`, `-frontend`, `-journeys`, `-runtime`, `-rust`. Fix the `catalog/observed/**` claim (C7). Add a rule pointing at `MAPPING.md` as the naming authority. Check `AGENTS.md` (current hits are incidental prose only). | | Every identifier named in a rule file resolves in the tree. |
| **P5** Exporter | `src/export_sysml.rs` + `src/bin/sysml_export.rs`, emitting `.sysml` textual notation. Imports `Requirements::*`, `VerificationCases::*`, `ModelingMetadata::*` from the submodule library. Local `metadata def Evidence` / `RuntimeCapture`. Golden-file gate in `gate.sh`, matching the `api_contracts --check` pattern. | Committed golden `.sysml` output | `gate.sh` green including the new golden check. Manual (not in `gate.sh`): emitted `.sysml` parses under the JVM pilot implementation, mirroring how `--strict-goldens` stays opt-out. |

Max 30 iterations. No commits or PRs unless asked.

## Notes

**Why deletion precedes renaming.** P1 removes code that P2 would otherwise have to
rename, and `commit-granularity` forbids editing the same line twice across a series.

**Why P0 cannot be skipped.** Without it, every Tier A rename drags baseline churn
along with it, and a real regression would be indistinguishable from rename noise in
a 400 KB JSON diff. After P0, "all baselines byte-identical" is a usable P2 gate.

**Submodule cost.** Pinned shallow + sparse (`bnf`, `sysml.library`) at 5.5 MB
against a 1.9 GB upstream. `shallow = true` is set in `.gitmodules`; the sparse
config is per-clone, so the lean init is:

```sh
git submodule update --init catalog/spec/sysml-v2-release
git -C catalog/spec/sysml-v2-release sparse-checkout set bnf sysml.library
```

Bumping the pin means re-verifying the citations in `MAPPING.md`; the KerML 1.1 /
SysML 2.1 RTFs are active, so keyword and library changes are expected over time.

## Open decisions

| # | Decision | Recommendation |
|---|---|---|
| D1 | `ServiceDefinition` -> `ServiceJudgment`? The layers would read `ObservedFacts` / `ServiceJudgment` / `RuntimeFacts`. | Take it. Alternative `AssertedFacts` is symmetric but "facts" misdescribes judgment. |
| D2 | Tier B: do `UserJourney` -> `UseCase` and `Edge` -> `Connection`, skip the rest? | Yes to both, per the table. |
| D3 | P3(d): `Availability` as SysML `variation`/`variant`, or as plain metadata? | Defer to P3 once the exporter shape is concrete. Variability is the semantically right answer; metadata is the cheap one. |
| D4 | Does P3(a) reclassification need a preserved view of the 36 constraint requirements for RTM continuity? | Decide before P3 lands; the RTM is a published artifact shape. |
