import asyncio
from typing import Any

import aiohttp


class Mavlink2RestClient:
    def __init__(self, device_host: str, session: aiohttp.ClientSession) -> None:
        self._url = f"http://{device_host}/mavlink2rest/mavlink"
        self._session = session

    async def send_arm_command(self, vehicle_system_id: int = 1) -> None:
        message = {
            "header": {"system_id": 255, "component_id": 0, "sequence": 0},
            "message": {
                "type": "COMMAND_LONG",
                "param1": 1,
                "param2": 0,
                "param3": 0,
                "param4": 0,
                "param5": 0,
                "param6": 0,
                "param7": 0,
                "command": {"type": "MAV_CMD_COMPONENT_ARM_DISARM"},
                "target_system": vehicle_system_id,
                "target_component": 1,
                "confirmation": 0,
            },
        }
        for _ in range(5):
            await self._post_message(message)
            if await self._vehicle_is_armed(vehicle_system_id):
                return
            await asyncio.sleep(1.0)
        raise RuntimeError("Vehicle did not arm after repeated attempts")

    async def send_disarm_command(self, vehicle_system_id: int = 1) -> None:
        message = {
            "header": {"system_id": 255, "component_id": 0, "sequence": 0},
            "message": {
                "type": "COMMAND_LONG",
                "param1": 0,
                "param2": 0,
                "param3": 0,
                "param4": 0,
                "param5": 0,
                "param6": 0,
                "param7": 0,
                "command": {"type": "MAV_CMD_COMPONENT_ARM_DISARM"},
                "target_system": vehicle_system_id,
                "target_component": 1,
                "confirmation": 0,
            },
        }
        await self._post_message(message)

    async def send_video_capture_command(
        self,
        start_capture: bool,
        vehicle_system_id: int,
        camera_component_id: int,
    ) -> None:
        message = {
            "header": {"system_id": 255, "component_id": 0, "sequence": 0},
            "message": {
                "type": "COMMAND_LONG",
                "param1": 0,
                "param2": 0,
                "param3": 0,
                "param4": 0,
                "param5": 0,
                "param6": 0,
                "param7": 0,
                "command": {"type": "MAV_CMD_VIDEO_START_CAPTURE" if start_capture else "MAV_CMD_VIDEO_STOP_CAPTURE"},
                "target_system": vehicle_system_id,
                "target_component": camera_component_id,
                "confirmation": 0,
            },
        }
        await self._post_message(message)

    async def _post_message(self, message: dict[str, Any]) -> None:
        async with self._session.post(
            self._url,
            json=message,
            timeout=aiohttp.ClientTimeout(total=5.0),
        ) as response:
            if response.status >= 400:
                body = await response.text()
                raise RuntimeError(f"mavlink2rest POST failed with {response.status}: {body}")

    async def _vehicle_is_armed(self, vehicle_system_id: int) -> bool:
        request_url = f"{self._url}/vehicles/{vehicle_system_id}/components/1/messages/HEARTBEAT"
        async with self._session.get(request_url, timeout=aiohttp.ClientTimeout(total=5.0)) as response:
            if response.status >= 400:
                return False
            payload = await response.json()
            base_mode_bits = payload["message"]["base_mode"]["bits"]
            return bool(base_mode_bits & 128)
