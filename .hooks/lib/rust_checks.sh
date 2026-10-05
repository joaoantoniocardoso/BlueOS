#!/usr/bin/env bash

: "${fixing:=false}"

RUST_NO_STD_TARGET=thumbv7em-none-eabihf
RUST_WASM32_TARGET=wasm32-unknown-unknown
# The quality gate tools and the thailint linters it runs (D-30).
RUST_RUSTQUAL_VERSION=1.8.3
RUST_THAILINT_VERSION=0.25.0
RUST_THAILINT_LINTERS=(unwrap-abuse clone-abuse blocking-async)
# Features of test-only backends: enabled in [dev-dependencies] only, so they never reach a shipped binary (D-02).
RUST_TEST_ONLY_FEATURES=(blueos-comms/channel blueos-service/testing)

# Prints "<unit> <folder>" for a crate directory: "libs logic" for libs/logic/jobs, "calibration app" for
# services/calibration/app, "calibration api" for services/calibration/logic/api, "multicall app" for core/app/blueos.
crate_place() {
    if [[ $1 =~ /services/([^/]+)/logic/api$ ]]; then
        echo "${BASH_REMATCH[1]} api"
    elif [[ $1 =~ /services/([^/]+)/([^/]+) ]]; then
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
    local idl_root="$workspace_dir/libs/idl"
    local idl_generated="$idl_root/src/generated"
    local idl_test_generated="$idl_root/tests/generated"
    local idl_typescript="$idl_root/typescript"
    local temporary
    temporary=$(mktemp -d)
    (
        cd "$workspace_dir" || exit 1
        mkdir -p "$temporary/tests/generated"
        cargo run --quiet -p blueos-idl-codegen --bin blueos-idl-codegen -- \
            --idl-root "$idl_root" \
            --core-dir "$workspace_dir" \
            --output "$temporary/generated" \
            --typescript-output "$temporary/typescript" \
            --test-generated-output "$temporary/tests/generated"
        cargo run --quiet -p blueos-idl-codegen --bin blueos-idl-codegen-catalog -- \
            --idl-root "$idl_root" \
            --generated-output "$temporary/generated" \
            --typescript-output "$temporary/typescript"
    )
    if ! diff -ru "$idl_generated" "$temporary/generated" >/dev/null; then
        rm -rf "$temporary"
        printf 'Committed IDL Rust is stale; fix with: (cd core && cargo run -p blueos-idl-codegen --bin blueos-idl-codegen -- --write)\n' >&2
        exit 1
    fi
    if ! diff -q "$idl_test_generated/cdr_codec_dispatch.rs" "$temporary/tests/generated/cdr_codec_dispatch.rs" >/dev/null; then
        rm -rf "$temporary"
        printf 'Committed IDL test dispatch is stale; fix with: (cd core && cargo run -p blueos-idl-codegen --bin blueos-idl-codegen -- --write)\n' >&2
        exit 1
    fi
    if ! diff -ru "$idl_typescript" "$temporary/typescript" >/dev/null; then
        rm -rf "$temporary"
        printf 'Committed IDL TypeScript is stale; fix with: (cd core && cargo run -p blueos-idl-codegen --bin blueos-idl-codegen -- --write)\n' >&2
        exit 1
    fi
    rm -rf "$temporary"
}

