# isort: skip_file
import importlib.util
import sys
from pathlib import Path
from typing import Any, List

import aiohttp
import pytest

_KRAKEN_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(_KRAKEN_DIR))
# Root pytest also collects the other services, whose top-level `config` and
# `settings` modules would otherwise be the ones imported here.
_saved = {module_name: sys.modules.get(module_name) for module_name in ("config", "settings", "utils")}
for module_name in _saved:
    sys.modules.pop(module_name, None)

from commonwealth.settings.manager import Manager  # noqa: E402
from config import SERVICE_NAME  # noqa: E402
from manifest import ManifestManager  # noqa: E402
from manifest.exceptions import ManifestBackendOffline  # noqa: E402
from manifest.models import RepositoryEntry, UpdateManifestSource  # noqa: E402
from settings import ManifestSettings, SettingsV2  # noqa: E402

importlib.import_module("extension.extension")

for module_name, module in _saved.items():
    if module is not None:
        sys.modules[module_name] = module
    else:
        sys.modules.pop(module_name, None)

pytestmark = pytest.mark.asyncio

GOOD_URL = "https://good.example/manifest.json"
BAD_URL = "https://bad.example/manifest.json"


def make_entry(identifier: str) -> RepositoryEntry:
    return RepositoryEntry(
        identifier=identifier,
        name=identifier,
        website="https://example.com",
        docker=f"example/{identifier}",
        description="",
    )


async def fetch_manifest_data(_self: Any, url: str) -> List[RepositoryEntry]:
    if url == GOOD_URL:
        return [make_entry("good")]
    raise ManifestBackendOffline("Unable to fetch manifest, backend is offline") from aiohttp.ClientConnectionError()


@pytest.fixture(name="manager")
def manager_fixture(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> ManifestManager:
    settings_manager = Manager(SERVICE_NAME, SettingsV2, config_folder=tmp_path)
    monkeypatch.setattr(ManifestManager, "_manager", settings_manager)
    monkeypatch.setattr(ManifestManager, "_settings", settings_manager.settings)
    monkeypatch.setattr(ManifestManager, "_instance", None)
    monkeypatch.setattr(ManifestManager, "_source_errors", {}, raising=False)
    monkeypatch.setattr(ManifestManager, "_fetch_manifest_data", fetch_manifest_data)
    return ManifestManager.instance()


def set_sources(*urls: str) -> None:
    ManifestManager._settings.manifests = [
        ManifestSettings(
            identifier=f"source-{index}",
            enabled=True,
            priority=index,
            factory=False,
            name=f"source-{index}",
            url=url,
        )
        for index, url in enumerate(urls)
    ]


async def test_fetch_reports_the_error_of_a_failed_source_next_to_a_working_one(
    manager: ManifestManager,
) -> None:
    set_sources(BAD_URL, GOOD_URL)

    manifests = await manager.fetch(fetch_data=True)

    failed, working = manifests
    assert failed.data is None
    assert failed.error is not None and "offline" in failed.error
    assert working.error is None
    assert working.data is not None and [entry.identifier for entry in working.data] == ["good"]


async def test_fetch_without_data_sets_no_error_for_a_source_never_fetched(
    manager: ManifestManager,
) -> None:
    set_sources(BAD_URL)

    manifests = await manager.fetch(fetch_data=False)

    assert manifests[0].error is None


async def test_listing_without_data_shows_the_error_of_the_last_failed_fetch(
    manager: ManifestManager,
) -> None:
    set_sources(BAD_URL, GOOD_URL)
    await manager.fetch(fetch_data=True)

    failed, working = await manager.fetch(fetch_data=False)

    assert failed.error is not None and "offline" in failed.error
    assert working.error is None


async def test_a_later_good_fetch_clears_the_error(manager: ManifestManager) -> None:
    set_sources(BAD_URL)
    with pytest.raises(ManifestBackendOffline):
        await manager.fetch(fetch_data=True)
    set_sources(GOOD_URL)
    manager._source_errors["source-0"] = "old"

    await manager.fetch(fetch_data=True)

    assert (await manager.fetch(fetch_data=False))[0].error is None


async def test_changing_a_source_forgets_its_error(manager: ManifestManager) -> None:
    set_sources(BAD_URL)
    with pytest.raises(ManifestBackendOffline):
        await manager.fetch(fetch_data=True)

    await manager.update_source("source-0", UpdateManifestSource(url=GOOD_URL), validate_url=False)

    assert (await manager.fetch(fetch_data=False))[0].error is None
