from load_scenario.schedule import alternating_recorder_binaries


def test_alternating_schedule_starts_with_reference() -> None:
    schedule = alternating_recorder_binaries(3)
    assert schedule == ["reference", "candidate", "reference", "candidate", "reference", "candidate"]


def test_alternating_schedule_requires_at_least_one_per_binary() -> None:
    assert alternating_recorder_binaries(10)[0] == "reference"
    assert alternating_recorder_binaries(10)[-1] == "candidate"
    assert len(alternating_recorder_binaries(10)) == 20
