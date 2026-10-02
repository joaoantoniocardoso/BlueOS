#!/usr/bin/env bash

: "${fixing:=false}"

RUST_NO_STD_TARGET=thumbv7em-none-eabihf
RUST_WASM32_TARGET=wasm32-unknown-unknown

# Prints "<unit> <folder>" for a crate directory: "libs logic" for libs/logic/jobs, "calibration app" for
# services/calibration/app, "multicall app" for core/app/blueos.
crate_place() {
    if [[ $1 =~ /services/([^/]+)/([^/]+) ]]; then
        echo "${BASH_REMATCH[1]} ${BASH_REMATCH[2]}"
    elif [[ $1 =~ /libs/([^/]+) ]]; then
        echo "libs ${BASH_REMATCH[1]}"
    elif [[ $1 =~ /app/blueos$ ]]; then
        echo "multicall app"
    else
        echo "none none"
    fi
}

# Usage: check_rust_style_copies <repository_dir>
# The Rust checklist in docs/architecture/rust-style.md is copied verbatim into the files developers and agents
# read; a copy that drifts teaches a different rule.
check_rust_style_copies() {
    local repository_dir="$1"
    local source="$repository_dir/docs/architecture/rust-style.md"
    local markers='/<!-- rust-style:begin -->/,/<!-- rust-style:end -->/p'
    local checklist copy
    checklist=$(sed -n "$markers" "$source")
    if [ -z "$checklist" ]; then
        printf 'No rust-style checklist markers in %s\n' "$source" >&2
        exit 1
    fi
    for copy in "$repository_dir/AGENTS.md" "$repository_dir/.cursor/rules/rust-blueos.mdc"; do
        if [ "$(sed -n "$markers" "$copy")" != "$checklist" ]; then
            printf 'The Rust checklist in %s differs from %s; copy the block between the rust-style markers\n' \
                "$copy" "$source" >&2
            exit 1
        fi
    done
}

# Regenerates IDL output into a temp dir and byte-compares with committed `src/generated`.
check_generated_idl() {
    local workspace_dir="$1"
    local idl_generated="$workspace_dir/libs/idl/src/generated"
    local idl_typescript="$workspace_dir/libs/idl/typescript"
    local temporary
    temporary=$(mktemp -d)
    (
        cd "$workspace_dir" || exit 1
        cargo run --quiet -p blueos-idl-codegen --bin blueos-idl-codegen -- \
            --output "$temporary/generated" \
            --typescript-output "$temporary/typescript"
    )
    if ! diff -ru "$idl_generated" "$temporary/generated" >/dev/null; then
        rm -rf "$temporary"
        printf 'Committed IDL Rust is stale; fix with: (cd core && cargo run -p blueos-idl-codegen --bin blueos-idl-codegen -- --write)\n' >&2
        exit 1
    fi
    if ! diff -ru "$idl_typescript" "$temporary/typescript" >/dev/null; then
        rm -rf "$temporary"
        printf 'Committed IDL TypeScript is stale; fix with: (cd core && cargo run -p blueos-idl-codegen --bin blueos-idl-codegen -- --write)\n' >&2
        exit 1
    fi
    rm -rf "$temporary"
}

check_api_lock() {
    local workspace_dir="$1"
    (
        cd "$workspace_dir" || exit 1
        cargo test --quiet -p blueos-idl --test api_lock
    )
}

