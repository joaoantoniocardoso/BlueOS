# isort: skip_file
import sys
from pathlib import Path
from typing import Iterator
from unittest import mock

import pytest

_KRAKEN_DIR = str(Path(__file__).resolve().parent)
sys.path.insert(0, _KRAKEN_DIR)
_saved = {module_name: sys.modules.get(module_name) for module_name in ("config", "settings", "utils")}
for module_name in _saved:
    sys.modules.pop(module_name, None)

from extension import extension as extension_module  # noqa: E402
from extension.extension import Extension  # noqa: E402
from extension.models import ExtensionSource  # noqa: E402
from harbor import ContainerManager  # noqa: E402
from kraken import Kraken  # noqa: E402
from settings import ExtensionSettings  # noqa: E402

for module_name, module in _saved.items():
    if module is not None:
        sys.modules[module_name] = module
    else:
        sys.modules.pop(module_name, None)

pytestmark = pytest.mark.asyncio

NOW = 100_000
STABLE_RUN_SECONDS = 60


def make_settings() -> ExtensionSettings:
    return ExtensionSettings(
        identifier="test.crashy",
        name="crashy",
        docker="test/crashy",
        tag="1.0",
        permissions="{}",
        enabled=True,
        user_permissions="{}",
    )


KEY = "test.crashy1.0"


@pytest.fixture(autouse=True)
def clock() -> Iterator[None]:
    Extension.start_attempts.clear()
    Extension.locked_entries.clear()
    with mock.patch("time.monotonic", side_effect=lambda: NOW):
        yield
    Extension.start_attempts.clear()


async def run_starter(settings: ExtensionSettings, is_running: bool) -> None:
    container = mock.Mock()
    container.name = f"/{settings.container_name()}"
    kraken = object.__new__(Kraken)
    kraken.manifest = mock.Mock(fetch_extension_version=mock.AsyncMock(return_value=None))
    with mock.patch.object(
        ContainerManager, "get_running_containers", return_value=[container] if is_running else []
    ), mock.patch.object(Extension, "_fetch_settings", return_value=[settings]), mock.patch.object(
        Extension, "start", mock.AsyncMock()
    ):
        await kraken.init_dead_extensions()


async def test_good_start_keeps_the_attempt_count() -> None:
    settings = make_settings()
    client = mock.AsyncMock()
    client.images.inspect.return_value = {"Id": "sha256:abc"}
    docker_ctx = mock.MagicMock()
    docker_ctx.return_value.__aenter__.return_value = client
    with mock.patch.object(extension_module, "DockerCtx", docker_ctx), mock.patch.object(
        Extension, "settings", settings
    ), mock.patch.object(Extension, "_start_or_create", mock.AsyncMock()):
        await Extension(ExtensionSource.from_settings(settings)).start()

    assert Extension.start_attempts[KEY] == (1, NOW)
    # The container died again: the next check must wait before it starts it
    assert not object.__new__(Kraken)._extension_start_try_valid(settings)


async def test_delay_grows_and_is_capped() -> None:
    settings = make_settings()
    kraken = object.__new__(Kraken)
    for attempts, age, valid in [(1, 11, False), (1, 13, True), (10, 599, False), (10, 601, True)]:
        Extension.start_attempts[KEY] = (attempts, NOW - age)
        assert kraken._extension_start_try_valid(settings) is valid, (attempts, age)


async def test_attempts_reset_after_a_stable_run() -> None:
    Extension.start_attempts[KEY] = (3, NOW - STABLE_RUN_SECONDS - 1)
    await run_starter(make_settings(), True)
    assert KEY not in Extension.start_attempts


async def test_attempts_kept_while_the_run_is_short() -> None:
    Extension.start_attempts[KEY] = (3, NOW - 5)
    await run_starter(make_settings(), True)
    assert Extension.start_attempts[KEY][0] == 3
