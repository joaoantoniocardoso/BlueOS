from load_scenario.clip import ClipMetadata
from load_scenario.constants import PHASE_DURATION_SECONDS
from load_scenario.integrity import (
    StreamByteLedger,
    expected_stream_ledger,
    recording_dropped_samples,
)


def test_recording_dropped_samples_when_frames_or_bytes_are_short() -> None:
    sent = StreamByteLedger(frame_count=100, payload_bytes=1_000_000)
    assert recording_dropped_samples(sent, StreamByteLedger(frame_count=99, payload_bytes=1_000_000))
    assert recording_dropped_samples(sent, StreamByteLedger(frame_count=100, payload_bytes=999_999))
    assert not recording_dropped_samples(sent, StreamByteLedger(frame_count=100, payload_bytes=1_000_000))


def test_expected_stream_ledger_scales_with_phase_duration() -> None:
    clip = ClipMetadata(byte_size=10_000, duration_seconds=5.0, frame_rate=30.0)
    ledger = expected_stream_ledger(PHASE_DURATION_SECONDS, clip, stream_count=1)
    assert ledger.frame_count == 1_800
    assert ledger.payload_bytes == 120_000

    two_streams = expected_stream_ledger(PHASE_DURATION_SECONDS, clip, stream_count=2)
    assert two_streams.frame_count == 3_600
    assert two_streams.payload_bytes == 240_000
