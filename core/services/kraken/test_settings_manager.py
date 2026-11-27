import json
from typing import Any, Dict
from unittest import mock

import pytest

from extension.extension import (  # type: ignore[attr-defined]
    Extension,
    ExtensionSettings,
)
from manifest import ManifestManager
from manifest.models import ManifestSource

pytestmark = pytest.mark.asyncio


def saved_file() -> Dict[str, Any]:
    return json.loads(Extension._manager.settings_file_path().read_text(encoding="utf-8"))  # type: ignore[no-any-return]


@pytest.fixture(autouse=True)
def clean_settings() -> None:
    Extension._settings.extensions = []
    Extension._manager.save()
    ManifestManager.instance()
    ManifestManager._settings.manifests = [m for m in ManifestManager._settings.manifests if m.factory]
    ManifestManager._manager.save()


async def add_custom_manifest() -> None:
    manifest = ManifestManager.instance()
    with mock.patch.object(ManifestManager, "_fetch_manifest", new=mock.AsyncMock()):
        await manifest.add_source(
            ManifestSource(name="custom", url="http://example.invalid/manifest.json", enabled=True), False
        )


def save_extension() -> None:
    Extension._settings.extensions.append(
        ExtensionSettings(identifier="test.ext", name="Test", docker="test/ext", tag="3.18", enabled=True)
    )
    Extension._manager.save()


async def test_adding_a_manifest_source_keeps_installed_extensions() -> None:
    save_extension()
    await add_custom_manifest()

    assert [ext["identifier"] for ext in saved_file()["extensions"]] == ["test.ext"]


async def test_installing_an_extension_keeps_custom_manifest_sources() -> None:
    await add_custom_manifest()
    save_extension()

    assert "custom" in [manifest["name"] for manifest in saved_file()["manifests"]]


def test_settings_share_one_manager() -> None:
    assert Extension._manager is ManifestManager._manager
