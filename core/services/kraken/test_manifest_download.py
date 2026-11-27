# isort: skip_file
# pylint: disable=redefined-outer-name
import asyncio
import json
import sys
from pathlib import Path
from typing import Any, AsyncIterator, Dict, List

import aiohttp
import pytest
from aiohttp import web

_KRAKEN_DIR = str(Path(__file__).resolve().parent)
sys.path.insert(0, _KRAKEN_DIR)
# Root pytest also collects the other services, whose top-level `config` and
# `settings` modules would otherwise be the ones imported here.
_saved = {module_name: sys.modules.get(module_name) for module_name in ("config", "settings", "utils")}
for module_name in _saved:
    sys.modules.pop(module_name, None)

from manifest import manifest as manifest_module  # noqa: E402
from manifest.exceptions import ManifestBackendOffline  # noqa: E402
from manifest.manifest import ManifestManager  # noqa: E402

for module_name, module in _saved.items():
    if module is not None:
        sys.modules[module_name] = module
    else:
        sys.modules.pop(module_name, None)

pytestmark = pytest.mark.asyncio

CATALOG: List[Dict[str, Any]] = []


class CatalogServer:
    def __init__(self) -> None:
        self.requests = 0
        self.release = asyncio.Event()
        self.mode = "answer"

    async def handle(self, _request: web.Request) -> web.StreamResponse:
        self.requests += 1
        if self.mode == "silent":
            await asyncio.sleep(3600)
        if self.mode == "stall_body":
            response = web.StreamResponse(headers={"Content-Type": "application/json"})
            await response.prepare(_request)
            await response.write(b"[")
            await asyncio.sleep(3600)
        if self.mode == "slow_body":
            response = web.StreamResponse(headers={"Content-Type": "application/json"})
            await response.prepare(_request)
            for chunk in (b"[", b" ", b" ", b" ", b" ", b" ", b"]"):
                await response.write(chunk)
                await asyncio.sleep(0.1)
            return response
        await self.release.wait()
        return web.Response(text=json.dumps(CATALOG), content_type="application/json")


@pytest.fixture
async def server() -> AsyncIterator[Any]:
    catalog = CatalogServer()
    app = web.Application()
    app.router.add_get("/manifest.json", catalog.handle)
    runner = web.AppRunner(app)
    await runner.setup()
    site = web.TCPSite(runner, "127.0.0.1", 0)
    await site.start()
    port = site._server.sockets[0].getsockname()[1]  # type: ignore[union-attr]
    catalog.url = f"http://127.0.0.1:{port}/manifest.json"  # type: ignore[attr-defined]
    yield catalog
    await runner.cleanup()


@pytest.fixture
def manager(monkeypatch: pytest.MonkeyPatch) -> ManifestManager:
    monkeypatch.setattr(
        manifest_module,
        "MANIFEST_DOWNLOAD_TIMEOUT",
        aiohttp.ClientTimeout(total=None, sock_connect=0.3, sock_read=0.3),
        raising=False,
    )
    return ManifestManager.__new__(ManifestManager)


async def test_silent_host_raises_offline_with_reason(server: Any, manager: ManifestManager) -> None:
    server.mode = "silent"
    with pytest.raises(ManifestBackendOffline, match="no data for 0.3 s"):
        await asyncio.wait_for(manager._fetch_manifest_data(server.url), timeout=5)


async def test_stalled_body_raises_offline_not_parse_error(server: Any, manager: ManifestManager) -> None:
    server.mode = "stall_body"
    with pytest.raises(ManifestBackendOffline):
        await asyncio.wait_for(manager._fetch_manifest_data(server.url), timeout=5)


async def test_parallel_cache_misses_make_one_request(server: Any, manager: ManifestManager) -> None:
    calls = [asyncio.create_task(manager._fetch_manifest_data(server.url)) for _ in range(5)]
    await asyncio.sleep(0.1)
    server.release.set()
    results = await asyncio.wait_for(asyncio.gather(*calls), timeout=5)
    assert results == [[]] * 5
    assert server.requests == 1


async def test_slow_but_progressing_download_is_not_cut(server: Any, manager: ManifestManager) -> None:
    server.mode = "slow_body"
    start = asyncio.get_running_loop().time()
    assert await manager._fetch_manifest_data(server.url) == []
    assert asyncio.get_running_loop().time() - start > 0.5
