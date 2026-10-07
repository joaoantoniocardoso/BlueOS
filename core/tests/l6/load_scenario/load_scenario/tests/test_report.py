from load_scenario.models import LoadScenarioRun
from load_scenario.report import build_load_scenario_report

PHASE = "one_stream_redirected"


def _run(recorder_binary: str, mean: float, throttled: bool = False) -> LoadScenarioRun:
    return LoadScenarioRun(
        recorder_binary=recorder_binary,
        throttled=throttled,
        phase_means={PHASE: mean},
    )


def test_known_difference_is_detected() -> None:
    runs = [
        _run("reference", mean)
        for mean in [10.0, 10.5, 9.8, 10.2, 9.9, 10.1, 10.0, 10.3, 9.7, 10.4]
    ] + [
        _run("candidate", mean)
        for mean in [20.0, 19.5, 20.2, 19.8, 20.1, 19.9, 20.0, 19.7, 20.3, 19.6]
    ]

    report = build_load_scenario_report(runs)
    statistics = report[PHASE]

    assert statistics.median_difference > 5.0
    assert statistics.confidence_interval_95[0] > 0.0
    assert statistics.p_value < 0.05


def test_identical_inputs_give_high_p_value_and_interval_containing_zero() -> None:
    shared_means = [12.0, 11.5, 12.2, 11.8, 12.1, 11.9, 12.0, 11.7, 12.3, 11.6]
    runs = [_run("reference", mean) for mean in shared_means] + [
        _run("candidate", mean) for mean in shared_means
    ]

    report = build_load_scenario_report(runs)
    statistics = report[PHASE]

    assert statistics.median_difference == 0.0
    assert statistics.confidence_interval_95[0] <= 0.0 <= statistics.confidence_interval_95[1]
    assert statistics.p_value > 0.9


def test_throttled_runs_are_excluded() -> None:
    runs = [
        _run("reference", 10.0),
        _run("reference", 10.0),
        _run("reference", 10.0),
        _run("reference", 10.0),
        _run("reference", 10.0),
        _run("candidate", 10.0),
        _run("candidate", 10.0),
        _run("candidate", 10.0),
        _run("candidate", 10.0),
        _run("candidate", 10.0),
        _run("candidate", 999.0, throttled=True),
    ]

    report = build_load_scenario_report(runs)
    statistics = report[PHASE]

    assert statistics.median_difference == 0.0
    assert statistics.p_value > 0.9
