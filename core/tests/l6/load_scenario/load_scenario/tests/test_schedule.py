from load_scenario.constants import MIN_RUNS_PER_BINARY
from load_scenario.schedule import (
    alternating_recorder_binaries,
    reference_recorder_schedule,
)


def test_alternating_schedule_starts_with_reference() -> None:
    schedule = alternating_recorder_binaries(3)
    assert schedule == ["reference", "candidate", "reference", "candidate", "reference", "candidate"]


def test_alternating_schedule_requires_at_least_one_per_binary() -> None:
    assert alternating_recorder_binaries(10)[0] == "reference"
    assert alternating_recorder_binaries(10)[-1] == "candidate"


def test_reference_schedule_requires_minimum_run_count() -> None:
    schedule = reference_recorder_schedule(MIN_RUNS_PER_BINARY)
    assert schedule == ["reference"] * MIN_RUNS_PER_BINARY
    assert len(alternating_recorder_binaries(10)) == 20
