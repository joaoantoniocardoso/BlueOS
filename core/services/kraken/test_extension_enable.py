# isort: skip_file
import sys
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

import pytest

_KRAKEN_DIR = str(Path(__file__).resolve().parent)
sys.path.insert(0, _KRAKEN_DIR)
# Root pytest also collects the other services, whose top-level `config` and
# `settings` modules would otherwise be the ones imported here.
_saved = {module_name: sys.modules.get(module_name) for module_name in ("config", "settings", "utils")}
for module_name in _saved:
    sys.modules.pop(module_name, None)

from extension.extension import Extension  # noqa: E402
from extension.models import ExtensionSource  # noqa: E402
from settings import ExtensionSettings  # noqa: E402

for module_name, module in _saved.items():
    if module is not None:
        sys.modules[module_name] = module
    else:
        sys.modules.pop(module_name, None)


pytestmark = pytest.mark.asyncio


def make_settings(tag: str, enabled: bool, identifier: str = "bluerobotics.example") -> ExtensionSettings:
    return ExtensionSettings(
        identifier=identifier,
        name="Example",
        docker="bluerobotics/example",
        tag=tag,
        permissions="{}",
        enabled=enabled,
        user_permissions="",
    )


def make_extension(tag: str) -> Extension:
    return Extension(
        ExtensionSource(
            identifier="bluerobotics.example",
            tag=tag,
            name="Example",
            docker="bluerobotics/example",
            enabled=False,
            permissions="{}",
        )
    )


async def test_enable_disables_other_tags_and_stops_their_containers() -> None:
    entries = [
        make_settings("1.0.0", enabled=True),
        make_settings("2.0.0", enabled=False),
        make_settings("1.0.0", enabled=True, identifier="bluerobotics.other"),
    ]
    other_identifier_entry = entries[2]
    with mock.patch.object(Extension, "_settings", SimpleNamespace(extensions=entries)), mock.patch.object(
        Extension, "_manager"
    ), mock.patch.object(Extension, "remove", new_callable=mock.AsyncMock) as remove:
        await make_extension("2.0.0").enable()

    enabled = {(ext.identifier, ext.tag): ext.enabled for ext in entries}
    assert enabled == {
        ("bluerobotics.example", "1.0.0"): False,
        ("bluerobotics.example", "2.0.0"): True,
        ("bluerobotics.other", "1.0.0"): True,
    }
    assert other_identifier_entry.enabled
    remove.assert_awaited_once_with(make_settings("1.0.0", False).container_name(), False)
