import json
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path

from load_scenario.integrity import StreamByteLedger

_COMPRESSED_VIDEO_SCHEMA = "foxglove.CompressedVideo"


@dataclass(frozen=True)
class PhaseTimeWindow:
    start_unix_seconds: float
    end_unix_seconds: float


def compressed_video_ledger(
    mcap_path: Path,
    time_window: PhaseTimeWindow | None = None,
) -> StreamByteLedger:
    if shutil.which("mcap") is None:
        raise RuntimeError("Install the mcap CLI on the topside computer to verify recording integrity.")

    command = ["mcap", "cat", str(mcap_path), "--json"]
    completed = subprocess.run(command, check=True, capture_output=True, text=True)
    frame_count = 0
    payload_bytes = 0
    for line in completed.stdout.splitlines():
        if not line.strip():
            continue
        payload = json.loads(line)
        if payload.get("schema") != _COMPRESSED_VIDEO_SCHEMA:
            continue
        log_time_seconds = _log_time_to_unix_seconds(payload.get("logTime"))
        if time_window is not None and not _time_window_contains(time_window, log_time_seconds):
            continue
        message = payload.get("message", {})
        data = message.get("data")
        if not isinstance(data, list):
            continue
        frame_count += 1
        payload_bytes += len(data)
    return StreamByteLedger(frame_count=frame_count, payload_bytes=payload_bytes)


def _log_time_to_unix_seconds(log_time: object) -> float:
    if log_time is None:
        return 0.0
    if isinstance(log_time, (int, float)):
        return float(log_time) / 1_000_000_000.0
    return 0.0


def _time_window_contains(time_window: PhaseTimeWindow, log_time_seconds: float) -> bool:
    return time_window.start_unix_seconds <= log_time_seconds <= time_window.end_unix_seconds
