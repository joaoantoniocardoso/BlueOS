import ast
from pathlib import Path

import pytest

SERVICES_PATH = Path(__file__).parents[6] / "services"
SERVICE_MAIN_FILES = sorted(SERVICES_PATH.glob("*/main.py"))


def uvicorn_configs_without_graceful_shutdown(main_file: Path) -> list[int]:
    return [
        node.lineno
        for node in ast.walk(ast.parse(main_file.read_text(encoding="utf-8")))
        if isinstance(node, ast.Call)
        and isinstance(node.func, ast.Name)
        and node.func.id == "Config"
        and "timeout_graceful_shutdown" not in {keyword.arg for keyword in node.keywords}
    ]


def test_services_were_found() -> None:
    assert SERVICE_MAIN_FILES


# Without it, uvicorn waits for open keep-alive and websocket connections after SIGTERM, which holds the whole
# container stop until Docker kills it
@pytest.mark.parametrize("main_file", SERVICE_MAIN_FILES, ids=lambda main_file: main_file.parent.name)
def test_service_bounds_uvicorn_graceful_shutdown(main_file: Path) -> None:
    assert uvicorn_configs_without_graceful_shutdown(main_file) == []
