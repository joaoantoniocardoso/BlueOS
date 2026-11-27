import ast
from pathlib import Path

import pytest

KRAKEN_DIR = Path(__file__).resolve().parent

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("version", ["v1", "v2"])
async def test_install_and_update_routes_send_heartbeats(version: str) -> None:
    source = (KRAKEN_DIR / "api" / version / "routers" / "extension.py").read_text()
    calls = [
        node
        for node in ast.walk(ast.parse(source))
        if isinstance(node, ast.Call) and getattr(node.func, "id", None) == "streamer"
    ]
    assert calls
    for call in calls:
        assert "heartbeats" in [keyword.arg for keyword in call.keywords], ast.unparse(call)
