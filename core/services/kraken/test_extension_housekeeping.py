# isort: skip_file
import sys
from pathlib import Path
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


@pytest.mark.asyncio
@pytest.mark.parametrize("stable, word", [(True, "stable versions"), (False, "versions")])
async def test_from_latest_message_has_spaces(stable: bool, word: str) -> None:
    manager = mock.Mock()
    manager.fetch_extension = mock.AsyncMock(return_value=object())
    manager.fetch_latest_extension_version = mock.AsyncMock(return_value=None)
    with mock.patch("extension.extension.ManifestManager.instance", return_value=manager):
        with pytest.raises(ExtensionNotFound) as error:
            await Extension.from_latest("id", stable)

    assert str(error.value) == f"Extension id has no {word}"
