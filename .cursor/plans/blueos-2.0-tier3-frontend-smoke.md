# Tier-3 frontend page-load smoke

Tier-3 proves **Actor::Frontend** journey entry points are reachable in a real browser. v1 covers the three calibration-related pages that match F2/F3 frontend journeys.

## Scope (v1)

| page_id | catalog route | concrete path |
|---|---|---|
| `vehicle_setup` | `/vehicle/setup/:tab?/:subtab?` | `/vehicle/setup` |
| `video_manager` | `/vehicle/video-manager` | `/vehicle/video-manager` |
| `parameter_editor` | `/vehicle/parameters` | `/vehicle/parameters` |

Full calibration wizards (MAVLink commands, vehicle state) are **not** automated yet — only page-load smoke.

## How it works

1. `cargo run --bin frontend_smoke` reads `Catalog::bootstrap().pages()` and emits `pages.json` with concrete paths (`concrete_page_path` strips Vue `:param` segments).
2. Playwright (`catalog/e2e/`) loads `pages.json`, opens the SPA at `/`, then client-side-routes to each path (BlueOS nginx does not serve `index.html` for deep links).

## Run locally

```bash
cd catalog/e2e
npm install
npx playwright install chromium   # first time only
BLUEOS_BASE=http://192.168.0.177 npm run smoke
```

`BLUEOS_BASE` defaults to `http://192.168.0.177` in `playwright.config.ts`.

## Gate policy

Unlike Tier-1 GET smoke (`journey_http --smoke`, wired into `catalog/gate.sh`), Tier-3 is an **optional live check**. Offline development stays green via Rust unit tests for `concrete_page_path` and `bash catalog/gate.sh` (which does not run Playwright).