# Usage: collect_folder_violations <cargo-metadata-json>
# Prints one violation per line. Exits 1 when any violation exists.
collect_folder_violations() {
    local metadata="$1"
    local violations=() package_args=() idl_package_args=()
    local name directory unit folder
    while IFS=$'\t' read -r name directory; do
        read -r unit folder <<<"$(crate_place "$directory")"
        case "$folder" in
            logic) package_args+=("$name") ;;
            idl) [[ $directory == */codegen ]] || idl_package_args+=("$name") ;;
            adapters | app | api | tools) ;;
            *) violations+=("$name is not in logic/, adapters/, app/, idl/, or api/ under libs/ or services/<name>/") ;;
        esac
    done < <(jq -r '.packages[] | [.name, (.manifest_path | rtrimstr("/Cargo.toml"))] | @tsv' <<<"$metadata")
    local dependency dependency_directory dependency_unit dependency_folder parent
    while IFS=$'\t' read -r name directory dependency dependency_directory; do
        read -r unit folder <<<"$(crate_place "$directory")"
        read -r dependency_unit dependency_folder <<<"$(crate_place "$dependency_directory")"
        if [ "$dependency_unit" != libs ] && [ "$dependency_unit" != "$unit" ]; then
            if [ "$unit" = multicall ] && [ "$dependency_folder" = app ]; then
                continue
            fi
            violations+=("$name depends on $dependency, which belongs to another service")
        elif [ "$folder" = logic ] \
            && [ "$dependency_unit/$dependency_folder" != libs/logic ] \
            && { [ "$dependency_folder" != logic ] || [ "$dependency_unit" != "$unit" ]; }; then
            violations+=("$name is logic, so it may only depend on libs/logic or sibling logic in the same service, not on $dependency")
        elif [ "$folder" = adapters ] && [ "$dependency_folder" != adapters ] && [ "$dependency_unit/$dependency_folder" != libs/idl ]; then
            violations+=("$name is an adapter, so it may only depend on adapters, not on $dependency")
        fi
        parent=$(dirname "$dependency_directory")
        if [ -f "$parent/Cargo.toml" ] && [ "$directory" != "$parent" ] && [ "$(dirname "$directory")" != "$parent" ]; then
            violations+=("$name depends on $dependency, which is private to the crate in $parent")
        fi
    done < <(jq -r '.packages[] | .name as $name | (.manifest_path | rtrimstr("/Cargo.toml")) as $directory
        | .dependencies[] | select(.kind != "dev" and .path != null) | [$name, $directory, .name, .path] | @tsv' \
        <<<"$metadata")
    if [ ${#violations[@]} -gt 0 ]; then
        printf '%s\n' "${violations[@]}"
        return 1
    fi
    return 0
}

# Usage: run_rust_lint_checks <workspace_dir>
run_rust_lint_checks() {
    local workspace_dir="$1"

    if ! rustup target list --installed | grep -qx "$RUST_NO_STD_TARGET"; then
        printf 'Rust target not installed, run: rustup target add %s\n' "$RUST_NO_STD_TARGET" >&2
        exit 1
    fi
    if ! rustup target list --installed | grep -qx "$RUST_WASM32_TARGET"; then
        printf 'Rust target not installed, run: rustup target add %s\n' "$RUST_WASM32_TARGET" >&2
        exit 1
    fi

    echo "Running Rust lint checks for ${workspace_dir}"
    echo "Checking the Rust checklist copies.."
    check_rust_style_copies "$(git -C "$workspace_dir" rev-parse --show-toplevel)"
    (
        cd "$workspace_dir" || exit 1

        if [ "$fixing" = true ]; then
            echo "Regenerating committed IDL Rust.."
            cargo run --quiet -p blueos-idl-codegen --bin blueos-idl-codegen -- --write
            echo "Running cargo fmt.."
            cargo fmt --all
            exit 0
        fi

        echo "Checking committed IDL Rust.."
        check_generated_idl "$workspace_dir"

        echo "Checking api.lock.."
        check_api_lock "$workspace_dir"

        echo "Running cargo fmt.."
        cargo fmt --all --check

        echo "Running cargo clippy.."
        cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

        echo "Running blueos rust style check.."
        cargo run --locked -q -p blueos-rust-style-check -- "$workspace_dir"

        local metadata
        metadata=$(cargo metadata --format-version 1 --no-deps --locked)

        echo "Checking crate folders.."
        if ! collect_folder_violations "$metadata"; then
            exit 1
        fi

        local package_args=() idl_package_args=()
        local name directory unit folder
        while IFS=$'\t' read -r name directory; do
            read -r unit folder <<<"$(crate_place "$directory")"
            case "$folder" in
                logic) package_args+=(-p "$name") ;;
                idl) [[ $directory == */codegen ]] || idl_package_args+=(-p "$name") ;;
            esac
        done < <(jq -r '.packages[] | [.name, (.manifest_path | rtrimstr("/Cargo.toml"))] | @tsv' <<<"$metadata")

        echo "Building logic crates for ${RUST_NO_STD_TARGET} and ${RUST_WASM32_TARGET}.."
        if [ ${#package_args[@]} -gt 0 ]; then
            for no_std_target in "$RUST_NO_STD_TARGET" "$RUST_WASM32_TARGET"; do
                cargo check --locked --target "$no_std_target" "${package_args[@]}"
            done
        fi
        if [ ${#idl_package_args[@]} -gt 0 ]; then
            cargo check --locked --target "$RUST_NO_STD_TARGET" --no-default-features "${idl_package_args[@]}"
        fi
    )
}

# Usage: run_rust_supply_chain_checks <workspace_dir>
run_rust_supply_chain_checks() {
    local workspace_dir="$1"
    local repository_dir
    repository_dir=$(git -C "$workspace_dir" rev-parse --show-toplevel)

    echo "Running Rust supply-chain checks for ${workspace_dir}"
    (
        cd "$workspace_dir" || exit 1

        echo "Running cargo deny.."
        cargo deny check bans licenses sources

        echo "Running cargo machete.."
        cargo machete
    )

    echo "Running typos on the Rust workspace.."
    local typos_paths=() path
    for path in .hooks/lib core/libs core/app core/Cargo.toml core/deny.toml core/clippy.toml core/coverage-ratchet.toml; do
        if [ -e "$repository_dir/$path" ]; then
            typos_paths+=("$repository_dir/$path")
        fi
    done
    if ! command -v typos >/dev/null 2>&1; then
        printf 'typos not installed; install typos-cli (e.g. cargo install --locked typos-cli)\n' >&2
        exit 1
    fi
    typos --config "$repository_dir/typos.toml" "${typos_paths[@]}"
}

# Usage: run_rust_test_checks <workspace_dir>
run_rust_test_checks() {
    local workspace_dir="$1"

    echo "Running Rust test checks for ${workspace_dir}"
    (
        cd "$workspace_dir" || exit 1

        echo "Running cargo nextest.."
        cargo nextest run --workspace --locked

        echo "Running cargo test --doc.."
        cargo test --doc --workspace --locked
    )
}

# Usage: check_rust_coverage_ratchet <workspace_dir> [ratchet_file]
# Runs instrumented tests and fails when measured line coverage is below the committed floors.
check_rust_coverage_ratchet() {
    local workspace_dir="$1"
    local ratchet_file="${2:-$workspace_dir/coverage-ratchet.toml}"
    local report_json measured_json

    if [ ! -f "$ratchet_file" ]; then
        printf 'Missing coverage ratchet file: %s\n' "$ratchet_file" >&2
        exit 1
    fi

    echo "Running Rust coverage ratchet for ${workspace_dir}"
    (
        cd "$workspace_dir" || exit 1

        echo "Resetting prior coverage artifacts.."
        cargo llvm-cov clean --workspace

        echo "Running cargo llvm-cov nextest.."
        cargo llvm-cov nextest --workspace --locked

        report_json=$(mktemp)
        measured_json=$(mktemp)
        trap 'rm -f "$report_json" "$measured_json"' EXIT

        cargo llvm-cov report --json --summary-only >"$report_json"

        jq --arg workspace_dir "$workspace_dir" '
            def line_percent($count; $covered):
                if $count == 0 then 100 else ($covered * 100 / $count) end;
            .data[0] as $root |
            ($root.files // []) as $files |
            ($root.totals.lines.count // 0) as $workspace_count |
            ($root.totals.lines.covered // 0) as $workspace_covered |
            {
                workspace: line_percent($workspace_count; $workspace_covered),
                libs_logic: (
                    [$files[] | select(.filename | contains("/libs/logic/")) | .summary.lines]
                    | if length == 0 then 100
                      else line_percent((map(.count) | add); (map(.covered) | add))
                      end
                ),
                services_logic: (
                    [$files[] | select(.filename | test("/services/[^/]+/logic/")) | .summary.lines]
                    | if length == 0 then 100
                      else line_percent((map(.count) | add); (map(.covered) | add))
                      end
                ),
                services_app: (
                    [$files[] | select(.filename | test("/services/[^/]+/app/")) | .summary.lines]
                    | if length == 0 then 100
                      else line_percent((map(.count) | add); (map(.covered) | add))
                      end
                )
            }
        ' "$report_json" >"$measured_json"

        local layer floor measured
        while IFS='=' read -r layer floor; do
            layer=${layer// /}
            floor=${floor// /}
            [ -n "$layer" ] || continue
            measured=$(jq -r --arg layer "$layer" '.[$layer] // empty' "$measured_json")
            if [ -z "$measured" ]; then
                printf 'Unknown coverage layer %s in %s\n' "$layer" "$ratchet_file" >&2
                exit 1
            fi
            if awk -v measured="$measured" -v floor="$floor" 'BEGIN { exit !(measured + 0.0001 < floor + 0) }'; then
                printf 'Coverage for %s is %.2f%%, below ratchet floor %s%%\n' "$layer" "$measured" "$floor" >&2
                exit 1
            fi
            printf 'Coverage %s: %.2f%% (floor %s%%)\n' "$layer" "$measured" "$floor"
        done < <(awk -F= '
            /^\[lines\]/ { section = 1; next }
            /^\[/ { section = 0 }
            section && $1 !~ /^#/ && NF >= 2 {
                key = $1
                gsub(/^[ \t]+|[ \t]+$/, "", key)
                value = $2
                gsub(/^[ \t]+|[ \t]+$/, "", value)
                if (key != "") print key "=" value
            }
        ' "$ratchet_file")
    )
}

# Usage: run_rust_checks <workspace_dir>
run_rust_checks() {
    local workspace_dir="$1"

    run_rust_lint_checks "$workspace_dir"
    if [ "$fixing" = true ]; then
        return 0
    fi
    run_rust_supply_chain_checks "$workspace_dir"
    if [ "${SKIP_CARGO_TEST:-}" != "1" ]; then
        run_rust_test_checks "$workspace_dir"
    fi
}
