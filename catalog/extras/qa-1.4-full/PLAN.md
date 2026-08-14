# BlueOS 1.4-dev — Full feature / journey QA (happy + sad)

**Campaign root:** `catalog/extras/qa-1.4-full/`
**Target:** live `bluerobotics/blueos-core:1.4-dev @ sha256:f615d7caef4d3e99f1c068e082350c1af43d5fcc0f45379ed97ea27c6dc89805`
**Harness:** `catalog` crate (`journey_http`, wifi RF, mutating smoke) — extend, do not fork.
**Orchestration:** Composer-2.5 agents for inventory / runs / harness fixes; Opus-5 for sad-path taxonomy, failure classification, and cross-DUT nuance.
**Loop rule:** a finding is never a stop. Catalog it, continue, fix harness when the harness is wrong, re-run only the affected class.

---

## 0. Assumptions (object if wrong)

| Assumption | Default |
|---|---|
| Play / sacrificial DUT | `192.168.0.177` (historical play Pi; reimage OK) |
| USB vehicle | `192.168.2.2` on `enp17s0f3u2u4c2` — **no stranding** (no static-IP / DHCP / interface-priority that can drop the USB path) |
| Shutdown | **Never** (`ShutdownOnboardComputer` stays hard-excluded) |
| Firmware flash / EEPROM / core switch / settings reset / host reboot | **177 only**, one at a time, with declared restore |
| RF wifi | **Serial, one DUT at a time** — one host radio (`HOST_WIFI_IFACE`) |
| Credentials | ssh `pi` / `raspberry`; HTTP via nginx (`http://<ip>/…`) |
| Keep all data | every run writes `--report` JSON + stdout under `catalog/extras/qa-1.4-full/reports/` |

---

## Part 1 — Sad / bad-path strategy

The catalog is a **happy-path spine**: 101 operator journeys, mostly success contracts (`expected_status: 200`). Wifi is the exception that already models failure as a first-class journey (`RejectInvalidWifiCredentials` → HTTP 500) plus mid-flow disruption (`DetectWifiApLoss`) and recovery (`AutoconnectToSavedWifiNetwork`).

We do **not** invent ~80 new `JourneyId`s. Catalog rules forbid journeys without a Doc or Source anchor, and each new id taxes `ALL`, presence generation, coverage mappings, and frontend provenance. Close-to-release coverage comes from **four tracks**.

### Track A — Operator-visible failure journeys (new `JourneyId`, wifi-style)

Promote a negative case to a catalog journey **only when all of**:

1. The operator sees a **named failure UX** (wrong password, confirm rejected, invalid color, bad file type).
2. It is **Doc- or Source-anchored** (frontend error handler or backend 4xx/5xx branch with a `file:line`).
3. The non-success status is a **contract**, not a bug (`RejectInvalidWifiCredentials` is the template: `POST /connect` → 500).
4. Restore is declared (`SmokeRepair`) and the case is automatable over HTTP (or HostWifiRf).

Wifi already owns Track A for RF. First promotions (source-grounded, low blast radius):

| Candidate | Anchor | Expected | Restore |
|---|---|---|---|
| Reject unconfirmed dangerous op | commander `i_know_what_i_am_doing=false` → 400 | 400 | none |
| Reject invalid theme color | customization `parse_hex` → 400 | 400 | none |
| Reject disallowed upload suffix | branding/model 400 | 400 | none |
| Reject missing bag path | bag `GET /get/{path}` → 400 | 400 | none |
| Reject host command without ack | commander 400 | 400 | none |

These are added **after** live capture of the real status/body (never guessed). Until then they live as Track B probes.

### Track B — Negative probes attached to existing journeys (harness, not new ids)

For **every** happy journey, the harness runs a probe family against the same `RouteRef`. A probe is a finding, not a catalog journey. Implementation: `catalog/src/negative_probes.rs` + `journey_http --negative` (mutually exclusive with `--smoke` / `--mutating-smoke`).

Probe classes (apply only where the route exists):

| Class | Example | Pass means |
|---|---|---|
| **B1 Invalid input** | malformed JSON, wrong types, empty required, garbage hex | documented 4xx (or cataloged 5xx if that is the live contract) |
| **B2 Confirm / gate** | `i_know_what_i_am_doing=false` or omitted | 400, **no mutation** |
| **B3 Missing precondition** | delete missing file, uninstall missing extension, connect to unknown SSID | 4xx/5xx **or** idempotent 200 — record which |
| **B4 Double-apply / idempotency** | DELETE twice, POST enable when already enabled, stop when stopped | second call does not 500-crash; state remains valid |
| **B5 Resource / limit** | oversize upload 413, disk speed 507, huge bag overwrite | status matches `failure_modes` |
| **B6 Service-down** | hit route after tmux stop of that service (177 only) | not silent 200 with empty success; record real status |
| **B7 Mid-flow disruption** | wifi AP drop is the template; analog: stop autopilot mid-MAVLink inspect, kill dnsmasq mid-DHCP | detection + optional recovery |
| **B8 Offline / no-internet** | helper ping, kraken manifest, firmware URL, version pull | skip vs fail distinction is explicit |
| **B9 Auth / pirate / advanced** | bag editor, advanced pages without fixture | skip (fixture) vs 403/empty |

