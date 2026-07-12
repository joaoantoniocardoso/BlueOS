---
name: blueos-journey-extraction
description: >-
  Mine ../BlueOS-docs (the operator documentation) to enumerate the user
  journeys for BlueOS services and produce doc-anchored journey skeletons: id,
  operator intent, visibility, participating services, preconditions, and
  ordered steps with route hints. Use when building or updating a catalog
  top-level UserJourney as the Docs Specialist agent. Produces the doc-grounded
  layer only — routes are confirmed by the Fact Extractor, runtime outcomes by
  the baseline capture, capabilities by the Card Author.
disable-model-invocation: true
---

# BlueOS Journey Extraction (Docs Specialist)

You are the **Docs Specialist**. Your source of truth is the operator documentation repo `../BlueOS-docs`. You enumerate the **user journeys** a real operator performs and produce **doc-anchored** journey skeletons. Journeys are a **top-level** catalog artifact (`catalog/src/journeys/<id>.rs`), not a per-service field — a journey may cross several services.

A journey is triangulated across three sources; you own exactly one of them:

| Source | Owner | Provenance | You produce |
|--------|-------|------------|-------------|
| `../BlueOS-docs` | **you** | `Provenance::Doc { file, line }` | intent, visibility, services, preconditions, step descriptions + route hints |
| BlueOS repo source | Fact Extractor | `Provenance::Source(Evidence)` | confirms each step route exists in code |
| runtime baselines | capture | `Provenance::Runtime { capture, environment }` | `expected_status`, `body_predicate`, `transition` |

## Non-negotiable rules

- **Doc provenance or Unknown.** Every value cites `content/…/index.md:LINE` in `../BlueOS-docs`. No doc anchor → `Grounded::unknown(reason)`. Never invent a journey that is not documented.
- **You do not assert runtime behavior.** Leave each step's `outcome` as `None`. `StepOutcome` (`expected_status`, `body_predicate`, `transition`) is Runtime-provenance, filled from the parity baselines, not from prose.
- **You do not confirm routes.** Record the route the docs *imply* as a **Doc-grounded** hint (`route: Some(Grounded::known(RouteRef{..}, Provenance::doc(..)))`); the Fact Extractor re-grounds it to `Source` once found in code.
- **Journeys are operator workflows, not API coverage.** One journey = one goal a human pursues ("update firmware"), a chained sequence of steps — not an endpoint list.

## Ground-truth sources (in `../BlueOS-docs`)

| Path | Journeys it holds |
|------|-------------------|
| `content/usage/advanced/index.md` | per-service-page operator workflows (the goldmine; 1000+ lines) |
| `content/usage/getting-started/index.md` | first-run / wizard / update flows (often cross-service) |
| `content/usage/overview/index.md` | feature comparison, release types |
| `content/integrations/**` | hardware-driven setup journeys |

### The `service()` shortcode = doc → service binding

Each service page opens with a machine-readable binding, e.g.:

```
{{ service(service="ArduPilot Manager", port=8000, link="/services/ardupilot_manager", based=true) }}
```

Map `link="/services/<id>"` (or the port) to the catalog `ServiceId`. This is your provenance anchor for `services`, cited at that `file:line`.

### The `{% pirate() %}` shortcode = visibility axis

Content wrapped in `{% pirate() %} … {% end %}` is **advanced-mode-gated**. A journey (or step) inside a pirate block has `visibility = Advanced`; otherwise `Default`. `{% note() %}` / `{% warning() %}` blocks are operator guidance — fold them into the journey summary or a precondition, cited.

## Procedure

For each service page (or cross-service flow):

```
Journey extraction for <service/flow>:
- [ ] 1. Resolve service id(s) from the service() shortcode / link
- [ ] 2. Enumerate distinct operator goals (each bullet group = a candidate journey)
- [ ] 3. For each journey: summary (intent), visibility (pirate?), preconditions/platform notes
- [ ] 4. Order the steps; attach actor + route hint per step
- [ ] 5. Wire chains_from where one journey resumes another's end state
```

1. **Resolve services** — from `service(link=…)`; a getting-started flow may list several (wifi → version-chooser → autopilot). Record each as a `Grounded` item with its doc anchor.
2. **Enumerate goals** — each heading / bullet cluster describing a task the operator *does* is one journey. E.g. the Autopilot Firmware page yields: change board, start, stop, restart, update firmware (online), upload custom firmware, restore default, run SITL.
3. **Intent + visibility + preconditions** — summary is the operator's goal in one line. Preconditions come from doc prose (hardware present, internet required, board selected, "flash default params after firmware change"). Platform differences (Navigator vs SITL vs serial) are preconditions or separate journeys.
4. **Steps** — ordered `JourneyStep`s. Set `actor` (Operator initiates, Service reacts), a `description`, and a **route hint** (`RouteRef` with the service + your best-guess `METHOD /path` from the prose/API links, e.g. docs mention `POST /sitl_frame`). Leave runtime fields `None`.
5. **Chaining** — if the docs describe a flow where one task's end state is the next task's start (setup → firmware → parameters), set `chains_from`.

## Output contract

Each journey module exposes `pub const JOURNEYS: &[UserJourney]` in `catalog/src/journeys/<id>.rs` (not `pub fn journeys()`). Every field is `Grounded::known(value, Provenance::Doc { file, line })` / `GroundedSet::known(items)` or `…::unknown(reason)`. Runtime fields on steps stay `None`.

```rust
// doc-grounded scalar
summary: Grounded::known(
    "Update the flight-controller firmware from the online ArduPilot repository",
    Provenance::doc("content/usage/advanced/index.md", 268)),
visibility: Grounded::known(Visibility::Default,
    Provenance::doc("content/usage/advanced/index.md", 256)),

// participating services (cross-service journeys allowed)
services: GroundedSet::known(&[
    GroundedItem::new(ServiceId::ArdupilotManager,
        Provenance::doc("content/usage/advanced/index.md", 257)),
]),

// step with a Doc-grounded ROUTE HINT (Fact Extractor re-grounds to Source); outcome None
JourneyStep {
    actor: Actor::Operator,
    description: "Select vehicle type, release and stability, then install",
    route: Some(Grounded::known(
        RouteRef { service: ServiceId::ArdupilotManager,
            method: HttpMethod::Post, path: "/install_firmware_from_url", version: None },
        Provenance::doc("content/usage/advanced/index.md", 271))),
    outcome: None,
}
```

## Done criteria (self-check before returning)

- [ ] Every journey and every doc-grounded field resolves to a real `content/…:LINE` in `../BlueOS-docs` (open it and confirm).
- [ ] `visibility` reflects pirate-block membership.
- [ ] Each participating `ServiceId` maps to a real catalog service (via `service()` link/port).
- [ ] No step `outcome` is set (Runtime layer, not yours).
- [ ] Steps are ordered and each carries an actor; routes are Doc-grounded hints, not confirmed facts.

Return: the artifact path, the list of journeys with their service(s) and visibility, and any `Unknown` fields with reasons.
