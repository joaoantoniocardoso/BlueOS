#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
cd "$ROOT_DIR"

checker="$ROOT_DIR/.hooks/lib/check_docs_paths.py"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

fail() {
    printf 'docs_paths_test: %s\n' "$1" >&2
    exit 1
}

git_quiet() {
    local repository=$1
    shift
    git -C "$repository" -c core.hooksPath=/dev/null "$@"
}

init_repository() {
    local repository=$1
    mkdir -p "$repository"
    git_quiet "$repository" init -q
    git_quiet "$repository" add --force --all
}

build_fixture() {
    local repository=$1
    mkdir -p \
        "$repository/core/services/example/app/src" \
        "$repository/core/app/blueos/src" \
        "$repository/core/libs/adapters/comms-zenoh" \
        "$repository/core/frontend/node_modules/pkg" \
        "$repository/core/target" \
        "$repository/.hooks" \
        "$repository/docs/architecture/draft-1" \
        "$repository/node_modules/pkg" \
        "$repository/target" \
        "$repository/vendor" \
        "$repository/notes"

    printf 'kept\n' >"$repository/core/real.rs"
    printf 'kept\n' >"$repository/core/services/kept.txt"
    printf 'kept\n' >"$repository/core/services/example/app/src/lib.rs"
    printf 'manifest\n' >"$repository/core/services/example/app/endpoints.toml"
    printf 'kept\n' >"$repository/core/app/blueos/src/lib.rs"
    printf 'kept\n' >"$repository/core/libs/adapters/comms-zenoh/lib.rs"
    printf '#!/bin/sh\n' >"$repository/.hooks/pre-push"
    printf 'x\n' >"$repository/docs/picture.png"
    printf '# spaced\n' >"$repository/docs/space file.md"

    cat >"$repository/docs/guide.md" <<'EOF'
# Guide

Exists as a backtick: `core/real.rs`, `core/services`, `core/services/`, `.hooks/pre-push`.

Missing backtick: `core/missing.rs`.

Missing dot-directory backtick: `.hooks/missing.sh`.

Repo-root link from this folder: [real](core/real.rs).

Absolute repo link: [absolute](/core/real.rs) and [absolute missing](/core/missing-absolute.rs).

Outside the repository: [passwd](/etc/passwd).

Relative link that exists: [self](./guide.md) and [up](../core/real.rs).

Relative link that is missing: [nope](./missing-guide.md) and [root](../missing-root.md).

Missing link: [missing](../core/missing-link.rs).

URL: [web](https://example.com/core/not-a-file.md) and `https://example.com/core/not-a-file.rs`.

Protocol-relative: [proto](//example.com/core/not-a-file.md).

Anchor: [section](#section) and [file](../core/real.rs#section).

Anchor in inline code is not a line suffix: `core/missing-anchor.rs#section`.

Placeholders: `core/<name>/file.rs`, `core/{name}/file.rs`, `core/*/file.rs`, `core/**/x.rs`.

Placeholder link: [skip](core/<service>/lib.rs).

Command: `cat core/missing-command.rs`.

Spaced link exists: [space](<space file.md>).

Spaced link missing: [missing space](<missing space.md>).

Line suffix exists: `core/real.rs:12`, `core/real.rs:12:4`, `core/real.rs#L10`, `core/real.rs#L10-L20`.

Line suffix missing: `core/missing-line.rs:3` and `core/missing-hash.rs#L4`.

Link line suffix missing: [line](../core/missing-link-line.rs:15).

Link with title missing: [title](../core/missing-title.rs "gone").

Link with title exists: [title](../core/real.rs "present").

Image exists: ![pic](./picture.png).

Image missing: ![missing pic](./missing-picture.png).

Manifest: `service = "tank"` and `service="tank"`.

Inline ./ and ../ are not resolved from this file: `./.hooks/pre-push` and `../core/missing-relative.rs`.

Bare relative name is not a repository path: `tests/missing.rs` and `lib.rs`.

Directory missing: `core/missing-directory`.

```toml
service = "tank"
core/fenced-missing.rs
See `core/fenced-span-missing.rs`.
```
EOF

    cat >"$repository/docs/extra.md" <<'EOF'
Missing from a docs page that is not a README: `core/docs-extra-missing.rs`.
EOF

    cat >"$repository/README.md" <<'EOF'
Missing from the repository README: `core/readme-missing.rs`.
EOF

    cat >"$repository/AGENTS.md" <<'EOF'
Missing from AGENTS: `core/agents-missing.rs`.
EOF

    cat >"$repository/GLOSSARY.md" <<'EOF'
Missing from the glossary: `core/glossary-missing.rs`.
EOF

    cat >"$repository/core/services/example/README.md" <<'EOF'
Missing from a nested README: `core/nested-readme-missing.rs`.
The service app source exists: `app/src/lib.rs`.
EOF
    cat >"$repository/docs/architecture/layout.md" <<'EOF'
The removed service is `services/tank/...`.
The zenoh adapter crate is not `libs/adapters/comms/zenoh/`.
This service directory exists: `services/example`.
A service manifest is not the multicall crate: `app/endpoints.toml`.
A first segment that is not a directory stays prose: `not-a-base/missing.rs`.
EOF

    cat >"$repository/docs/architecture/draft-1/old.md" <<'EOF'
Ignored draft record: `core/draft-missing.rs`.
EOF
    cat >"$repository/docs/architecture/draft-1/README.md" <<'EOF'
Ignored draft README: `core/draft-readme-missing.rs`.
EOF
    cat >"$repository/docs/architecture/profiling-research.md" <<'EOF'
Ignored research snapshot: `core/research-missing.rs`.
EOF
    cat >"$repository/notes/other.md" <<'EOF'
Unscoped: `core/unscoped-missing.rs`.
EOF
    cat >"$repository/node_modules/pkg/README.md" <<'EOF'
Node: `core/node-modules-missing.rs`.
EOF
    cat >"$repository/core/frontend/node_modules/pkg/README.md" <<'EOF'
Nested node: `core/nested-node-missing.rs`.
EOF
    cat >"$repository/target/README.md" <<'EOF'
Target: `core/target-missing.rs`.
EOF
    cat >"$repository/core/target/README.md" <<'EOF'
Nested target: `core/nested-target-missing.rs`.
EOF
    cat >"$repository/vendor/README.md" <<'EOF'
Submodule: `core/submodule-missing.rs`.
EOF

    cat >"$repository/.gitmodules" <<'EOF'
[submodule "vendor"]
	path = vendor
	url = https://example.com/vendor.git
EOF

    init_repository "$repository"
}

build_clean_repository() {
    local repository=$1
    mkdir -p "$repository/core" "$repository/docs/architecture/draft-1"
    printf 'kept\n' >"$repository/core/real.rs"
    cat >"$repository/README.md" <<'EOF'
See `core/real.rs`, `core/real.rs:10`, and `core/real.rs#L2`.
[real](core/real.rs) [relative](./core/real.rs) [absolute](/core/real.rs)
[web](https://example.com/core/nope.rs)
[section](#top)
`cat core/nope.rs`
`core/<name>/nope.rs`
`service = "tank"`
`service="tank"`

```toml
service = "tank"
core/fenced-missing.rs
```
EOF
    cat >"$repository/docs/architecture/draft-1/old.md" <<'EOF'
Stale on purpose: `core/draft-missing.rs`.
EOF
    init_repository "$repository"
}

build_fixture "$work/fixture"
build_clean_repository "$work/clean"

if output=$(python3 "$checker" "$work/clean" 2>"$work/clean-err"); then
    :
else
    fail "clean repository should pass: $(cat "$work/clean-err") $output"
fi
if [ -s "$work/clean-err" ]; then
    fail "clean repository wrote to stderr: $(cat "$work/clean-err")"
fi
if [ -n "$output" ]; then
    fail "clean repository reported paths: $output"
fi

if output=$(python3 "$checker" "$work/fixture" 2>"$work/fixture-err"); then
    fail "fixture should report missing paths"
fi
if [ -s "$work/fixture-err" ]; then
    fail "fixture wrote to stderr: $(cat "$work/fixture-err")"
fi

cat >"$work/expected" <<'EOF'
AGENTS.md core/agents-missing.rs
GLOSSARY.md core/glossary-missing.rs
README.md core/readme-missing.rs
core/services/example/README.md core/nested-readme-missing.rs
docs/architecture/layout.md core/libs/adapters/comms/zenoh
docs/architecture/layout.md core/services/tank/...
docs/extra.md core/docs-extra-missing.rs
docs/guide.md .hooks/missing.sh
docs/guide.md core/missing-absolute.rs
docs/guide.md core/missing-directory
docs/guide.md core/missing-hash.rs
docs/guide.md core/missing-line.rs
docs/guide.md core/missing-link-line.rs
docs/guide.md core/missing-link.rs
docs/guide.md core/missing-title.rs
docs/guide.md core/missing.rs
docs/guide.md docs/missing space.md
docs/guide.md docs/missing-guide.md
docs/guide.md docs/missing-picture.png
docs/guide.md missing-root.md
EOF
sort -o "$work/expected" "$work/expected"

sed -E "s/^([^:]+):[0-9]+: missing repository path '([^']*)'.*$/\\1 \\2/" <<<"$output" | sort >"$work/actual"

if ! diff -u "$work/expected" "$work/actual"; then
    printf 'docs_paths_test: missing-path set mismatch\n%s\n' "$output" >&2
    exit 1
fi

# A file deleted from the working tree but still in the index is gone from the commit about to be made.
rm "$work/clean/core/real.rs"
if output=$(python3 "$checker" "$work/clean" 2>&1); then
    fail "a reference to a file deleted from the working tree should fail"
fi
if ! grep -F -q "missing repository path 'core/real.rs'" <<<"$output"; then
    fail "expected the deleted core/real.rs to be named in: $output"
fi
