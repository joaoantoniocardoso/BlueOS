# squash_merge field QA

Root cause: `bluerobotics/BlueOS` repo settings are `allow_merge_commit=false,
allow_squash_merge=false, allow_rebase_merge=true` (`gh api repos/bluerobotics/BlueOS`).
Every merged PR is **rebase-merged**: each original commit is replayed onto
`master` with a *new* SHA (different parent ⇒ different hash), so
`merge_commit_sha` (the rebased tip) is, by construction, never a member of
the PR's pre-rebase `commit_shas`. The current formula
(`merge_commit_sha not in commit_shas`) treats that as "squash", so it fires
on virtually every PR — squash and rebase are indistinguishable to it. Only
true 2-parent merge commits (8 exist in ~4722 commits, historical/admin
overrides) would ever legitimately clear the check, and the current formula
also mis-flags those as squash (never checks parent count).

## 1. Per-PR actual method vs formula

Verified via `git rev-list --parents -n1 <merge_sha>` (parent count) +
`git log --format=%s -n<N> <merge_sha>` vs `gh pr view --json commits`
headlines (subject-chain alignment, oldest→newest, all matched exactly):

| PR | feature | parents | commits | chain subjects match PR commits? | **actual method** | **current formula** |
|---|---|---|---|---|---|---|
| #3300 | zenoh | 1 | 12 | yes, 1:1 | rebase | squash_merge=true (**wrong**) |
| #3669 | disk_usage | 1 | 2 | yes, 1:1 | rebase | squash_merge=true (**wrong**) |
| #3930 | customization | 1 | 17 | yes, 1:1 | rebase | squash_merge=true (**wrong**) |
| #3826 | LevelHorizon | 1 | 1 | yes (trivial) | rebase | squash_merge=true (**wrong**) |
| #3602 | internet speed | 1 | 1 | yes (trivial) | rebase | squash_merge=true (**wrong**) |
| #871 | old nginx cluster | 1 | 18 | yes, 1:1 | rebase | squash_merge=true (**wrong**) |

All 6 are rebase merges, none squash, none ordinary 2-parent merge. `#3300`
is not a special case — the bug is systemic (100% false-positive rate on
this sample), not a one-off.

## 2. Recommended formula (git/gh only, no new deps)

```
parents = git rev-list --parents -n1 <merge_commit_sha>   // drop leading sha token
if parents.len() >= 2:            → MergeCommit   (squash_merge = false)
else:
  n = commit_shas.len()
  chain = git log --format=%s -n<n> <merge_commit_sha>       // first-parent, newest→oldest
  original = commit_subjects reversed (already fetched via gh pr view --json commits, messageHeadline)
  if chain == original (element-wise): → RebaseMerge (squash_merge = false)
  else:                                 → SquashMerge (squash_merge = true)
```

- Needs one new stored field: commit `messageHeadline`/subject per entry in
  `commit_shas` (already fetched by `gh pr view --json commits`, just not
  persisted on `PrEntry` today — currently only `oid` survives into
  `commit_shas`).
- For `n==1` squash vs rebase are structurally identical (both yield one new
  commit); subject match still correctly classifies as rebase since GitHub
  preserves the message verbatim on rebase, whereas a real squash commonly
  appends `(#NNNN)` — acceptable secondary tie-break, not required for BlueOS
  today since squash is disabled repo-wide.
- Store `merge_method: MergeCommit | Rebase | Squash` (enum) instead of/alongside
  the bool; keep `squash_merge` as a derived `== Squash` for backward compat.

## 3. `intro_sha_in_pr_commits` semantics

Same root cause hits this field: it checks `intro_sha ∈ commit_shas` (pre-rebase
SHAs), which is structurally always false under rebase-merge, same as
`squash_merge`. `SCHEMA_V2.md`'s doc comment ("intro commit *is* the squash
merge commit... common BlueOS case") is factually wrong — it's the *rebased*
merge commit, not squash. Recommend: keep the field's computation as-is
(sha-membership is still a correct, cheap check), but fix the doc comment to
say "rebase merge tip" instead of "squash merge commit", and stop treating
`false` as a squash signal — it should instead be read as "intro_sha predates
rebase (expected for any rebase-merged PR)", orthogonal to `merge_method`.
