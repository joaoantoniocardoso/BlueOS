# Catalog ontology -> SysML v2 concept mapping

Normative source: `catalog/spec/sysml-v2-release` submodule, pinned at tag **2026-05**
(commit `de1070ae`). Every claim below is cited to a file in that submodule. When the
submodule is bumped, re-verify the citations.

Short paths used in citations:
- `BNF` = `catalog/spec/sysml-v2-release/bnf/SysML-textual-bnf.kebnf`
- `KBNF` = `catalog/spec/sysml-v2-release/bnf/KerML-textual-bnf.kebnf`
- `LIB` = `catalog/spec/sysml-v2-release/sysml.library`

## 1. Name collisions (the reason this rework exists)

These are cases where a catalog identifier already means something different in
KerML/SysML v2. Exporting without fixing them produces a model that reads wrong to
anyone fluent in the standard.

| Catalog identifier | Collides with | Standard meaning | Citation |
|---|---|---|---|
| `Feature`, `FeatureId`, `FeatureCatalog`, `FeatureLens`, `FeatureAvailability`, `FeatureTraces` | KerML `feature` | The most general KerML element: *any* member of a type. Attributes, ports, parts, actions are all Features. | `KBNF` `Feature =` production; `'feature'` is a reserved keyword |
| `RequirementKind` (6 variants) | SysML v2 `RequirementKind` | Exactly two values, distinguishing an assumption constraint from a required one: `'assume' \| 'require'` | `BNF` `RequirementKind : RequirementConstraintMembership` |
| `Port(pub u16)` | SysML v2 `port` | An interaction point on a part, with a conjugate (`~P`) for the opposite direction. Not an address or a number. | `BNF` `PortDefinition`; `LIB/Systems Library/Ports.sysml` |
| `Function`, `FunctionId`, `FunctionIo` | KerML `function` | A Behavior that returns a result. Catalog `Function` has no result and models a performed behavior, which is `action def`. | `KBNF` `'function'` keyword; `LIB/Systems Library/Actions.sysml` |
| `ServiceDefinition` | SysML v2 `def` | "Definition" is the definition half of the def/usage pair. This type is the asserted-judgment half of a single *usage*, not a definition. | `BNF` `PartDefinition` vs `PartUsage` |
| `Interface` (enum of `Rest`/`Zenoh`/`Mavlink`/...) | SysML v2 `interface def` | An `interface` connects ports. The catalog enum enumerates *kinds of port*, one level below. | `BNF` `InterfaceDefinition`, `PortDefinition` |
| `HardwareRequirement`, `SoftwareRequirement`, `DataRequirement` | SysML v2 `requirement` | These are journey preconditions, which are `assume constraint` members, not requirements. | `LIB/Systems Library/Requirements.sysml` `assumptions[0..*]` |

## 2. Structural mapping

| Catalog | SysML v2 | Notes / citation |
|---|---|---|
| `Catalog` | root `package` | |
| `Domain` (6) | `package` | Pure grouping. |
| `Aggregate` (20) | nested `package` | DDD vocabulary; no SysML counterpart. Keep as a package label. |
| `ServiceId` + `Service` | `part def` + `part` | `LIB/Systems Library/Parts.sysml` |
| `ObservedFacts` | attributes of the part, each annotated with evidence | See section 4. |
| `ServiceDefinition` | attributes of the part, each annotated with rationale | See section 4. |
| `RuntimeFacts` | attributes annotated with a runtime capture reference | See section 4. |
| `Interface` variants | `port def` set | Conjugation is a direct fit: `Zenoh { topics_produced, topics_consumed }` is a port and its conjugate. |
| `Bus` | `interface def` kind | |
| `Edge` | `interface` usage (`connect A to B`) | `BNF` `InterfaceUsage`, `ConnectionUsage`. `Edge.via: Bus` makes this a typed interface, not a bare `connect`. |
| `Port(u16)`, `PortRef`, `PathRef` | plain `attribute` values | Not SysML ports. |
| `CapabilityId` (144) | requirement `subject` | `LIB/Systems Library/Requirements.sysml`: `subject subj : Anything[1]` -- "the entity being checked for satisfaction". |
| `CapabilityDef` / `FrontendCapabilityDef` | the registry that binds a subject to its owning part | |
| `Function` (108) | `action def` | `LIB/Systems Library/Actions.sysml` |
| (pending) action I/O | action `in` / `out` parameters | not on `Action` yet; FastAPI extract is method+path only. |
| `UserJourney` (101) | `use case def` | `LIB/Systems Library/UseCases.sysml`: `use case def UseCase :> Case` with `subj`, `obj`, `subUseCases`, `includedUseCases`. |
| `UserJourney.chains_from` | `includedUseCases` or a succession | |
| `JourneyStep` | `action` usage in the case body | No independent identity in either model; addressed positionally. |
| `Actor` | `actor` | `BNF` `ActorMember : ActorMembership` |
| `Precondition` | `assume constraint` | `LIB/.../Requirements.sysml` `assumptions[0..*]` |
| `AcceptanceCriterion` | `require constraint` | `LIB/.../Requirements.sysml` `constraints[0..*]` |
| `Requirement.functional_children` | `subrequirements` | `LIB/.../Requirements.sysml`: `abstract requirement subrequirements[0..*]`. Exact library name. |
| `Requirement.verifying_journeys` | `requirementVerifications` / `verify requirement` | `LIB/Systems Library/VerificationCases.sysml`; `BNF` `RequirementVerificationMember` |
| `Requirement.{feature_id, journey_id, function_id}` | one `subject` | The three-field spread collapses to the standard single subject. |
| a harness run against a DUT | `verification def` + `return verdict : VerdictKind` | `LIB/.../VerificationCases.sysml`. Note the keyword is `verification def`, **not** `verification case def`. |
| `StateMachine` | `state def` | `LIB/Systems Library/States.sysml` |
| `Resource` + `ResourceOwnership` | `item def` + `allocate` | `LIB/Systems Library/Items.sysml`, `Allocations.sysml` |
| `Page` | `part def` in the presentation package | |
| `PageServiceCall` | `interface` usage | |
| `SloBaseline` | `PerformanceRequirementCheck`, or an `analysis def` | Keyword is `analysis def`, **not** `analysis case def`. |
| `FeatureAvailability` | `variation` / `variant` | `BNF` `isVariation ?= 'variation'`, `VariantUsageMember`. Alternative: plain metadata. See PLAN Phase 3. |
| `CriticalityTier`, `PrivilegeLevel`, `BlastRadius`, `Visibility` | `enum def` or metadata attributes | |

