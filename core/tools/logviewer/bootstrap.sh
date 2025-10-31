#!/usr/bin/env bash
set -e

VERSION="v1.0.1"
REPOSITORY_ORG="Ardupilot"
REPOSITORY_NAME="UAVLogViewer"
PROJECT_NAME="logviewer"

echo "Installing project $PROJECT_NAME version $VERSION"

# Step 1: Determine architecture

ARCH="$(uname -m)"
BUILD_NAME=""
echo "For architecture $ARCH, using build $BUILD_NAME"

# Step 2: Prepare the installation path

INSTALL_FOLDER="/var/www/html/$PROJECT_NAME"
mkdir -p "$INSTALL_FOLDER"

# Step 3: Download and verify sources

ARTIFACT_NAME="$PROJECT_NAME.tar.gz"
SCRIPTDIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source="SCRIPTDIR/../bootstrap_from_github.sh"
source "$SCRIPTDIR/../bootstrap_from_github.sh"
mapfile -t BOOTSTRAP_OUTPUT < <(
    bootstrap_from_github \
    "$REPOSITORY_ORG" \
    "$REPOSITORY_NAME" \
    "$VERSION" \
    "$PROJECT_NAME" \
    "$BUILD_NAME" \
    "$ARTIFACT_NAME" \
    ""
)
DOWNLOADED_ASSET_PATH="${BOOTSTRAP_OUTPUT[0]}"
TMP_DIR="${BOOTSTRAP_OUTPUT[1]}"

# Step 4: Install

echo "Installing to $INSTALL_FOLDER"
tar -zxf "$DOWNLOADED_ASSET_PATH" -C "$INSTALL_FOLDER"
find "$INSTALL_FOLDER/dist" -name "*.gz" -type f -delete
mv "$INSTALL_FOLDER"/dist/* "$INSTALL_FOLDER"
rm -rf "$INSTALL_FOLDER/dist"

# Step 5: Cleanup temp files

rm -rf "$TMP_DIR"

echo "Finished installing $PROJECT_NAME"
