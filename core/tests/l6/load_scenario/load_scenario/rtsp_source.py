import hashlib
import shutil
import socket
import subprocess
from pathlib import Path


class RtspSource:
    def __init__(
        self,
        assets_directory: Path,
        mount_name: str = "load_scenario",
        port: int = 8554,
    ) -> None:
        self._assets_directory = assets_directory
        self._mount_name = mount_name
        self._port = port
        self._process: subprocess.Popen[bytes] | None = None

    @property
    def clip_path(self) -> Path:
        return self._assets_directory / "clip.h264"

    @property
    def checksum_path(self) -> Path:
        return self._assets_directory / "clip.sha256"

    def verify_clip(self) -> None:
        if not self.clip_path.is_file():
            raise FileNotFoundError(
                f"Missing {self.clip_path}; run {self._assets_directory / 'build_clip.sh'} on the topside computer."
            )
        expected = self.checksum_path.read_text(encoding="ascii").strip()
        digest = hashlib.sha256(self.clip_path.read_bytes()).hexdigest()
        if digest != expected:
            raise ValueError(f"Clip checksum mismatch: expected {expected}, got {digest}")

    def local_ip_for_device(self, device_host: str) -> str:
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as datagram_socket:
            datagram_socket.connect((device_host, 80))
            return datagram_socket.getsockname()[0]

    def rtsp_url(self, device_host: str) -> str:
        topside_address = self.local_ip_for_device(device_host)
        return f"rtsp://{topside_address}:{self._port}/{self._mount_name}"

    def start(self, device_host: str) -> str:
        self.verify_clip()
        if self._process is not None:
            raise RuntimeError("RTSP source is already running")

        rtsp_url = self.rtsp_url(device_host)
        server_script = self._assets_directory / "rtsp_server.sh"
        command = [str(server_script), str(self.clip_path), rtsp_url]
        # The server outlives this call and is stopped by stop().
        self._process = subprocess.Popen(command)  # pylint: disable=consider-using-with
        return rtsp_url

    def stop(self) -> None:
        if self._process is None:
            return
        self._process.terminate()
        try:
            self._process.wait(timeout=10.0)
        except subprocess.TimeoutExpired:
            self._process.kill()
            self._process.wait(timeout=5.0)
        self._process = None

    def ensure_tools(self) -> None:
        if shutil.which("test-launch") is None and shutil.which("ffmpeg") is None:
            raise RuntimeError("Install gst-rtsp-server (test-launch) or ffmpeg on the topside computer.")
