from collections.abc import Sequence
from dataclasses import dataclass

from load_scenario.constants import MONITORED_PROCESSES


@dataclass(frozen=True)
class ProcessSample:
    cpu_percent: float
    memory_rss_kib: int


@dataclass(frozen=True)
class ResourceSample:
    processes: dict[str, ProcessSample]


@dataclass(frozen=True)
class ProcessMeans:
    cpu_percent_mean: float
    memory_rss_kib_mean: float


def parse_remote_sample_line(line: str) -> ResourceSample:
    processes: dict[str, ProcessSample] = {}
    for field in line.strip().split():
        name, cpu_text, memory_text = field.split(":", maxsplit=2)
        processes[name] = ProcessSample(cpu_percent=float(cpu_text), memory_rss_kib=int(memory_text))
    missing = set(MONITORED_PROCESSES) - processes.keys()
    if missing:
        raise ValueError(f"Remote sample missing processes: {sorted(missing)}")
    return ResourceSample(processes=processes)


def mean_process_metrics(samples: Sequence[ResourceSample]) -> dict[str, ProcessMeans]:
    if not samples:
        raise ValueError("Cannot compute means from an empty sample list")

    means: dict[str, ProcessMeans] = {}
    for process_name in MONITORED_PROCESSES:
        cpu_values = [sample.processes[process_name].cpu_percent for sample in samples]
        memory_values = [sample.processes[process_name].memory_rss_kib for sample in samples]
        means[process_name] = ProcessMeans(
            cpu_percent_mean=sum(cpu_values) / len(cpu_values),
            memory_rss_kib_mean=sum(memory_values) / len(memory_values),
        )
    return means
