from load_scenario.clients.http_device import HttpDeviceClient


class ArdupilotClient:
    def __init__(self, http_client: HttpDeviceClient) -> None:
        self._http_client = http_client

    async def ensure_sitl_running(self) -> None:
        available_boards = await self._http_client.get_json("/ardupilot-manager/v1.0/available_boards")
        sitl_board = next(board for board in available_boards if board.get("name") == "SITL")
        await self._http_client.post_json("/ardupilot-manager/v1.0/board", sitl_board)
        await self._http_client.post_json("/ardupilot-manager/v1.0/start")
