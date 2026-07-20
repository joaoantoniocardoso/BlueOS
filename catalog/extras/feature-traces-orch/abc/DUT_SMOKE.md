# Feature Provenance — DUT Smoke Test

**Date:** 2026-07-19  
**Result:** PASS

## DUT

| Field | Value |
|---|---|
| Address | http://192.168.0.177 |
| Tag | `1.4-dev` |
| SHA | `sha256:6db7e42ab466be3565862bc38cbd21b387fad4ae4a5a02d7c23273f63436ea53` |
| `/status` | HTTP 204 |

## Test harness

Local Vite dev server proxied to DUT:

```bash
BLUEOS_ADDRESS=http://192.168.0.177 yarn --cwd core/frontend dev
```

Page is **not** deployed on the Pi image yet; smoke exercised via localhost proxy only.

## URLs hit

| URL | Status | Notes |
|---|---|---|
| `http://localhost:8080/tools/feature-provenance` | 200 | HTML shell (`<!DOCTYPE html>`) |
| `http://localhost:8080/assets/feature-provenance.json` | 200 | Valid JSON |
| `http://localhost:8080/version-chooser/v1.0/version/current` | 200 | Proxied; tag `1.4-dev` |
| `http://192.168.0.177/status` | 204 | Direct DUT health |
| `http://192.168.0.177/version-chooser/v1.0/version/current` | 200 | Direct DUT version |

## Asset validation (`feature-provenance.json`)

| Check | Result |
|---|---|
| `journeys` count | **8** (≥8 required) |
| Each journey has `skip_reasons` | **yes** (8/8) |
| `schema_version` | 1 |
| `reference_dut_tags` | `master`, `1.4-dev` |

## Blockers

- Feature Provenance page not on-device until frontend image ships; no on-Pi `/tools/feature-provenance` test possible today.
