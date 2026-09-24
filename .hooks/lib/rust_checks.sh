#!/usr/bin/env bash

: "${fixing:=false}"

RUST_NO_STD_TARGET=thumbv7em-none-eabihf

# Prints "<unit> <folder>" for a crate directory: "libs logic" for libs/logic/jobs, "calibration app" for
# services/calibration/app.
crate_place() {
    if [[ $1 =~ /services/([^/]+)/([^/]+) ]]; then
        echo "${BASH_REMATCH[1]} ${BASH_REMATCH[2]}"
    elif [[ $1 =~ /libs/([^/]+) ]]; then
        echo "libs ${BASH_REMATCH[1]}"
    else
        echo "none none"
    fi
}

# Usage: run_rust_checks <workspace_dir>
run_rust_checks() {
    local workspace_dir="$1"

    if ! rustup target list --installed | grep -qx "$RUST_NO_STD_TARGET"; then
        printf 'Rust target not installed, run: rustup target add %s\n' "$RUST_NO_STD_TARGET" >&2
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
        cargo clippy --workspace --all-targets --locked -- -D warnings

        echo "Running cargo test.."
        cargo test --workspace --locked

        echo "Running cargo deny.."
        cargo deny check bans

        local metadata
        metadata=$(cargo metadata --format-version 1 --no-deps --locked)

        echo "Checking crate folders.."
        # libs/{logic,adapters,app}/ and services/<name>/{logic,adapters,app}/ decide who may depend on whom:
        # logic uses only libs/logic, adapters use only adapters, and apps use anything in libs/ or in their
        # own service. No crate reaches into another service, and a crate nested in another crate's folder
        # is private to that crate and its siblings.
        local violations=() package_args=()
        local name directory unit folder
        while IFS=$'\t' read -r name directory; do
            read -r unit folder <<<"$(crate_place "$directory")"
            case "$folder" in
                logic) package_args+=(-p "$name") ;;
                adapters | app) ;;
                *) violations+=("$name is not in logic/, adapters/ or app/ under libs/ or services/<name>/") ;;
            esac
        done < <(jq -r '.packages[] | [.name, (.manifest_path | rtrimstr("/Cargo.toml"))] | @tsv' <<<"$metadata")
        local dependency dependency_directory dependency_unit dependency_folder parent
        while IFS=$'\t' read -r name directory dependency dependency_directory; do
            read -r unit folder <<<"$(crate_place "$directory")"
            read -r dependency_unit dependency_folder <<<"$(crate_place "$dependency_directory")"
            if [ "$dependency_unit" != libs ] && [ "$dependency_unit" != "$unit" ]; then
                violations+=("$name depends on $dependency, which belongs to another service")
            elif [ "$folder" = logic ] && [ "$dependency_unit/$dependency_folder" != libs/logic ]; then
                violations+=("$name is logic, so it may only depend on libs/logic, not on $dependency")
            elif [ "$folder" = adapters ] && [ "$dependency_folder" != adapters ]; then
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
            printf '%s\n' "${violations[@]}" >&2
            exit 1
        fi

        echo "Building logic crates for ${RUST_NO_STD_TARGET}.."
        # A target without std rejects any dependency, direct or transitive, that can do I/O.
        if [ ${#package_args[@]} -gt 0 ]; then
            cargo check --locked --target "$RUST_NO_STD_TARGET" "${package_args[@]}"
        fi
    )
}
