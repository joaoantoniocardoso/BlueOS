import argparse
import json
from pathlib import Path

from load_scenario.campaign import load_runs_from_directory
from load_scenario.constants import MIN_RUNS_PER_BINARY
from load_scenario.driver import run_load_scenario_sync
from load_scenario.report import build_load_scenario_report
from load_scenario.schedule import alternating_recorder_binaries


def main() -> None:
    parser = argparse.ArgumentParser(description="Run the D-33 load scenario on a BlueOS device (topside driver).")
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_parser = subparsers.add_parser("run", help="Run all load scenario phases once on the device.")
    _add_common_device_arguments(run_parser)

    campaign_parser = subparsers.add_parser(
        "campaign",
        help="Alternate reference and candidate Recorder binaries for many runs.",
    )
    _add_common_device_arguments(campaign_parser)
    campaign_parser.add_argument(
        "--runs-per-binary",
        type=int,
        default=MIN_RUNS_PER_BINARY,
        help=f"Runs per Recorder binary (default {MIN_RUNS_PER_BINARY}).",
    )

    report_parser = subparsers.add_parser("report", help="Build per-phase statistics from result JSON files.")
    report_parser.add_argument(
        "--results-directory",
        type=Path,
        default=Path("load-scenario-results"),
        help="Directory with one JSON file per run.",
    )

    arguments = parser.parse_args()
    if arguments.command == "run":
        output_path = run_load_scenario_sync(
            device_host=arguments.device_host,
            output_directory=arguments.output_directory,
            assets_directory=arguments.assets_directory,
            recorder_binary=arguments.recorder_binary,
            ssh_user=arguments.ssh_user,
        )
        print(output_path)
        return

    if arguments.command == "campaign":
        schedule = alternating_recorder_binaries(arguments.runs_per_binary)
        for index, recorder_binary in enumerate(schedule, start=1):
            print(
                f"Run {index}/{len(schedule)}: install the {recorder_binary} Recorder binary on the device, "
                "then press Enter to continue.",
                flush=True,
            )
            input()
            output_path = run_load_scenario_sync(
                device_host=arguments.device_host,
                output_directory=arguments.output_directory,
                assets_directory=arguments.assets_directory,
                recorder_binary=recorder_binary,
                ssh_user=arguments.ssh_user,
            )
            print(output_path)
        return

    if arguments.command == "report":
        runs = load_runs_from_directory(arguments.results_directory)
        report = build_load_scenario_report(runs)
        payload = {
            phase: {
                "median_difference": statistics.median_difference,
                "confidence_interval_95": statistics.confidence_interval_95,
                "p_value": statistics.p_value,
            }
            for phase, statistics in report.items()
        }
        print(json.dumps(payload, indent=2, sort_keys=True))
        return


def _add_common_device_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("device_host", help="Device hostname or IP reachable over HTTP and SSH.")
    parser.add_argument(
        "--output-directory",
        type=Path,
        default=Path("load-scenario-results"),
        help="Directory where one JSON file is written per run.",
    )
    parser.add_argument(
        "--assets-directory",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "assets",
        help="Directory with clip.sha256, build_clip.sh and rtsp_server.sh.",
    )
    parser.add_argument(
        "--recorder-binary",
        choices=("reference", "candidate"),
        default="candidate",
        help="Which Recorder binary is installed on the device (campaign mode sets this each run).",
    )
    parser.add_argument("--ssh-user", default="pi", help="SSH user on the device.")


if __name__ == "__main__":
    main()
