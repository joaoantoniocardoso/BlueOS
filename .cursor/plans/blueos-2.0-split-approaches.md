# BlueOS 2.0 — Service Split: Multiple Approaches + Consensus

**Branch:** `2.0-dev/model`
**Status:** M4 input (human decision). **Generated from the catalog; these are candidate signals, not boundary decisions.**
**Regenerate:** `cargo run --quiet --bin cluster` (deterministic). Feature-first views: `cargo run --quiet --bin features`.

The split is exercised through **five independent lenses** over the same 26 services, then reconciled by a **cross-lens consensus**. Different lenses encode different notions of "belongs together"; boundaries that survive *multiple* lenses are the defensible ones.

---

## The five lenses

| Lens | Signal | "Together" means… |
|---|---|---|
| `coupling_only` | structural edges (calls, shared resources) | services wired to each other |
| `coupling_trust` | + privilege-level affinity | wired **and** same trust tier |
| `coupling_domain` | + declared `bounded_context` affinity | wired **and** same asserted domain |
| `page_cooccurrence` | frontend pages' `consumes` | the operator uses them **on the same page** |
| `journey_cooccurrence` | journeys' `services` + step routes | they **collaborate in one workflow** |

The first three are variations of the structural edge graph (M3). The last two are new — they exploit the F2 (pages) and F1/F3 (journeys) layers, capturing *usage* and *workflow* coupling that the edge graph cannot see.

---

## Per-lens partitions (groups of ≥2; the rest are singletons)

### `coupling_only` — Q ≈ 0.245 (16 groups)
```
ardupilot_manager, mavlink-camera-manager, zenohd
helper, kraken, versionchooser
mavlink2rest, nmea_injector, ping
bridget, linux2rest
cable_guy, wifi
recorder, recorder_extractor
ttyd, user_terminal
```

### `coupling_trust` — Q ≈ 0.008 (18 groups)
```
ardupilot_manager, mavlink-camera-manager, mavlink2rest, ping
bridget, linux2rest
cable_guy, wifi
kraken, versionchooser
recorder, recorder_extractor
ttyd, user_terminal
```
Q collapses to ~0 because **almost every service runs as root** — trust affinity connects nearly everything, so it stops discriminating. That is itself a finding for M4 (privilege is not a useful boundary axis today).

### `coupling_domain` — Q ≈ 0.245 (16 groups)
Identical to `coupling_only` here (the asserted `bounded_context` tags don't yet pull services apart from their structural grouping).

### `page_cooccurrence` — Q ≈ −0.025 (18 groups)
```
bag_of_holding, kraken, zenohd
commander, customization, filebrowser
mavlink-camera-manager, mavlink2rest, versionchooser
ardupilot_manager, bridget
linux2rest, ping
```

### `journey_cooccurrence` — Q ≈ −0.080 (23 groups)
```
beacon, commander, mavlink-camera-manager
ardupilot_manager, mavlink2rest
```

**Negative modularity on the two co-occurrence lenses is the headline finding.** Pages and workflows do **not** decompose into clean communities: a handful of hub services (`mavlink2rest`, `mavlink-camera-manager`, `ardupilot_manager`) co-occur with almost everything, so the graph is a hub-and-spoke, not a set of islands. Interpretation for M4: BlueOS's operator experience is organized around a **shared MAVLink/data plane**, not around separable per-domain stacks. A 2.0 split must decide deliberately how to treat those cross-cutting hubs (shared kernel? published-language integration service?) rather than expecting clustering to hand over the answer.

---

## Cross-lens consensus (5 lenses, majority ≥ 3)

**Pairs agreed on by a majority (3/5) — the robust "keep together" dyads:**

| Pair | Agreement | Why it's stable |
|---|---|---|
| `ardupilot_manager` ↔ `mavlink-camera-manager` | 3/5 | co-tuned via MAVLink (5777 pairing) + used together on vehicle pages |
| `bridget` ↔ `linux2rest` | 3/5 | bridget enumerates serial ports through linux2rest |
| `cable_guy` ↔ `wifi` | 3/5 | shared `/etc/dhcpcd.conf` + dnsmasq (networking) |
| `kraken` ↔ `versionchooser` | 3/5 | both drive Docker via `/var/run/docker.sock` |
| `mavlink2rest` ↔ `ping` | 3/5 | ping forwards distance as MAVLink through mavlink2rest |
| `recorder` ↔ `recorder_extractor` | 3/5 | producer→consumer over the shared recording dir |
| `ttyd` ↔ `user_terminal` | 3/5 | ttyd attaches the user_terminal tmux session |

**Consensus clusters (majority-agreed pairs → connected components):** the 7 dyads above, plus 12 singletons. No dyad grows into a triad at the majority threshold — i.e. the only boundaries all lenses agree on are these tight pairs; everything larger is lens-dependent and is a genuine M4 judgement call.

**Weaker signals (2/5)** worth discussing but not settled: `ardupilot_manager↔mavlink2rest`, `ardupilot_manager↔zenohd`, `helper↔kraken`, `helper↔versionchooser`, `mavlink-camera-manager↔mavlink2rest`, `mavlink-camera-manager↔zenohd`, `mavlink2rest↔nmea_injector`, `nmea_injector↔ping`.

---

## Feature-first views (orthogonal, capability-level — from `features` bin)

The lenses above split *services*. The feature model splits *capabilities* independently of today's services:

- **View A — aggregates (19):** capabilities grouped by the entity they act on (e.g. `autopilot` 15, `branding_ui` 14, `camera` 11, `extensions` 8, `files_kv` 6…). This is the entity-first target for 2.0 bounded contexts.
- **View B — journey communities (7):** capabilities grouped by co-occurrence in journeys (Q ≈ 0.238).
- `view_divergence()` surfaces where A and B disagree — candidate seams to examine in the event-storm.

Use A/B to propose *what the 2.0 services should be*, and the five service lenses + consensus to understand *how the current 26 would have to be cut/merged to get there*.

---

## How to use this in M4

1. **Lock the 7 consensus dyads** as near-certain co-location (or deliberately justify splitting any of them).
2. **Decide the hub treatment** for `mavlink2rest` / `mavlink-camera-manager` / `ardupilot_manager` — the negative-Q result says they can't be cleanly assigned; pick shared-kernel vs integration-service explicitly.
3. **Adjudicate the 2/5 pairs** against quality attributes (deploy independence, blast radius, team ownership).
4. **Bind features → 2.0 services** using View A as the primary target and View B / divergence as a cross-check.
5. Record each boundary decision as an **ADR**.

Nothing here is a decision — the algorithms produced the inputs; the cut is human.
