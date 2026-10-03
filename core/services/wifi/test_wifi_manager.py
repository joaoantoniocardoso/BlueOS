import asyncio
import signal
import sys
from pathlib import Path
from types import SimpleNamespace
from typing import Any
from unittest.mock import AsyncMock

import pytest

_WIFI_DIR = str(Path(__file__).resolve().parent)
sys.path.insert(0, _WIFI_DIR)
_saved = {name: sys.modules.get(name) for name in ("exceptions", "settings", "typedefs")}
for name in _saved:
    sys.modules.pop(name, None)

from wifi_handlers.networkmanager.networkmanager import NetworkManagerWifi
from wifi_handlers.wpa_supplicant.WifiManager import WifiManager

for name, module in _saved.items():
    if module is not None:
        sys.modules[name] = module
    else:
        sys.modules.pop(name, None)

pytestmark = pytest.mark.asyncio


async def test_reports_unavailable_without_a_socket() -> None:
    manager = WifiManager.__new__(WifiManager)

    assert (await manager.status()).state == "unavailable"
    assert await manager.get_wifi_available() == []
    assert await manager.get_saved_wifi_network() == []
    assert await manager.supports_hotspot() is False
    assert await manager.hotspot_is_running() is False


async def test_reports_available_after_connecting(monkeypatch: pytest.MonkeyPatch) -> None:
    manager = WifiManager.__new__(WifiManager)
    monkeypatch.setattr(manager.wpa, "run", lambda _target: None)
    monkeypatch.setattr(WifiManager, "get_wifi_available", _no_networks)

    # The udp socket is the fallback when there is no wpa_supplicant socket, it still means wifi works
    await manager.connect(("127.0.0.1", 6664))

    assert manager.wpa_path is not None


# uvicorn re-raises the SIGTERM it captured once it has served. A service handler that swallows it keeps the process
# alive after the server ends, and the non-daemon zenoh callback thread then blocks the interpreter exit forever.
async def test_start_leaves_sigterm_to_the_server(monkeypatch: pytest.MonkeyPatch) -> None:
    manager = NetworkManagerWifi.__new__(NetworkManagerWifi)
    manager._nm = SimpleNamespace(get_devices=AsyncMock(return_value=[]))  # type: ignore[assignment]
    manager._tasks = []
    for method in ("_create_virtual_interface", "_autoscan", "hotspot_watchdog"):
        monkeypatch.setattr(NetworkManagerWifi, method, AsyncMock())
    sigterm_handler = signal.getsignal(signal.SIGTERM)

    try:
        await manager.start()
        assert signal.getsignal(signal.SIGTERM) is sigterm_handler
    finally:
        asyncio.get_running_loop().remove_signal_handler(signal.SIGTERM)
        asyncio.get_running_loop().remove_signal_handler(signal.SIGINT)
        signal.signal(signal.SIGTERM, sigterm_handler)


async def _no_networks(_self: Any) -> list[Any]:
    return []
