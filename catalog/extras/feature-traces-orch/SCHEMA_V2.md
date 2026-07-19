# `feature_traces.json` — schema_version 2 (frozen proposal)

Design-only. No enricher code changes here. Aligns 1:1 with
`DISCOVERY_PROBE.md`'s algorithms A (backport) and B (follow-up).

## Top level

```json
{
  "schema_version": 2,
  "repo": "bluerobotics/BlueOS",
  "source_presence": "catalog/feature_presence_map.json",
  "tool": "gh + git (catalog/scripts/enrich_feature_traces.py)",
  "generated_at": "2026-07-19T12:33:00-03:00",
  "commits": { "<full_sha>": Commit },
  "pull_requests": { "<number_as_string>": PullRequest },
  "issues": { "<number_as_string>": Issue },
  "intro_clusters": { "<intro_sha>": IntroCluster },
  "journeys": [ Journey ]
}
```

`commits` / `pull_requests` / `issues` are deduped entity indexes (gap 7).
Every relationship is a reference (sha string or number), never an inline
copy — this replaces v1's per-journey duplicated `trace` blob.

## `Commit`

```json
{
  "sha": "127f885b2dafa5a2c4ac39420fb1f5a6bc38bbab",
  "short": "127f885b2daf",
  "subject": "core: frontend: views: ZenohInspectorView: First commit",
  "author_name": "Patrick José Pereira",
  "author_email": "patrickelectric@gmail.com",
  "authored_at": "2025-05-08T11:00:05-03:00",
  "body": "Signed-off-by: ...",
  "url": "https://github.com/bluerobotics/BlueOS/commit/127f885b2daf...",
  "files_changed": ["core/frontend/src/views/ZenohInspectorView.vue"],
  "files_changed_truncated": false
}
```

`files_changed` via local `git show --name-only`; cap at 400 paths, set
`files_changed_truncated: true` and keep the first 400 if exceeded (gap 4).

## `PullRequest`

```json
{
  "number": 3300,
  "title": "Add initial frontend zenoh integration",
  "url": "https://github.com/bluerobotics/BlueOS/pull/3300",
  "state": "MERGED",
  "author": "patrickelectric",
  "merged_at": "2025-05-15T21:54:13Z",
  "closed_at": "2025-05-15T21:54:13Z",
  "base_ref": "master",
  "head_ref": "zenoh-integration",
  "labels": ["docs-needed"],
  "body": "## Summary\n...",
  "body_truncated": false,
  "commit_shas": ["caf22779272b469216ca4249fc5c0e4108728f32"],
  "commit_headlines": ["core: frontend: Add initial zenoh integration"],
  "files_changed": ["core/frontend/src/views/ZenohInspectorView.vue"],
  "files_changed_truncated": false,
  "merge_commit_sha": "127f885b2dafa5a2c4ac39420fb1f5a6bc38bbab"
}
```

`commit_shas`/`files_changed` are refs/paths only; full commit detail lives
in the `commits` index (gap 4/7). `commit_headlines` (same order as
`commit_shas`, from `gh pr view --json commits[].messageHeadline`) exists
solely to feed the `merge_method` classification below — `gh` truncates long
headlines with a trailing `…`, so comparisons against it must be prefix-aware.
`body` is stored in full (gap 5); cap at 20000 chars only if exceeded, then
set `body_truncated: true`. `base_ref` alone tells the reader whether a PR is
a `1.4`/`1.4-dev` backport (gap 1) — no redundant boolean needed on the PR
itself.

## `Issue`

```json
{
  "number": 3512,
  "title": "Zenoh inspector crashes on...",
  "url": "https://github.com/bluerobotics/BlueOS/issues/3512",
  "state": "CLOSED",
  "author": "someuser",
  "created_at": "2025-05-10T00:00:00Z",
  "closed_at": "2025-05-16T00:00:00Z",
  "labels": ["bug"],
  "missing": false
}
```

Unchanged from v1 shape (kept). `missing: true` + null fields when `gh
issue view` 404s (deleted/transferred/cross-repo).

## `IntroCluster` — the relational record gaps 1/2/3/6 attach to

```json
{
  "intro_commit": "127f885b2dafa5a2c4ac39420fb1f5a6bc38bbab",
  "landing_prs": [3300],
  "squash_merge": false,
  "merge_method": "rebase",
  "intro_sha_in_pr_commits": false,
  "merge_commit_sha": "127f885b2dafa5a2c4ac39420fb1f5a6bc38bbab",
  "backport_prs": [3350],
  "follow_up_prs": [3313, 3368, 3376, 3381, 3386],
  "issues": [
    { "number": 3512, "sources": [{ "kind": "closing", "pr": 3300 }] },
    { "number": 3520, "sources": [{ "kind": "body", "pr": 3313 }, { "kind": "timeline", "pr": null }] }
  ]
}
```

