import uuid

import zenoh
from commonwealth.utils import blueos_idl


class RecorderClient:
    def __init__(self, device_host: str) -> None:
        self._device_host = device_host
        self._session: zenoh.Session | None = None

    def connect(self) -> None:
        config = zenoh.Config()
        config.insert_json5("mode", '"client"')
        config.insert_json5("connect/endpoints", f'["tcp/{self._device_host}:7447"]')
        self._session = zenoh.open(config)
        blueos_idl.ensure_idl_loaded()

    def close(self) -> None:
        if self._session is not None:
            self._session.close()
            self._session = None

    def start_recording(self) -> None:
        if self._session is None:
            raise RuntimeError("Recorder client is not connected")

        goal = {"rotate_if_active": False}
        schema_name = "blueos_recorder_msgs/action/StartRecording_Goal"
        payload = blueos_idl.encode(schema_name, goal)
        encoding = blueos_idl.cdr_encoding(schema_name)
        command = blueos_idl.command_key("recorder", "Start")
        job_id = str(uuid.uuid4())

        replies = self._session.get(
            command,
            payload=payload,
            encoding=encoding,
            attachment=job_id.encode("utf-8"),
            timeout=30.0,
        )
        for reply in replies:
            if reply.ok is None:
                continue
            ack = blueos_idl.decode("blueos_msgs/msg/CommandAck", bytes(reply.ok.payload))
            if not ack.get("accepted", False):
                reason = ack.get("reason", "")
                raise RuntimeError(f"Recorder Start rejected: {reason}")
            return
        raise RuntimeError("Recorder Start returned no reply")
