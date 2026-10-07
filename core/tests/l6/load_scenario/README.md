# D-33 load scenario (L6)

Python driver run from a topside computer against a BlueOS device over SSH and HTTP. It is not run in CI.

Terminology matches decision D-33:

- **reference** — out-of-tree (legacy) Recorder binary on the device
- **candidate** — new in-tree Recorder binary under test

Each run writes one JSON file with per-phase Recorder CPU means, throttle state, and integrity flags.

## Prerequisites (topside)

- `uv` (from the `core/` workspace)
- GStreamer and OpenCV tooling used by `assets/build_clip.sh` and the RTSP source
- SSH access to the device as `--ssh-user` (default `pi`)

From `core/`:

```bash
uv run load-scenario --help
```

## Prerequisites (device — human steps)

1. Use the **same** BlueOS image and board for every run in a comparison (reference hardware: Pi 4, 32-bit bullseye, unless D-33 has been updated).
2. Fix the CPU governor to `performance` (the driver sets this when possible; confirm on the device).
3. Install dependencies for MAVLink Camera Manager redirects, SITL, and the Recorder Zenoh command path.
4. Build or copy the Recorder binary you will test:
   - **reference**: out-of-tree Recorder build for the device architecture
   - **candidate**: in-tree `blueos` Recorder from this repository
5. Swap **only** the Recorder binary between reference and candidate runs; keep the rest of the image unchanged.
6. After each binary swap, restart the Recorder service the same way you would on a vehicle.

## Capture reference (out-of-tree) baseline JSON

Run at least ten reference runs (adjust after a trial if needed):

```bash
uv run load-scenario reference-campaign DEVICE_HOST \
  --output-directory reference-recorder-results \
  --run-count 10
```

Install the reference binary once at the prompt; the command records JSON for every run without swapping again.

## Compare candidate against reference

**Option A — alternating campaign (same session):**

```bash
uv run load-scenario campaign DEVICE_HOST \
  --output-directory load-scenario-results \
  --runs-per-binary 10
```

Swap binaries when prompted between reference and candidate runs.

**Option B — candidate only, reuse saved reference JSON:**

```bash
uv run load-scenario run DEVICE_HOST \
  --recorder-binary candidate \
  --output-directory candidate-recorder-results
```

Repeat until you have at least ten non-throttled candidate runs, then:

```bash
uv run load-scenario report \
  --results-directory candidate-recorder-results \
  --reference-results-directory reference-recorder-results
```

## Report output

The `report` command prints per-phase JSON:

- `reference_median` / `candidate_median` — Recorder CPU percent means (median across runs)
- `median_percentage_of_reference` — candidate as a percentage of reference (`100` means equal CPU)
- `median_difference`, `confidence_interval_95`, `p_value` — statistics from D-33

Throttled runs are excluded. Runs with `recording_dropped_samples: true` in the JSON should be investigated before trusting the comparison.
