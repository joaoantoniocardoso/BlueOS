# BlueOS 2.0 — Service Catalog Model

**Branch:** `2.0-dev/model`
**Status:** Planning / scaffolding
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
| **Fact Extractor** | composer-2.5 | Observed layer for one process (with provenance) | Make judgment calls; guess |
| **Card Author** | composer-2.5 | Asserted semantics for one service | Touch observed fields |
| **QA Reviewer** | composer-2.5 (fresh context) | Independent verification of one worker output → ACCEPT/BOUNCE verdict | Review work it produced; fix the artifact itself |
| **Clustering Analyst** | composer-2.5 | Multi-policy clustering + stability metrics | Decide final boundaries |

**QA independence is structural:** the QA Reviewer is a *separate* composer-2.5 instance with fresh context — never the author of the artifact, never the orchestrator. The orchestrator dispatches it, then adjudicates its verdict; it never performs the review itself. This removes both author bias and the orchestrator bottleneck.

### Skills (to be created under `.cursor/skills/`)

Each skill is a `SKILL.md` the worker reads first. They encode the precision rules so quality does not depend on prompt wording.

| Skill | Loaded by | Encodes |
|-------|-----------|---------|
| `blueos-catalog-crate` | Crate Engineer | Rust conventions, top-down ordering, `cargo fmt`/`cargo test`, type-design invariants (parse-don't-validate, `PortRef`, `Unknown`) |
| `blueos-service-extraction` | Fact Extractor | The frozen extraction protocol + provenance format + output contract |
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
| Contracts | OpenAPI refs **[O]**, compatibility policy **[A]** |
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
│   ├── id.rs               # ServiceId, CapabilityId, Port, PathRef, PortRef
│   ├── provenance.rs       # Evidence{file,line}; Observed<T>={value,evidence}|Unknown; Asserted<T>={value,rationale}|Unknown
│   ├── criticality.rs
│   ├── trust.rs
│   ├── interface.rs        # tagged enum Interface { Rest, Zenoh, Mavlink, ... }
│   ├── resource.rs
│   ├── state.rs
│   ├── edge.rs
│   ├── lifecycle.rs
│   ├── observed.rs         # ObservedFacts (extractor output shape)
│   ├── service.rs          # ServiceDefinition (asserted) + link to ObservedFacts
│   ├── catalog.rs          # Catalog + indexing + coupling_matrix()
│   ├── resolve.rs          # PortRef::Env → literal
│   ├── validate.rs         # cross-service rules + coverage gate
│   ├── drift.rs            # diff(asserted, observed)
│   ├── export.rs           # JSON, JSON Schema, mermaid
│   └── services/
│       ├── mod.rs
│       ├── ardupilot_manager.rs
│       └── kraken.rs
├── observed/               # GENERATED by `extract` — never hand-edited
└── src/bin/
    ├── extract.rs          # repo → observed/
    └── drift.rs            # asserted vs observed → CI gate
```

### Key design choices

1. **Parse, don't validate** — enums/newtypes; `Bus::Mavlink` cannot appear under REST.
2. **Definition vs resolved** — `PortRef::Literal(14001)` vs `PortRef::Env("MAV_SYSTEM_ID")`; `resolve()` pass for tooling.
3. **Observed vs asserted are distinct types** — `ObservedFacts` (generated) is separate from `ServiceDefinition` (authored); `drift.rs` reconciles them.
4. **Two provenance wrappers, `Unknown { reason }` first-class** — observed fields use `Observed<T>` (carries `Evidence{file,line}`); asserted fields use `Asserted<T>` (carries `rationale`, never `file:line`). Both have an `Unknown` arm feeding the coverage gate; no silent blanks.
5. **Unified edges** — replace separate callers/callees/producers/consumers lists.
6. **Registration** — start with a plain slice/`Vec` registry for M0; consider `inventory`/`linkme` only if it pays for itself later.
7. **Declaration ergonomics** — start with `const ServiceDefinition`; optional `service!` macro deferred to M1+.

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
- [ ] `blueos-catalog` crate with core types incl. `Observed<T>` / `Unknown{reason}` / `PortRef`
- [ ] `Catalog`, `validate()` (invariants + coverage), `export json` + `--schema`
- [ ] `extract` + `drift` binaries (may start as stubs that read the seed table)
- [x] Skill files for the 5 agent profiles under `.cursor/skills/`
- [x] Catalog rules under `.cursor/rules/` (`blueos-catalog-two-layer`, `blueos-catalog-rust`)
- [ ] CI test: empty/minimal catalog validates; schema round-trips

### M1 — Calibration (reference services + frozen rubric)
- [ ] `extract` produces real `observed/` for `ardupilot_manager` and `kraken`
- [ ] Full asserted cards for both (MAVLink router owner; Docker/extensions/Zenoh/jobs)
- [ ] Drift gate green; exclusive-resource-uniqueness validation
- [ ] Inter-rater pass on 2–3 services → **freeze extraction rubric + skills**

### M2 — Coverage
- [ ] Observed + asserted cards for **all 26 processes** (from `start-blueos-core` + nginx)
- [ ] External binaries first-class (`mavlink2rest`, `mavlink-camera-manager`, `linux2rest`, `zenohd`, `blueos-recorder`, `filebrowser`, `ttyd`, `iperf3`, `nginx`)
- [ ] Edge graph connects known static HTTP/MAVLink/Zenoh links

### M3 — Agent tooling
- [ ] JSON Schema export validated against sample LLM output
- [ ] Drift detector run in CI against the repo
- [ ] Clustering helper: `coupling_matrix` + weights + stability export

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

- [ ] Coverage threshold value (max `Unknown` on required fields) — set during M1 calibration.
- [ ] Exact edge weights per bus — set during M3.

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
