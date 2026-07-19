# Schema v2 enricher — implementation notes

`catalog/scripts/enrich_feature_traces.py` rewritten in place. Validated with
`python3 -m py_compile` and one `--journey InspectZenohNetwork` dry run
(output deleted afterwards — not the real deliverable, just a smoke test).

## Implemented (all 7 gaps)

1. **Backport discovery**: `discover_release_branches()` reads `git branch -r`
   and keeps only `origin/<n>.<n>` / `origin/<n>.<n>-dev` refs (currently
   1.0…1.4). For each, path-scoped `git log --format=%H <branch> -- <paths>`
   (never `--all`), new shas resolved via `commits/{sha}/pulls`, kept only if
   the resolved PR's `baseRefName` matches `^\d+\.\d+(-dev)?$`.
2. **Follow-up discovery**: same path-scoped `git log` over `origin/master`,
   resolved the same way, deduped by PR number, landing PR(s) and PRs merged
   before the intro commit's `authored_at` dropped.
3. **squash_merge / intro_sha_in_pr_commits / merge_commit_sha**: computed
   exactly per SCHEMA_V2's formula from the first `landing_prs` entry.
4. **files_changed**: `git show --name-only` (commits, cap 400) and `gh pr
   view --json files` (PRs, cap 400), both with a `*_truncated` flag.
5. **PR body**: stored in full, capped at 20000 chars with `body_truncated`.
6. **Issues sources[]**: `closing` (`closingIssuesReferences`), `body`/
   `commit` (widened `ISSUE_REF_RE` — comma/`and`-separated closing lists,
   bare `#N` 1–6 digits, `github.com/owner/repo/issues/N` URLs), `timeline`
   (`gh api issues/{pr}/timeline`, cross-referenced events — PRs are issues
   in the GH REST API, so this reads *from* each cluster PR rather than
   needing to already know a candidate issue number), `search` (`gh issue
   list --search "<module> in:title"`, only tried when a cluster has zero
   issues from every other signal).
7. **Output**: `schema_version: 2`, `generated_at`, deduped `commits{}` /
   `pull_requests{}` / `issues{}` indexes, `intro_clusters{}` keyed by intro
   sha, `journeys[]` passed through unchanged from the presence map (already
   carries `intro_commit`, no embedded trace blob).

Kept: `--journey`/`--refresh`/`--only-1_5_exclusive`, `catalog/.cache/gh_traces/`
cache dir, `bluerobotics/BlueOS` repo. Rate limiting: the small sleep moved
*inside* each `fetch()` closure passed to `cached_json`, so it only fires on
a cold (cache-miss) fetch, never on a warm cache hit. PR detail cache key
bumped `pr_{number}` → `pr_v2_{number}` since the v1 cache shape lacks
`files`/`mergeCommit` — reusing it as-is would have silently produced
incomplete v2 records.

## Deferred / known limitations

- **squash vs. rebase-merge ambiguity**: SCHEMA_V2's formula (`merge_commit_sha
  present and not in commit_shas`) can't tell "squash merge" apart from
  "rebase and merge" — both leave the merged sha out of the PR's (pre-rebase)
  commit list. Confirmed on PR #3300 (an ordinary merge, not a squash), which
  still reports `squash_merge: true`. Implemented exactly as specified rather
  than deviating from SCHEMA_V2.
- Only the *first* `landing_prs` entry feeds squash/`intro_sha_in_pr_commits`/
  `merge_commit_sha` when a sha resolves to more than one PR (schema itself
  calls this rare).
- `search` keyword is the journey's `module` field only, no synonym list.
- `git show --name-only` on a true (non-squash, 2-parent) merge commit
  returns no files by default; no `--first-parent -m` fallback was added
  since no intro commit in the current presence map hit this case.
- Full enricher run intentionally not executed (implementation-only task).
- **Post-impl tighten (required for quality):** unbounded `git log origin/master`
  on shared paths (e.g. `start-blueos-core`) was pathological. Fixed with
  `intro_sha..origin/master`, `MAX_DISCOVERY_SHAS=80`, expanded noise denylist,
  module-preferred `discovery_paths()`, backport **title** filter
  (`backport` / `[1.4…`), and issue body/timeline harvest limited to landing PRs.
  See `VALIDATION_REPORT.md`.
