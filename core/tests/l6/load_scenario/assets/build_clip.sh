#!/usr/bin/env bash
set -euo pipefail

script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
clip_path="${script_directory}/clip.h264"
checksum_path="${script_directory}/clip.sha256"

ffmpeg -y -hide_banner -loglevel error -f lavfi -i testsrc=duration=5:size=1920x1080:rate=30 \
  -c:v libx264 -b:v 50M -maxrate 50M -bufsize 100M -g 30 -an -f h264 "${clip_path}"

sha256sum "${clip_path}" | awk '{print $1}' > "${checksum_path}"
echo "Wrote ${clip_path} and ${checksum_path}"
