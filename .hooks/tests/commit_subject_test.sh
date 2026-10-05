#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
# shellcheck disable=SC1091
source "$ROOT_DIR/.hooks/lib/commit_subject.sh"

fail() {
    printf 'commit_subject_test: %s\n' "$1" >&2
    exit 1
}

expect_accept() {
    local label=$1
    local subject=$2
    shift 2
    local output
    if ! output=$(check_commit_subject "$subject" "$@"); then
        printf 'commit_subject_test: %s: expected accept\nsubject: %s\n%s\n' "$label" "$subject" "$output" >&2
        exit 1
    fi
}

expect_reject() {
    local label=$1
    local pattern=$2
    local subject=$3
    shift 3
    local output
    if output=$(check_commit_subject "$subject" "$@"); then
        printf 'commit_subject_test: %s: expected reject for %s\n' "$label" "$subject" >&2
        exit 1
    fi
    if ! grep -F -q -- "$pattern" <<<"$output"; then
        printf 'commit_subject_test: %s: expected %q in:\n%s\n' "$label" "$pattern" "$output" >&2
        exit 1
    fi
}

expect_log_accept() {
    local label=$1
    local log=$2
    local output
    if ! output=$(check_commit_subjects_from_log "$log"); then
        printf 'commit_subject_test: %s: expected the log to pass\n%s\n' "$label" "$output" >&2
        exit 1
    fi
}

expect_log_reject() {
    local label=$1
    local pattern=$2
    local log=$3
    local output
    if output=$(check_commit_subjects_from_log "$log"); then
        fail "$label: expected the log to fail"
    fi
    if ! grep -F -q -- "$pattern" <<<"$output"; then
        printf 'commit_subject_test: %s: expected %q in:\n%s\n' "$label" "$pattern" "$output" >&2
        exit 1
    fi
}

test_style_guide_examples() {
    expect_accept "webrtc sink stem" \
        "src: lib: stream: sink: webrtc_sink: Fix double-free on pipeline teardown" \
        "src/lib/stream/sink/webrtc_sink.rs"
    expect_accept "mavlink directory" \
        "src: lib: mavlink: Fix CAMERA_INFORMATION using wrong fields" \
        "src/lib/mavlink/camera.rs"
    expect_accept "cargo alias" \
        "cargo: Add tikv-jemallocator dependency" \
        "core/Cargo.toml" "core/Cargo.lock"
    expect_accept "tests alias" \
        "tests: Add redirect pipeline integration tests for H264 and H265" \
        "core/libs/app/service/tests/redirect.rs" \
        "video/test_redirect.py"
    expect_accept ".github workflows path" \
        ".github: workflows: Adopt cargo-nextest for parallel test execution" \
        ".github/workflows/test-and-deploy.yml"
    expect_accept "build script alias" \
        "build: Add the crate build script" \
        "core/libs/idl/build.rs" \
        "deploy/build/package.sh"
}

test_prefix_covers_every_file() {
    expect_accept "directory covers nested cargo manifest" \
        "core: services: recorder: app: Add the handler" \
        "core/services/recorder/app/src/lib.rs" \
        "core/services/recorder/app/Cargo.toml"
    expect_accept "raised prefix covers the lockfile" \
        "core: Add the domain crate" \
        "core/Cargo.lock" \
        "core/libs/logic/domain/src/lib.rs"
    expect_accept "stem and basename both name the file" \
        ".hooks: lib: rust_checks: Add the folder gate" \
        ".hooks/lib/rust_checks.sh"
    expect_accept "full basename names the file" \
        ".hooks: lib: rust_checks.sh: Allow the cookbook crate folder" \
        ".hooks/lib/rust_checks.sh"
    expect_reject "sibling file outside the prefix" \
        "src/other.rs" \
        "src: mcap: Fix the reader" \
        "src/mcap.rs" "src/other.rs"
}

test_lowercase_segments_and_capital_description() {
    expect_reject "uppercase cargo prefix" \
        "path segments must be lowercase" \
        "Cargo: update lock file" \
        "core/Cargo.lock"
    expect_reject "uppercase segment before the description" \
        "path segments must be lowercase" \
        "core: Services: Add the handler" \
        "core/services/app.rs"
    expect_reject "description does not start with a capital" \
        "description must start with a capital letter" \
        "cargo: add a dependency" \
        "core/Cargo.toml"
    expect_reject "missing path prefix" \
        "missing path prefix" \
        "Fix some bug" \
        "src/mcap.rs"
    expect_reject "slash is not a segment separator" \
        "missing path prefix" \
        "core/frontend: Map the codec import" \
        "core/frontend/vite.config.js"
}

test_conventional_commit_prefixes() {
    local conventional_prefix
    for conventional_prefix in feat fix perf chore deps runtime refactor style ci test revert; do
        expect_reject "conventional $conventional_prefix" \
            "conventional-commit prefix" \
            "$conventional_prefix: Add the change" \
            "src/mcap.rs"
    done
    expect_accept "docs is a path when it covers the files" \
        "docs: adr: decisions: Record the measured release binary sizes" \
        "docs/adr/decisions.md"
    expect_reject "docs used as a type rather than a path" \
        "README.md" \
        "docs: Update the readme" \
        "README.md"
}

