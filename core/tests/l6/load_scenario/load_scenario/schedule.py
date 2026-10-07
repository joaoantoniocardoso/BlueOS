from load_scenario.models import RecorderBinary

_MIN_RUNS_PER_BINARY = 1


def alternating_recorder_binaries(runs_per_binary: int) -> list[RecorderBinary]:
    if runs_per_binary < _MIN_RUNS_PER_BINARY:
        raise ValueError(f"runs_per_binary must be at least {_MIN_RUNS_PER_BINARY}")

    schedule: list[RecorderBinary] = []
    for index in range(runs_per_binary):
        schedule.append("reference")
        schedule.append("candidate")
    return schedule
