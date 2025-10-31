#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   bootstrap_from_github \
#     REPOSITORY_ORG \
#     REPOSITORY_NAME \
#     VERSION \
#     PROJECT_NAME \
#     BUILD_NAME \
#     [ARTIFACT_NAME] \
#     [SKIP_VERIFICATION]
#
# Outputs (to stdout):
#   DOWNLOADED_ASSET_PATH
#   TMP_DIR
#
# All logs go to stderr.

bootstrap_from_github() {
    if [ "$#" -lt 5 ]; then
        echo "ERROR: bootstrap_from_github requires at least 5 arguments." >&2
        return 1
    fi

    local REPOSITORY_ORG=$1
    local REPOSITORY_NAME=$2
    local VERSION=$3
    local PROJECT_NAME=$4
    local BUILD_NAME=$5
    local ARTIFACT_NAME=${6:-${PROJECT_NAME}-${BUILD_NAME}}
    local SKIP_VERIFICATION=${7:-}

    echo "For architecture $(uname -m), using build $BUILD_NAME" >&2
    echo "Asset name: $ARTIFACT_NAME" >&2

    local GITHUB_API_URL="https://api.github.com/repos"
    local API_URL="${GITHUB_API_URL}/${REPOSITORY_ORG}/${REPOSITORY_NAME}/releases/tags/${VERSION}"

    echo "Querying GitHub API for release assets..." >&2
    local ASSET_INFO
    ASSET_INFO=$(curl -s "$API_URL" | jq -r ".assets[] | select(.name == \"${ARTIFACT_NAME}\")")
    if [ -z "$ASSET_INFO" ] || [ "$ASSET_INFO" = "null" ]; then
        echo "Error: Asset '${ARTIFACT_NAME}' not found in release '${VERSION}'." >&2
        return 1
    fi

    local REMOTE_URL
    REMOTE_URL=$(echo "$ASSET_INFO" | jq -r '.browser_download_url')
    if [ -z "$REMOTE_URL" ] || [ "$REMOTE_URL" = "null" ]; then
        echo "Error: Could not extract download URL from GitHub API." >&2
        return 1
    fi
    echo "Remote URL is: $REMOTE_URL" >&2

    local EXPECTED_SHA256=""
    if [ -z "${SKIP_VERIFICATION:+x}" ]; then
        EXPECTED_SHA256=$(echo "$ASSET_INFO" | jq -r '.digest | sub("^sha256:"; "")')
        echo "Expected SHA256: $EXPECTED_SHA256" >&2
    fi

    local TMP_DIR
    TMP_DIR=$(mktemp -d "/tmp/${PROJECT_NAME}.XXXXXX")
    local DOWNLOADED_ASSET_PATH="$TMP_DIR/$ARTIFACT_NAME"

    echo "Downloading to temporary location: $DOWNLOADED_ASSET_PATH" >&2
    wget -q "$REMOTE_URL" -O "$DOWNLOADED_ASSET_PATH"

    if [ -z "${SKIP_VERIFICATION:+x}" ]; then
        local ACTUAL_SHA256
        ACTUAL_SHA256=$(sha256sum "$DOWNLOADED_ASSET_PATH" | cut -d' ' -f1)
        if [ "$EXPECTED_SHA256" != "$ACTUAL_SHA256" ]; then
            echo "ERROR: Checksum mismatch!" >&2
            echo "Expected: $EXPECTED_SHA256" >&2
            echo "Actual:   $ACTUAL_SHA256" >&2
            rm -rf "$TMP_DIR"
            return 1
        else
            echo "Checksum verified successfully." >&2
        fi
    else
        echo "WARNING: Checksum skipped." >&2
    fi

    # Output ONLY the two paths (for capture)
    echo "$DOWNLOADED_ASSET_PATH"
    echo "$TMP_DIR"
}