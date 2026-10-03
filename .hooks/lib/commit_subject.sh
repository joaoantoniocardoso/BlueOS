#!/usr/bin/env bash

# Commit subjects are `path: to: module: Capitalized description`.
# The prefix is one path: `: ` separates segments, every segment is lowercase,
# and the prefix must cover every file the commit changes. The last segment may
# be a directory, the file's stem, or the full basename (`src: mcap:` covers
# `src/mcap.rs`, and `rust_checks.sh` covers `.hooks/lib/rust_checks.sh`).
#
# Conventions read off the existing history, beyond a byte-for-byte path join:
# - Segments match path components case-insensitively. Root files keep their
#   uppercase names, so `glossary:` covers `GLOSSARY.md`, `agents:` covers
#   `AGENTS.md`, and `readme:` covers `README.md`.
# - `cargo:` covers `Cargo.toml` and `Cargo.lock` in any directory. A path
#   prefix still covers a manifest that lives under that directory; `cargo:`
#   is the alias for a commit that only changes those files.
# - `tests:` covers a test file anywhere (a `tests/` or `test/` directory, a
#   `test_*` / `*_test` / `*.test.*` / `*.spec.*` name, or a `tests.rs` module).
#   Further segments are the path under that directory: `tests: effects:`
#   covers `core/libs/app/service/tests/effects.rs`.
# - `build:` covers a Rust `build.rs` or any file under a `build/` directory.
# - `.github: workflows:` is the ordinary path of `.github/workflows/`.
# - Renames are passed as the deleted path and the added path (`git log
#   --no-renames`), and a deletion is passed as the deleted path. The prefix
#   has to cover each of those paths; there is no separate rename exception.
# - `docs:` is a path. It is rejected when the files are not under `docs/`.
#   `feat`, `fix`, `perf`, `chore`, `deps`, `runtime`, `refactor`, `style`,
#   `ci`, `test`, and `revert` are conventional-commit types, not paths.
#   `build` is the build-script alias, so it is not in that list.

conventional_commit_prefix() {
    case "$1" in
        feat|fix|perf|chore|deps|runtime|refactor|style|ci|test|revert) return 0 ;;
        *) return 1 ;;
    esac
}

file_stem() {
    local base=$1
    # `.gitignore` has no extension. The leading dot is the whole name.
    if [[ $base == .* && $base != *.*.* ]]; then
        printf '%s\n' "$base"
        return 0
    fi
    if [[ $base == *.* ]]; then
        printf '%s\n' "${base%.*}"
        return 0
    fi
    printf '%s\n' "$base"
}

file_is_cargo_manifest() {
    local base
    base=$(basename "$1")
    base=${base,,}
    if [ "$base" = cargo.toml ] || [ "$base" = cargo.lock ]; then
        return 0
    fi
    return 1
}

