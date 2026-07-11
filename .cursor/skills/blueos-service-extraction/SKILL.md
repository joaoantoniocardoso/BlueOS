---
name: blueos-service-extraction
description: >-
  Extract the observed-facts layer for one BlueOS process (ports, tmux/nginx
  names, resource limits, subprocess/MAVLink/Zenoh/HTTP interfaces) directly from
  the repo, with file:line provenance on every field. Use when building or
  updating a catalog observed/<id> artifact as the Fact Extractor agent. Produces
  facts only — no judgment, no authorities, no tiers.
disable-model-invocation: true
---

# BlueOS Service Extraction

You are the **Fact Extractor**. Produce the observed-facts artifact for exactly **one** process. Every field is either `{ value, evidence }` or `Unknown { reason }`. No judgment fields.

## Non-negotiable rules

- **Provenance or Unknown.** Every value cites `file:line` or the exact shell command that produced it. No evidence → `Unknown { reason }`. Never guess or infer.
- **Facts only.** Do NOT set authorities, criticality, trust, `bounded_context`, blast radius, or failure modes. Those belong to the Card Author.
- **Key on the process, not the directory.** ~10 processes are external binaries with no `core/services/` dir but own critical ports/routes. Start from the `start-blueos-core` process tuple.
- Output is the `observed_facts() -> ObservedFacts` builder function in `catalog/src/services/<id>.rs`. Populate each field with `Observed::known(value, Evidence { file, line })` or `Observed::unknown(reason)`. (`catalog/observed/**` JSON is a generated export — do not write there.)
- Do NOT touch `service_definition()` in the same file — that is the Card Author's asserted layer.

## Ground-truth sources

| Source | Truth it holds |
|--------|----------------|
| `core/start-blueos-core` | tmux name, tier (PRIORITY vs SERVICES), mem/cpu/io limits, `nice`, run-as, exact command |
| `core/tools/nginx/nginx.conf` | nginx prefix(es) → port (may be many-to-one) |
| `core/tools/zenoh/blueos-zenoh.json5` | Zenoh broker config |
| `core/services/<id>/` (if present) | argparse port defaults, HTTP calls, MAVLink strings, settings paths |
| `core/libs/commonwealth/.../zenoh_helper.py` | Zenoh topic conventions |

## Procedure (frozen rubric)

Run each step; record `{ value, evidence }` or `Unknown { reason }`.

```
Extraction checklist for <id>:
- [ ] 1. Identity & deployment
- [ ] 2. Listener port(s)
- [ ] 3. nginx route(s) — ALL of them
- [ ] 4. MAVLink role
- [ ] 5. Zenoh topics
- [ ] 6. Hardware exclusivity
- [ ] 7. Settings / file paths
- [ ] 8. Subprocesses / external binaries
- [ ] 9. Outbound edges (implicit HTTP/Zenoh calls)
- [ ] 10. Env coupling
```

