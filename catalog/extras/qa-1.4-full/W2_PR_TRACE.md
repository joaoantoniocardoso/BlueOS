# W2 contract-vs-live: git diffs and PRs

Traced by Composer-2.5 against this tree and `gh` on **bluerobotics/BlueOS**
(commit→PR API; `BlueOS-docker` search does not index these SHAs).

DUT pin when this file was written: `bluerobotics/blueos-core:1.4-dev @
sha256:5b50dfafc3114651d459993c0c65639d7205634613fb62e0dea5ee04de8eebb1`
(2026-08-13T21:23:50Z). Theme / disk-usage / recorder still **404** on this
image — those nine W2 fails remain “not on 1.4-dev”, not contract mismatches.

---

## NP-31 — unknown SSID `POST /remove` → 200, not 400

**Verdict: product bug (minor).** Harness already retargeted to 200.

| | |
|---|---|
| Behavioral commit | `58094a5f0` 2023-07-26 |
| Subject | `core: services: wifi: Remove all wifis with same ssid in database` |
| PR | https://github.com/bluerobotics/BlueOS/pull/1899 |
| Relocate commit | `634f37fb5` 2024-12-02 / merged 2025-01-10 |
| Relocate PR | https://github.com/bluerobotics/BlueOS/pull/3018 (bookworm / NetworkManager; the wpa subfolder move rode along) |

`58094a5f0` replaced `next(filter(...))` (raises `StopIteration` → 400) with a
list comprehension. Empty match → loop does nothing → HTTP 200. `634f37fb5`
moved that loop into `WifiManager.remove_network(ssid)` and left the dead
`except StopIteration` in `main.py`. 400 has been unreachable since **2023-07-26**,
including all of 1.4.0 / 1.4-dev.

**User-visible:** UI/API reports success for a typo or never-saved SSID; saved
networks are unchanged. Not idempotent DELETE — the original contract was
`Network '{ssid}' not saved.` Fix: raise 400/404 when `match_networks` is empty.

---

## NP-38 — concurrent `GET /scan` → 200,200 not 425

**Verdict: tolerable (intentional).** Harness still expects 425 (`runner.rs`).

| Commit | Date | PR | What |
|---|---|---|---|
| `0f359674c` | 2021-05-20 | (wifi BusyError→425) | `/scan` maps `BusyError` → 425 |
| `cd2feef74` | 2021-07-14 | [#383](https://github.com/bluerobotics/BlueOS/pull/383) | FAIL-BUSY retry; `get_wifi_available` swallows `BusyError` as `FetchError` (would be **500**, not 425) |
| `f6f5e87ce` | 2021-07-14 | [#383](https://github.com/bluerobotics/BlueOS/pull/383) | `_scan_task` coalescing: second caller waits |
| `6c14140cc` | 2022-07-22 | [#1019](https://github.com/bluerobotics/BlueOS/pull/1019) | 30 s `_time_last_scan` cache |

PR #383 body: *“If there's one scan happening when you make a request, its
result will be used.”* Concurrent HTTP cannot hit 425. A wpa `FAIL-BUSY` that
lasts the full SCAN timeout (~30 s) becomes `FetchError` → **500**, because
`BusyError` is caught by `except Exception` in `WifiManager.get_wifi_available`.

**User-visible:** parallel UI polls do not hammer the radio. Clients cannot
learn “radio busy” via overlapping `/scan`. Not a regression.

---

## NP-53 — `POST /extension/{id}/restart` missing id → 400, not 404

**Verdict: product bug (minor, semantic).** Still rejects. Harness retargeted to 400.

| Commit | Date | PR |
|---|---|---|
| `638f98428` | 2024-05-16 | [#2604](https://github.com/bluerobotics/BlueOS/pull/2604) — V2 restart was 501 |
| `8cf701065` | 2024-06-03 | [#2641](https://github.com/bluerobotics/BlueOS/pull/2641) — `from_running`, tag-gated `ExtensionNotFound` |
| `62a153849` | 2024-06-03 | [#2641](https://github.com/bluerobotics/BlueOS/pull/2641) — restart → `from_running`; `ExtensionNotRunning` → **400** |

PR #2641 does not mention 400 vs 404. Asymmetry is emergent: untagged
`from_running` with empty install list raises `ExtensionNotRunning`
(`"Extension {id} have no running versions"`) instead of `ExtensionNotFound`.
Tagged install/uninstall still 404. `ExtensionNotRunning` has never been 404.

**User-visible:** request does not restart anything (safe). Clients/UI may show
“not running” for an identifier that was never installed. Fix: empty `installed`
→ `ExtensionNotFound`; installed-but-disabled → keep `ExtensionNotRunning`.

---

## NP-54 — `GET /container/{name}/log` missing → HTTP 200, not 404

**Verdict: product bug (real false success).** Harness retargeted to 200.

| Commit | Date | PR | What |
|---|---|---|---|
| `a41963910` | 2023-12-05 | [#2292](https://github.com/bluerobotics/BlueOS/pull/2292) | V1 `StreamingResponse` before existence check |
| `36142005a` / `43f7a54bd` | 2024-05-30 | [#2640](https://github.com/bluerobotics/BlueOS/pull/2640) | Harbor raises 404 **inside** the generator; V2 log returns `StreamingResponse` immediately |
| `f2f449d8e` | 2024-07-30 | [#2852](https://github.com/bluerobotics/BlueOS/pull/2852) | Default path is continuous `streamer` (heartbeats) — still no pre-check |

`GET /{container_name}/details` awaits and **does** return HTTP 404. Log commits
200 first; 404 may appear only as a JSON fragment in the stream body (and
heartbeats can arrive first). No later commit added a pre-check.

**User-visible:** HTTP-only clients (`curl`, monitors, NP-54) treat missing
container as success. Extension UI today uses Zenoh for logs, so frontend
impact is low. Not a tolerable quirk unless the API is documented as
“always 200; errors only in stream JSON” — it is not (`responses={404}`).

Product fix: await container existence **before** constructing `StreamingResponse`.

---

## Open issues (filed 2026-08-13, type Bug)

| Probe | Issue | Affects |
|---|---|---|
| NP-31 | https://github.com/bluerobotics/BlueOS/issues/4159 | master + 1.4-dev |
| NP-38 | not filed (by design) | — |
| NP-53 | https://github.com/bluerobotics/BlueOS/issues/4160 | master + 1.4-dev |
| NP-54 | https://github.com/bluerobotics/BlueOS/issues/4161 | master + 1.4-dev |

---

## Summary

| Probe | Observed | Bug? | Ship-blocking? | Harness |
|---|---|---|---|---|
| NP-31 | 200 | yes — silent success on unknown SSID | no | already 200 |
| NP-38 | 200,200 | no — coalescing by design; 425 dead | no | still expects 425 |
| NP-53 | 400 | yes — wrong error class/message | no | already 400 |
| NP-54 | 200 | **yes — false success** | no for UI; yes for HTTP monitors | already 200 |