- **`landing_prs`** (array, usually len 1): PRs resolved from
  `commits/{intro_sha}/pulls` — v1's only PR link. Array because that API
  can return >1 PR for a shared sha (rare, but v1 already looped it).
- **merge/squash fields** (gap 3, revised per `SQUASH_QA.md`): BlueOS
  disables squash and merge-commit merges repo-wide (rebase-only), so
  `merge_commit_sha not in commit_shas` is *always* true and cannot
  distinguish squash from rebase on its own. `merge_method` (`merge_commit
  | rebase | squash | unknown`) is classified instead via
  `git rev-list --parents` (≥2 parents ⇒ `merge_commit`) and, for 1-parent
  tips, by comparing the rebased subject chain (`git log --format=%s`) to
  the PR's pre-rebase commit headlines (`commit_headlines`, from `gh pr
  view --json commits`) — a match (either order) ⇒ `rebase`, a mismatch ⇒
  `squash`; no `merge_commit_sha` ⇒ `unknown`. `squash_merge` is now a
  derived `merge_method == "squash"`. `intro_sha_in_pr_commits = intro_commit
  in landing_pr.commit_shas`; under rebase-merge it is `false` by
  construction (rebase gives every commit a new sha) — read it as "intro
  predates the rebase", not as a squash signal. `merge_commit_sha` comes
  from `gh pr view --json mergeCommit`.
- **`backport_prs`** (gap 1, Probe §A): for each release line, `git log
  origin/<line>-dev -- <intro commit's files_changed>` (and `origin/<line>`
  too); empty = no backport for that line. Non-empty SHAs resolved via
  `commits/{sha}/pulls`, kept only if `baseRefName` matches a release line.
  Title-keyword search (`gh pr list --search`) is a pre-filter only, never
  the sole signal, per Probe's false-positive note.
- **`follow_up_prs`** (gap 2, Probe §B): `git log origin/master -- <path>`,
  resolve each new SHA via `commits/{sha}/pulls`, dedup by PR number,
  discard PRs merged before `intro_commit.authored_at` and the landing PR
  itself.
- **`issues[].sources[].kind`** (gap 6) — enum: `closing | body | commit |
  timeline | search`.
  - `closing`: PR's `closingIssuesReferences`.
  - `body`: `#NNNN` regex hit in a PR body (existing `ISSUE_REF_RE`).
  - `commit`: `#NNNN` regex hit in a commit subject/body (intro, PR
    branch, or follow-up/backport commit).
  - `timeline`: confirmation pass — issue's own cross-reference timeline
    mentions one of this cluster's PRs/commits, found without a prior
    text-regex hit.
  - `search`: `gh issue list --search` keyword fallback — lowest
    confidence, used sparingly (mirrors Probe's backport-title caveat).
  Same issue can carry multiple source entries; dedup by `number` in the
  `issues` index, keep all sources here.

## `Journey`

```json
{
  "journey": "InspectZenohNetwork",
  "module": "zenohd",
  "method": "path:core/frontend/src/views/ZenohInspectorView.vue",
  "first_tag": "1.4.4-beta.16",
  "present_on_master": true,
  "present_on_1_4_dev": false,
  "present_in_tags": ["1.4.4-beta.16", "..."],
  "intro_commit": "127f885b2dafa5a2c4ac39420fb1f5a6bc38bbab"
}
```

Passthrough fields unchanged from `feature_presence_map.json`/v1. The only
structural change: no embedded `trace` — resolve via
`intro_clusters[intro_commit]` (gap 7). Multiple journeys sharing an intro
commit now share one cluster instead of duplicating it N times.

## v1 → v2 keep / drop

**Keep as-is:** journey passthrough fields; `Commit`/`PullRequest`/`Issue`
field names (`sha`, `short`, `subject`, `number`, `base_ref`, `missing`,
...); `ISSUE_REF_RE` regex; `catalog/.cache/gh_traces/` cache dir and
per-key JSON cache files; `--journey`/`--refresh`/`--only-1_5_exclusive`
CLI flags.

**Drop:** per-journey embedded `trace: {intro_commit, pull_requests,
issues}` blob (→ indexes + `intro_clusters` + `intro_commit` ref);
inline `commits` arrays nested inside each PR object (→ `commit_shas`
refs); `schema_version: 1`.

**Add:** `generated_at`; `commits`/`pull_requests`/`issues` top-level
indexes; `intro_clusters` (backport/follow-up/squash/multi-source issues);
`files_changed` on `Commit` and `PullRequest`; `body`/`body_truncated` on
`PullRequest`; `merge_commit_sha`; `commit_headlines` on `PullRequest`;
`merge_method` on `IntroCluster`.
