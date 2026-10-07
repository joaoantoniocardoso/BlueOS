from typing import Any

import aiohttp


class HttpDeviceClient:
    def __init__(self, device_host: str, session: aiohttp.ClientSession) -> None:
        self._base_url = f"http://{device_host}"
        self._session = session

    async def get_json(self, path: str) -> Any:
        url = f"{self._base_url}{path}"
        async with self._session.get(url, timeout=aiohttp.ClientTimeout(total=60.0)) as response:
            if response.status >= 400:
                body = await response.text()
                raise RuntimeError(f"GET {path} failed with {response.status}: {body}")
            return await response.json()

    async def post_json(self, path: str, payload: dict[str, Any] | None = None) -> None:
        url = f"{self._base_url}{path}"
        async with self._session.post(url, json=payload, timeout=aiohttp.ClientTimeout(total=120.0)) as response:
            if response.status >= 400:
                body = await response.text()
                raise RuntimeError(f"POST {path} failed with {response.status}: {body}")

    async def delete(self, path: str, params: dict[str, str] | None = None) -> None:
        url = f"{self._base_url}{path}"
        async with self._session.delete(url, params=params, timeout=aiohttp.ClientTimeout(total=60.0)) as response:
            if response.status >= 400:
                body = await response.text()
                raise RuntimeError(f"DELETE {path} failed with {response.status}: {body}")
