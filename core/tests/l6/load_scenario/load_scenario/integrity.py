from dataclasses import dataclass

from load_scenario.clip import ClipMetadata


@dataclass(frozen=True)
class StreamByteLedger:
    frame_count: int
    payload_bytes: int


def expected_stream_ledger(
    phase_duration_seconds: float,
    clip: ClipMetadata,
    stream_count: int,
) -> StreamByteLedger:
    if stream_count < 1:
        raise ValueError("stream_count must be at least 1")

    loops = phase_duration_seconds / clip.duration_seconds
    frame_count = int(loops * clip.frame_rate * clip.duration_seconds * stream_count)
    payload_bytes = int(loops * clip.byte_size * stream_count)
    return StreamByteLedger(frame_count=frame_count, payload_bytes=payload_bytes)


def recording_dropped_samples(sent: StreamByteLedger, recorded: StreamByteLedger) -> bool:
    return recorded.frame_count < sent.frame_count or recorded.payload_bytes < sent.payload_bytes
