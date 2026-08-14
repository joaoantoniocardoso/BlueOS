# W2 negative-probe triage: on-1.4-dev fails

Scope: the four W2 fails that reproduce identically on all four DUTs and are *not* absent-service
405/404. Each was checked against this tree's `core/services` **and** against the `1.4.0` tag
(`git show 1.4.0:...`), because this repo is 2.0-dev/model and may be newer than the DUT image.

**No source divergence found.** For all four probes the 1.4.0 tag and this tree contain byte-identical
logic, so the DUT is *not* older than the catalog source here; the observed statuses are the contract
on both. Every failure is a harness expectation error (the `NEGATIVE_PROBES.md` anchor was read as the
reachable path when it is not), with a genuine product wart worth keeping as a finding in three cases.

Exact diffs + PRs: `W2_PR_TRACE.md` (Composer-2.5, 2026-08-13).

---

## NP-31 — `POST /wifi-manager/v1.0/remove?ssid=__np_no_such_ssid__` — expected 400, got 200

**Verdict: harness** (expected_status wrong). Product note: silent success on unknown SSID.

Source:
- `core/services/wifi/main.py:89-99` — the handler catches `StopIteration` and maps it to 400.
- `core/services/wifi/wifi_handlers/wpa_supplicant/WifiManager.py:273-288` — `remove_network(ssid)`
  filters with a list comprehension and loops over the matches. An unknown SSID yields an empty list,
  the loop body never runs, and the coroutine returns `None`.

`StopIteration` cannot escape `remove_network`, so the `except` in `main.py:96-98` is dead code and the
400 is unreachable. The `next()`-based lookup that *did* raise lived in `main.py` before
`634f37fb5` (2024-12-02, "Wifi: move wpa implementation to subfolder"), which predates `1.4.0`
(2025-04-04) — the 1.4-dev DUT has the post-refactor behavior. Confirmed at
`git show 1.4.0:core/services/wifi/wifi_handlers/wpa_supplicant/WifiManager.py:274-288`.

**Action:** retarget `expected_status` to `200` in `catalog/src/negative_probes.rs` (NP-31) and keep the
"forget unknown network reports success" wart as a product finding. The `NEGATIVE_PROBES.md` anchor
"remove-400 `core/services/wifi/main.py:96-98`" should be annotated as unreachable.

---

## NP-38 — two concurrent `GET /wifi-manager/v1.0/scan` — expected 425, got 200,200

**Verdict: harness / limitation** — 425 is not reachable through concurrent front-door scans.

Source:
- `core/services/wifi/main.py:63-71` — `/scan` returns 425 only on `BusyError`.
- `core/services/wifi/wifi_handlers/wpa_supplicant/wpa_supplicant.py:76-82` — `BusyError` is raised only
  when wpa_supplicant answers `FAIL-BUSY` continuously past the command timeout (30 s for `SCAN`).
- `WifiManager.py:214-230` — two guards defeat the probe before wpa is ever touched twice:
  a 30-second result cache (`if time.time() - self._time_last_scan < 30: return ...`), and, when the
  cache is cold, a wait loop that *awaits* the in-flight `_scan_task` instead of issuing a second scan.

Concurrency is handled by coalescing, by design, so both requests return the same cached/awaited result
with 200. `FM:wifi/scan_busy` needs wpa_supplicant itself to stay busy >30 s (radio-level contention),
not two overlapping HTTP calls. Identical in `1.4.0`.

**Action:** the NP-38 code path in `runner.rs:262-299` hard-fails unless some response is 425, so this is
not a one-constant fix — it needs the probe demoted to `UnknownLive` (capture the observed pair) or
dropped from the front-door suite. Left unmodified pending that decision; `NEGATIVE_PROBES.md:129`
("highest-value probe in this group") is wrong and should be corrected.

---

## NP-53 — `POST /kraken/v2.0/extension/np.no.such.extension/restart` — expected 404, got 400

**Verdict: harness** (expected_status wrong). Product note: unknown identifier is reported as
"not running" rather than "not found".

Source:
- `core/services/kraken/api/v2/routers/extension.py:129-136` — `restart` calls `Extension.from_running`.
- `core/services/kraken/extension/extension.py:393-398` — `from_running` calls `from_settings(identifier)`,
  which for an unknown identifier returns an empty list (`ExtensionNotFound` at
  `extension.py:107-109` is raised only when *both* identifier and tag are supplied), so the empty
  `enabled` list raises `ExtensionNotRunning`.
- `core/services/kraken/api/v2/routers/extension.py:34-35` — `ExtensionNotRunning` maps to **400**.

This is why NP-51/NP-52 pass with 404 while NP-53 does not: those routes take an explicit tag and go
through the `ExtensionNotFound` path. Identical in `1.4.0`
(`git show 1.4.0:core/services/kraken/extension/extension.py:318-325`).

**Action:** retarget `expected_status` to `400` for NP-53 and keep the 404-vs-400 asymmetry between
tagged and untagged extension routes as a product finding.

---

## NP-54 — `GET /kraken/v2.0/container/np_no_such_container/log` — expected 404, got 200

**Verdict: harness** (expected_status wrong). Product note: 404 is unreachable on a streaming route.

Source:
- `core/services/kraken/api/v2/routers/container.py:51-63` — the handler builds the async generator but
  does not iterate it, then returns `StreamingResponse`. Headers (200) are committed immediately.
- `core/services/kraken/harbor/container.py:115-120` — `get_container_log_by_name` raises the
  `StackedHTTPException(404)` on `ContainerNotFound`, but only on first iteration of the generator,
  i.e. after the response has already started; the `container_to_http_exception` wrapper
  (`container.py:20-30`) has already returned by then.

The client therefore sees 200 with an empty/aborted body. Contrast `/{container_name}/details`
(`container.py:42-48`), which awaits and does return 404. Identical in `1.4.0`.

**Action:** retarget `expected_status` to `200` for NP-54 and keep "unknown container log streams 200
instead of 404" as a product finding (a `ContainerNotFound` pre-check before constructing the
`StreamingResponse` would be the product fix).

---

## Summary

| probe | observed | verdict | harness retarget |
|---|---|---|---|
| NP-31 | 200 | harness (dead 400 branch) + product wart | `expected_status: Some(200)` |
| NP-38 | 200,200 | harness/limitation (425 unreachable via HTTP concurrency) | needs `UnknownLive` + `runner.rs` change, not a constant |
| NP-53 | 400 | harness (ExtensionNotRunning maps to 400) + product wart | `expected_status: Some(400)` |
| NP-54 | 200 | harness (404 raised after stream headers) + product wart | `expected_status: Some(200)` |

None of these are "DUT older than catalog source".
