import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from harbor.uptime import compute_uptime_seconds  # noqa: E402

NOW = datetime(2026, 1, 1, 12, 0, tzinfo=timezone.utc)
HOUR = 3600.0


def test_uptime_is_clamped_to_host_uptime_after_clock_step() -> None:
    assert compute_uptime_seconds(NOW - timedelta(hours=17), NOW, 4 * HOUR) == 4 * HOUR


def test_uptime_without_clock_step() -> None:
    assert compute_uptime_seconds(NOW - timedelta(minutes=5), NOW, 4 * HOUR) == 300.0


def test_uptime_of_future_start_is_zero() -> None:
    assert compute_uptime_seconds(NOW + timedelta(hours=1), NOW, 4 * HOUR) == 0.0
