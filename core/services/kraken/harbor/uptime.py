from datetime import datetime, timezone
from pathlib import Path


def compute_uptime_seconds(started_at: datetime, now: datetime, host_uptime_s: float) -> float:
    return max(0.0, (now - started_at).total_seconds())


def host_uptime_seconds() -> float:
    return float(Path("/proc/uptime").read_text(encoding="utf-8").split()[0])


def parse_docker_time(value: str) -> datetime:
    return datetime.fromisoformat(value)


def now_utc() -> datetime:
    return datetime.now(timezone.utc)
