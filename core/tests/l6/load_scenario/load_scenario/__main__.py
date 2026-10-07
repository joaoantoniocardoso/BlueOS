import argparse
from pathlib import Path

from load_scenario.driver import run_one_stream_redirected_phase_sync


def main() -> None:
    parser = argparse.ArgumentParser(description="Run the D-33 load scenario on a BlueOS device (topside driver).")
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
        help="Which Recorder binary is installed on the device for this run.",
    )
    parser.add_argument("--ssh-user", default="pi", help="SSH user on the device.")
    arguments = parser.parse_args()

    output_path = run_one_stream_redirected_phase_sync(
        device_host=arguments.device_host,
        output_directory=arguments.output_directory,
        assets_directory=arguments.assets_directory,
        recorder_binary=arguments.recorder_binary,
        ssh_user=arguments.ssh_user,
    )
    print(output_path)


if __name__ == "__main__":
    main()
