#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
cd "$ROOT_DIR"

report_script="$ROOT_DIR/.hooks/lib/ergonomics_report.sh"

assert_report_shape() {
    local output=$1
    local label=$2
    local measure_rows
    measure_rows=$(grep -c '^| [^|]* | [^|]* | [^|]* | [^|]* |$' <<<"$output" || true)
    if [ "$measure_rows" -lt 7 ]; then
        printf 'ergonomics_report_test: %s: expected at least seven measure rows\n%s\n' "$label" "$output" >&2
        exit 1
    fi
    while IFS='|' read -r _ measure measured _ status _; do
        measure=$(echo "$measure" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
        measured=$(echo "$measured" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
        status=$(echo "$status" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
        [ -n "$measure" ] || continue
        [[ $measure == Measure ]] && continue
        [[ $measure == ---* ]] && continue
        if [[ $measure == *"Time to first endpoint"* ]]; then
            [[ $measured == "(not measured in CI)" ]] || {
                printf 'ergonomics_report_test: %s: time row measured column\n' "$label" >&2
                exit 1
            }
            [[ $status == "documented goal only" ]] || {
                printf 'ergonomics_report_test: %s: time row status\n' "$label" >&2
                exit 1
            }
            continue
        fi
        if [[ $measure == *"Files touched"* ]]; then
            [[ $measured =~ ^[0-9]+$ ]] || {
                printf 'ergonomics_report_test: %s: files row not numeric: %s\n' "$label" "$measured" >&2
                exit 1
            }
            continue
        fi
        if [[ ! $measured =~ ^[0-9]+$ ]]; then
            printf 'ergonomics_report_test: %s: non-numeric measured for %s: %s\n' "$label" "$measure" "$measured" >&2
            exit 1
        fi
        case "$status" in
            "meets goal ("*) ;;
            "above goal ("*) ;;
            "reported only") ;;
            "documented goal only") ;;
            *)
                printf 'ergonomics_report_test: %s: bad status for %s: %s\n' "$label" "$measure" "$status" >&2
                exit 1
                ;;
        esac
    done <<<"$(grep '^| ' <<<"$output")"
}

build_fixture() {
    local fixture_root=$1
    mkdir -p "$fixture_root/core/ergonomics"
    mkdir -p "$fixture_root/core/services/example/app/src"
    mkdir -p "$fixture_root/core/services/example/logic/domain/src"
    mkdir -p "$fixture_root/core/services/example/logic/api/src"
    mkdir -p "$fixture_root/core/libs/logic/domain/src"
    mkdir -p "$fixture_root/core/libs/logic/jobs/src"
    mkdir -p "$fixture_root/core/libs/app/service/src/builder"
    mkdir -p "$fixture_root/core/libs/app/service/src/entry"
    mkdir -p "$fixture_root/core/libs/app/service/src/kernel"

    cat >"$fixture_root/core/ergonomics/d31-example-command-concepts.txt" <<'EOF'
alpha
beta
EOF

    cat >"$fixture_root/core/services/example/app/endpoints.toml" <<'EOF'
service = "example"

[command]
Ping = { request = "example_msgs/Ping" }
EOF

    cat >"$fixture_root/core/services/example/app/Cargo.toml" <<'EOF'
[package]
name = "fixture-app"
EOF

    cat >"$fixture_root/core/services/example/logic/domain/Cargo.toml" <<'EOF'
[package]
name = "fixture-domain"
EOF

    cat >"$fixture_root/core/services/example/logic/api/Cargo.toml" <<'EOF'
[package]
name = "fixture-api"
EOF

    cat >"$fixture_root/core/services/example/logic/domain/src/lib.rs" <<'EOF'
pub enum Request { Ping }
pub fn handle(_request: Request) {}
EOF

    cat >"$fixture_root/core/services/example/logic/api/src/lib.rs" <<'EOF'
pub fn ping(_request: ()) {}
EOF

    cat >"$fixture_root/core/services/example/app/src/service.rs" <<'EOF'
pub struct Service;
EOF

    for framework_file in \
        "$fixture_root/core/libs/logic/domain/src/lib.rs" \
        "$fixture_root/core/libs/logic/jobs/src/lib.rs" \
        "$fixture_root/core/libs/app/service/src/builder/mod.rs" \
        "$fixture_root/core/libs/app/service/src/kernel/mod.rs" \
        "$fixture_root/core/libs/app/service/src/entry/run.rs"; do
        printf '// fixture framework\n' >"$framework_file"
    done
}

fixture_root=$(mktemp -d)
trap 'rm -rf "$fixture_root"' EXIT
build_fixture "$fixture_root"

fixture_output=$("$report_script" "$fixture_root")
assert_report_shape "$fixture_output" "fixture"

if ! grep -Fq '| Concepts per Command (example-minimal) | 2 |' <<<"$fixture_output"; then
    printf 'ergonomics_report_test: fixture concepts count\n%s\n' "$fixture_output" >&2
    exit 1
fi

if ! grep -Fq '| Smallest complete Service (hand-written lines) | 14 |' <<<"$fixture_output"; then
    printf 'ergonomics_report_test: fixture service line count (expected 14)\n%s\n' "$fixture_output" >&2
    exit 1
fi

live_output=$("$report_script" "$ROOT_DIR")
assert_report_shape "$live_output" "live tree"
