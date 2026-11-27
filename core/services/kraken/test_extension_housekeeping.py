# isort: skip_file
import sys
from pathlib import Path
from typing import Iterator
from unittest import mock

import pytest

_KRAKEN_DIR = str(Path(__file__).resolve().parent)
sys.path.insert(0, _KRAKEN_DIR)
# Root pytest also collects the other services, whose top-level `config`, `settings` and `utils`
# modules would otherwise be the ones imported here.
_saved = {module_name: sys.modules.get(module_name) for module_name in ("config", "settings", "utils")}
for module_name in _saved:
    sys.modules.pop(module_name, None)

from extension.exceptions import ExtensionNotFound  # noqa: E402
from extension.extension import Extension  # noqa: E402

for module_name, module in _saved.items():
    if module is not None:
        sys.modules[module_name] = module
    else:
        sys.modules.pop(module_name, None)


@pytest.fixture(autouse=True)
def clean_activity() -> Iterator[None]:
    Extension.temp_extension_activity.clear()
    yield
    Extension.temp_extension_activity.clear()


def test_keep_alive_uses_monotonic_clock() -> None:
    settings = mock.Mock(identifier="")
    with mock.patch.object(Extension, "_fetch_settings", return_value=settings), mock.patch(
        "extension.extension.time"
    ) as fake_time:
        fake_time.monotonic.return_value = 1000.0
        fake_time.time.return_value = 5_000_000_000.0
        Extension.keep_temporary_extension_alive("temp-tag")

    assert Extension.temp_extension_activity["temp-tag"] == (1000, 1000)


@pytest.mark.asyncio
async def test_cleanup_ignores_wall_clock_step() -> None:
    settings = mock.Mock(identifier="", docker="img", tag="temp-tag")
    Extension.temp_extension_activity["temp-tag"] = (1000, 1000)
    with mock.patch.object(Extension, "_fetch_settings", return_value=[settings]), mock.patch.object(
        Extension, "_cleanup_orphan_temp_tags", new=mock.AsyncMock()
    ), mock.patch("extension.extension.ExtensionSource"), mock.patch.object(
        Extension, "uninstall", new=mock.AsyncMock()
    ) as uninstall, mock.patch(
        "extension.extension.time"
    ) as fake_time:
        fake_time.monotonic.return_value = 1001.0
        fake_time.time.return_value = 5_000_000_000.0
        await Extension.cleanup_temporary_extensions()

    uninstall.assert_not_awaited()


@pytest.mark.asyncio
@pytest.mark.parametrize("stable, word", [(True, "stable versions"), (False, "versions")])
async def test_from_latest_message_has_spaces(stable: bool, word: str) -> None:
    manager = mock.Mock()
    manager.select_extension = mock.AsyncMock(return_value=object())
    manager.versions_from_entry = mock.Mock(return_value=[])
    with mock.patch("extension.extension.ManifestManager.instance", return_value=manager):
        with pytest.raises(ExtensionNotFound) as error:
            await Extension.from_latest("id", stable)

    assert str(error.value) == f"Extension id has no {word}"
