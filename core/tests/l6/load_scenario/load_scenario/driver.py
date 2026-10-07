import asyncio
import math
import tempfile
from dataclasses import dataclass
from pathlib import Path

import aiohttp
from load_scenario.clients.ardupilot import ArdupilotClient
from load_scenario.clients.http_device import HttpDeviceClient
from load_scenario.clients.mavlink2rest import Mavlink2RestClient
from load_scenario.clients.mavlink_camera import MavlinkCameraClient
from load_scenario.clients.recorder import RecorderClient
from load_scenario.clip import ClipMetadata, load_clip_metadata
from load_scenario.constants import (
    ACTIVE_RECORDING_FILE_NAME,
    DEVICE_RECORDINGS_DIRECTORY,
    LOAD_SCENARIO_PHASES,
    PHASE_ARMED_MAVLINK_ONLY,
    PHASE_DURATION_SECONDS,
    PHASE_IDLE,
    PHASE_ONE_STREAM_RECORDING,
    PHASE_ONE_STREAM_REDIRECTED,
    PHASE_TWO_STREAMS_RECORDING,
    RESULT_SCHEMA_VERSION,
    SAMPLE_INTERVAL_SECONDS,
)
from load_scenario.integrity import (
    StreamByteLedger,
    expected_stream_ledger,
    recording_dropped_samples,
)
from load_scenario.mcap_video import compressed_video_ledger
from load_scenario.models import RecorderBinary
from load_scenario.result import (
    LoadScenarioResult,
    PhaseResult,
    new_run_id,
    utc_now_iso,
    write_result_file,
)
from load_scenario.rtsp_source import RtspSource
from load_scenario.sampling import mean_process_metrics
from load_scenario.ssh_device import SshDevice
from load_scenario.throttle import parse_vcgencmd_throttled, run_was_throttled

_VEHICLE_SYSTEM_ID = 1
_PRIMARY_STREAM_NAME = "load-scenario-primary"
_SECONDARY_STREAM_NAME = "load-scenario-secondary"
_PRIMARY_RTSP_PORT = 8554
_SECONDARY_RTSP_PORT = 8555


@dataclass(frozen=True)
class RecordingPhaseExpectation:
    phase_name: str
    stream_count: int


class _PhaseController:
    def __init__(
        self,
        mavlink_client: Mavlink2RestClient,
        primary_camera: MavlinkCameraClient,
        secondary_camera: MavlinkCameraClient,
    ) -> None:
        self._mavlink_client = mavlink_client
        self._primary_camera = primary_camera
        self._secondary_camera = secondary_camera
        self._armed = False
        self._primary_capture = False
        self._secondary_capture = False

    async def apply(self, phase_name: str, primary_rtsp_url: str, secondary_rtsp_url: str) -> None:
        await self._reset_streams()
        if phase_name == PHASE_IDLE:
            return
        if phase_name == PHASE_ONE_STREAM_REDIRECTED:
            await self._primary_camera.create_redirect_stream(primary_rtsp_url)
            return
        if phase_name == PHASE_ARMED_MAVLINK_ONLY:
            await self._mavlink_client.send_arm_command(_VEHICLE_SYSTEM_ID)
            self._armed = True
            return
        if phase_name == PHASE_ONE_STREAM_RECORDING:
            await self._primary_camera.create_redirect_stream(primary_rtsp_url)
            component_id = await self._primary_camera.camera_component_id()
            await self._mavlink_client.send_video_capture_command(True, _VEHICLE_SYSTEM_ID, component_id)
            self._primary_capture = True
            return
        if phase_name == PHASE_TWO_STREAMS_RECORDING:
            await self._primary_camera.create_redirect_stream(primary_rtsp_url)
            await self._secondary_camera.create_redirect_stream(secondary_rtsp_url)
            primary_component_id = await self._primary_camera.camera_component_id()
            secondary_component_id = await self._secondary_camera.camera_component_id()
            await self._mavlink_client.send_video_capture_command(True, _VEHICLE_SYSTEM_ID, primary_component_id)
            await self._mavlink_client.send_video_capture_command(True, _VEHICLE_SYSTEM_ID, secondary_component_id)
            self._primary_capture = True
            self._secondary_capture = True
            return
        raise ValueError(f"Unknown phase {phase_name}")

    async def teardown(self) -> None:
        await self._reset_streams()

    async def _reset_streams(self) -> None:
        if self._primary_capture:
            component_id = await self._primary_camera.camera_component_id()
            await self._mavlink_client.send_video_capture_command(False, _VEHICLE_SYSTEM_ID, component_id)
            self._primary_capture = False
        if self._secondary_capture:
            component_id = await self._secondary_camera.camera_component_id()
            await self._mavlink_client.send_video_capture_command(False, _VEHICLE_SYSTEM_ID, component_id)
            self._secondary_capture = False
        if self._armed:
            await self._mavlink_client.send_disarm_command(_VEHICLE_SYSTEM_ID)
            self._armed = False
        for camera_client in (self._primary_camera, self._secondary_camera):
            try:
                await camera_client.delete_stream()
            except RuntimeError:
                pass