1. **Identity & deployment** — find the tuple in `PRIORITY_SERVICES` or `SERVICES` in `start-blueos-core`. Record tmux name, tier, `MEMORY_MB/CPU_PERCENT/IO_READ/IO_WRITE`, `nice` value, run-as (`RUN_AS_REGULAR_USER_*`), and the exact command. → `core/start-blueos-core:LINE`.
2. **Listener port(s)** — grep the main/command for `--port`, `--server`, `listen`, `argparse` defaults, or hardcoded ports.
3. **nginx route(s) + API version(s)** — grep `nginx.conf` for the `proxy_pass` matching the port; record **every** `location` prefix (e.g. `/ardupilot-manager/` and `/autopilot-manager/` both → `:8000`). → `core/tools/nginx/nginx.conf:LINE`. Also record the **API version prefix(es)**: grep for `VersionedFastAPI`/`prefix_format` (e.g. `"/v{major}.{minor}"` → `/v1.0`, `/v2.0`) and set `Interface::Rest { versions }`. **Cite the line that yields the URL prefix** — the `prefix_format="/v{major}.{minor}"` argument or a `@version(1, 0)` decorator — NOT the semantic `version="1.0.0"` string (that string is not the `/v1.0` prefix). Full per-route+verb enumeration is deferred to the M2 automated extractor. **`Interface::Rest` anchoring convention:** the single `Evidence` on the `Interface::Rest` value cites the `prefix_format`/`@version` line (it grounds the judgment-prone `versions` field). The concrete `path_prefix` and `port` are corroborated by the separately-recorded nginx `location` line (step 3) and the uvicorn port line (step 2) — QA should NOT bounce the Rest anchor merely because the `path_prefix` string does not appear verbatim on the `prefix_format` line (see `helper.rs`/`cable_guy.rs` as the accepted pattern).
4. **MAVLink role** — grep for `udpin|udpout|tcpin|tcpout|--connect|--mavlink|MAV_SYSTEM_ID|component-id`. Record each connect string as `Interface::Mavlink { connect, role }`. The `role` is *directional and factual*: inbound/listen (`udpin`/`tcpin`) → `Endpoint`; outbound (`udpout`/`tcpout`) → `Consumer`; forwarding → `Bridge`. Do NOT assert router ownership — that is `Authority::MavlinkRouterOwner` in the asserted layer, not an observed role.
5. **Zenoh** — grep for `zenoh`, `zenoh_helper`, session/topic names; record topics with evidence.
6. **Hardware exclusivity** — serial (`/dev/tty*`), camera (`/dev/video*`), `wlan0`, GPIO. Record device path + evidence.
7. **Settings / files / caches** — writes under `/usr/blueos/userdata`, settings dirs, AND cache/manifest dirs (e.g. `.../ardupilot-manager/manifest-cache`). Record every distinct path as a `Resource`/`Interface::File`. For the settings file, record its path + root shape (e.g. `{version, content}`) via `Interface::Settings`. Record path only (who-else-writes is a cross-service concern for validation, not this card). Per-transition settings *mutation* is runtime, not source — it belongs in the `RuntimeFacts` layer, not here.
8. **Subprocesses / external binaries** — spawned processes (`mavlink-camera-manager`, `linux2rest`, `mavlink2rest`, `zenohd`, `nginx`, `filebrowser`, `ttyd`, `iperf3`, `blueos-recorder`).
9. **Outbound edges** — grep for `http://`, `127.0.0.1:`, `aiohttp`, other services' base URLs. Record `to` target + endpoint + evidence. Leave `purpose`/`failure_impact` empty (assertion). **NEVER synthesize a URL.** An `Interface::OutboundHttp { url }` value must be a string literally present at the cited `file:line`. If the source only has a host/path/port (e.g. a `{"hostname": "amazon.com", "port": 80, "path": "/"}` dict), record the literal host string (no invented scheme/port) and cite that exact line — do not assemble `http://amazon.com:80/`. If a full URL is composed elsewhere, cite that composition line. When in doubt, use the narrowest literal value with a real citation.
10. **Env coupling** — `MAV_SYSTEM_ID`, `BLUEOS_*`, secondary venv (`BLUEOS_VENV_SECONDARY`/`venv2`).

## Provenance format

Scalar fields use `Observed<T>` (one evidence). Collection fields (`aliases`, `nginx_prefixes`, `listen`, `interfaces`, `resources`, `openapi_refs`) use `ObservedSet<T>` — **each item carries its own `Evidence`** via `Evidenced::new`. Never bundle facts from different `file:line`s under one citation.

```rust
// scalar
tmux_name: Observed::known("autopilot".into(),
    Evidence { file: "core/start-blueos-core".into(), line: 118 }),
run_as: Observed::unknown("no RUN_AS_REGULAR_USER wrapper on line 118, so runs as container default"),

// collection — one Evidence PER item
nginx_prefixes: ObservedSet::known(vec![
    Evidenced::new(PathRef("/ardupilot-manager/".into()),
        Evidence { file: "core/tools/nginx/nginx.conf".into(), line: 76 }),
    Evidenced::new(PathRef("/autopilot-manager/".into()),
        Evidence { file: "core/tools/nginx/nginx.conf".into(), line: 81 }),
]),
interfaces: ObservedSet::unknown("none found"),
```

## Done criteria (self-check before returning)

- [ ] Every populated field resolves to a real `file:line` (open it and confirm).
- [ ] Ports and routes reconcile with both `nginx.conf` and `start-blueos-core`.
- [ ] No authority/tier/context/failure field is set.
- [ ] `drift` binary passes for this observed artifact.

Return: the artifact path, a one-line summary, and a list of any `Unknown` fields with reasons.
