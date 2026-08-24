# Type inventory vs PLAN.md rename table

Scope: `catalog/src` only. Counts: `rg -w <ident> catalog/src` (hits / files).
Snapshot: 2026-08-23. No renames applied.

Legend: EXISTS = planned current identifier found; SKIP = PLAN defers rename; DELETE = P1 collapse, not rename.

## Tier A -- collisions (in scope)

| Planned current | Actual Rust identifier | File | Hits / files | Exists | Serde | Recommended actual-from -> actual-to |
|---|---|---|---:|---|---|---|
| Feature | `struct Feature` | `feature.rs` | 18 / 10 | yes | `Serialize`, `JsonSchema` | DELETE (collapse into `CapabilityDef`) |
| FeatureId | `struct FeatureId(pub CapabilityId)` | `feature.rs` | 57 / 4 | yes | `serde(transparent)` | DELETE -> use `CapabilityId` |
| FeatureCatalog | `struct FeatureCatalog` | `feature.rs` | 36 / 7 | yes | `Serialize`, `JsonSchema` | DELETE |
| FeatureLens | `struct FeatureLens` | `feature.rs` | 7 / 2 | yes | `Serialize`, `JsonSchema` | `FeatureLens` -> `CapabilityLens` |
| FeatureCommunity | `struct FeatureCommunity` | `feature.rs` | 7 / 2 | yes | `Serialize`, `JsonSchema` | `FeatureCommunity` -> `CapabilityCommunity` |
| FeaturePairAgreement | `struct FeaturePairAgreement` | `feature.rs` | 4 / 2 | yes | `Serialize`, `JsonSchema` | `FeaturePairAgreement` -> `CapabilityPairAgreement` |
| FeatureSplitConsensus | `struct FeatureSplitConsensus` | `feature.rs` | 4 / 2 | yes | `Serialize`, `JsonSchema` | `FeatureSplitConsensus` -> `CapabilitySplitConsensus` |
| JourneyView | `struct JourneyView` | `feature.rs` | 4 / 2 | yes | `Serialize`, `JsonSchema` | `JourneyView` -> `CapabilityJourneyView` |
| FeatureAvailability | `struct FeatureAvailability` | `version.rs` | 165 / 17 | yes | `Serialize`, `JsonSchema` (no `Deserialize`) | `FeatureAvailability` -> `Availability` |
| FeatureTraces | `struct FeatureTraces` | `feature_trace.rs` | 13 / 3 | yes | `Deserialize` only (wire via `feature_traces.json`) | `FeatureTraces` -> `IntroTraces` |
| Port(pub u16) | `struct Port(pub u16)` | `id.rs` | 9 / 4 | yes | `serde(transparent)` | `Port` -> `TcpPort` |
| Function | `struct Function` | `function.rs` | 9 / 3 | yes | `Serialize`, `JsonSchema` | `Function` -> `Action` |
| FunctionId | `struct FunctionId(pub String)` | `function.rs` | 12 / 3 | yes | `serde(transparent)` | `FunctionId` -> `ActionId` |
| FunctionIo | `enum FunctionIo` | `function.rs` | 9 / 2 | yes | `rename_all = "snake_case"` | `FunctionIo` -> `ActionParameter` |
| FunctionCatalog | `struct FunctionCatalog` | `function.rs` | 26 / 5 | yes | `Serialize`, `JsonSchema` | `FunctionCatalog` -> `ActionCatalog` |
| FUNCTION_COUNT | `const FUNCTION_COUNT` | `function.rs` | 5 / 3 | yes | n/a | `FUNCTION_COUNT` -> `ACTION_COUNT` |
| RequirementKind | `enum RequirementKind` | `requirement.rs` | 43 / 4 | yes | `rename_all = "snake_case"`, `Deserialize` | `RequirementKind` -> `RequirementClass` |
| RequirementKind::Constraint | variant `Constraint` | `requirement.rs` | (in enum) | yes | serializes `constraint` | REMOVE variant; reclassify reqs (P3) |
| ServiceDefinition | `struct ServiceDefinition` | `service.rs` | 98 / 33 | yes | `Serialize`, `JsonSchema` | `ServiceDefinition` -> `ServiceJudgment` |
| Interface | `enum Interface` | `interface.rs` | 237 / 33 | yes | `tag = "bus"`, `rename_all = "snake_case"` | `Interface` -> `PortKind` |
| HardwareRequirement | `enum HardwareRequirement` | `journey.rs` | 33 / 9 | yes | `rename_all = "snake_case"` | `HardwareRequirement` -> `HardwareAssumption` |
| SoftwareRequirement | `enum SoftwareRequirement` | `journey.rs` | 56 / 15 | yes | `rename_all = "snake_case"` | `SoftwareRequirement` -> `SoftwareAssumption` |
| DataRequirement | `enum DataRequirement` | `journey.rs` | 44 / 11 | yes | `rename_all = "snake_case"` | `DataRequirement` -> `DataAssumption` |
| Requirement.functional_children | field on `Requirement` | `requirement.rs` | 37 / 4 | yes | n/a (struct not `Deserialize`) | `functional_children` -> `subrequirements` |
| Requirement.verifying_journeys | field on `Requirement` | `requirement.rs` | 37 / 4 | yes | n/a | `verifying_journeys` -> `requirement_verifications` |
| Automatable | `enum Automatable` | `journey.rs` | 39 / 7 | yes | `rename_all = "snake_case"` | `Automatable` -> `VerificationMethod` |
| StepResult | `enum StepResult` | `runner.rs` | 129 / 4 | yes | none | fold into `Verdict` |
| CellState | `enum CellState` | `journey_matrix.rs` | 37 / 2 | yes | none | fold into `Verdict` |

