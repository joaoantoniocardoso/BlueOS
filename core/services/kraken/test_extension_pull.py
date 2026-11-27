# isort: skip_file
import asyncio
import sys
import time
from pathlib import Path
from typing import Any, AsyncGenerator, Optional
from unittest import mock

import pytest

_KRAKEN_DIR = str(Path(__file__).resolve().parent)
sys.path.insert(0, _KRAKEN_DIR)
# Root pytest also collects the other services, whose top-level `config` and
# `settings` modules would otherwise be the ones imported here.
_saved = {module_name: sys.modules.get(module_name) for module_name in ("config", "settings", "utils")}
for module_name in _saved:
    sys.modules.pop(module_name, None)

import extension.extension as extension_module  # noqa: E402
from aiodocker.exceptions import DockerError  # noqa: E402
from extension.exceptions import ExtensionPullFailed  # noqa: E402
from extension.extension import Extension  # noqa: E402
from extension.models import ExtensionInstallSource  # noqa: E402

for module_name, module in _saved.items():
    if module is not None:
        sys.modules[module_name] = module
    else:
        sys.modules.pop(module_name, None)

pytestmark = pytest.mark.asyncio


def make_extension() -> Extension:
    source = ExtensionInstallSource(
        identifier="test.ext", tag="1.0", name="Test", docker="test/ext", enabled=True, permissions="{}"
    )
    return Extension(source)


def stalled_pull(extension: Extension) -> Any:
    async def pull(_auth: Optional[str]) -> AsyncGenerator[bytes, None]:
        yield b"first"
        await asyncio.sleep(5)
        yield b"never"

    return mock.patch.multiple(
        extension,
        _pull_docker_image=pull,
        _switch_from=mock.AsyncMock(return_value=None),
        _sibling_image_id=mock.AsyncMock(return_value=None),
        _can_use_local_image=mock.AsyncMock(return_value=False),
        lock=mock.Mock(),
        unlock=mock.Mock(),
        reset_start_attempt=mock.Mock(),
    )


async def test_install_fails_when_the_pull_goes_silent() -> None:
    extension = make_extension()
    lines = []
    with stalled_pull(extension), mock.patch.object(extension_module, "PULL_IDLE_TIMEOUT_SECONDS", 0.1, create=True):
        with pytest.raises(ExtensionPullFailed):
            async for line in extension.install():
                lines.append(line)
        assert lines == [b"first"]
        extension._switch_from.assert_not_awaited()  # type: ignore


async def test_install_keeps_going_while_the_pull_has_progress() -> None:
    extension = make_extension()

    async def pull(_auth: Optional[str], _docker: Any = None) -> AsyncGenerator[bytes, None]:
        for index in range(4):
            await asyncio.sleep(0.05)
            yield str(index).encode()

    with stalled_pull(extension), mock.patch.object(extension_module, "PULL_IDLE_TIMEOUT_SECONDS", 0.12, create=True):
        extension._pull_docker_image = pull  # type: ignore
        extension._clear_remaining_tags = mock.AsyncMock()  # type: ignore
        assert [line async for line in extension.install()] == [b"0", b"1", b"2", b"3"]


async def test_start_gives_up_on_a_pull_that_goes_silent() -> None:
    extension = make_extension()
    client = mock.AsyncMock()
    client.images.inspect.side_effect = DockerError(404, {"message": "no such image"})  # type: ignore

    async def stalled_stream() -> AsyncGenerator[dict[str, str], None]:
        yield {"status": "Pulling"}
        await asyncio.sleep(5)
        yield {"status": "never"}

    async def stalled_pull_call() -> list[dict[str, str]]:
        await asyncio.sleep(5)
        return []

    def pull(*_args: Any, stream: bool = False, **_kwargs: Any) -> Any:
        return stalled_stream() if stream else stalled_pull_call()

    client.images.pull = pull
    docker_ctx = mock.MagicMock()
    docker_ctx.return_value.__aenter__.return_value = client
    with mock.patch.multiple(
        extension,
        lock=mock.Mock(),
        unlock=mock.Mock(),
        mark_start_attempt=mock.Mock(),
        _set_container_config_host_config=mock.Mock(),
        _set_container_config_default_env_variables=mock.Mock(),
    ), mock.patch.object(extension_module, "DockerCtx", docker_ctx), mock.patch.object(
        extension_module, "PULL_IDLE_TIMEOUT_SECONDS", 0.1, create=True
    ), mock.patch.object(
        Extension, "settings", mock.MagicMock(), create=True
    ):
        started = time.monotonic()
        with pytest.raises(ExtensionPullFailed):
            await extension.start()
        assert time.monotonic() - started < 2
