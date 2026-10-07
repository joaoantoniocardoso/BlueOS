from load_scenario.clients.http_device import HttpDeviceClient


class MavlinkCameraClient:
    def __init__(self, http_client: HttpDeviceClient, stream_name: str = "load-scenario-redirect") -> None:
        self._http_client = http_client
        self._stream_name = stream_name

    async def create_redirect_stream(self, rtsp_endpoint: str) -> None:
        payload = {
            "name": self._stream_name,
            "source": "Redirect",
            "stream_information": {
                "endpoints": [rtsp_endpoint],
                "configuration": {
                    "type": "Redirect",
                },
                "extended_configuration": {
                    "thermal": False,
                    "disable_lazy": False,
                    "disable_mavlink": False,
                    "disable_thumbnails": True,
                    "disable_zenoh": False,
                },
            },
        }
        await self._http_client.post_json("/mavlink-camera-manager/streams", payload)

    async def delete_stream(self) -> None:
        await self._http_client.delete(
            "/mavlink-camera-manager/delete_stream",
            params={"name": self._stream_name},
        )

    async def camera_component_id(self) -> int:
        streams = await self._http_client.get_json("/mavlink-camera-manager/streams")
        for stream in streams:
            if stream.get("name") != self._stream_name:
                continue
            component_id = stream.get("mavlink_component_id")
            if component_id is None:
                break
            return int(component_id)
        raise RuntimeError(f"Stream {self._stream_name} was not found in MAVLink Camera Manager")
