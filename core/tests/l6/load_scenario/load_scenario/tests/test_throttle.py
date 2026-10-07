from load_scenario.throttle import parse_vcgencmd_throttled, run_was_throttled


def test_parse_vcgencmd_throttled() -> None:
    assert parse_vcgencmd_throttled("throttled=0x0") == 0
    assert parse_vcgencmd_throttled("throttled=0x50005") == 0x50005


def test_run_was_throttled_detects_throttle_bits() -> None:
    assert not run_was_throttled(0, 0)
    assert run_was_throttled(0, 0x4)


def test_run_was_throttled_detects_throttle_that_happened_and_ended_during_the_run() -> None:
    assert run_was_throttled(0, 0x50000)
    assert not run_was_throttled(0x50000, 0x50000)
