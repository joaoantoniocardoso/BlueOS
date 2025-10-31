#!/usr/bin/env bash
set -e

VERSION="v2.30.0"
REPOSITORY_ORG="filebrowser"
REPOSITORY_NAME="$REPOSITORY_ORG"
PROJECT_NAME="$REPOSITORY_ORG"

echo "Installing project $PROJECT_NAME version $VERSION"

# Step 1: Determine architecture

ARCH="$(uname -m)"
case "$ARCH" in
  x86_64 | amd64)
    BUILD_NAME="linux-amd64"
    ;;
  armv7l | armhf)
    BUILD_NAME="linux-armv7"
    ;;
  aarch64 | arm64)
    BUILD_NAME="linux-arm64"
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

ARTIFACT_NAME="$BUILD_NAME-$PROJECT_NAME.tar.gz"
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
tar -zxf "$DOWNLOADED_ASSET_PATH" -C "$TMP_DIR"
mv "$TMP_DIR/$PROJECT_NAME" "$BINARY_PATH"
chmod +x "$BINARY_PATH"

# Step 5: Cleanup temp files

rm -rf "$TMP_DIR"

echo "Installed binary type: $(file "$BINARY_PATH")"

# Step 6: Create configuration files
DATABASE_PATH="/etc/filebrowser/filebrowser.db"
mkdir -p "$(dirname "$DATABASE_PATH")"
filebrowser config init --address=0.0.0.0 --port=7777 --auth.method=noauth --log=stdout --root=/shortcuts --database="$DATABASE_PATH"
filebrowser users add pi raspberry --database="$DATABASE_PATH"

echo "Finished installing $PROJECT_NAME"
