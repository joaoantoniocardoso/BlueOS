import json
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any
from uuid import uuid4

from load_scenario.constants import PROCESS_RECORDER
from load_scenario.models import LoadScenarioRun, RecorderBinary
from load_scenario.sampling import ProcessMeans


@dataclass(frozen=True)
class PhaseResult:
    duration_seconds: float
    processes: dict[str, ProcessMeans]


@dataclass(frozen=True)
class LoadScenarioResult:  # pylint: disable=too-many-instance-attributes
    schema_version: int
    run_id: str
    started_at: str
    ended_at: str
    recorder_binary: RecorderBinary
    device_host: str
    cpu_governor: str
    throttle_before: str
    throttle_after: str
    throttled: bool
    recording_dropped_samples: bool
    phases: dict[str, PhaseResult]

    def to_load_scenario_run(self) -> LoadScenarioRun:
        phase_means = {
            phase_name: phase.processes[PROCESS_RECORDER].cpu_percent_mean for phase_name, phase in self.phases.items()
        }
        return LoadScenarioRun(
            recorder_binary=self.recorder_binary,
            throttled=self.throttled,
            phase_means=phase_means,
        )


def write_result_file(result: LoadScenarioResult, output_directory: Path) -> Path:
    output_directory.mkdir(parents=True, exist_ok=True)
    output_path = output_directory / f"{result.run_id}.json"
    payload = _result_to_json(result)
    output_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="ascii")
    return output_path


def read_result_file(path: Path) -> LoadScenarioResult:
    payload = json.loads(path.read_text(encoding="ascii"))
    return _result_from_json(payload)


def new_run_id() -> str:
    return uuid4().hex


def utc_now_iso() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def _result_to_json(result: LoadScenarioResult) -> dict[str, Any]:
    payload = asdict(result)
    payload["phases"] = {
        phase_name: {
            "duration_seconds": phase.duration_seconds,
            "processes": {
                process_name: {
                    "cpu_percent_mean": process_means.cpu_percent_mean,
                    "memory_rss_kib_mean": process_means.memory_rss_kib_mean,
                }
                for process_name, process_means in phase.processes.items()
            },
        }
        for phase_name, phase in result.phases.items()
    }
    return payload


def _result_from_json(payload: dict[str, Any]) -> LoadScenarioResult:
    phases: dict[str, PhaseResult] = {}
    for phase_name, phase_payload in payload["phases"].items():
        processes = {
            process_name: ProcessMeans(
                cpu_percent_mean=process_payload["cpu_percent_mean"],
                memory_rss_kib_mean=process_payload["memory_rss_kib_mean"],
            )
            for process_name, process_payload in phase_payload["processes"].items()
        }
        phases[phase_name] = PhaseResult(
            duration_seconds=float(phase_payload["duration_seconds"]),
            processes=processes,
        )
    recording_dropped_samples = bool(payload.get("recording_dropped_samples", False))
    return LoadScenarioResult(
        schema_version=int(payload["schema_version"]),
        run_id=str(payload["run_id"]),
        started_at=str(payload["started_at"]),
        ended_at=str(payload["ended_at"]),
        recorder_binary=payload["recorder_binary"],
        device_host=str(payload["device_host"]),
        cpu_governor=str(payload["cpu_governor"]),
        throttle_before=str(payload["throttle_before"]),
        throttle_after=str(payload["throttle_after"]),
        throttled=bool(payload["throttled"]),
        recording_dropped_samples=recording_dropped_samples,
        phases=phases,
    )
