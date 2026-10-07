from pathlib import Path

from load_scenario.constants import PHASE_ONE_STREAM_REDIRECTED, PROCESS_RECORDER
from load_scenario.result import LoadScenarioResult, PhaseResult, read_result_file, write_result_file
from load_scenario.sampling import ProcessMeans


def test_result_round_trip(tmp_path: Path) -> None:
    result = LoadScenarioResult(
        schema_version=1,
        run_id="abc123",
        started_at="2026-01-01T00:00:00Z",
        ended_at="2026-01-01T00:01:00Z",
        recorder_binary="candidate",
        device_host="blueos.local",
        cpu_governor="performance",
        throttle_before="throttled=0x0",
        throttle_after="throttled=0x0",
        throttled=False,
        phases={
            PHASE_ONE_STREAM_REDIRECTED: PhaseResult(
                duration_seconds=60.0,
                processes={
                    PROCESS_RECORDER: ProcessMeans(cpu_percent_mean=5.5, memory_rss_kib_mean=1000.0),
                    "mavlink_camera_manager": ProcessMeans(cpu_percent_mean=10.0, memory_rss_kib_mean=2000.0),
                    "zenohd": ProcessMeans(cpu_percent_mean=1.0, memory_rss_kib_mean=500.0),
                    "system": ProcessMeans(cpu_percent_mean=40.0, memory_rss_kib_mean=0.0),
                },
            )
        },
    )
    output_path = write_result_file(result, tmp_path)
    loaded = read_result_file(output_path)
    assert loaded.run_id == result.run_id
    run = loaded.to_load_scenario_run()
    assert run.phase_means[PHASE_ONE_STREAM_REDIRECTED] == 5.5
