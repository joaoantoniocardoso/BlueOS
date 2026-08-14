# W2 negative — consolidated (all four DUTs identical)

`journey_http --negative` 63 probes. Core `1.4-dev @ sha256:f615d7caef4d3e99f1c068e082350c1af43d5fcc0f45379ed97ea27c6dc89805`.

| DUT | passed | failed | skipped | unasserted | exit |
|---|---:|---:|---:|---:|---:|
| 192.168.0.177 | 22 | 13 | 0 | 28 | 1 |
| 192.168.0.87 | 22 | 13 | 0 | 28 | 1 |
| 192.168.2.2 | 22 | 13 | 0 | 28 | 1 |
| 192.168.0.124 | 22 | 13 | 0 | 28 | 1 |

NP-62 (delete running tag) **500 Pass** on all four — running-image guard holds.

## Failures (same 13 on every DUT)

### Product / contract on 1.4-dev (service exists)

| probe | expected | got | note |
|---|---|---|---|
| NP-31 forget unknown SSID | 400 | **200** | `POST /wifi-manager/v1.0/remove?ssid=__np_no_such_ssid__` is idempotent success |
| NP-38 concurrent scan | 425 | **[200,200]** | `scan_busy` 425 never fired; both scans 200 |
| NP-53 restart missing extension | 404 | **400** | kraken restart unknown id |
| NP-54 missing container log | 404 | **200** | kraken log unknown container |

### Environment — route absent on 1.4-dev (405/404; F-022)

| probes | got | first tag |
|---|---|---|
| NP-12, NP-13 theme PUT | 405 | 1.5.0+ customization |
| NP-19 model DELETE | 405 | 1.5.0+ |
| NP-28, NP-29 disk DELETE | 405 | 1.4.4+ disk_usage |
| NP-30 disk speed | 404 | 1.4.4+ |
| NP-68, NP-69, NP-70 recorder | 404/405 | 1.4.4+ recorder_extractor |

Traversal probes (NP-69) returned **405 not 2xx** — no arbitrary-delete blocker.

Commander B2 cluster (NP-01–08) **passed 400**. Versionchooser NP-60/61/62/63 passed.
