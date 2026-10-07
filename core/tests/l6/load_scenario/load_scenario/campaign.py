from pathlib import Path

from load_scenario.models import LoadScenarioRun
from load_scenario.result import read_result_file


def load_runs_from_directory(results_directory: Path) -> list[LoadScenarioRun]:
    runs: list[LoadScenarioRun] = []
    for result_path in sorted(results_directory.glob("*.json")):
        result = read_result_file(result_path)
        if result.throttled:
            continue
        runs.append(result.to_load_scenario_run())
    return runs