## 3. The requirement taxonomy is already in the library

The catalog's six `RequirementKind` variants do not need a bespoke mapping. Five of
them land on library types, which carry *typed subjects* that independently confirm
the rest of this table. From `LIB/Systems Library/Requirements.sysml`:

```
requirement def FunctionalRequirementCheck  :> RequirementCheck { subject: Action; }
requirement def InterfaceRequirementCheck   :> RequirementCheck { subject: Interface; }
requirement def PerformanceRequirementCheck :> RequirementCheck { subject: AttributeValue; }
requirement def PhysicalRequirementCheck    :> RequirementCheck { subject: Part; }
requirement def DesignConstraintCheck       :> RequirementCheck { subject: Part; }
```

| Catalog kind | Derived in catalog from | Library type | Subject agreement |
|---|---|---|---|
| `Functional` | `Function` | `FunctionalRequirementCheck` | `Function` -> `action def`; library subject is `Action`. Agrees. |
| `Interface` | distinct `RouteRef`s | `InterfaceRequirementCheck` | `RouteRef` -> interface; library subject is `Interface`. Agrees. |
| `Performance` | `SloBaseline` | `PerformanceRequirementCheck` | SLO is a measured attribute; library subject is `AttributeValue`. Agrees. |
| `System` | one per capability | local specialization of `RequirementCheck` | No library type. Subject is the whole-system `Part`. |
| `Robustness` | `NEGATIVE_PROBES` | local specialization of `RequirementCheck` | No library type. |
| `Constraint` | journey `Precondition`s | **none - misclassified** | Preconditions are `assume constraint` members of the requirements they gate, not requirements in their own right. See PLAN Phase 3. |

Defining local specializations for `System` and `Robustness` is the intended
extension mechanism, not a workaround: every library type above is itself a
specialization of `RequirementCheck`.

## 4. Provenance maps onto standard metadata

This is the part that most needed checking, because the catalog's evidence discipline
is its whole value and a lossy export would defeat the purpose.

| Catalog | SysML v2 | Citation |
|---|---|---|
| `Asserted::Established { rationale }` | `@Rationale { text = "..."; }` | `LIB/Domain Libraries/Metadata/ModelingMetadata.sysml`: `metadata def Rationale { attribute text : String; }` -- exact field match |
| `Unknown { reason }` | `@StatusInfo { status = StatusKind::tbd; }` plus `@Issue { text = reason; }` | same file: `enum def StatusKind { open; tbd; tbr; tbc; done; closed; }`, `metadata def StatusInfo`, `metadata def Issue { attribute text : String; }` |
| `Observed::Known { evidence: Evidence { file, line, anchor } }` | local `metadata def Evidence { attribute file : String; attribute line : Natural; attribute anchor : String; }` | No library counterpart, correctly: source-citation evidence is domain metadata. `BNF` `MetadataDefinition`, annotation via `@` or `#`. |
| `Provenance::Runtime { capture, environment }` | local `metadata def RuntimeCapture` | Same reasoning. |

`Unknown { reason }` surviving the export intact is what keeps the coverage gate and
the harness ratchet meaningful on the far side. `StatusKind::tbd` plus `Issue` says
"not yet determined, and here is why", which is exactly the catalog's semantics --
distinct from an element that is simply absent.

## 5. Two catalog enums reinvent standard ones

| Catalog | Standard | Citation |
|---|---|---|
| `Automatable { Http, Frontend, Hardware, ExternalGcs, Manual }` | `VerificationMethodKind { inspect; analyze; demo; test; }`, applied as `@VerificationMethod { kind = (test); }` | `LIB/Systems Library/VerificationCases.sysml` |
| runner step results, `journey_matrix::CellState` | `VerdictKind { pass; fail; inconclusive; error; }` | same file |

`pass_with_finding` (see `extras/qa-1.4-closeout/G0_TAXONOMY.md`, F-068) becomes
`VerdictKind::pass` plus an `@Issue` annotation. A typed skip becomes
`VerdictKind::inconclusive` plus the skip reason as an `@Issue`.

## 6. Deliberately not adopted

| Standard concept | Why not |
|---|---|
| `PartDef` / `PartUsage` as distinct Rust types throughout | The catalog is almost entirely usages: 26 specific services, 101 specific journeys, 24 specific pages. The definition side would have to be invented. Where a real definition exists it is already there (`CapabilityDef`, and the library types in section 3, which we import rather than declare). |
| KerML abstract syntax (`Element`, `Membership`, `Relationship`) | That is building a SysML v2 metamodel in Rust, which is the pivot we chose not to make. The exporter emits textual notation; it does not model the metamodel. |
| Systems Modeling API as the store | Out of scope during reverse engineering. Revisit if BlueOS 2.0 work needs SysON as the authoring surface. |