test_aliases() {
    expect_reject "cargo alias does not cover source" \
        "core/services/recorder/app/src/lib.rs" \
        "cargo: Update the lockfile" \
        "core/Cargo.lock" \
        "core/services/recorder/app/src/lib.rs"
    expect_accept "tests alias plus the file stem" \
        "tests: effects: Apply cargo fmt" \
        "core/libs/app/service/tests/effects.rs"
    expect_reject "tests alias does not cover production source" \
        "core/libs/app/service/src/lib.rs" \
        "tests: Add panic recovery coverage" \
        "core/libs/app/service/tests/recovery.rs" \
        "core/libs/app/service/src/lib.rs"
    expect_reject "workflows prefix does not cover other github files" \
        ".github/lib/install_ast_metrics.sh" \
        ".github: workflows: Add report-only Rust CI" \
        ".github/workflows/test-and-deploy.yml" \
        ".github/lib/install_ast_metrics.sh"
    expect_reject "build alias does not cover crate source" \
        "core/libs/idl/src/lib.rs" \
        "build: Add the crate build script" \
        "core/libs/idl/src/lib.rs"
}

test_root_files_renames_and_deletions() {
    expect_accept "glossary stem is lowercase" \
        "glossary: Define the DomainState" \
        "GLOSSARY.md"
    expect_accept "agents stem is lowercase" \
        "agents: Point Rust work at the decisions" \
        "AGENTS.md"
    expect_accept "readme stem is lowercase" \
        "readme: Update the install instructions" \
        "README.md"
    expect_accept "dotfile basename" \
        ".gitignore: Ignore the Rust target folder" \
        ".gitignore"
    expect_accept "rename covers the old and new path" \
        "docs: architecture: Move draft 1 reviews into draft-1" \
        "docs/architecture/rust-service-overview.md" \
        "docs/architecture/draft-1/rust-service-overview.md"
    expect_reject "rename prefix misses the old path" \
        "docs/architecture/rust-service-overview.md" \
        "docs: architecture: draft-1: Move draft 1 reviews into draft-1" \
        "docs/architecture/rust-service-overview.md" \
        "docs/architecture/draft-1/rust-service-overview.md"
    expect_accept "deletion is covered by its path" \
        "core: services: tank: Remove the service" \
        "core/services/tank/app/src/lib.rs"
}

test_real_errors() {
    expect_reject "glued prefixes" \
        "core/services/recorder/src/lib.rs" \
        "core: libs: logic: ros2-names: core: services: recorder: Fix ros2 name parsing" \
        "core/libs/logic/ros2-names/src/lib.rs" \
        "core/services/recorder/src/lib.rs"
    expect_reject "recorder app commit also changes the lockfile" \
        "core/Cargo.lock" \
        "core: services: recorder: app: Add endpoint compile-fail tests" \
        "core/services/recorder/app/tests/compile_errors.rs" \
        "core/Cargo.lock"
    expect_accept "rust-style subject covers the style guide" \
        "docs: architecture: rust-style: Allow the paused-clock drain sleep in tests" \
        "docs/architecture/rust-style.md"
    expect_reject "rust-style commit also changes a test file" \
        "core/libs/tools/rust-style-check/tests/fixtures.rs" \
        "docs: architecture: rust-style: Allow the paused-clock drain sleep in tests" \
        "docs/architecture/rust-style.md" \
        "core/libs/tools/rust-style-check/tests/fixtures.rs"
}

test_log_splits_commits() {
    local good_log bad_log
    good_log=$(cat <<'EOF'
---
aaaaaaaa
docs: architecture: Move draft 1 reviews into draft-1

docs/architecture/rust-service-overview.md
docs/architecture/draft-1/rust-service-overview.md
EOF
)
    expect_log_accept "two paths of one rename" "$good_log"

    bad_log=$(cat <<'EOF'
---
aaaaaaaa
docs: architecture: Move draft 1 reviews into draft-1

docs/architecture/rust-service-overview.md
---
bbbbbbbb
core: services: recorder: app: Add endpoint compile-fail tests

core/services/recorder/app/tests/compile_errors.rs
core/Cargo.lock
EOF
)
    expect_log_reject "second commit in the log" "core/Cargo.lock" "$bad_log"
}

test_push_base_missing_from_the_clone() {
    local output
    if ! output=$(BASE_SHA=0000000000000000000000000000000000000000 HEAD_SHA=HEAD check_commit_subjects 2>&1); then
        printf 'commit_subject_test: a push that creates a branch must pass\n%s\n' "$output" >&2
        exit 1
    fi
    if ! grep -F -q -- "not in this clone" <<<"$output"; then
        printf 'commit_subject_test: expected the skip to be named in:\n%s\n' "$output" >&2
        exit 1
    fi
}

main() {
    test_style_guide_examples
    test_prefix_covers_every_file
    test_lowercase_segments_and_capital_description
    test_conventional_commit_prefixes
    test_aliases
    test_root_files_renames_and_deletions
    test_real_errors
    test_log_splits_commits
    test_push_base_missing_from_the_clone
    printf 'commit_subject_test: ok\n'
}

main "$@"
