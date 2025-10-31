#!/bin/bash

# Immediately exit on errors
set -e

VERSION="1.3.4"
PROJECT_NAMES=(
  "zenoh"
  "zenoh-plugin-webserver"
  "zenoh-backend-filesystem"
  "zenoh-ts"
)
REPOSITORY_ORG="eclipse-zenoh"

echo "Installing projects ${PROJECT_NAMES[*]} version $VERSION"

# Step 1: Determine architecture

ARCH="$(uname -m)"
case "$ARCH" in
  x86_64 | amd64)
    BUILD_NAME="x86_64-unknown-linux-gnu"
    ;;
  armv7l | armhf)
    BUILD_NAME="armv7-unknown-linux-gnueabihf"
    ;;
  aarch64 | arm64)
    BUILD_NAME="aarch64-unknown-linux-gnu"
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
    LIB_DIR="$BIN_DIR"
else
    BIN_DIR="/usr/bin"
    LIB_DIR="/usr/lib"
fi
mkdir -p "$BIN_DIR" "$LIB_DIR"

# Step 3: Download and verify sources

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
for PROJECT_NAME in "${PROJECT_NAMES[@]}"; do
  REPOSITORY_NAME="$PROJECT_NAME"
  ARTIFACT_NAME="$PROJECT_NAME-$VERSION-$BUILD_NAME-standalone.zip"
  read -r DOWNLOADED_ASSET_PATH TMP_DIR < <(
  env \
    REPOSITORY_ORG="$REPOSITORY_ORG" \
    REPOSITORY_NAME="$REPOSITORY_NAME" \
    VERSION="$VERSION" \
    PROJECT_NAME="$PROJECT_NAME" \
    BUILD_NAME="$BUILD_NAME" \
    ARTIFACT_NAME="$ARTIFACT_NAME" \
    "$SCRIPT_DIR/../bootstrap_from_github.sh"
)

  # Step 4: Install
  unzip -q "$DOWNLOADED_ASSET_PATH" -d "$TMP_DIR"
  chmod +x "$TMP_DIR"/*
  mapfile -t LIB_FILES < <(find "$TMP_DIR" -name "*.so" -printf '%f\n')
  mapfile -t BIN_FILES < <(find "$TMP_DIR" -type f ! -name "*.so" -printf '%f\n')

  if [ "${#LIB_FILES[@]}" -gt 0 ]; then
    echo "Installing shared libraries to $LIB_DIR:"
    for f in "${LIB_FILES[@]}"; do
      echo "  → $f"
      mv "$TMP_DIR/$f" "$LIB_DIR/"
    done
  fi

  if [ "${#BIN_FILES[@]}" -gt 0 ]; then
    echo "Installing binaries to $BIN_DIR:"
    for f in "${BIN_FILES[@]}"; do
      echo "  → $f"
      mv "$TMP_DIR/$f" "$BIN_DIR/"
    done
  fi

  echo "Finished installing $PROJECT_NAME"
done
