import asyncio
import math
from pathlib import Path

import aiohttp

from load_scenario.clients.ardupilot import ArdupilotClient
from load_scenario.clients.http_device import HttpDeviceClient
from load_scenario.clients.mavlink2rest import Mavlink2RestClient
from load_scenario.clients.mavlink_camera import MavlinkCameraClient
from load_scenario.clients.recorder import RecorderClient
from load_scenario.constants import (
    PHASE_DURATION_SECONDS,
    PHASE_ONE_STREAM_REDIRECTED,
    RESULT_SCHEMA_VERSION,
    SAMPLE_INTERVAL_SECONDS,
)
from load_scenario.models import RecorderBinary
from load_scenario.result import LoadScenarioResult, PhaseResult, new_run_id, utc_now_iso, write_result_file
from load_scenario.rtsp_source import RtspSource
from load_scenario.sampling import mean_process_metrics
from load_scenario.ssh_device import SshDevice
from load_scenario.throttle import parse_vcgencmd_throttled, run_was_throttled


async def run_one_stream_redirected_phase(
    device_host: str,
    output_directory: Path,
    assets_directory: Path,
    recorder_binary: RecorderBinary = "candidate",
    ssh_user: str = "pi",
) -> Path:
    run_id = new_run_id()
    started_at = utc_now_iso()
    ssh_device = SshDevice(device_host, user=ssh_user)
    rtsp_source = RtspSource(assets_directory)
    rtsp_source.ensure_tools()

    governor = ssh_device.read_cpu_governor()
    ssh_device.set_cpu_governor_performance()
    governor = ssh_device.read_cpu_governor()
    throttle_before_text = ssh_device.read_throttled()
    throttle_before = parse_vcgencmd_throttled(throttle_before_text)

    rtsp_url = rtsp_source.start(device_host)
    recorder_client = RecorderClient(device_host)
    recorder_client.connect()

    try:
        async with aiohttp.ClientSession() as session:
            http_client = HttpDeviceClient(device_host, session)
            ardupilot_client = ArdupilotClient(http_client)
            mavlink_camera_client = MavlinkCameraClient(http_client)
            mavlink_client = Mavlink2RestClient(device_host, session)

            await ardupilot_client.ensure_sitl_running()
            recorder_client.start_recording()
            await mavlink_client.send_arm_command()
            await mavlink_camera_client.create_redirect_stream(rtsp_url)

            sample_count = max(1, math.ceil(PHASE_DURATION_SECONDS / SAMPLE_INTERVAL_SECONDS))
            samples = ssh_device.collect_resource_samples(sample_count, SAMPLE_INTERVAL_SECONDS)
            process_means = mean_process_metrics(samples)
            phase_result = PhaseResult(
                duration_seconds=PHASE_DURATION_SECONDS,
                processes=process_means,
            )

            try:
                await mavlink_camera_client.delete_stream()
            except RuntimeError:
                pass
    finally:
        recorder_client.close()
        rtsp_source.stop()

    throttle_after_text = ssh_device.read_throttled()
    throttle_after = parse_vcgencmd_throttled(throttle_after_text)
    ended_at = utc_now_iso()

    result = LoadScenarioResult(
        schema_version=RESULT_SCHEMA_VERSION,
        run_id=run_id,
        started_at=started_at,
        ended_at=ended_at,
        recorder_binary=recorder_binary,
        device_host=device_host,
        cpu_governor=governor,
        throttle_before=throttle_before_text,
        throttle_after=throttle_after_text,
        throttled=run_was_throttled(throttle_before, throttle_after),
        phases={PHASE_ONE_STREAM_REDIRECTED: phase_result},
    )
    return write_result_file(result, output_directory)


def run_one_stream_redirected_phase_sync(
    device_host: str,
    output_directory: Path,
    assets_directory: Path,
    recorder_binary: RecorderBinary = "candidate",
    ssh_user: str = "pi",
) -> Path:
    return asyncio.run(
        run_one_stream_redirected_phase(
            device_host=device_host,
            output_directory=output_directory,
            assets_directory=assets_directory,
            recorder_binary=recorder_binary,
            ssh_user=ssh_user,
        )
    )
