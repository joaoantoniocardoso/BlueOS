# isort: skip_file
import sys
from pathlib import Path
from typing import Any, Callable, List

import pytest

_KRAKEN_DIR = str(Path(__file__).resolve().parent)
sys.path.insert(0, _KRAKEN_DIR)
# Root pytest also collects the other services, whose top-level `config` and
# `settings` modules would otherwise be the ones imported here.
_saved = {module_name: sys.modules.get(module_name) for module_name in ("config", "settings", "utils")}
for module_name in _saved:
    sys.modules.pop(module_name, None)

from fastapi import HTTPException  # noqa: E402
from loguru import logger  # noqa: E402
from api.v2.routers.container import container_to_http_exception  # noqa: E402
from api.v2.routers.extension import extension_to_http_exception  # noqa: E402
from api.v2.routers.jobs import jobs_to_http_exception  # noqa: E402
from api.v2.routers.manifest import manifest_to_http_exception  # noqa: E402

for module_name, module in _saved.items():
    if module is not None:
        sys.modules[module_name] = module
    else:
        sys.modules.pop(module_name, None)

pytestmark = pytest.mark.asyncio

WRAPPERS = [
    container_to_http_exception,
    extension_to_http_exception,
    jobs_to_http_exception,
    manifest_to_http_exception,
]


@pytest.mark.parametrize("wrapper", WRAPPERS)
async def test_unexpected_error_is_logged_with_traceback_and_answers_500(wrapper: Callable[..., Any]) -> None:
    @wrapper
    async def failing_route() -> None:
        raise RuntimeError("boom")

    records: List[Any] = []
    sink_id = logger.add(records.append, level="ERROR")
    try:
        with pytest.raises(HTTPException) as raised:
            await failing_route()
    finally:
        logger.remove(sink_id)

    assert raised.value.status_code == 500
    assert raised.value.detail == "boom"
    assert len(records) == 1
    assert "failing_route" in records[0].record["message"]
    assert records[0].record["exception"].type is RuntimeError