file_is_build_script() {
    local file=$1
    local base
    base=$(basename "$file")
    if [ "$base" = build.rs ]; then
        return 0
    fi
    case "$file" in
        build/*|*/build/*) return 0 ;;
        *) return 1 ;;
    esac
}

file_is_test_file() {
    local file=$1
    local base stem component
    local -a components=()
    base=$(basename "$file")
    case "$base" in
        test_*|*_test|*_test.*|*.test.*|*.spec.*) return 0 ;;
    esac
    stem=$(file_stem "$base")
    case "$stem" in
        test|tests) return 0 ;;
    esac
    IFS=/ read -ra components <<< "$file"
    for component in "${components[@]}"; do
        case "$component" in
            test|tests) return 0 ;;
        esac
    done
    return 1
}

# 0 when every segment equals the leading path components and the file lives
# strictly inside that directory, or when the final segment is the file's stem
# or basename and the earlier segments are its parent directory.
relative_path_covered() {
    local relative=$1
    shift
    local -a segments=("$@")
    local -a components=()
    local count file_count index matched last base stem

    if [ -z "$relative" ] || [ "${#segments[@]}" -eq 0 ]; then
        return 1
    fi
    IFS=/ read -ra components <<< "$relative"
    count=${#segments[@]}
    file_count=${#components[@]}

    if [ "$file_count" -gt "$count" ]; then
        matched=1
        for index in "${!segments[@]}"; do
            if [ "${components[$index]}" != "${segments[$index]}" ]; then
                matched=0
                break
            fi
        done
        if [ "$matched" -eq 1 ]; then
            return 0
        fi
    fi

    if [ "$file_count" -ne "$count" ]; then
        return 1
    fi
    last=$((count - 1))
    for ((index = 0; index < last; index++)); do
        if [ "${components[$index]}" != "${segments[$index]}" ]; then
            return 1
        fi
    done
    base=${components[$last]}
    stem=$(file_stem "$base")
    if [ "$base" = "${segments[$last]}" ] || [ "$stem" = "${segments[$last]}" ]; then
        return 0
    fi
    return 1
}

tests_alias_covers_file() {
    local file=$1
    shift
    local -a segments=("$@")
    local -a components=()
    local index component_index relative

    if [ "${#segments[@]}" -eq 1 ]; then
        file_is_test_file "$file"
        return
    fi

    IFS=/ read -ra components <<< "$file"
    for index in "${!components[@]}"; do
        if [ "${components[$index]}" != tests ] && [ "${components[$index]}" != test ]; then
            continue
        fi
        relative=""
        for ((component_index = index + 1; component_index < ${#components[@]}; component_index++)); do
            if [ -n "$relative" ]; then
                relative=$relative/${components[$component_index]}
            else
                relative=${components[$component_index]}
            fi
        done
        if relative_path_covered "$relative" "${segments[@]:1}"; then
            return 0
        fi
    done
    return 1
}

prefix_covers_file() {
    local file=$1
    shift
    local -a segments=("$@")
    local lower=${file,,}

    if [ "${#segments[@]}" -eq 1 ] && [ "${segments[0]}" = cargo ] && file_is_cargo_manifest "$file"; then
        return 0
    fi
    if [ "${#segments[@]}" -eq 1 ] && [ "${segments[0]}" = build ] && file_is_build_script "$lower"; then
        return 0
    fi
    if [ "${segments[0]}" = tests ] && tests_alias_covers_file "$lower" "${segments[@]}"; then
        return 0
    fi
    relative_path_covered "$lower" "${segments[@]}"
}

# Fills the caller's segment array and description from one subject line.
parse_commit_subject() {
    local -n parsed_segments=$1
    local -n parsed_description=$2
    local subject=$3
    local rest=$subject

    parsed_segments=()
    while [[ $rest =~ ^(\.[a-z0-9._-]+|[a-z0-9][a-z0-9._-]*):\ (.+)$ ]]; do
        parsed_segments+=("${BASH_REMATCH[1]}")
        rest=${BASH_REMATCH[2]}
    done
    # nameref: this writes the caller's description variable.
    # shellcheck disable=SC2034
    parsed_description=$rest
}

# Prints one reason per line and returns 1 when the subject is not allowed for
# these paths. `changed files` may be empty, in which case only the subject
# shape is checked.
check_commit_subject() {
    local subject=$1
    shift
    local -a changed_files=("$@")
    local -a segments=()
    local description=""
    local changed_file rejected=0

    parse_commit_subject segments description "$subject"

    if [ "${#segments[@]}" -eq 0 ]; then
        if [[ $subject =~ ^[A-Z][A-Za-z0-9._-]*:\  ]]; then
            printf '%s\n' "path segments must be lowercase"
        else
            printf '%s\n' "missing path prefix"
        fi
        return 1
    fi

    if conventional_commit_prefix "${segments[0]}"; then
        printf 'conventional-commit prefix %s\n' "${segments[0]}"
        return 1
    fi

    # `core: Services: Add the handler` parses `Services` as the start of the
    # description. A capitalised token, a colon, and a capitalised word is an
    # uppercase path segment, not a description.
    if [[ $description =~ ^[A-Z][A-Za-z0-9._-]*:\ [A-Z] ]]; then
        printf '%s\n' "path segments must be lowercase"
        return 1
    fi

    if [[ ! $description =~ ^[A-Z] ]]; then
        printf '%s\n' "description must start with a capital letter"
        return 1
    fi

    if [ "${#changed_files[@]}" -eq 0 ]; then
        return 0
    fi
    for changed_file in "${changed_files[@]}"; do
        if ! prefix_covers_file "$changed_file" "${segments[@]}"; then
            printf 'prefix does not cover %s\n' "$changed_file"
            rejected=1
        fi
    done
    return "$rejected"
}

# `log` is `git log --no-merges --no-renames --name-only --format='---%n%H%n%s'`.
# One record is a `---` line, the hash, the subject, a blank line, then paths.
check_commit_subjects_from_log() {
    local log=$1
    local padded_log commit_hash="" commit_subject="" phase="seek" line failed=0
    local -a changed_files=()

    padded_log=${log}$'\n---'
    while IFS= read -r line || [ -n "$line" ]; do
        if [ "$line" = "---" ]; then
            if [ -n "$commit_hash" ]; then
                if ! judge_logged_commit "$commit_hash" "$commit_subject" \
                    "${changed_files[@]+"${changed_files[@]}"}"; then
                    failed=1
                fi
            fi
            commit_hash=""
            commit_subject=""
            changed_files=()
            phase="hash"
            continue
        fi
        case "$phase" in
            hash)
                commit_hash=$line
                phase="subject"
                ;;
            subject)
                commit_subject=$line
                phase="files"
                ;;
            files)
                if [ -n "$line" ]; then
                    changed_files+=("$line")
                fi
                ;;
            *) ;;
        esac
    done <<< "$padded_log"
    return "$failed"
}

judge_logged_commit() {
    local commit_hash=$1
    local commit_subject=$2
    shift 2
    local output
    if ! output=$(check_commit_subject "$commit_subject" "$@"); then
        printf '%s %s\n%s\n' "$commit_hash" "$commit_subject" "$output"
        return 1
    fi
}

# One `git log` for the whole range. Merge commits are omitted. `--no-renames`
# lists both sides of a rename, so each side has to be covered.
check_commit_subjects() {
    local log
    if [ -n "${BASE_SHA:-}" ] && [ -n "${HEAD_SHA:-}" ]; then
        # A push that creates a branch has an all-zero base, and a force push's base may not be fetched.
        if ! git cat-file -e "${BASE_SHA}^{commit}" 2>/dev/null; then
            printf 'Skipping commit subjects: base %s is not in this clone\n' "$BASE_SHA"
            return 0
        fi
        printf 'Checking commit subjects in %s..%s\n' "$BASE_SHA" "$HEAD_SHA"
        log=$(git log --no-merges --no-renames --name-only --format='---%n%H%n%s' "${BASE_SHA}..${HEAD_SHA}")
    else
        printf 'Checking commit subjects that are on no remote\n'
        log=$(git log --no-merges --no-renames --name-only --format='---%n%H%n%s' HEAD --not --remotes)
    fi
    check_commit_subjects_from_log "$log"
}

commit_is_amend() {
    local command_line process_id=$PPID depth
    for ((depth = 0; depth < 4; depth++)); do
        if [ ! -r "/proc/${process_id}/cmdline" ]; then
            break
        fi
        command_line=$(tr '\0' ' ' < "/proc/${process_id}/cmdline")
        case " $command_line " in
            *" --amend "*) return 0 ;;
        esac
        process_id=$(awk '/^PPid:/ { print $2 }' "/proc/${process_id}/status")
        if [ -z "$process_id" ] || [ "$process_id" = 0 ]; then
            break
        fi
    done
    return 1
}

# Paths about to be committed. An amend's parent is HEAD^; the index is the
# new tree. `--no-renames` keeps both sides of a rename.
commit_index_changed_paths() {
    local parent_revision
    if commit_is_amend; then
        if git rev-parse --verify --quiet 'HEAD^' >/dev/null; then
            parent_revision='HEAD^'
        else
            parent_revision=$(git hash-object -t tree /dev/null)
        fi
    elif git rev-parse --verify --quiet HEAD >/dev/null; then
        parent_revision=HEAD
    else
        parent_revision=$(git hash-object -t tree /dev/null)
    fi
    git diff --cached --name-only --no-renames "$parent_revision"
}
