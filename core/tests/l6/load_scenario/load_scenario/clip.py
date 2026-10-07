from dataclasses import dataclass
from pathlib import Path

from load_scenario.constants import CLIP_DURATION_SECONDS, CLIP_FRAME_RATE


@dataclass(frozen=True)
class ClipMetadata:
    byte_size: int
    duration_seconds: float
    frame_rate: float


def load_clip_metadata(clip_path: Path) -> ClipMetadata:
    if not clip_path.is_file():
        raise FileNotFoundError(f"Missing clip at {clip_path}")
    return ClipMetadata(
        byte_size=clip_path.stat().st_size,
        duration_seconds=CLIP_DURATION_SECONDS,
        frame_rate=CLIP_FRAME_RATE,
    )
