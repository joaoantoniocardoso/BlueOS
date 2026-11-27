# isort: skip_file
import importlib.util
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional
from unittest import mock

import aiohttp
import pytest
from fastapi import HTTPException
from fastapi.routing import APIRoute

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
from manifest.models import RepositoryEntry  # noqa: E402
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
    monkeypatch.setattr(ManifestManager, "_fetch_manifest_data", fetch_manifest_data)
    return ManifestManager.instance()


def set_sources(*urls: str) -> None:
    ManifestManager._settings.manifests = [
        ManifestSettings(
            identifier=f"source-{index}", enabled=True, priority=index, factory=False, name=f"source-{index}", url=url
        )
        for index, url in enumerate(urls)
    ]


async def test_fetch_consolidated_raises_when_every_source_fails(manager: ManifestManager) -> None:
    set_sources(BAD_URL, BAD_URL + "?2")

    with pytest.raises(ManifestBackendOffline):
        await manager.fetch_consolidated()


async def test_fetch_consolidated_keeps_good_source_when_another_fails(manager: ManifestManager) -> None:
    set_sources(BAD_URL, GOOD_URL)

    entries = await manager.fetch_consolidated()

    assert [entry.identifier for entry in entries] == ["good"]


async def test_fetch_without_data_does_not_raise(manager: ManifestManager) -> None:
    set_sources(BAD_URL)

    manifests = await manager.fetch(fetch_data=False)

    assert [manifest.url for manifest in manifests] == [BAD_URL]


def load_router(relative_path: str, extra_modules: Optional[Dict[str, Any]] = None) -> Any:
    # Loading by path skips the `api` package, whose import starts the whole service.
    name = "kraken_router_" + relative_path.replace("/", "_").removesuffix(".py")
    spec = importlib.util.spec_from_file_location(name, _KRAKEN_DIR / relative_path)
    assert spec is not None and spec.loader is not None
    router = importlib.util.module_from_spec(spec)
    # The root test environment installs neither fastapi_versioning nor python-multipart.
    versioning = mock.Mock(versioned_api_route=lambda *_: APIRoute)
    modules = {name: router, "fastapi_versioning": versioning, **(extra_modules or {})}
    with mock.patch.dict(sys.modules, modules), mock.patch("fastapi.dependencies.utils.check_file_field"):
        spec.loader.exec_module(router)
    return router


@pytest.mark.usefixtures("manager")
async def test_consolidated_route_answers_502_when_every_source_fails() -> None:
    set_sources(BAD_URL)
    router = load_router("api/v2/routers/manifest.py")

    with pytest.raises(HTTPException) as error:
        await router.fetch_consolidated()

    assert error.value.status_code == 502


EXTENSION_ROUTES = [
    ("v2", "install_by_identifier", {"identifier": "org.ext"}),
    ("v2", "install_by_identifier_and_tag", {"identifier": "org.ext", "tag": "1.0"}),
    ("v2", "update_to_latest", {"identifier": "org.ext"}),
    ("v2", "update_to_tag", {"identifier": "org.ext", "tag": "1.0"}),
    ("v1", "update_extension", {"extension_identifier": "org.ext", "new_version": "1.0"}),
]


@pytest.mark.usefixtures("manager")
@pytest.mark.parametrize("api_version, route, arguments", EXTENSION_ROUTES)
async def test_extension_routes_answer_502_when_every_source_fails(
    api_version: str, route: str, arguments: Dict[str, str]
) -> None:
    set_sources(BAD_URL)
    v2_router = load_router("api/v2/routers/extension.py")
    router = v2_router
    if api_version == "v1":
        stubs = {name: mock.Mock() for name in ("api", "api.v2", "api.v2.routers")}
        router = load_router("api/v1/routers/extension.py", {**stubs, "api.v2.routers.extension": v2_router})

    with pytest.raises(HTTPException) as error:
        await getattr(router, route)(**arguments)

    assert error.value.status_code == 502
