from dataclasses import dataclass
from typing import Literal

RecorderBinary = Literal["reference", "candidate"]


@dataclass(frozen=True)
class LoadScenarioRun:
    recorder_binary: RecorderBinary
    throttled: bool
    phase_means: dict[str, float]


@dataclass(frozen=True)
class PhaseStatistics:
    reference_median: float
    candidate_median: float
    median_percentage_of_reference: float | None
    median_difference: float
    confidence_interval_95: tuple[float, float]
    p_value: float
