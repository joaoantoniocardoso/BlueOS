#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ]; then
  echo "Usage: $0 <clip.h264> <rtsp://host:port/mount>" >&2
  exit 1
fi

clip_path="$1"
rtsp_url="$2"

if command -v test-launch >/dev/null 2>&1; then
  exec test-launch "( filesrc location=${clip_path} ! h264parse ! rtph264pay name=pay0 pt=96 )"
fi

if [ "${LOAD_SCENARIO_RTSP_FALLBACK:-}" = "1" ] && command -v ffmpeg >/dev/null 2>&1; then
  exec ffmpeg -re -stream_loop -1 -i "${clip_path}" -c copy -f rtsp -rtsp_flags listen "${rtsp_url}"
fi

echo "Install gst-rtsp-server (test-launch) or set LOAD_SCENARIO_RTSP_FALLBACK=1 with ffmpeg." >&2
exit 1