# Regenerates every Service's endpoint code from its endpoint manifest and compares it with what is committed.
check_generated_endpoints() {
    local workspace_dir="$1"
    if ! (cd "$workspace_dir" && cargo run --quiet -p blueos-idl-codegen --bin blueos-idl-codegen -- \
        --check-endpoints --idl-root "$workspace_dir/libs/idl" --core-dir "$workspace_dir"); then
        printf 'Committed endpoint code is stale; fix with: (cd core && cargo run -p blueos-idl-codegen --bin blueos-idl-codegen -- --write)\n' >&2
        exit 1
    fi
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
            adapters | app | api | cookbook | tools) ;;
            *) violations+=("$name is not in logic/, adapters/, app/, idl/, cookbook/, or api/ under libs/ or services/<name>/") ;;
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
        elif [ "$folder" = api ] && [ "$unit" != libs ] \
            && [ "$dependency_unit/$dependency_folder" != libs/logic ] \
            && [ "$dependency_unit/$dependency_folder" != libs/idl ] \
            && { [ "$dependency_folder" != logic ] || [ "$dependency_unit" != "$unit" ]; }; then
            violations+=("$name is logic/api, so it may only depend on libs/logic, blueos-idl or logic in the same service, not on $dependency")
        elif [ "$folder" = adapters ] && [ "$dependency_folder" != adapters ] \
            && [ "$dependency_unit/$dependency_folder" != libs/idl ] \
            && { [ "$dependency_folder" != logic ] || [ "$dependency_unit" = libs ]; }; then
            violations+=("$name is an adapter, so it may only depend on adapters, blueos-idl or logic in the same service, not on $dependency")
        fi
        parent=$(dirname "$dependency_directory")
        if [ -f "$parent/Cargo.toml" ] && [ "$directory" != "$parent" ] && [ "$(dirname "$directory")" != "$parent" ]; then
            violations+=("$name depends on $dependency, which is private to the crate in $parent")
        fi
    done < <(jq -r '.packages[] | .name as $name | (.manifest_path | rtrimstr("/Cargo.toml")) as $directory
        | .dependencies[] | select(.kind != "dev" and .path != null) | [$name, $directory, .name, .path] | @tsv' \
        <<<"$metadata")
    while IFS=$'\t' read -r name directory; do
        read -r unit folder <<<"$(crate_place "$directory")"
        if [ "$folder" = logic ] || [ "$folder" = api ]; then
            violations+=("$name is logic, so it may not depend on metrics: logic exposes a count through its Snapshot and a Projection, and only a Task or an adapter records it (D-35)")
        fi
    done < <(jq -r '.packages[] | select(any(.dependencies[]; .name == "metrics" and .kind != "dev"))
        | [.name, (.manifest_path | rtrimstr("/Cargo.toml"))] | @tsv' <<<"$metadata")
    if [ ${#violations[@]} -gt 0 ]; then
        printf '%s\n' "${violations[@]}"
        return 1
    fi
    return 0
}

# Usage: service_app_block_folders <service_dir>
# Prints one Block folder name per line that may appear under app/src/ (D-23).
service_app_block_folders() {
    local service_dir="$1"
    local logic_dir="$service_dir/logic"
    local domain_folder=api
    if [ -d "$logic_dir/domain" ]; then
        domain_folder=domain
    elif [ -d "$logic_dir/$(basename "$service_dir")" ]; then
        domain_folder=$(basename "$service_dir")
    fi
    local entry
    for entry in "$logic_dir"/*; do
        [ -d "$entry" ] || continue
        local name
        name=$(basename "$entry")
        case "$name" in
            api | "$domain_folder" | paths | schema-gate) continue ;;
        esac
        printf '%s\n' "$name"
    done
}

# Usage: collect_app_src_violations <repository_dir>
# Enforces D-23 layout under services/<name>/app/src. Prints one violation per line.
collect_app_src_violations() {
    local root_dir="$1"
    local services_root
    if [ -d "$root_dir/core/services" ]; then
        services_root="$root_dir/core/services"
    elif [ -d "$root_dir/services" ]; then
        services_root="$root_dir/services"
    else
        return 0
    fi
    local violations=() service_dir app_src entry name stem
    local -a service_wide=(cli context endpoints handlers io service settings)
    while IFS= read -r -d '' service_dir; do
        app_src="$service_dir/app/src"
        [ -d "$app_src" ] || continue
        local -a block_folders=()
        while IFS= read -r name; do
            [ -n "$name" ] && block_folders+=("$name")
        done < <(service_app_block_folders "$service_dir")
        while IFS= read -r -d '' entry; do
            name=$(basename "$entry")
            if [ -f "$entry" ]; then
                [ "$name" = lib.rs ] && continue
                stem=${name%.rs}
                local allowed=false
                for stem in "${service_wide[@]}"; do
                    if [ "$name" = "${stem}.rs" ]; then
                        allowed=true
                        break
                    fi
                done
                if [ "$allowed" = false ]; then
                    violations+=("$app_src: unknown top-level module ${name%.rs}")
                fi
            elif [ -d "$entry" ]; then
                case "$name" in
                    tasks) ;;
                    *)
                        local block_allowed=false
                        for stem in "${block_folders[@]}"; do
                            if [ "$name" = "$stem" ]; then
                                block_allowed=true
                                break
                            fi
                        done
                        if [ "$block_allowed" = false ]; then
                            violations+=("$app_src: unknown top-level module $name")
                        fi
                        ;;
                esac
            fi
        done < <(find "$app_src" -mindepth 1 -maxdepth 1 -print0)
    done < <(find "$services_root" -mindepth 1 -maxdepth 1 -type d -print0)
    if [ ${#violations[@]} -gt 0 ]; then
        printf '%s\n' "${violations[@]}"
        return 1
    fi
    return 0
}

# Usage: collect_test_only_feature_violations <workspace_dir>
# Resolves features the way a non-test build does (no dev edges, every target) and prints one violation per
# test-only feature it finds enabled. Exits 1 when any violation exists.
collect_test_only_feature_violations() {
    local workspace_dir="$1"
    local violations=() feature package tree
    for feature in "${RUST_TEST_ONLY_FEATURES[@]}"; do
        package=${feature%/*}
        tree=$(cargo tree --manifest-path "$workspace_dir/Cargo.toml" --workspace --target all \
            --edges normal,build,features --invert "$package" --prefix none) || return 1
        if grep -qF "$package feature \"${feature#*/}\"" <<<"$tree"; then
            violations+=("$feature is enabled outside [dev-dependencies], so it would reach a shipped binary")
        fi
    done
    if [ ${#violations[@]} -gt 0 ]; then
        printf '%s\n' "${violations[@]}"
        return 1
    fi
    return 0
}

# Usage: collect_dependency_violations <cargo-metadata-json> <workspace-manifest> <exceptions-file>
# Prints one violation per line. Exits 1 when any violation exists (D-30). cargo metadata names the members; their
# manifests are read as written because the metadata hides whether an entry inherited from the workspace.
collect_dependency_violations() {
    local metadata="$1"
    local workspace_manifest="$2"
    local exceptions_file="$3"
    local violations=() name manifest_path
    while IFS=$'\t' read -r name manifest_path; do
        while IFS= read -r violation; do
            violations+=("$violation")
        done < <(toml_to_json "$manifest_path" | jq -r --arg name "$name" '
            def sections: ["dependencies", "dev-dependencies", "build-dependencies"][] as $section
                | (.[$section] // {} | to_entries[] | {section: $section, entry: .}),
                  (.target // {} | .[] | .[$section] // {} | to_entries[] | {section: $section, entry: .});
            sections
            | select(.entry.value | type != "object" or .workspace != true)
            | "\($name): [\(.section)] \(.entry.key) is not workspace = true"')
    done < <(jq -r '.packages[] | [.name, .manifest_path] | @tsv' <<<"$metadata")

    local workspace exceptions
    workspace=$(toml_to_json "$workspace_manifest")
    exceptions=$(toml_to_json "$exceptions_file")
    while IFS= read -r violation; do
        violations+=("$violation")
    done < <(jq -r --argjson exceptions "$exceptions" --arg file "$(basename "$exceptions_file")" '
        (.workspace.dependencies // {}) as $dependencies
        | ($exceptions.exceptions // {}) as $listed
        | ($dependencies | to_entries[]
            | select(.value | type != "object" or .["default-features"] != false)
            | select(.key as $key | $listed | has($key) | not)
            | "workspace: \(.key) keeps default features; set default-features = false or list it in \($file) with a reason"),
          ($listed | to_entries[] | .key as $key | .value as $reason
            | if ($reason | type != "string" or (gsub("\\s"; "") == "")) then "exceptions: \($key) has no reason in \($file)"
              elif ($dependencies | has($key) | not) then "exceptions: \($key) is listed but is not a workspace dependency"
              elif ($dependencies[$key] | type == "object" and .["default-features"] == false) then
                "exceptions: \($key) is listed but already sets default-features = false"
              else empty end)' <<<"$workspace")
    if [ ${#violations[@]} -gt 0 ]; then
        printf '%s\n' "${violations[@]}"
        return 1
    fi
    return 0
}

# Usage: check_dependency_gate <workspace_dir>
check_dependency_gate() {
    local workspace_dir="$1"
    local metadata
    metadata=$(cargo metadata --manifest-path "$workspace_dir/Cargo.toml" --format-version 1 --no-deps --locked) \
        || return 1
    collect_dependency_violations "$metadata" "$workspace_dir/Cargo.toml" "$workspace_dir/dependency-exceptions.toml"
}

# Usage: run_shipped_clippy
# Run from the Cargo workspace. Lints the feature set build_cross.sh ships (D-16).
# --all-features hides an item that only a feature the shipped binary leaves off refers to.
run_shipped_clippy() {
    # shellcheck disable=SC1091
    source shipped_features.sh
    cargo clippy --locked -p "$BLUEOS_SHIPPED_PACKAGE" \
        --features "${BLUEOS_SHIPPED_FEATURES[*]}" -- -D warnings
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
            echo "Regenerating committed IDL and endpoint Rust.."
            cargo run --quiet -p blueos-idl-codegen --bin blueos-idl-codegen -- \
                --write --idl-root "$workspace_dir/libs/idl" --core-dir "$workspace_dir"
            echo "Running cargo fmt.."
            cargo fmt --all
            exit 0
        fi

        echo "Checking committed IDL Rust.."
        check_generated_idl "$workspace_dir"

        echo "Checking committed endpoint code.."
        check_generated_endpoints "$workspace_dir"

        echo "Checking api.lock.."
        check_api_lock "$workspace_dir"

        echo "Running cargo fmt.."
        cargo fmt --all --check

        echo "Running cargo clippy.."
        cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

        echo "Running cargo clippy for the shipped configuration.."
        run_shipped_clippy

        echo "Running blueos rust style check.."
        cargo run --locked -q -p blueos-rust-style-check -- "$workspace_dir"

        echo "Running rustqual and thailint.."
        check_rust_quality "$workspace_dir"

        local metadata
        metadata=$(cargo metadata --format-version 1 --no-deps --locked)

        echo "Checking crate folders.."
        if ! collect_folder_violations "$metadata"; then
            exit 1
        fi

        echo "Checking application crate layout.."
        if ! collect_app_src_violations "$(git -C "$workspace_dir" rev-parse --show-toplevel)"; then
            exit 1
        fi

        echo "Checking dependencies.."
        if ! check_dependency_gate "$workspace_dir"; then
            exit 1
        fi

        echo "Checking test-only features.."
        if ! collect_test_only_feature_violations "$workspace_dir"; then
            exit 1
        fi

        local package_args=() idl_package_args=()
        local name directory unit folder
        while IFS=$'\t' read -r name directory; do
            read -r unit folder <<<"$(crate_place "$directory")"
            case "$folder" in
                logic) package_args+=(-p "$name") ;;
                api) [ "$unit" = libs ] || package_args+=(-p "$name") ;;
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
    check_typos "$repository_dir"
}

# Usage: check_typos <repository_dir>
check_typos() {
    local repository_dir="$1"
    local typos_paths=() path
    for path in .hooks/lib core/libs core/app core/services core/Cargo.toml core/deny.toml core/clippy.toml \
        core/coverage-ratchet.toml; do
        if [ -e "$repository_dir/$path" ]; then
            typos_paths+=("$path")
        fi
    done
    if ! command -v typos >/dev/null 2>&1; then
        printf 'typos not installed; install typos-cli (e.g. cargo install --locked typos-cli)\n' >&2
        exit 1
    fi
    # typos matches the typos.toml excludes against the paths as given, so they must be relative to the repository.
    (cd "$repository_dir" && typos --config typos.toml "${typos_paths[@]}")
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

# Usage: export_rust_llvm_cov_tools
# Points cargo-llvm-cov at the llvm-profdata/llvm-cov that match the active rustc (not a distro symlink).
export_rust_llvm_cov_tools() {
    local sysroot host bindir
    sysroot=$(rustc --print sysroot)
    host=$(rustc -vV | sed -n 's/^host: //p')
    bindir="$sysroot/lib/rustlib/$host/bin"
    export LLVM_PROFDATA="$bindir/llvm-profdata"
    export LLVM_COV="$bindir/llvm-cov"
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
        export_rust_llvm_cov_tools

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

# Usage: toml_to_json <file>
toml_to_json() {
    python3 -c 'import json, sys, tomllib; print(json.dumps(tomllib.load(open(sys.argv[1], "rb"))))' "$1"
}

# Usage: check_ratchet_prerequisites
# The ratchet files are read with python3's tomllib (3.11 or later) and compared with jq.
check_ratchet_prerequisites() {
    if ! python3 -c 'import tomllib' 2>/dev/null; then
        printf 'python3 3.11 or later not installed; the ratchet checks read TOML with its tomllib\n' >&2
        exit 1
    fi
    if ! command -v jq >/dev/null 2>&1; then
        printf 'jq not installed; install it with your package manager (e.g. apt install jq)\n' >&2
        exit 1
    fi
}

# Usage: check_rust_quality <workspace_dir>
# Fails on any rustqual or thailint finding, on a file rustqual cannot parse, and when the rustqual suppressions
# exceed max_suppression_ratio. Every tool runs, so one run lists every finding (D-30).
check_rust_quality() {
    local workspace_dir="$1"
    if [ "$(rustqual --version 2>/dev/null)" != "rustqual $RUST_RUSTQUAL_VERSION" ]; then
        printf 'rustqual %s not installed; install it with: cargo install --locked rustqual@%s\n' \
            "$RUST_RUSTQUAL_VERSION" "$RUST_RUSTQUAL_VERSION" >&2
        exit 1
    fi
    if [ "$(thailint --version 2>/dev/null)" != "thailint, version $RUST_THAILINT_VERSION" ]; then
        printf 'thailint %s not installed; install it with: pip install thailint==%s\n' \
            "$RUST_THAILINT_VERSION" "$RUST_THAILINT_VERSION" >&2
        exit 1
    fi
    (
        cd "$workspace_dir" || exit 1
        local status=0 linter output
        output=$(rustqual --fail-on-warnings --findings . 2>&1) || status=1
        printf '%s\n' "$output"
        # rustqual skips a file it cannot parse with only a warning, which would hide that file's findings.
        if grep -q 'Could not parse' <<<"$output"; then
            status=1
        fi
        for linter in "${RUST_THAILINT_LINTERS[@]}"; do
            thailint "$linter" libs app services || status=1
        done
        exit "$status"
    )
}

# Usage: collect_ratchet_loosening <base-file> <head-file>
# Prints one line per floor that fell since the base, or was removed. Exits 1 when any exists (D-30).
collect_ratchet_loosening() {
    local base head loosened
    base=$(toml_to_json "$1") || return 1
    head=$(toml_to_json "$2") || return 1
    loosened=$(jq -rn --argjson base "$base" --argjson head "$head" '
        $base | to_entries[] | .key as $table | .value | to_entries[] | .key as $key | .value as $was
        | $head[$table][$key] as $now
        | if $now == null then "\($table).\($key) was removed"
          elif $now < $was then "\($table).\($key) floor fell from \($was) to \($now)"
          else empty end')
    if [ -n "$loosened" ]; then
        printf '%s\n' "$loosened"
        return 1
    fi
    return 0
}

# Usage: check_ratchets_not_loosened <repository_dir> <base_revision>
# Fails when the coverage ratchet loosened since the base revision, and when the base revision cannot be resolved. A
# base without the file has nothing to loosen.
check_ratchets_not_loosened() {
    local repository_dir="$1"
    local base_revision="$2"
    local path=core/coverage-ratchet.toml
    local base_file status=0
    check_ratchet_prerequisites
    if ! git -C "$repository_dir" cat-file -e "$base_revision^{commit}" 2>/dev/null; then
        printf 'cannot resolve the base revision %s; fetch it before comparing the ratchet files\n' \
            "$base_revision" >&2
        return 1
    fi
    git -C "$repository_dir" cat-file -e "$base_revision:$path" 2>/dev/null || return 0
    base_file=$(mktemp)
    git -C "$repository_dir" show "$base_revision:$path" >"$base_file"
    if ! collect_ratchet_loosening "$base_file" "$repository_dir/$path"; then
        printf '%s loosened since %s; a ratchet may only tighten (D-30)\n' "$path" "$base_revision" >&2
        status=1
    fi
    rm -f "$base_file"
    return "$status"
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
