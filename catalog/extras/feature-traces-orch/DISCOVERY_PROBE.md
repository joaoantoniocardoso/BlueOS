# Feature Provenance Discovery Probe

Repo: `bluerobotics/BlueOS` via `gh` (origin remote = `bluerobotics/BlueOS-docker`).
Release branches exist locally as `origin/1.0-dev` … `origin/1.4-dev`, `origin/1.4`, `origin/1.3`, etc.

## A) Backport discovery — recommended algorithm

1. Get the intro commit's changed file paths (from PR's `files[].path`, already in `feature_traces.json`).
2. Run `git log --oneline origin/<release>-dev -- <paths...>` (and `origin/<release>` too) for each candidate release line. **Empty output = confidently no backport** (verified: zero hits for all 3 probed features on `1.4-dev`/`1.4`, matching `present_on_1_4_dev: false`).
3. If non-empty, resolve each new SHA via `gh api repos/{repo}/commits/{sha}/pulls` → gives real backport PR + `baseRefName`.
4. Secondary/confirmatory signal only (noisy, many false positives): `gh pr list --search "<keyword> in:title" --base <release>-dev --state all`. Real backport PRs use consistent title markers: `[backport/1.4]`, `[1.4 backport]`, `[1.4]`, `(1.4 backport)` — case/bracket varies, so match loosely (`backport`) then filter by `baseRefName`.
5. **Do not** rely on `git log --all -- path`: local clone has 200+ contributor/topic remotes, producing massive noise (dozens of unrelated feature branches touch the same path via rebases/cherry-picks that never became backport PRs). Restrict to `origin/<branch>` only.

False-positive risks: title keyword search (step 4) catches unrelated PRs sharing the word (e.g. "customization in:title" matched a wizard-step PR and a deploy-overlay PR, neither related to the customization service). Path-based `git log` (step 2/3) is precise and should be the primary signal; title search is only a cheap pre-filter.

## B) Follow-up discovery — recommended algorithm

1. `git log --oneline origin/master -- <path-or-dir-from-intro-commit>` → full chronological list of every commit touching that path, oldest = intro commit.
2. For each SHA not already in the landing PR's known commit list, resolve via `gh api repos/{repo}/commits/{sha}/pulls --jq '.[0]|"\(.number)\t\(.baseRefName)\t\(.mergedAt)\t\(.title)"'`. Dedup by PR number.
3. Discard PRs merged **before** the intro commit's `authored_at` (pre-existing/renamed history) and the landing PR itself.
4. This is far more precise than `gh pr list --search "<keyword> in:title"`, which returns unrelated PRs sharing generic words (disk/customization/zenoh appear in many unrelated titles).

False-positive risk: none observed with the path+commits/pulls method (every resolved PR was genuinely on that feature's files). Risk is only incompleteness if a follow-up PR touched *only* a renamed/moved path not covered by the glob.

## Rate limits

- `commits/{sha}/pulls` uses core REST quota (5000/hr authenticated) — cheap, ~1 req/commit, ~1s latency each; looping 20+ sequentially is fine.
- `gh pr list --search` uses Search API quota (30/min) — the tight one; batch/cache aggressively, avoid loops per-SHA.

## Example PRs found

- **Zenoh** (intro `127f885b2daf`, landing #3300): follow-ups on master: **#3313, #3368, #3376, #3381, #3386, #3387, #3391, #3392, #3393, #3407, #3635, #3820, #3953**. No backport PR (0 commits on `origin/1.4-dev`/`1.4` touching `ZenohInspectorView.vue`/`zenoh-inspector/`).
- **DiskUsage** (intro `c29e24679e3d`, landing #3669): follow-up **#3691** ("Add speed test", extended by later same-path commits). No backport (0 commits on `1.4-dev`/`1.4` for `core/services/disk_usage`).
- **Customization** (intro `2d797c10c681`, landing #3930): no follow-up commits yet on `core/services/customization` beyond the landing PR's own commits. No backport (0 commits on `1.4-dev`/`1.4`), consistent with "1.5-only".

Written to: `catalog/extras/feature-traces-orch/DISCOVERY_PROBE.md`