Wifi already covers B1 (wrong password), B7 (AP loss), recovery (autoconnect). Other services get B1–B5 first (safe). B6/B7 disruptive only on 177.

### Track C — Failure-mode ledger (every asserted `failure_modes` id)

Each service card already names failure modes (`wpa_supplicant_connection_failure`, `firmware_flash_failure`, `i_know_what_i_am_doing_rejected`, …). The ledger maps **every** id to exactly one of:

- `probed` — Track A or B ran; result attached
- `skipped` — reason (no hardware, hard-exclude, would strand DUT)
- `limitation` — product/platform ceiling; not a harness bug
- `harness_gap` — we should probe it and currently cannot; fix the harness

File: `catalog/extras/qa-1.4-full/FAILURE_MODE_LEDGER.md` (append-only updates). This is how “full coverage” is claimed without pretending every mode is a journey.

### Track D — Absence is not failure

Missing Ping sonar, USB camera, or Pixhawk-vs-Navigator is a **skip with typed precondition**, never a Fail. Wrong skip (hardware present but harness skipped) is a **harness bug** and must be fixed. Cross-DUT: 87 is Pixhawk1; 177/124/2.2 are Navigator — firmware and serial-bridge probes must not assume Navigator on 87.

### What “pass” means on the sad side

- **Expected rejection** (wrong password, no confirm, bad hex) → Pass if status matches contract **and** system state is unchanged (or restored).
- **Unexpected 500** on a happy path → Fail (product or harness).
- **Unexpected 200** on a negative probe that should reject → Fail (missing validation) — catalog as product finding, continue.
- **Skip** → only with a typed reason (`no_wifi_radio`, `not_on_1.4-dev`, `hard_exclude_shutdown`, `would_strand_usb`, `deferred_core_switch`).
- **Limitation** → product/platform cannot do it (e.g. WPA3 reject when `EXPECT_WPA3=no`); not a test fail.

### Harness changes (Part 1 execute)

1. Add `negative_probes.rs`: table of `{journey_id, class, method, path, body, expected_status, restore, dut_affinity, blast}`.
2. Wire `journey_http --negative [--journey id] [--report path]`.
3. Expected non-200 is a **Pass** (extend `run_http_step` / mutating expected-status override — wifi already does 500 for reject).
4. `--report` JSON must include `suite: negative`, probe class, raw status/body (truncated), restore result.
5. Do **not** require new `JourneyId`s to ship Track B. Track A promotions happen after live capture, in a later commit.

Wifi stays the gold standard; do not regress RF skip/restore.

### Part 1 done criteria

- [ ] `SAD_PATH_MATRIX.md` — every one of 101 journeys has ≥1 sad class (or explicit `n/a` with reason)
- [ ] `FAILURE_MODE_LEDGER.md` — every service `failure_modes` id classified
- [ ] `journey_http --negative --dry-run` lists probes
- [ ] Safe subset (B1/B2/B3/B5, no B6) compiles; `cargo test` + `gate.sh` offline green
- [ ] Findings schema in `FINDINGS.md` ready for live fills

---

## Part 2 — Organize and run on four machines

### DUT roles (live inventory 2026-08-13)

| DUT | Board | Link | Role |
|---|---|---|---|
| `192.168.0.177` | Navigator | LAN `eno1` | **Play.** RF wifi first. Destructive / firmware / EEPROM / reboot / core-switch / settings-reset / B6 service-down. Full mutating. |
| `192.168.0.87` | **Pixhawk1** `/dev/ttyACM0` | LAN | Platform contrast. GET smoke + reversible mutating. Firmware/serial against Pixhawk, not Navigator. No EEPROM (Pi-specific still OK). No RF until 177/124 done. |
| `192.168.0.124` | Navigator | LAN | Second Navigator. GET smoke + reversible. RF wifi **second** (serial). No destructive. |
| `192.168.2.2` | Navigator | USB `192.168.2.0/24` | Physical ROV. GET smoke + conservative mutating. **Never** DHCP/static-IP/priority/hotspot that can drop USB. No RF unless wlan confirmed and ethernet management stays up. |

