import random
import statistics
from collections.abc import Sequence

from load_scenario.models import LoadScenarioRun, PhaseStatistics

_BOOTSTRAP_RESAMPLES = 10_000
_PERMUTATION_RESAMPLES = 10_000
_REPORT_RANDOM_SEED = 33


def build_load_scenario_report(
    runs: Sequence[LoadScenarioRun],
    reference_runs: Sequence[LoadScenarioRun] | None = None,
) -> dict[str, PhaseStatistics]:
    included_runs = [run for run in runs if not run.throttled]
    included_reference_runs = (
        [run for run in reference_runs if not run.throttled] if reference_runs is not None else None
    )
    phase_names = sorted({phase for run in included_runs for phase in run.phase_means})
    if included_reference_runs is not None:
        phase_names = sorted(set(phase_names) | {phase for run in included_reference_runs for phase in run.phase_means})
    random_generator = random.Random(_REPORT_RANDOM_SEED)

    report: dict[str, PhaseStatistics] = {}
    for phase in phase_names:
        if included_reference_runs is not None:
            reference_values = [run.phase_means[phase] for run in included_reference_runs if phase in run.phase_means]
            candidate_values = [
                run.phase_means[phase]
                for run in included_runs
                if run.recorder_binary == "candidate" and phase in run.phase_means
            ]
        else:
            reference_values = [
                run.phase_means[phase]
                for run in included_runs
                if run.recorder_binary == "reference" and phase in run.phase_means
            ]
            candidate_values = [
                run.phase_means[phase]
                for run in included_runs
                if run.recorder_binary == "candidate" and phase in run.phase_means
            ]
        report[phase] = _phase_statistics(reference_values, candidate_values, random_generator)
    return report


def _phase_statistics(
    reference_values: list[float],
    candidate_values: list[float],
    random_generator: random.Random,
) -> PhaseStatistics:
    reference_median = statistics.median(reference_values) if reference_values else 0.0
    candidate_median = statistics.median(candidate_values) if candidate_values else 0.0
    observed_difference = candidate_median - reference_median
    confidence_interval = _bootstrap_median_difference_interval(
        reference_values,
        candidate_values,
        random_generator,
    )
    p_value = _permutation_test_p_value(reference_values, candidate_values, random_generator)
    median_percentage = _median_percentage_of_reference(reference_median, candidate_median)
    return PhaseStatistics(
        reference_median=reference_median,
        candidate_median=candidate_median,
        median_percentage_of_reference=median_percentage,
        median_difference=observed_difference,
        confidence_interval_95=confidence_interval,
        p_value=p_value,
    )


def _median_percentage_of_reference(reference_median: float, candidate_median: float) -> float | None:
    if reference_median == 0.0:
        return None
    return 100.0 * candidate_median / reference_median


def _bootstrap_median_difference_interval(
    reference_values: list[float],
    candidate_values: list[float],
    random_generator: random.Random,
) -> tuple[float, float]:
    if not reference_values or not candidate_values:
        return (0.0, 0.0)

    bootstrap_differences: list[float] = []
    for _ in range(_BOOTSTRAP_RESAMPLES):
        resampled_reference = random_generator.choices(reference_values, k=len(reference_values))
        resampled_candidate = random_generator.choices(candidate_values, k=len(candidate_values))
        bootstrap_differences.append(statistics.median(resampled_candidate) - statistics.median(resampled_reference))
    bootstrap_differences.sort()
    lower_index = int(0.025 * len(bootstrap_differences))
    upper_index = int(0.975 * len(bootstrap_differences)) - 1
    return (bootstrap_differences[lower_index], bootstrap_differences[upper_index])


def _permutation_test_p_value(
    reference_values: list[float],
    candidate_values: list[float],
    random_generator: random.Random,
) -> float:
    if not reference_values or not candidate_values:
        return 1.0

    pooled_values = reference_values + candidate_values
    reference_count = len(reference_values)
    observed_difference = abs(statistics.median(candidate_values) - statistics.median(reference_values))

    at_least_as_extreme = 0
    for _ in range(_PERMUTATION_RESAMPLES):
        shuffled_values = list(pooled_values)
        random_generator.shuffle(shuffled_values)
        permuted_reference = shuffled_values[:reference_count]
        permuted_candidate = shuffled_values[reference_count:]
        permuted_difference = abs(statistics.median(permuted_candidate) - statistics.median(permuted_reference))
        if permuted_difference >= observed_difference:
            at_least_as_extreme += 1

    return (at_least_as_extreme + 1) / (_PERMUTATION_RESAMPLES + 1)
