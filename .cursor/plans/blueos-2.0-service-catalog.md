# BlueOS 2.0 — Service Catalog Model

**Branch:** `2.0-dev/model`
**Status:** M0–M3 DONE (harness, calibration, journey spine, full 26-service model, complete edge graph, clustering engine); F1 DONE (feature-first Views A+B); **F2 DONE (all 24 frontend pages modeled; frontend features merged into the inventory via `Feature.origin`).** The catalog is now a **fully const/static, strongly-typed** model (see [Typed const-catalog model](#typed-const-catalog-model-post-refactor)). M4 (human architecture-decision step) still pending. (Detailed run log: `.cursor/plans/blueos-2.0-autonomous-run.md`.)
**Audience:** Orchestrator agent (Opus) and human architects
**Phase 1 objective:** Build a precise, agentic harness — every step of the process has a defined agent profile, its skills, and an acceptance gate. Precision over speed.

---

## Goals

BlueOS 2.0 is a major rework from scratch. A central early task is **reorganizing services and their responsibilities**. Before merging/splitting services, we need a **machine-verifiable, human-readable model** of what each service does today.

### Primary deliverable

A **`blueos-catalog` Rust crate** that is the **single source of truth** for the *semantic* model of every BlueOS service, layered on top of an **auto-extracted facts layer** derived from the repo. See [Two-layer model](#two-layer-model-the-core-principle).

The crate captures:

- Capabilities and exclusive authorities
- Interfaces (REST, Zenoh, MAVLink, WebSocket, HTTP streams, subprocesses, files, settings, hardware, Docker)
- States and boot/degraded conditions
- Resource ownership (exclusive / shared read / shared write)
- Interaction graph (unified edges: callers, callees, producers, consumers)
- Deployment truth (startup tier, cgroup limits, nginx paths, tmux names, run-as)
- Criticality, trust, lifecycle, observability, failure modes
- Extension platform surface (kraken and extension contracts)
- Domain placement for clustering into 2.0 bounded contexts

### Secondary deliverables

- **Validation** — cross-service invariants (no duplicate exclusive ports, no conflicting authorities, known edge targets)
- **Export** — JSON + JSON Schema (`serde` + `schemars`) for agent tooling
- **Drift detection** — the correctness engine, not an afterthought: diff the *asserted* semantic layer against the *observed* facts layer (which itself is regenerated from `start-blueos-core`, `nginx.conf`, `blueos-zenoh.json5`, and service code)
- **Reference service cards** — full definitions for `ardupilot_manager` and `kraken` first (stress MAVLink + Docker + extensions), used to **calibrate the rubric**

### Non-goals (for this branch)

- Rewriting Python services
- Implementing 2.0 runtime architecture
- Auto-migrating services based on clustering output

---

## Two-layer model (the core principle)

The single biggest risk to this project is **quality drift**: card #18 researched less rigorously than card #1, or a card that *claims* a port/edge that no longer exists in the code. The defense is to never mix two fundamentally different kinds of information:

| Layer | Contains | Ground truth | Who authors | Trust mechanism |
|-------|----------|--------------|-------------|-----------------|
| **Observed** | Ports, tmux name, nginx prefix(es), startup tier, cgroup limits, subprocess command, MAVLink connect strings, settings paths, outbound HTTP/Zenoh calls found by search | Exists in the repo | **Extractor agent** (mechanical) | Every field carries `evidence: file:line` or the command that produced it |
| **Asserted** | Authorities, criticality tier, trust/privilege, blast radius, failure modes, `bounded_context`, capability verbs | Human/LLM judgment | **Card Author agent** | Reviewed by orchestrator; `Unknown { reason }` where not established |

**Rules that follow from this split:**

1. **Never hand-type an observed fact.** It drifts the instant someone edits `start-blueos-core`. The extractor regenerates the observed layer; humans never edit it by hand.
2. **Drift detection = diff(asserted, observed).** CI fails when a card asserts `port 8000` but the observed layer (from nginx + startup) disagrees. This is the primary correctness gate.
3. **No field is silently blank.** Every field is either present-with-evidence or `Unknown { reason }`. A card over the `Unknown` threshold fails the coverage gate. This forces uniform depth across all services.

> SSOT precedence: repo → observed layer (generated) → asserted layer (authored) → exports (JSON/schema/mermaid). Data only ever flows downward; drift detection enforces it.

---

## User journeys — the spine (three-source triangulation)

Services are *nouns*; the model also needs the *verbs* — the workflows an operator actually performs. **User journeys are the spine**: routes, state transitions, error semantics, and platform behavior are all **projections of the journey set**. A journey is therefore a **top-level** artifact (`catalog/src/journeys/<id>.rs`) that references services/routes/states by id — never trapped inside one service module, because real journeys (e.g. first-boot wizard) cross several services.

This forces a **third provenance class** beyond source-`Evidence` and asserted-`rationale`. A journey is only *fully grounded* when three independent sources agree:

| Source repo | Supplies | Provenance | Owner |
|-------------|----------|------------|-------|
| `../BlueOS-docs` | operator intent, visibility (pirate/advanced), preconditions, step order | `Provenance::Doc{file,line}` | Docs Specialist |
| BlueOS repo source | the route each step hits actually exists | `Provenance::Source(Evidence)` | Fact Extractor |
| **live BlueOS capture** (Raspberry Pi 4; identity = image digest, not floating tag) | what actually happens: status, body, transition, latency | `Provenance::Runtime{capture,environment}` | Runtime Specialist |

> **Version-pin policy (runtime captures):** a floating tag like `master` is a channel label, not a pin. Every `Provenance::Runtime.environment` MUST record `bluerobotics/blueos-core:<tag> @ sha256:<digest>` (and ideally `captured_at`). Capture filenames may keep `__master` as the channel suffix; the digest inside the artifact JSON / `environment` string is the identity. Re-read `GET /version-chooser/v1.0/version/current` on the play Pi before claiming an environment — the Pi may drift. Historical baseline digest: `sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e`.

> The POC `../microservices_core_prototype` is **design-reference only** — it revealed *which* runtime dimensions exist (state contracts, SLOs, settings mutations, platform matrix) but is **never a value source**. Runtime values come only from capturing a live BlueOS instance; absent a capture, the field is `Unknown`. These runtime facts have their own per-service **`RuntimeFacts`** layer (see `runtime.rs`), parallel to observed/asserted, so they can *diff against* the asserted layer (e.g. a captured `500`-when-stopped falsifies `api_stable: true`).

**Anti-drift payoff:** a documented journey with no source route = doc drift; a route in no journey = undocumented/dead surface; a runtime outcome contradicting the doc = behavioral drift. `Catalog::validate()` enforces the join: every journey route/service/capability/state/`chains_from` reference must resolve. Journeys are also *replayable* (the prototype does baseline→compare), so the spine is executable, not just prose.

> The existing two-layer `Evidence`/`rationale` split is unchanged for the observed and asserted layers. `Provenance` (with a `Source(Evidence)` arm) is introduced only for the journey layer, which legitimately mixes all three kinds per item.

---

## Background — why not manual WBS + clustering?

We considered: per-service WBS (features, inputs, outputs, states) → flatten under BlueOS → mental cluster analysis by responsibility.

### Flaws of that approach

1. **Documents declared boundaries, not actual coupling** — MAVLink ports, nginx routing, shared `/usr/blueos/userdata`, implicit HTTP calls (`helper` → `version-chooser`, `bridget` → `linux2rest`) are invisible in feature lists.
2. **Static and subjective** — naming drift (`pardal` → `/network-test/` → `:9120`; `autopilot`/`ardupilot_manager`/`/autopilot-manager/`), dead routes, startup priority tiers.
3. **Wrong primary ontology** — "features" under-weight safety, trust boundaries, consistency models, lifecycle, and operational SLOs.
4. **Extensions break clean boxes** — kraken is a platform inside the platform.
5. **Human clustering is path-dependent** — multiple valid groupings, no objective scoring, ignores Conway's law and migration cost.
6. **Hard to falsify** — weeks of docs without runtime validation.

### Classically used complements

| Method | Role |
|--------|------|
| Event storming | Find cross-service workflows |
| DDD bounded contexts | Name and scope 2.0 domains |
| C4 model | Communicate layers |
| Dependency / coupling analysis | Bottom-up truth from code |
| Quality attributes + ADRs | Constrain merges/splits |
| Strangler fig migration | Make 2.0 shippable incrementally |
| Team topology + user journeys | Align ownership and UX |

### Agent-augmented workflow

```text
Repo + runtime config  →  Extractor agents: observed facts layer (provenance-checked)
                       →  Card Author agents: asserted semantics on top
                       →  Orchestrator QA: validate + drift + coverage + provenance spot-check
                       →  Human: event storm + journeys
                       →  Clustering agent: multi-policy proposals + stability metric
                       →  Human: score with quality attributes + ADRs
                       →  Target boundaries + strangler slices
                       →  Continuous drift checks vs catalog
```

Agents **extract and propose**; humans **decide** on safety, ownership, and vehicle-critical paths.

---

## Anti-drift process controls

These are what keep card #18 as good as card #1. They are as important as the type system.

1. **Frozen extraction protocol (rubric).** An ordered, *executable* checklist of where to look and which command produces each fact. See [Extraction protocol](#extraction-protocol-rubric). It is frozen after calibration; changes require re-running affected cards.
2. **Provenance on every observed field.** `file:line` or the exact command. No evidence → `Unknown`, never a guess. Enables second-based reviewer spot-checks.
3. **Coverage gate via types.** No "minimal vs full" profiles. `Unknown { reason }` is a first-class value; a coverage threshold (e.g. ≤ N `Unknown` on required fields) is enforced in CI.
4. **Calibrate, then measure inter-rater reliability before scaling.** Do `ardupilot_manager` + `kraken` first *with a human in the loop*, freeze the rubric, then run **two independent passes** (two composer-2.5 agents, or agent + human) on 2–3 services and measure disagreement on authorities/tiers. Ambiguity → fix the rubric before batching the other ~24 processes.
5. **Harness before content.** Crate types + extractor + validation/drift gate + export exist *before* any card is authored, so every card is born validated.
6. **One process per task, independent QA.** Each subagent handles exactly one service/process (bounded blast radius, parallelizable). The orchestrator that QAs a card never authored it.

---

## Agentic harness

Phase 1 delivers a harness where each step has an **agent profile** (mission, inputs, tools/skills, procedure, output contract, done-criteria, forbidden actions) and a **skill** file the agent loads. The orchestrator (Opus) spawns **composer-2.5** workers via the Task tool, dispatches an independent **QA Reviewer** for every output, and merges only on an ACCEPT verdict.

### Roles overview

| Role | Model | Owns | Never does |
|------|-------|------|------------|
| **Orchestrator** | Opus (this agent) | Planning, spawning workers, dispatching QA, adjudicating verdicts, merges, rubric ownership | Produce or QA the artifact itself |
| **Crate Engineer** | composer-2.5 | `blueos-catalog` crate: types, registry, `validate()`, export, extractor + drift binaries | Encode service data by hand |
| **Fact Extractor** | composer-2.5 | Observed layer for one process (with provenance); confirms journey route hints → `Source` | Make judgment calls; guess |
| **Docs Specialist** | composer-2.5 | Doc-grounded journey skeletons from `../BlueOS-docs` (intent, visibility, services, preconditions, steps) | Assert runtime behavior; confirm routes; invent undocumented journeys |
| **Runtime Specialist** | composer-2.5 | `RuntimeFacts` + journey step `outcome`s captured from a **live BlueOS Pi** (state contracts, SLO baselines, settings mutations, platform matrix) | Take values from the POC prototype; assert runtime from docs/source; guess uncaptured boards/states |
| **Card Author** | composer-2.5 | Asserted semantics for one service; journey `capability_refs` + per-service `journey_refs` | Touch observed fields |
| **QA Reviewer** | composer-2.5 (fresh context) | Independent verification of one worker output → ACCEPT/BOUNCE verdict | Review work it produced; fix the artifact itself |
| **Clustering Analyst** | composer-2.5 | Multi-policy clustering + stability metrics | Decide final boundaries |

**QA independence is structural:** the QA Reviewer is a *separate* composer-2.5 instance with fresh context — never the author of the artifact, never the orchestrator. The orchestrator dispatches it, then adjudicates its verdict; it never performs the review itself. This removes both author bias and the orchestrator bottleneck.

### Skills (to be created under `.cursor/skills/`)

Each skill is a `SKILL.md` the worker reads first. They encode the precision rules so quality does not depend on prompt wording.

| Skill | Loaded by | Encodes |
|-------|-----------|---------|
| `blueos-catalog-crate` | Crate Engineer | Rust conventions, top-down ordering, `cargo fmt`/`cargo test`, type-design invariants (parse-don't-validate, `PortRef`, `Unknown`) |
| `blueos-service-extraction` | Fact Extractor | The frozen extraction protocol + provenance format + output contract |
| `blueos-journey-extraction` | Docs Specialist | Mining `../BlueOS-docs` for operator journeys; `service()`/`pirate()` shortcodes; doc-anchored journey skeletons + route hints |
| `blueos-runtime-capture` | Runtime Specialist | Capturing a live BlueOS Pi into `RuntimeFacts` + step outcomes; capture-artifact provenance; POC prototype is reference-only, never a value source |
| `blueos-service-card` | Card Author | Disambiguation rules (capability vs authority vs interface vs edge vs resource), the BlueOS-specific checklist, `Unknown{reason}` policy |
| `blueos-catalog-validate` | QA Reviewer (+ orchestrator adjudication) | Running `validate()`, drift diff, coverage + provenance spot-check; ACCEPT/BOUNCE verdict |
| `blueos-clustering` | Clustering Analyst | Edge weights, named policies, stability/modularity metrics |

### Rules vs skills (guardrails vs procedures)

Rules and skills are complementary, not redundant:

- **Skills** are *opt-in procedures* a worker loads for a step. If an agent skips the skill, there is no guardrail.
- **Rules** (`.cursor/rules/*.mdc`) are *always-on invariants* auto-attached by glob whenever a matching file is edited — they fire even when a worker forgets the skill. They are the drift protection that cannot be forgotten.

The orchestrator QA protocol stays in this plan (it governs orchestrator behavior, not a file-triggered edit) and must not be duplicated as a rule. Global repo rules (`top-down-ordering`, `cargo-fmt`, `minimal-diff`) already apply; catalog rules add only crate-specific constraints.

| Rule | Glob | Enforces |
|------|------|----------|
| `blueos-catalog-two-layer` | `catalog/**` | Observed/asserted separation; `observed/` generated-only; provenance required; `Unknown{reason}` over blanks; `validate()` + `drift` before done |
| `blueos-catalog-rust` | `catalog/**/*.rs` | Parse-don't-validate, `PortRef` literal-vs-env, approved deps only, `cargo fmt`/`clippy`/`test` |
| `blueos-catalog-journeys` | `catalog/src/journeys/**`, `catalog/src/journey.rs` | Journeys are top-level + cross-service; three-source triangulation (`Doc`/`Source`/`Runtime`); layer ownership; `validate()` cross-refs required for every reference field |
| `blueos-catalog-runtime` | `catalog/src/runtime.rs`, `catalog/src/services/*.rs`, `catalog/src/journeys/*.rs`, `catalog/runtime-captures/**` | `RuntimeFacts` is a third layer grounded in `Provenance::Runtime`; only a live BlueOS capture is trusted (POC prototype is reference-only); capture-artifact + environment required; runtime never lives in the asserted layer |
| `blueos-capture-tools` | `catalog/runtime-captures/**` | Measurements go through committed, parameterized tools under `runtime-captures/tools/` (reuse/extend, never ad-hoc); self-describing JSON; CPU/mem/latency as a distribution over a window, never a single snapshot |

### Agent profiles

#### Crate Engineer (M0)
- **Mission:** Build/maintain the crate: types, registry, `validate()`, `export`, `extract` binary, `drift` binary.
- **Inputs:** This plan; `blueos-catalog-crate` skill.
- **Tools/skills:** Rust, `cargo`, `blueos-catalog-crate`.
- **Output contract:** Compiling crate; `cargo test` green; `Catalog::validate()` runs on empty/minimal catalog; `export json` and `--schema` produce valid output.
- **Done-criteria (gate):** `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` all pass; schema round-trips.
- **Forbidden:** Hand-encoding service facts; adding deps beyond the approved list without orchestrator sign-off.

#### Fact Extractor (M1+, one process per task)
- **Mission:** Produce the observed-facts artifact for a single process, every field with provenance.
- **Inputs:** process id; `start-blueos-core`, `nginx.conf`, `blueos-zenoh.json5`, the service dir/binary; `blueos-service-extraction` skill.
- **Tools/skills:** Grep/Glob/Read, `blueos-service-extraction`.
- **Output contract:** `observed/<id>` populated per the protocol; each field `{value, evidence}` or `Unknown{reason}`.
- **Done-criteria (gate):** Every populated field has resolvable `file:line`; ports/routes reconcile with nginx + startup; drift gate passes for the observed layer.
- **Forbidden:** Authorities, tiers, `bounded_context`, or any judgment field.

#### Docs Specialist (M1.5+, one service/flow per task)
- **Mission:** Mine `../BlueOS-docs` for the operator journeys of a service (or a cross-service flow) and produce the **doc-grounded** journey skeleton.
- **Inputs:** `../BlueOS-docs/content/usage/**` (esp. `advanced/index.md`, `getting-started`); `service()`/`pirate()`/`note()` shortcodes; `blueos-journey-extraction` skill.
- **Tools/skills:** Read/Grep over `../BlueOS-docs`, `blueos-journey-extraction`.
- **Output contract:** top-level `UserJourney` builders in `catalog/src/journeys/<id>.rs` with every doc-grounded field `Grounded::known(value, Provenance::Doc{file,line})`; step routes as **hints**; runtime fields `None`.
- **Done-criteria (gate):** every journey + field resolves to a real `content/…:LINE`; `visibility` matches pirate membership; each participating `ServiceId` maps to a catalog service; no runtime field set.
- **Forbidden:** Asserting `expected_status`/`body_predicate`/`transition`; confirming routes exist in code (Fact Extractor's job); inventing undocumented journeys.

#### Card Author (M1+, one service per task)
- **Mission:** Author the asserted semantics on top of a completed observed artifact.
- **Inputs:** the observed artifact; service source; `blueos-service-card` skill.
- **Tools/skills:** Read/Grep, `blueos-service-card`.
- **Output contract:** `const ServiceDefinition` (asserted fields only) registered in the catalog; `Unknown{reason}` where not established.
- **Done-criteria (gate):** compiles; `validate()` passes; coverage threshold met; no asserted field contradicts the observed layer.
- **Forbidden:** Editing observed fields; inventing edges without a capability/interface to justify them.

#### QA Reviewer (M1+, one worker output per task)
- **Mission:** Independently verify a single Crate Engineer / Fact Extractor / Card Author output and return an ACCEPT/BOUNCE verdict.
- **Inputs:** the artifact under review + its worker output summary; `blueos-catalog-validate` skill. **Not** given the author's chain of reasoning — reviews the artifact fresh.
- **Tools/skills:** `cargo`, Read/Grep, `blueos-catalog-validate`.
- **Output contract:** verdict = **ACCEPT** (all gates green) or **BOUNCE** (each failing gate with exact command output or bad `file:line` + required fix).
- **Done-criteria (gate):** ran every automated gate; performed the provenance spot-check; verdict is reproducible.
- **Forbidden:** Reviewing an artifact it authored; editing the artifact to "fix" it (BOUNCE instead); rubber-stamping without opening cited evidence.

#### Clustering Analyst (M3–M4)
- **Mission:** Run `coupling_matrix()` under multiple policies, compute stability, emit boundary proposals.
- **Output contract:** ≥3 proposals; per-policy modularity; pairwise co-occurrence matrix; list of unstable services.
- **Done-criteria (gate):** results reproducible from committed weights; no proposal presented as "the answer."
- **Forbidden:** Choosing final boundaries (human decision).

### QA flow (delegated + adjudicated)

QA is a separate step performed by an independent QA Reviewer, not folded into any producing role. Per worker output:

1. **Ground-truth-first ordering:** the orchestrator never dispatches a Card Author before the matching observed artifact has an ACCEPT verdict.
2. **Dispatch independent review:** the orchestrator spawns a **QA Reviewer** (fresh composer-2.5, not the author) with the `blueos-catalog-validate` skill. The reviewer runs the automated gates (`cargo fmt --check` + `clippy` + `test` + `Catalog::validate()` + `drift` + coverage) and the provenance spot-check, then returns ACCEPT/BOUNCE.
3. **Adjudicate:** the orchestrator reads the verdict. BOUNCE → return to the original worker with the reviewer's evidence; never let the reviewer edit the artifact. ACCEPT → proceed.
4. **Inter-rater check (calibration only):** on 2–3 services, run two independent QA Reviewers (or reviewer + human); disagreement on authorities/tiers/context → refine rubric/skill, not just the card.
5. **Merge discipline:** orchestrator merges; one process (or one milestone) per PR; commit style per repo convention.

> **Automated vs judgment QA.** The automated gates are deterministic and could also run in CI; the QA Reviewer adds the judgment layer (provenance truthfulness, authority/edge justification) that CI cannot. Both are required before ACCEPT.

---

## Extraction protocol (rubric)

Executable steps the `blueos-service-extraction` skill encodes. For each process:

1. **Identity & deployment** — find the tuple in `start-blueos-core` (`PRIORITY_SERVICES` or `SERVICES`): tmux name, tier, mem/cpu/io limits, `nice`, run-as, exact command. → `evidence: core/start-blueos-core:LINE`.
2. **Listener(s)** — determine the port(s) it binds (grep `--port`, `--server`, `listen`, argparse defaults, hardcoded). → evidence in service main / start command.
3. **nginx route(s)** — grep `nginx.conf` for the `proxy_pass` matching the port; record **all** prefixes (a service may have several, e.g. `/ardupilot-manager/` + `/autopilot-manager/` → `:8000`). → `evidence: core/tools/nginx/nginx.conf:LINE`.
4. **MAVLink role** — grep for `udpin|udpout|tcpin|tcpout|--connect|--mavlink|MAV_SYSTEM_ID|component-id`; classify router-owner / endpoint / bridge / consumer.
5. **Zenoh** — grep for `zenoh`, `blueos-zenoh.json5`, `zenoh_helper`; record topics produced/consumed.
6. **Hardware exclusivity** — serial ports, camera nodes, `wlan0`, GPIO. Record device path.
7. **Settings / files** — writes under `/usr/blueos/userdata`, settings paths; who else writes the same path.
8. **Subprocesses / external binaries** — spawned processes (`mavlink-camera-manager`, `linux2rest`, `mavlink2rest`, `zenohd`, `nginx`, `filebrowser`, `ttyd`, `iperf3`, `blueos-recorder`).
9. **Outbound edges** — grep for `http://`, `127.0.0.1:`, `aiohttp`, service base URLs to find implicit calls to other services.
10. **Env coupling** — `MAV_SYSTEM_ID`, `BLUEOS_*`, secondary venv usage.

Each step yields `{value, evidence}` or `Unknown{reason}`. Steps map 1:1 to observed-layer fields.

---

## Service view schema (conceptual)

Each field is tagged **[O]** observed (extractor) or **[A]** asserted (author).

| Group | Key fields |
|-------|------------|
| Identity | `id`, `aliases` **[O]**, `kind` **[O]**, `entrypoint` **[O]**, `singleton` **[A]** |
| Deployment | `tmux_name` **[O]**, `startup_tier` **[O]**, `resource_limits` **[O]**, `nice` **[O]**, `run_as` **[O]**, `nginx_prefix` (list) **[O]**, `listen` **[O]** |
| Domain | `bounded_context` **[A]**, `user_journeys` **[A]** |
| Criticality & trust | `tier` **[A]**, `offline_required` **[A]**, `privilege_level` **[A]**, `dangerous_operations` **[A]**, `user_confirmation` **[A]** |
| Capabilities | verbs the service exposes **[A]** |
| Authorities | **exclusive** rights only this service should have **[A]** |
| Interfaces | typed: `rest`, `zenoh`, `mavlink`, `websocket`, `http_stream`, `subprocess`, `file`, `settings`, `hardware`, `docker` **[O]** (existence) + **[A]** (intent) |
| States | entity state machines, `boot_state`, `degraded_when` **[A]** |
| Resources | `exclusive`, `shared_read`, `shared_write` **[O]** paths + **[A]** mode |
| Edges | unified: `from`, `to`, `via` (bus), `sync`, `endpoint`, `purpose`, `required_at_boot`, `failure_impact` — target/endpoint **[O]**, purpose/impact **[A]** |
| Lifecycle | `triggers`, `ordered_after/before` **[O]**, `shutdown`, `upgrade_behavior` **[A]** |
| Observability | logs path **[O]**, zenoh log topic **[O]**, sentry **[O]**, health **[A]** |
| Extension | `is_platform` **[A]**, `api_stable` **[A]**, `permissions_model` **[A]** |
| Failure | modes, `blast_radius` **[A]** |
| Contracts | OpenAPI refs **[O]**, API version prefix(es) on `Interface::Rest` **[O]**, settings root shape **[O]**, compatibility policy **[A]** |
| Runtime **[R]** | `RuntimeFacts` (new layer, `Provenance::Runtime`): `state_contracts` (per-state endpoint status/body), `slo_baselines` (latency/cpu/mem), `platform_matrix` (per-board behavior), settings-mutation-per-transition — captured from parity baselines, never source/asserted |
| Meta | team **[A]**, `git_path` **[O]**, ADR refs **[A]** |

### Disambiguation rules

- **Capabilities** = what it can do
- **Authorities** = what only it may do
- **Interfaces** = how capabilities are reached
- **Edges** = who talks to whom on which bus
- **Resources** = what it owns or shares (typed ownership mode)

### BlueOS-specific checklist (per service)

1. MAVLink role? (router owner, endpoint, bridge, consumer)
2. Hardware exclusive? (serial, camera, WiFi iface)
3. Survives reboot / upgrade / settings reset?
4. Who else writes the same settings path?
5. nginx path(s) vs SERVICE_NAME vs tmux name — record all three (they frequently differ)
6. Priority startup tier?
7. Extension-visible?
8. External binary? (mavlink-camera-manager, linux2rest, mavlink2rest, zenohd, filebrowser, ttyd, iperf3, blueos-recorder, nginx)
9. Side effects on vehicle safety?
10. Implicit env var coupling? (`MAV_SYSTEM_ID`, `BLUEOS_*`, secondary venv)

---

## Rust type system design

**Principle:** Rust source is SSOT for the *asserted* layer; the observed layer is *generated*; TOML/JSON/YAML are **build artifacts** only.

### Crate layout

```text
blueos-catalog/
├── src/
│   ├── lib.rs
│   ├── id.rs               # ServiceId/CapabilityId/JourneyId/PageId ENUMS (serde-renamed) + Entity trait; PathRef/PortRef (&'static str)
│   ├── provenance.rs       # Evidence{file,line}; Observed<T>|Unknown; Asserted<T>|Unknown; Grounded/Provenance (all const, &'static)
│   ├── capability.rs       # Aggregate enum; CapabilityDef + CAPABILITIES (backend registry); FrontendCapabilityDef + FRONTEND_CAPABILITIES
│   ├── criticality.rs
│   ├── trust.rs
│   ├── interface.rs        # tagged enum Interface { Rest{versions}, Zenoh, Mavlink, OutboundHttp, ... }
│   ├── resource.rs
│   ├── state.rs
│   ├── edge.rs
│   ├── lifecycle.rs
│   ├── observed.rs         # ObservedFacts (extractor output shape)
│   ├── service.rs          # Service AGGREGATE { id, observed, definition, runtime } + ServiceDefinition (asserted)
│   ├── journey.rs          # UserJourney, JourneyStep (route: Grounded<RouteRef>, outcome: Grounded<StepOutcome>), Actor, Visibility, Precondition
│   ├── runtime.rs          # RuntimeFacts (third layer, Provenance::Runtime): state_contracts, slo_baselines, platform_matrix, settings_mutations
│   ├── page.rs             # Page (frontend route) + PageId enum + ConsumeTarget{Service(ServiceId),External} + ClientState/StateOwnership
│   ├── feature.rs          # Feature { id, aggregate, origin: Origin{BackendService|FrontendPage}, rationale }; FeatureCatalog + A/B views
│   ├── cluster.rs          # coupling_matrix + greedy modularity + stability
│   ├── catalog.rs          # Catalog { services: Vec<Service>, journeys, pages } + indexing
│   ├── resolve.rs          # PortRef::Env → literal
│   ├── validate.rs         # cross-service rules + coverage gate
│   ├── drift.rs            # diff(asserted, observed) + runtime
│   ├── export.rs           # JSON, JSON Schema, mermaid
│   ├── services/           # per service: `pub const SERVICE: Service` (bundles observed/definition/runtime)
│   │   ├── mod.rs          # `pub const SERVICES: &[Service]` + all_services()
│   │   ├── ardupilot_manager.rs … (26 modules)
│   ├── pages/              # per page: `pub const PAGE: Page`
│   │   ├── mod.rs          # `pub const PAGES: &[Page]` + all_pages()  (24 modules)
│   └── journeys/           # TOP-LEVEL user journeys (may cross services)
│       ├── mod.rs          # `pub const JOURNEYS` per module, concatenated
│       └── <journey_id>.rs
├── observed/               # GENERATED by `extract` — never hand-edited
├── runtime-captures/       # raw live-BlueOS capture artifacts (Provenance::Runtime source)
└── src/bin/
    ├── extract.rs          # repo → observed/
    ├── drift.rs            # asserted vs observed → CI gate
    ├── export.rs, features.rs, cluster.rs
```

### Key design choices

1. **Parse, don't validate** — enums/newtypes; `Bus::Mavlink` cannot appear under REST.
2. **Definition vs resolved** — `PortRef::Literal(14001)` vs `PortRef::Env("MAV_SYSTEM_ID")`; `resolve()` pass for tooling.
3. **Observed vs asserted are distinct types** — `ObservedFacts` (generated) is separate from `ServiceDefinition` (authored); `drift.rs` reconciles them.
4. **Two provenance wrappers, `Unknown { reason }` first-class** — observed fields use `Observed<T>` (carries `Evidence{file,line}`); asserted fields use `Asserted<T>` (carries `rationale`, never `file:line`). Both have an `Unknown` arm feeding the coverage gate; no silent blanks.
5. **Unified edges** — replace separate callers/callees/producers/consumers lists.
6. **Registration** — plain `&'static [Service]`/`&'static [Page]` slices (`SERVICES`, `PAGES`, per-module `JOURNEYS`); `inventory`/`linkme` never needed.
7. **Fully const/static declaration (post-refactor).** The original assumption that owned `String`/`Vec` make `const` impossible was **retired**: every data type now uses `&'static str` and `&'static [T]`, so each service is `catalog/src/services/<id>.rs` exposing **`pub const SERVICE: Service`** (the aggregate bundling `id`/`observed`/`definition`/`runtime`), each page a `pub const PAGE: Page`, and journeys `pub const JOURNEYS`. `Deserialize` was dropped (the Rust source is the only input; JSON is an export). See [Typed const-catalog model](#typed-const-catalog-model-post-refactor).

### Dependencies (approved list)

```toml
schemars = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
# inventory = "0.3"   # only if compile-time registration proves worthwhile
```

### Export / command targets

| Command / API | Output |
|---------------|--------|
| `Catalog::validate()` | CI gate (invariants + coverage) |
| `extract` (bin) | Regenerate `observed/` from repo |
| `drift` (bin) | CI gate: asserted vs observed |
| `export json` | Agent input |
| `export --schema` (schemars) | LLM output validation |
| `coupling_matrix()` | Clustering proposals |
| `export mermaid` | Human-readable edge graph |

---

## Typed const-catalog model (post-refactor)

A late refactor (architect-requested) made the whole catalog **compile-time known and fully typed** — the compiler, not string matching, now guarantees referential integrity:

- **All IDs are enums.** `ServiceId`, `CapabilityId`, `JourneyId`, `PageId` are `#[serde(rename=…)]` enums (no more `ServiceId("mavlink2rest".to_string())`). They share an `Entity` trait (`const ALL`, `as_str()`). A page consuming a backend uses `ConsumeTarget::Service(ServiceId::X)` or `ConsumeTarget::External` — a typo is a compile error.
- **All data is `const`/`&'static`.** `String`→`&'static str`, `Vec<T>`→`&'static [T]`; builders became `pub const` items. `Deserialize` was removed (JSON is export-only).
- **`Service` aggregate.** `service.rs` bundles `Service { id, observed, definition, runtime }`; each service module is one `pub const SERVICE`, collected into `SERVICES: &[Service]`. `Catalog` holds `Vec<Service>` and exposes `definition_by_id`/`observed_by_id`/`runtime_by_id` through it.
- **Central capability registry.** `capability.rs` holds the `Aggregate` enum, `CAPABILITIES` (backend `CapabilityDef{id,aggregate,owner:ServiceId}`), and `FRONTEND_CAPABILITIES` (`FrontendCapabilityDef{id,aggregate,owner:PageId}`) — the single place that classifies every capability.
- **Typed `Feature.origin`.** `Feature { id: FeatureId(CapabilityId), aggregate: Aggregate, origin: Origin, rationale }` where `Origin = BackendService(ServiceId) | FrontendPage(PageId)`. The inventory is **143 features** = 129 backend + 14 frontend-origin; `FeatureCatalog::validate()` enforces the split (a page feature must be registered; no frontend id may collide with the backend registry).

---

## Current BlueOS landscape (calibration reference)

> **Seed observations, pending automated verification.** The `extract` binary regenerates this table; it is shown here to calibrate the extractor and to make the naming/route drift concrete. Note how `tmux` / `dir-or-binary` / `nginx prefix` frequently disagree.

**Priority tier** (`PRIORITY_SERVICES`, started first, then `sleep 5`):

| tmux | dir / binary | nginx prefix(es) | port | notes |
|------|--------------|------------------|------|-------|
| `autopilot` | `ardupilot_manager/main.py` | `/ardupilot-manager/`, `/autopilot-manager/` | 8000 | MAVLink router owner; `nice --19` |
| `cable_guy` | `cable_guy/main.py` | `/cable-guy/` | 9090 | network ifaces |
| `video` | `mavlink-camera-manager` (binary) | `/mavlink-camera-manager/`, `/webrtc/ws/` | 6020, 6021 | MAVLink `tcpout:127.0.0.1:5777`; zenoh; external recorder |
| `mavlink2rest` | `mavlink2rest` (binary) | `/mavlink2rest/` | 6040 | `--connect=udpout:127.0.0.1:14001` |

**Normal tier** (`SERVICES`):

| tmux | dir / binary | nginx prefix | port | notes |
|------|--------------|--------------|------|-------|
| `kraken` | `kraken/main.py` | `/kraken/` | 9134 | extension platform; secondary venv |
| `wifi` | `wifi/main.py` | `/wifi-manager/` | 9000 | `--socket wlan0` |
| `zenohd` | `zenohd` (binary) | `/zenoh/`, `/zenoh-api/` | 7117, 7118 | bus broker |
| `beacon` | `beacon/main.py` | `/beacon/` | 9111 | mem 250 |
| `bridget` | `bridget/main.py` | `/bridget/` | 27353 | runs as regular user |
| `commander` | `commander/main.py` | `/commander/` | 9100 | |
| `nmea_injector` | `nmea_injector/main.py` | `/nmea-injector/` | 2748 | |
| `helper` | `helper/main.py` | `/helper/` | 81 | secondary venv |
| `iperf3` | `iperf3` (binary) | — | 5201 | no nginx route |
| `linux2rest` | `linux2rest` (binary) | `/system-information/` | 6030 | |
| `filebrowser` | `filebrowser` (binary) | `/file-browser/` | 7777 | `--baseurl /file-browser` |
| `versionchooser` | `versionchooser/main.py` | `/version-chooser/` | 8081 | secondary venv |
| `pardal` | `pardal/main.py` | `/network-test/` | 9120 | **triple name drift** |
| `ping` | `ping/main.py` | `/ping/` | 9110 | runs as regular user |
| `user_terminal` | `cat /etc/motd` | — | — | tmux session host for ttyd |
| `ttyd` | `ttyd` (binary) | `/terminal/` | 8088 | web terminal |
| `nginx` | `nginx` (binary) | (serves all) | 80, 2770 | reverse proxy |
| `bag_of_holding` | `bag_of_holding/main.py` | `/bag/` | 9101 | |
| `recorder` | `blueos-recorder` (binary) | — | — | writes `userdata/recorder` |
| `recorder_extractor` | `recorder_extractor/main.py` | `/recorder-extractor/` | 9150 | range requests |
| `disk_usage` | `disk_usage/main.py` | `/disk-usage/` | 9151 | |
| `customization` | `customization/main.py` | `/customization/` | 9152 | `.glb` uploads |

**Observations that break a naive `services/`-keyed WBS:**
- 16 dirs in `core/services/`, but **26 runtime processes**; ~10 are binaries with no service dir yet own critical ports/routes.
- Keying extraction on the **process list + nginx routes**, not the `services/` directory, is mandatory.

**Cross-cutting:** MAVLink routing, Zenoh (`blueos-zenoh.json5`), nginx reverse proxy, `/usr/blueos/userdata`, cgroup limits, secondary venv, `nice` priorities.

---

## Clustering rigor (M3–M4)

"Mentally run a cluster analysis" is the subjective step we are eliminating — do not smuggle it back in.

- **Explicit edge weights per bus.** A shared exclusive hardware resource ≫ an occasional REST call. Weights are committed and versioned.
- **Multiple named policies** run mechanically: `coupling-only`, `coupling+trust`, `coupling+team`. Each yields a boundary proposal.
- **Stability metric.** Perturb weights and report how often two services co-cluster (pairwise co-occurrence) plus per-policy modularity. Boundaries that hold under only one weighting are not real boundaries.
- **Output ≥3 proposals**, never one "answer"; humans score against quality attributes + ADRs.

---

## Milestones

### M0 — Harness scaffold (this branch)
- [x] `blueos-catalog` crate with core types incl. `Observed<T>` / `Unknown{reason}` / `PortRef`
- [x] `Catalog`, `validate()` (invariants + coverage), `export json` + `--schema`
- [x] `extract` + `drift` binaries (real, not stubs — Phase 2)
- [x] Skill files for the 5 agent profiles under `.cursor/skills/` (+ journey/runtime/capture skills)
- [x] Catalog rules under `.cursor/rules/` (two-layer, rust, journeys, runtime, capture-tools)
- [x] CI test: empty/minimal catalog validates; schema round-trips (all tests green via `gate.sh`)

### M1 — Calibration (reference services + frozen rubric) — DONE
- [x] Registration plumbing: `services/mod.rs` + `Catalog::bootstrap()`; compiling `ardupilot_manager` skeleton
- [x] Fact Extractor authors provenance-backed `observed_facts()` for `ardupilot_manager` and `kraken` (M2 `extract` oracle)
- [x] Full asserted `service_definition()` for both (MAVLink router owner; Docker/extensions/Zenoh/jobs)
- [x] Drift gate green; exclusive-resource-uniqueness validation
- [x] Inter-rater pass → **froze extraction rubric v1.0 + skills** (Phase 2 item 4)

### M1.5 — Journey spine (top-level, triangulated) — DONE
- [x] Crate Engineer: `journey.rs` types (`UserJourney`, `JourneyStep`, `RouteRef`, `Actor`, `Visibility`, `Precondition`, `StateTransition`), `Provenance`/`Grounded`/`GroundedSet`, `id::JourneyId`; `Catalog.journeys` + `bootstrap`; `validate()` cross-refs (service/route/capability/state/`chains_from`); `service.journey_refs` projection
- [x] QA Reviewer: crate change (compiles, `validate()` cross-refs tested, schema round-trips)
- [x] Docs Specialist: `ardupilot_manager` journeys from `../BlueOS-docs` (doc-grounded skeletons + route hints)
- [x] Fact Extractor: confirm each step route → `Source`; Runtime: fill outcomes from **live BlueOS Pi** (NOT the POC — runtime values come only from live capture, per the resolved guardrail)
- [x] Card Author: `capability_refs` + `journey_refs`; triangulated `ardupilot_manager` reference journeys → QA
- [x] Fold remaining findings into rubric freeze: manifest-cache resource, `/v1.0` version path, error-semantics-per-state, SLO/perf baselines, settings schema, platform matrix

### M2 — Coverage + extractor automation — DONE
- [x] Real `extract` binary: scans `start-blueos-core` → self-checks observed facts; reproduces the M1 calibration artifacts (oracle)
- [x] Observed + asserted + **runtime** cards for **all 26 processes** (from `start-blueos-core` + nginx + live Pi)
- [x] External binaries first-class (`mavlink2rest`, `mavlink-camera-manager`, `linux2rest`, `zenohd`, `blueos-recorder`, `filebrowser`, `ttyd`, `iperf3`, `nginx`)
- [x] **Edge graph connects known static HTTP/MAVLink/Zenoh links** — edges-pass complete, **zero `edges: unknown` remain**. Concrete: mavlink2rest→ardupilot_manager, mavlink-camera-manager→ardupilot_manager, recorder→recorder_extractor (File), ttyd→user_terminal (Subprocess), helper→{versionchooser, mavlink2rest}, bridget→linux2rest, ping→mavlink2rest, nmea_injector→mavlink2rest (all Rest), **ardupilot_manager→zenohd** (Zenoh MAVLink bridge, port 7117). The 10 services with no outbound catalog coupling are now `established(vec![])` (determined-empty, distinct from Unknown — needed for the M3 coupling matrix).

### M3 — Agent tooling — DONE
- [x] JSON Schema export (`export --schema`) + `export json` + grouped-subgraph `export mermaid` + `export_proposals_json`
- [x] Drift detector implemented (`drift` bin) and run via `gate.sh` (no CI runner in this repo yet)
- [x] **Clustering helper: real `coupling_matrix(policy)` + committed weights (`WEIGHTS_VERSION="v1"`) + stability export** — `src/cluster.rs` + `src/bin/cluster.rs`. Weighted undirected coupling from established edges (per-bus × failure-impact + boot bonus) + shared non-exclusive resources + policy affinity; 3 named policies (`CouplingOnly`/`CouplingTrust`/`CouplingDomain`); greedy CNM modularity clustering (deterministic); `cluster_stability()` (seeded perturbation → co-occurrence + unstable pairs); `boundary_proposals()` → exactly 3 candidate partitions (NOT the answer). CouplingOnly Q≈0.245; reproducible; 33 tests; no new deps.

### F1 — Feature-first 2.0 redesign (inverts the model) — VIEWS A+B DONE
> Pivot requested by the architect: **don't re-cluster the 26 services — forget the current feature↔service map and author a new one.** Features (capabilities/journeys) are the durable SSOT; services are a rebindable projection.
- [x] `src/feature.rs`: grounded capabilities → service-agnostic `Feature { id, aggregate, origin, rationale }`, built mechanically from the catalog (none lost/invented; `validate()` enforces 1:1). `src/bin/features.rs`. *(Later typed: `origin: Origin{BackendService|FrontendPage}`; inventory grew to 143 with the F2 frontend merge.)*
- [x] **View A (aggregate-first):** features grouped by the entity they act on → 19 aggregates.
- [x] **View B (journey-first):** feature co-occurrence from journey `capability_refs` (+ `chains_from`) → 7 workflow communities (Q≈0.238); 47 features touched by no journey (read-only/status/mavlink/infra).
- [x] **A-vs-B divergence report:** where "same thing" (A) and "same workflow" (B) disagree = the 2.0 boundary tensions.
- [ ] **NEXT (after F2):** fold the ~19 aggregates / 7 workflow communities into a handful of candidate **2.0 bounded contexts**, then declare the new **feature→2.0-service** binding (some 1.x services split, some merge, infra dissolves into a platform layer). This is the human decision the two views feed.

### F2 — Frontend as a first-class subject (fills the biggest blind spot) — DONE
> Architect insight: BlueOS 1.x implements whole journeys **in the frontend** (calibration wizards, motor detection, param editing), holds **domain state in the browser**, and pages fan out to *several* services — none of which the backend-only model saw. The unit is the **Page** (a router route), not the 208 components.
- [x] `src/page.rs`: `Page { route/name/component/menu_title/advanced_only/stores/consumes (Observed) + frontend_features/client_state (Asserted) }`; `PageServiceCall`, `ClientState`, `StateOwnership {BackendOwned, FrontendOwned, Shared}`. Wired into `Catalog.pages` + `validate()` (page→service targets must be cataloged or `external`; no dup `PageId`).
- [x] **Frontend Extractor** agent: `.cursor/skills/blueos-frontend-extraction/SKILL.md` + rule `.cursor/rules/blueos-catalog-frontend.mdc`. Observed fields cite `core/frontend/src/...:LINE`; asserted fields (client-implemented features + state ownership) carry a rationale.
- [x] **Rubric frozen on a 3-page calibration set** (all QA-accepted, Page type unchanged across all three): `vehicle_setup` (calibration hub — 25 consume edges, 9 client-implemented features, 13 client-state entries incl. FrontendOwned calibration progress + Shared param-derived "is calibrated?"), `video_manager` (streaming — 12 edges to mavlink-camera-manager + commander, 5 features, backend-mirrored lists), `disk` (simple — backend-driven, `frontend_features` intentionally empty). Frozen conventions recorded in the skill.
- [x] **Scaled to all 24 pages** (4 Frontend Extractor batches + independent per-batch QA). QA caught six pages that mislabeled backend-owned capabilities as `frontend_features`; the rule now carries a mechanical registry test to prevent recurrence.
- [x] **Merged frontend features into the inventory** via typed `Feature.origin = Origin::{BackendService(ServiceId), FrontendPage(PageId)}`; the 14 frontend-origin capabilities are registered in `FRONTEND_CAPABILITIES` (calibration/param → `Autopilot`, streams → `Camera`). `FrontendOwned` client-state is now catalog-queryable per page (the direct 2.0 re-home input).
- [ ] Deferred type work (only if coupling analysis needs it): typed `consume.trigger` (page-triggered vs global-background) and `endpoint.protocol` tag.

### M4 — Architecture decisions (human)
- [ ] Event storm / journey workshop output → ADRs
- [ ] ≥3 boundary proposals from clustering policies
- [ ] Strangler migration slices (downstream, not in this crate)

---

## Orchestrator agent instructions

When picking up this branch:

1. Read this plan and `blueos-catalog/` (once it exists).
2. Implement **M0** first — harness (types + extractor + drift + skills) before any service data.
3. This crate is **greenfield Rust** (no existing Rust in BlueOS today). Use **top-down declaration order** (primary types first); run `cargo fmt` + `cargo clippy -D warnings` + `cargo test` after Rust edits.
4. **Spawn composer-2.5 workers** for extraction and card authoring, one process per task, in parallel where independent. **QA every output** against its acceptance gate; never author a card you will QA.
5. Enforce **ground-truth-first**: no Card Author before the matching observed artifact passes its gate.
6. Do **not** rewrite Python services on this branch.
7. Prefer **minimal diffs**; one process (or one milestone) per PR. Commit style per repo convention.
8. After M1, freeze the rubric and export sample JSON for human review.

### Suggested first PR scope

`blueos-catalog` crate skeleton + core types (`Observed`/`Unknown`/`PortRef`) + `validate()` (invariants + coverage) + `extract`/`drift` stubs + the 5 skill files + the 2 catalog rules. No service data yet.

---

## Open questions — resolved

- **Where does the crate live?** Repo root `catalog/` (or `blueos-catalog/`). It is analysis tooling, not shipped runtime — keep it out of `core/`.
- **`service!` macro?** Defer to M1+. `const` + slice registry for M0.
- **Drift detector home?** Rust binary (`src/bin/drift.rs`) in the same crate — shares types, one CI job.
- **Minimal vs full card?** Neither. `Unknown { reason }` + a coverage threshold enforced by `validate()`.
- **Compile-time registration (`inventory`/`linkme`)?** Deferred; plain slice registry unless it earns its keep.

## Still open

- [ ] Coverage threshold value (max `Unknown` on required fields) — deferred with rationale (Phase 3 audit); the per-field `Unknown{reason}` discipline + drift gate already enforce depth.
- [x] Exact edge weights per bus — committed as `WEIGHTS_VERSION="v1"` in `src/cluster.rs` (tunable defaults; humans re-score during M4).
- **Remaining = M4 (human):** event-storm/journey workshop → ADRs; score the 3 clustering proposals against quality attributes; pick target boundaries + strangler slices. Agents have produced the inputs; the decision is human.

---

## Links

- Startup truth: `core/start-blueos-core`
- Routing truth: `core/tools/nginx/nginx.conf`
- Zenoh config: `core/tools/zenoh/blueos-zenoh.json5`
- Zenoh helper: `core/libs/commonwealth/src/commonwealth/utils/zenoh_helper.py`
- Reference PR for new services: bluerobotics/BlueOS#3669 (disk_usage)

---

## Conversation summary

**Session focus:** Planning BlueOS 2.0 service reorganization methodology and its agentic harness.

1. User proposed WBS per service → flatten → cluster by responsibility. We identified flaws (implicit coupling, static docs, extensions, safety/trust gaps) and listed classic complements (event storming, DDD, coupling analysis, ADRs, strangler migration).

2. We designed a rich **service view schema** with capabilities, authorities, typed interfaces, states, resources, unified edges, lifecycle, trust, failure, and BlueOS-specific checklists.

3. User preferred **Rust types as SSOT** over TOML. We outlined the `blueos-catalog` crate structure, key enums/newtypes, validation, export for agents, and `const`/`service!` declaration patterns.

4. Grounding review against the repo revealed real drift (e.g. `pardal`→`/network-test/`→`:9120`; 26 runtime processes vs 16 service dirs; double route `/ardupilot-manager/`+`/autopilot-manager/`). This motivated the **two-layer model** (observed/extracted vs asserted/authored), **drift detection as the correctness engine**, and **anti-drift process controls** (frozen rubric, provenance, coverage gate, calibration + inter-rater reliability, harness-first).

5. User set the operating model: **Orchestrator (Opus) spawns composer-2.5 workers and QAs their work**. Phase 1 is precision engineering to build a full agentic harness — each step has an **agent profile** (Crate Engineer, Fact Extractor, Card Author, Clustering Analyst) and a **skill** file, gated by the orchestrator QA protocol.