All four: same tag+digest. Compare **behavior**, not version skew.

### Waves (do not stop between waves)

| Wave | What | Parallelism | Agents |
|---|---|---|---|
| **W0** | DUT inventory (hw, wifi, cameras, ping, pirate/advanced, disk, extensions, firmware) | 4 parallel | Composer-2.5 ×4 |
| **W1** | Tier-1 `journey_http --smoke --fixtures internet,pirate,advanced --report …` | 4 parallel | Composer-2.5 ×4 |
| **W2** | Track B safe `--negative` (B1–B5) | 4 parallel | Composer-2.5 ×4 |
| **W3** | Reversible mutating (theme, branding, bag, rename, NMEA, LAN speed, disk inspect) | 3-way (exclude DUT in RF) | Composer-2.5 |
| **W4** | RF wifi mutating — **serial** 177 → 124 → 87 (if radio) → 2.2 only if safe | 1 DUT | Composer-2.5 + host RF |
| **W5** | Disruptive mutating (stop/start autopilot, board/SITL, wifi non-RF HTTP) | one DUT at a time | Composer-2.5 |
| **W6** | Destructive **177 only** (reboot wait-for-recovery, firmware restore plan, settings reset, version switch alias) | 1 | Composer-2.5 |
| **W7** | Frontend / hardware journeys (calibration, cameras, ping) — skip if no fixture | per DUT capability | Composer-2.5 |
| **W8** | Opus-5 classifies every Fail/Skip: product vs harness vs limitation vs environment | after each wave | Opus-5 |
| **W9** | Harness fixes for `harness_gap` / wrong skip; re-run **only that class** | as needed | Composer-2.5 |

`--report` path convention:

```
catalog/extras/qa-1.4-full/reports/<dut-ip>/<wave>-<suite>-<utc>.json
catalog/extras/qa-1.4-full/reports/<dut-ip>/<wave>-<suite>-<utc>.log
```

Raw HTTP snippets, `docker inspect` RepoDigest, and `GET /version-chooser/v1.0/version/current` are copied into `captures/`.

### Isolation / safety

- One mutating owner per DUT (lock file `catalog/extras/qa-1.4-full/locks/<ip>`).
- RF: global lock `locks/host-wifi`.
- After every mutating wave: restore + GET `/status` 204 + version digest unchanged (unless the wave was a version switch).
- If a DUT drops: catalog `LIMITATION`, move on to the other three; recover 177 via SSH/reboot if it was the play Pi.
- `UpdateBlueosVersion` POST `/version/current` and bootstrap replace stay **deferred** (existing harness policy) unless a restore pin is proven first.

### Finding catalog (every result)

`catalog/extras/qa-1.4-full/FINDINGS.md` — one row per distinct issue:

| Field | Meaning |
|---|---|
| `id` | `F-NNN` |
| `dut` | ip or `all` |
| `journey` / `probe` | id + class |
| `kind` | `product` / `harness` / `limitation` / `environment` / `contract` |
| `severity` | blocker / major / minor / note |
| `expected` vs `got` | status/body |
| `restore` | ok / failed / n/a |
| `next` | fix harness / accept limitation / file for 1.4 |

Never delete a finding. Supersede with `F-NNN.1` if re-run changes the result.

### Agent protocol

- Composer-2.5: self-contained prompt (DUT ip, exact command, report path, **do not stop on fail**, append findings, return 20–40 line summary).
- Opus-5: sad-path matrix (Part 1), then per-wave Fail taxonomy, then global release-readiness read.
- Orchestrator does not wait on a red smoke to start the next **independent** wave (W1 on DUT B while W1 on DUT A failed).
- No iteration cap. Working memory: `catalog/extras/qa-1.4-full/memory.xml` (overwrite, keep short). Archive: `memory_archive.md` (append).

### Part 2 done criteria

- [ ] W0 inventory for all four DUTs
- [ ] W1 smoke JSON for all four (failed count may be >0; cataloged)
- [ ] W2 negative probes for all four
- [ ] W3 reversible mutating on ≥3 DUTs
- [ ] W4 RF on ≥1 DUT with radio (wifi gold path)
- [ ] W6 destructive subset on 177 or explicit skip-with-reason
- [ ] Every Fail/Skip in FINDINGS or ledger
- [ ] Harness diffs only for proven harness bugs
- [ ] `RELEASE_READINESS.md` — what 1.4-dev can claim

---

## Out of scope (still cataloged if stumbled on)

- Implementing BlueOS product fixes (findings only, unless the user later asks).
- `ShutdownOnboardComputer`.
- Replacing the catalog with a new test framework.
- Guessing runtime status codes without a live capture.
