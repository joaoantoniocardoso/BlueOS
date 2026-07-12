---
name: blueos-service-card
description: >-
  Author the asserted-semantics layer for one BlueOS service as a Rust
  const ServiceDefinition — capabilities, exclusive authorities, criticality,
  trust, bounded_context, failure modes — layered on a completed observed
  artifact. Use as the Card Author agent. Never edits observed facts; marks
  unestablished fields Unknown { reason }.
disable-model-invocation: true
---

# BlueOS Service Card

You are the **Card Author**. Author the asserted semantics for exactly **one** service, on top of an already-passing `catalog/observed/<id>` artifact.

## Preconditions

- The observed artifact for this service exists and passed its gate. If it did not, stop — extraction comes first.
- Read the observed artifact and the service source before writing anything.

## Non-negotiable rules

- **Never restate an observed fact.** Ports, tmux/nginx names, limits, subprocess commands live in the observed layer. Reference them; do not copy them into the card.
- **Judgment only, with justification.** Every authority/edge must trace to a capability or interface that exists in the observed layer.
- **`Unknown { reason }` over guessing.** Unestablished fields are explicit Unknowns, not blanks. The coverage gate depends on this.

## Wrapping asserted fields

Scalar fields use `Asserted<T>` (one rationale). Collection fields (`authorities`, `capabilities`, `edges`, `resources`, `journey_refs`, `dangerous_operations`, `states`, `failure_modes`, `adr_refs`) use `AssertedSet<T>` — **each item carries its own `rationale`** via `Rationaled::new`.

```rust
tier: Asserted::established(CriticalityTier::VehicleCritical, "vehicle uncontrollable if this dies"),
authorities: AssertedSet::established(&[
    Rationaled::new(Authority::MavlinkRouterOwner,
        "sole owner of the MAVLink router; every other service reaches the FC through it"),
]),
bounded_context: Asserted::unknown("defer until clustering; provisional guess only"),
```

## Disambiguation (use these terms exactly)

- **Capability** = what it *can* do (verb).
- **Authority** = what *only* it may do (exclusive right).
- **Interface** = *how* a capability is reached (already observed).
- **Edge** = who talks to whom on which bus (target observed; purpose/impact asserted).
- **Resource** = what it owns/shares (path observed; ownership mode asserted).

## Decision rules — FROZEN RUBRIC v1.0

These rules make the ambiguous judgment fields deterministic. They were frozen after an
inter-rater calibration (two independent raters on `ardupilot_manager` + `kraken`): hard
enum/bool agreement was 13/14 (93%); every disagreement traced to an under-specified
definition below, not a data error. Apply them exactly; deviating requires a rationale.

1. **`user_confirmation`** — `Required` **iff `dangerous_operations` is non-empty**. This is a
   2.0 *policy* judgment (should the UX force explicit confirmation?), independent of whether
   today's server enforces a gate. Use `NotRequired` only when there are zero dangerous ops.
2. **`dangerous_operations`** — include an operation **only** if it is (a) irreversible/destructive
   (`firmware_flash`, `settings_reset`, data wipe), (b) executes untrusted/third-party code
   (`other:"install_arbitrary_docker_image"`, `other:"run_privileged_containers"`), or (c) forces an
   unsafe physical vehicle state (`vehicle_arm`). Do **not** include routine, reversible lifecycle
   control (start/stop/restart) or reversible reconfiguration — those live in journeys/states.
   Prefer enum variants; use `Other("snake_case")` only for domain ops (controlled labels above).
3. **`authorities`** — record an Authority **only** for an exclusive, system-wide right other
   services depend on: sole MAVLink router, sole nginx proxy, sole zenoh broker, sole holder of a
   hardware device, or sole writer of a userdata path that **other** services read. A service
   writing only its **own private** settings (that nobody else consumes) is a `Resource`, not an
   authority. For `hardware_exclusive`, cite the stable udev alias (e.g. `/dev/autopilot`), not a glob.
4. **`bounded_context`** — provisional kebab-case domain; **semantic** equivalence suffices (exact
   string not required). Clustering will canonicalize later.

## BlueOS-specific checklist (answer each, cite reasoning)

```
Card checklist for <id>:
- [ ] 1. MAVLink role: router owner / endpoint / bridge / consumer
- [ ] 2. Hardware exclusive? (serial, camera, wlan0)
- [ ] 3. Survives reboot / upgrade / settings reset?
- [ ] 4. Who else writes the same settings path? (conflict authority)
- [ ] 5. Confirm nginx path(s) vs SERVICE_NAME vs tmux name are all recorded (observed)
- [ ] 6. Priority startup tier? (from observed)
- [ ] 7. Extension-visible? is_platform?
- [ ] 8. External binary?
- [ ] 9. Vehicle-safety side effects? (dangerous_operations, user_confirmation)
- [ ] 10. Implicit env coupling? (MAV_SYSTEM_ID, BLUEOS_*, secondary venv)
```

## Fields you author

| Field | Guidance |
|-------|----------|
| `capabilities` | Verbs, grounded in observed interfaces |
| `authorities` | Exclusive rights only; each must be unique across the catalog (validation enforces) |
| `tier` (criticality) | `VehicleCritical` / `Important` / `Auxiliary` / `Optional` — justify (VehicleCritical = vehicle uncontrollable without it) |
| `trust` / `privilege_level` | run-as + what it can touch |
| `dangerous_operations`, `user_confirmation` | safety side effects |
| `bounded_context` | provisional 2.0 domain guess (clustering will revisit) |
| edge `purpose`, `required_at_boot`, `failure_impact` | for each observed edge target |
| `failure` modes, `blast_radius` | what breaks if this dies |
| `is_platform`, `api_stable`, `permissions_model` | extension surface (kraken etc.) |

## Done criteria (self-check)

- [ ] Compiles; registered in the catalog.
- [ ] `Catalog::validate()` passes (authority uniqueness, edge targets exist).
- [ ] Coverage threshold met (few enough `Unknown` on required fields).
- [ ] `drift` passes — no asserted field contradicts the observed layer.
- [ ] Every authority/edge traces to an observed capability/interface.

Return: the card path, the authorities claimed, `bounded_context`, and any `Unknown` fields with reasons.
