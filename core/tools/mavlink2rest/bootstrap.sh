#!/usr/bin/env bash
set -e

VERSION="t0.11.24"
REPOSITORY_ORG="mavlink"
REPOSITORY_NAME="mavlink2rest"
PROJECT_NAME="$REPOSITORY_NAME"

echo "Installing project $PROJECT_NAME version $VERSION"

# Step 1: Determine architecture

ARCH="$(uname -m)"
case "$ARCH" in
  x86_64 | amd64)
    BUILD_NAME="x86_64-unknown-linux-musl"
    ;;
  armv7l | armhf)
    BUILD_NAME="armv7-unknown-linux-musleabihf"
    ;;
  aarch64 | arm64)
    BUILD_NAME="aarch64-unknown-linux-musl"
    ;;
  *)
    echo "Architecture: $ARCH is unsupported, please create a new issue on https://github.com/bluerobotics/BlueOS/issues"
    exit 1
    ;;
esac
echo "For architecture $ARCH, using build $BUILD_NAME"

# Step 2: Prepare the installation path

if [ -n "$VIRTUAL_ENV" ]; then
    BIN_DIR="$VIRTUAL_ENV/bin"
else
    BIN_DIR="/usr/bin"
fi
mkdir -p "$BIN_DIR"

# Step 3: Download and verify sources

ARTIFACT_NAME="$PROJECT_NAME-$BUILD_NAME"
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

BINARY_PATH="$BIN_DIR/$PROJECT_NAME"
echo "Installing to $BINARY_PATH"
mv "$DOWNLOADED_ASSET_PATH" "$BINARY_PATH"
chmod +x "$BINARY_PATH"

# Step 5: Cleanup temp files

rm -rf "$TMP_DIR"

echo "Installed binary type: $(file "$BINARY_PATH")"

echo "Finished installing $PROJECT_NAME"
