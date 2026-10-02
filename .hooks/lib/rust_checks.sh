#!/usr/bin/env bash

: "${fixing:=false}"

RUST_NO_STD_TARGET=thumbv7em-none-eabihf
RUST_WASM32_TARGET=wasm32-unknown-unknown
# Features of test-only backends: enabled in [dev-dependencies] only, so they never reach a shipped binary (D-02).
RUST_TEST_ONLY_FEATURES=(blueos-comms/channel)

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
            adapters | app | api) ;;
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

# Usage: run_rust_checks <workspace_dir>
run_rust_checks() {
    local workspace_dir="$1"

    if ! rustup target list --installed | grep -qx "$RUST_NO_STD_TARGET"; then
        printf 'Rust target not installed, run: rustup target add %s\n' "$RUST_NO_STD_TARGET" >&2
        exit 1
    fi
    if ! rustup target list --installed | grep -qx "$RUST_WASM32_TARGET"; then
        printf 'Rust target not installed, run: rustup target add %s\n' "$RUST_WASM32_TARGET" >&2
        exit 1
    fi

    echo "Running Rust checks for ${workspace_dir}"
    (
        cd "$workspace_dir" || exit 1

        echo "Running cargo fmt.."
        if [ "$fixing" = true ]; then
            cargo fmt --all
            exit 0
        fi
        cargo fmt --all --check

        echo "Running cargo clippy.."
        cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

        if [ "${SKIP_CARGO_TEST:-}" != "1" ]; then
            echo "Running cargo test.."
            cargo test --workspace --locked
        fi

        echo "Running cargo deny.."
        cargo deny check bans

        local metadata
        metadata=$(cargo metadata --format-version 1 --no-deps --locked)

        echo "Checking crate folders.."
        if ! collect_folder_violations "$metadata"; then
            exit 1
        fi

        echo "Checking test-only features.."
        if ! collect_test_only_feature_violations "$workspace_dir"; then
            exit 1
        fi

        local package_args=()
        local name directory unit folder
        while IFS=$'\t' read -r name directory; do
            read -r unit folder <<<"$(crate_place "$directory")"
            if [ "$folder" = logic ]; then
                package_args+=(-p "$name")
            fi
        done < <(jq -r '.packages[] | [.name, (.manifest_path | rtrimstr("/Cargo.toml"))] | @tsv' <<<"$metadata")

        echo "Building logic crates for ${RUST_NO_STD_TARGET} and ${RUST_WASM32_TARGET}.."
        if [ ${#package_args[@]} -gt 0 ]; then
            for no_std_target in "$RUST_NO_STD_TARGET" "$RUST_WASM32_TARGET"; do
                cargo check --locked --target "$no_std_target" "${package_args[@]}"
            done
        fi
    )
}
