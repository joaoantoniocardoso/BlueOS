from load_scenario.constants import MIN_RUNS_PER_BINARY
from load_scenario.models import RecorderBinary

_MIN_RUNS_PER_BINARY = 1


def alternating_recorder_binaries(runs_per_binary: int) -> list[RecorderBinary]:
    if runs_per_binary < _MIN_RUNS_PER_BINARY:
        raise ValueError(f"runs_per_binary must be at least {_MIN_RUNS_PER_BINARY}")

    schedule: list[RecorderBinary] = []
    for _index in range(runs_per_binary):
        schedule.append("reference")
        schedule.append("candidate")
    return schedule


def reference_recorder_schedule(run_count: int) -> list[RecorderBinary]:
    if run_count < MIN_RUNS_PER_BINARY:
        raise ValueError(f"run_count must be at least {MIN_RUNS_PER_BINARY}")
    return ["reference"] * run_count
