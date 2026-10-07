from pathlib import Path

from load_scenario.campaign import load_runs_from_directory
from load_scenario.constants import PHASE_IDLE
from load_scenario.result import LoadScenarioResult, PhaseResult, write_result_file
from load_scenario.sampling import ProcessMeans


def test_load_runs_from_directory_skips_throttled_files(tmp_path: Path) -> None:
    good = LoadScenarioResult(
        schema_version=2,
        run_id="good",
        started_at="2026-01-01T00:00:00Z",
        ended_at="2026-01-01T00:10:00Z",
        recorder_binary="reference",
        device_host="device",
        cpu_governor="performance",
        throttle_before="throttled=0x0",
        throttle_after="throttled=0x0",
        throttled=False,
        recording_dropped_samples=False,
        phases={
            PHASE_IDLE: PhaseResult(
                duration_seconds=60.0,
                processes={
                    "recorder": ProcessMeans(cpu_percent_mean=1.0, memory_rss_kib_mean=100.0),
                },
            )
        },
    )
    throttled = LoadScenarioResult(
        schema_version=2,
        run_id="bad",
        started_at=good.started_at,
        ended_at=good.ended_at,
        recorder_binary="candidate",
        device_host=good.device_host,
        cpu_governor=good.cpu_governor,
        throttle_before=good.throttle_before,
        throttle_after=good.throttle_after,
        throttled=True,
        recording_dropped_samples=False,
        phases=good.phases,
    )
    write_result_file(good, tmp_path)
    write_result_file(throttled, tmp_path)

    runs = load_runs_from_directory(tmp_path)
    assert len(runs) == 1
    assert runs[0].recorder_binary == "reference"
    assert runs[0].phase_means[PHASE_IDLE] == 1.0