async def run_load_scenario(  # pylint: disable=too-many-locals
    device_host: str,
    output_directory: Path,
    assets_directory: Path,
    recorder_binary: RecorderBinary = "candidate",
    ssh_user: str = "pi",
) -> Path:
    run_id = new_run_id()
    started_at = utc_now_iso()
    ssh_device = SshDevice(device_host, user=ssh_user)
    primary_rtsp_source = RtspSource(assets_directory, mount_name="load_scenario_primary", port=_PRIMARY_RTSP_PORT)
    secondary_rtsp_source = RtspSource(
        assets_directory,
        mount_name="load_scenario_secondary",
        port=_SECONDARY_RTSP_PORT,
    )
    primary_rtsp_source.ensure_tools()
    secondary_rtsp_source.ensure_tools()
    clip_metadata = load_clip_metadata(primary_rtsp_source.clip_path)

    governor = ssh_device.read_cpu_governor()
    ssh_device.set_cpu_governor_performance()
    governor = ssh_device.read_cpu_governor()
    throttle_before_text = ssh_device.read_throttled()
    throttle_before = parse_vcgencmd_throttled(throttle_before_text)

    primary_rtsp_url = primary_rtsp_source.start(device_host)
    secondary_rtsp_url = secondary_rtsp_source.start(device_host)
    recorder_client = RecorderClient(device_host)
    recorder_client.connect()

    phase_results: dict[str, PhaseResult] = {}
    recording_phase_expectations: list[RecordingPhaseExpectation] = []

    try:
        async with aiohttp.ClientSession() as session:
            http_client = HttpDeviceClient(device_host, session)
            ardupilot_client = ArdupilotClient(http_client)
            primary_camera = MavlinkCameraClient(http_client, stream_name=_PRIMARY_STREAM_NAME)
            secondary_camera = MavlinkCameraClient(http_client, stream_name=_SECONDARY_STREAM_NAME)
            mavlink_client = Mavlink2RestClient(device_host, session)
            phase_controller = _PhaseController(mavlink_client, primary_camera, secondary_camera)

            await ardupilot_client.ensure_sitl_running()
            recorder_client.start_recording()

            sample_count = max(1, math.ceil(PHASE_DURATION_SECONDS / SAMPLE_INTERVAL_SECONDS))
            for phase_name in LOAD_SCENARIO_PHASES:
                await phase_controller.apply(phase_name, primary_rtsp_url, secondary_rtsp_url)
                samples = ssh_device.collect_resource_samples(sample_count, SAMPLE_INTERVAL_SECONDS)
                process_means = mean_process_metrics(samples)
                phase_results[phase_name] = PhaseResult(
                    duration_seconds=PHASE_DURATION_SECONDS,
                    processes=process_means,
                )
                stream_count = _recording_stream_count_for_phase(phase_name)
                if stream_count > 0:
                    recording_phase_expectations.append(
                        RecordingPhaseExpectation(phase_name=phase_name, stream_count=stream_count)
                    )
            await phase_controller.teardown()
    finally:
        recorder_client.close()
        primary_rtsp_source.stop()
        secondary_rtsp_source.stop()

    throttle_after_text = ssh_device.read_throttled()
    throttle_after = parse_vcgencmd_throttled(throttle_after_text)
    ended_at = utc_now_iso()
    dropped_samples = _recording_dropped_samples(
        ssh_device=ssh_device,
        clip_metadata=clip_metadata,
        recording_phase_expectations=recording_phase_expectations,
    )

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
        recording_dropped_samples=dropped_samples,
        phases=phase_results,
    )
    return write_result_file(result, output_directory)


def run_load_scenario_sync(
    device_host: str,
    output_directory: Path,
    assets_directory: Path,
    recorder_binary: RecorderBinary = "candidate",
    ssh_user: str = "pi",
) -> Path:
    return asyncio.run(
        run_load_scenario(
            device_host=device_host,
            output_directory=output_directory,
            assets_directory=assets_directory,
            recorder_binary=recorder_binary,
            ssh_user=ssh_user,
        )
    )


async def run_one_stream_redirected_phase(
    device_host: str,
    output_directory: Path,
    assets_directory: Path,
    recorder_binary: RecorderBinary = "candidate",
    ssh_user: str = "pi",
) -> Path:
    return await run_load_scenario(
        device_host=device_host,
        output_directory=output_directory,
        assets_directory=assets_directory,
        recorder_binary=recorder_binary,
        ssh_user=ssh_user,
    )


def run_one_stream_redirected_phase_sync(
    device_host: str,
    output_directory: Path,
    assets_directory: Path,
    recorder_binary: RecorderBinary = "candidate",
    ssh_user: str = "pi",
) -> Path:
    return run_load_scenario_sync(
        device_host=device_host,
        output_directory=output_directory,
        assets_directory=assets_directory,
        recorder_binary=recorder_binary,
        ssh_user=ssh_user,
    )


def _recording_stream_count_for_phase(phase_name: str) -> int:
    if phase_name == PHASE_ONE_STREAM_RECORDING:
        return 1
    if phase_name == PHASE_TWO_STREAMS_RECORDING:
        return 2
    return 0


def _recording_dropped_samples(
    ssh_device: SshDevice,
    clip_metadata: ClipMetadata,
    recording_phase_expectations: list[RecordingPhaseExpectation],
) -> bool:
    if not recording_phase_expectations:
        return False
    remote_recording_path = f"{DEVICE_RECORDINGS_DIRECTORY}/{ACTIVE_RECORDING_FILE_NAME}"
    with tempfile.TemporaryDirectory(prefix="load-scenario-") as temporary_directory:
        local_recording_path = Path(temporary_directory) / ACTIVE_RECORDING_FILE_NAME
        ssh_device.copy_file_from_device(remote_recording_path, local_recording_path)
        expected_frames = 0
        expected_bytes = 0
        for recording_phase in recording_phase_expectations:
            expected = expected_stream_ledger(
                PHASE_DURATION_SECONDS,
                clip_metadata,
                recording_phase.stream_count,
            )
            expected_frames += expected.frame_count
            expected_bytes += expected.payload_bytes
        recorded = compressed_video_ledger(local_recording_path, None)
        return recording_dropped_samples(
            StreamByteLedger(frame_count=expected_frames, payload_bytes=expected_bytes),
            recorded,
        )