Note: `RequirementKind::Interface` (variant, serializes `interface`) is separate from `interface::Interface`; 237 hits include both contexts.

PLAN hit drift (actual vs PLAN table): FeatureId 57 vs 65; FeatureAvailability 165 vs 274; FeatureTraces 13 vs 14; FunctionIo 9 vs 10; Interface 237 vs 239.

## Tier B -- non-colliding (deferred by default)

| Planned current | Actual Rust identifier | File | Hits / files | Exists | Serde | PLAN | Recommended actual-from -> actual-to |
|---|---|---|---:|---|---|---|---|
| UserJourney | `struct UserJourney` | `journey.rs` | 294 / 50 | yes | `Serialize`, `JsonSchema` | Do it | `UserJourney` -> `UseCase` |
| JourneyId | `enum JourneyId` | `id.rs` | 1253 / 81 | yes | per-variant `serde(rename)` | SKIP (C2) | keep `JourneyId` |
| Edge | `struct Edge` | `edge.rs` | 27 / 14 | yes | `Serialize`, `JsonSchema` | Do it | `Edge` -> `Connection` |
| StateMachine | `struct StateMachine` | `state.rs` | 11 / 5 | yes | `Serialize`, `JsonSchema` | SKIP | keep `StateMachine` |
| Aggregate | `enum Aggregate` | `capability.rs` | 248 / 11 | yes | per-variant `serde(rename)` | SKIP | keep `Aggregate` |

PLAN hit drift: UserJourney 294 vs 366; JourneyId 1253 vs 1277.

## Omitted types -- SysML v2 / KerML collision suspects

Types the plan does not rename but share KerML/SysML vocabulary.

| Identifier | File | Hits / files | Serde | SysML overlap | PLAN coverage |
|---|---|---:|---|---|---|
| Service | `service.rs` | 295 / 78 | `Serialize` on `Service` | part / service usage | not renamed (`ServiceDefinition` only) |
| ServiceId | `id.rs` | 2251 / 99 | per-variant `serde(rename)` | service identity | not renamed |
| Catalog | `catalog.rs` | 332 / 47 | `Serialize`, `JsonSchema` | package / model root | mentioned in C1, no rename |
| Bus | `edge.rs` | 48 / 13 | `rename_all = "snake_case"` | connection / bus kind | not renamed (`Interface` covers iface) |
| PortRef | `id.rs` | 105 / 33 | `rename_all = "snake_case"` | port reference | not renamed (`Port` only) |
| Resource | `resource.rs` | 141 / 26 | `Serialize`, `JsonSchema` | resource usage | not renamed |
| ResourceOwnership | `resource.rs` | (in enum) | `rename_all = "snake_case"` | resource semantics | not renamed |
| Domain | `domain.rs` | 33 / 6 | per-variant `serde(rename)`, `Deserialize` | package / domain | not renamed |
| DomainDef | `domain.rs` | low | `Serialize` | package def | not renamed |
| Page | `page.rs` | 121 / 38 | mixed `Observed`/`Asserted` | presentation (weak) | not renamed |
| PageId | `page.rs` | 165 / 37 | per-variant `serde(rename)` | page identity | not renamed |
| Requirement | `requirement.rs` | 27 / 3 | `Serialize`, `JsonSchema` | requirement def | struct name not in table |
| RequirementCatalog | `requirement.rs` | 47 / 7 | `Serialize`, `JsonSchema` | requirement catalog | not renamed |
| CapabilityId | `id.rs` | 829 / 67 | per-variant `serde(rename)` | capability (target name) | target of P1 collapse |
| CapabilityDef | `capability.rs` | 134 / 3 | none on struct | `def` registry | target of P1 collapse |
| FrontendCapabilityDef | `capability.rs` | 18 / 2 | none | frontend capability def | not renamed |
| ObservedFacts | `observed.rs` | 95 / 38 | `Serialize`, `JsonSchema` | observation layer | not renamed |
| RuntimeFacts | `runtime.rs` | 98 / 34 | `Serialize`, `JsonSchema` | runtime layer | not renamed |
| Authority | `service.rs` | 65 / 26 | `rename_all = "snake_case"` | authority judgment | not renamed |

Related near-misses (same module family, not PLAN targets): `Origin` (`feature.rs`), `AggregateGroup`, `Divergence` (`feature.rs`); `JourneyLens`, `JourneyPairAgreement`, `JourneySplitConsensus` (`journey_group.rs`); `AvailabilitySkip` (`version.rs`); `AutomatableCounts` (`coverage.rs`).

## Summary counts

| Metric | Value |
|---|---|
| Tier A planned current identifiers | 27 |
| Tier B planned current identifiers | 5 |
| All exist as named in PLAN | 32 / 32 |
| Aborted (type missing) | 0 |
| PLAN hit-count drift | 7 rows |
| Highest-hit WILL rename (excl. SKIP/DELETE) | UserJourney 294, Interface 237, FeatureAvailability 165, StepResult 129, ServiceDefinition 98 |
