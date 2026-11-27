# isort: skip_file
import importlib.util
import os
import sys
import tempfile
from pathlib import Path
from types import SimpleNamespace
from typing import Any, Awaitable, Callable, Dict, List, cast
from unittest import mock

import pytest

_KRAKEN_DIR = str(Path(__file__).resolve().parent)
sys.path.insert(0, _KRAKEN_DIR)
# Kraken writes its settings to the user config dir when `ManifestManager.instance()` runs at import time.
_saved_env = {name: os.environ.get(name) for name in ("HOME", "XDG_CONFIG_HOME")}
os.environ["HOME"] = os.environ["XDG_CONFIG_HOME"] = tempfile.mkdtemp()
# Root pytest also collects the other services, whose top-level `config` and
# `settings` modules would otherwise be the ones imported here.
_saved = {module_name: sys.modules.get(module_name) for module_name in ("config", "settings", "utils")}
for module_name in _saved:
    sys.modules.pop(module_name, None)

_stubbed: List[Any] = []
# Another test module may already have put a stub without a spec in `sys.modules`, and `find_spec` refuses that.
_loaded = sys.modules.get("fastapi_versioning")
if (
    (getattr(_loaded, "__spec__", None) is None)
    if _loaded is not None
    else importlib.util.find_spec("fastapi_versioning") is None
):
    # Not a dev dependency: the routes only need `versioned_api_route` to give a route class.
    import types
    import fastapi.dependencies.utils as fastapi_utils
    from fastapi.routing import APIRoute

    stub = types.ModuleType("fastapi_versioning")
    stub.versioned_api_route = lambda *_args, **_kwargs: APIRoute  # type: ignore[attr-defined]
    stub.VersionedFastAPI = mock.MagicMock()  # type: ignore[attr-defined]
    sys.modules["fastapi_versioning"] = stub
    # The upload route declares a `File` field, which refuses to load without python-multipart.
    _stubbed = [fastapi_utils, fastapi_utils.check_file_field]
    fastapi_utils.check_file_field = lambda _field: None

from fastapi import HTTPException  # noqa: E402
from api.v1.routers import extension as v1_extension  # noqa: E402
from api.v1.routers import index as v1_index  # noqa: E402
from api.v2.routers import extension as v2_extension  # noqa: E402
from api.v2.routers import manifest as v2_manifest  # noqa: E402
from extension.exceptions import ExtensionNotFound  # noqa: E402
from extension.extension import Extension  # noqa: E402
from manifest import ManifestManager  # noqa: E402
from manifest.exceptions import ExtensionSourceConflict, ManifestNotFound  # noqa: E402
from manifest.models import ExtensionVersion, RepositoryEntry  # noqa: E402
from settings import ExtensionSettings, ManifestSettings  # noqa: E402

for env_name, env_value in _saved_env.items():
    if env_value is None:
        os.environ.pop(env_name, None)
    else:
        os.environ[env_name] = env_value
# Other services have their own top-level `api` package.
for module_name in [name for name in sys.modules if name == "api" or name.startswith("api.")]:
    del sys.modules[module_name]
if _stubbed:
    sys.modules.pop("fastapi_versioning", None)
    _stubbed[0].check_file_field = _stubbed[1]
for module_name, module in _saved.items():
    if module is not None:
        sys.modules[module_name] = module
    else:
        sys.modules.pop(module_name, None)

pytestmark = pytest.mark.asyncio

IDENTIFIER = "org.ext"
GOOD = "org/ext"
OTHER = "evil/ext"


def entry(docker: str, tags: List[str], identifier: str = IDENTIFIER) -> RepositoryEntry:
    versions = {
        tag: ExtensionVersion(type="example", images=[], authors=[], filter_tags=[], extra_links={}, tag=tag)
        for tag in tags
    }
    return RepositoryEntry(
        identifier=identifier, name=f"Ext {docker}", website="", docker=docker, description="", versions=versions
    )


class World:
    def __init__(self, monkeypatch: pytest.MonkeyPatch) -> None:
        self.monkeypatch = monkeypatch
        self.data: Dict[str, List[RepositoryEntry]] = {}
        self.sources: List[ManifestSettings] = []
        self.installed: List[ExtensionSettings] = []

        manager = ManifestManager.__new__(ManifestManager)
        monkeypatch.setattr(ManifestManager, "_instance", manager)
        monkeypatch.setattr(ManifestManager, "_settings", SimpleNamespace(manifests=self.sources))
        monkeypatch.setattr(ManifestManager, "_manager", mock.Mock())
        monkeypatch.setattr(ManifestManager, "_fetch_manifest_data", self._fetch_data)
        monkeypatch.setattr(Extension, "_settings", SimpleNamespace(extensions=self.installed))
        monkeypatch.setattr(v2_manifest, "manifest_manager", manager)
        monkeypatch.setattr(v1_index, "manifest_manager", manager)

    async def _fetch_data(self, url: str) -> List[RepositoryEntry]:
        return self.data[url]

    def add_source(self, identifier: str, entries: List[RepositoryEntry], enabled: bool = True) -> None:
        self.data[identifier] = entries
        self.sources.append(
            ManifestSettings(
                identifier=identifier,
                enabled=enabled,
                priority=len(self.sources),
                factory=False,
                name=f"Source {identifier}",
                url=identifier,
            )
        )

    def install(self, docker: str, tag: str = "1.0.0") -> None:
        self.installed.append(
            ExtensionSettings(
                identifier=IDENTIFIER, name="Ext", docker=docker, tag=tag, permissions="{}", enabled=True,
                user_permissions="",
            )
        )  # fmt: skip


@pytest.fixture(name="world")
def world_fixture(monkeypatch: pytest.MonkeyPatch) -> World:
    monkeypatch.setattr(Extension, "get_compatible_digest", lambda *_args, **_kwargs: "sha256:" + "a" * 64)
    return World(monkeypatch)


async def latest(identifier: str, **kwargs: Any) -> Extension:
    return await Extension.from_latest(identifier, **kwargs)


async def from_tag(identifier: str, tag: str, **kwargs: Any) -> Extension:
    return cast(Extension, await Extension.from_manifest(identifier, tag, **kwargs))


def two_repos(world: World) -> None:
    world.add_source("m1", [entry(OTHER, ["1.5.0", "2.0.0"])])
    world.add_source("m2", [entry(GOOD, ["1.0.0", "1.5.0"])])


async def test_same_repo_in_two_sources_is_not_a_conflict(world: World) -> None:
    world.add_source("m1", [entry(GOOD, ["1.0.0"])])
    world.add_source("m2", [entry(GOOD, ["1.0.0", "1.5.0"])])
    world.install(GOOD)

    extension = await latest(IDENTIFIER)
    assert extension.source.docker == GOOD
    consolidated = await ManifestManager.instance().fetch_consolidated()
    assert [item.conflicts for item in consolidated] == [[]]


async def test_two_repos_follow_priority_by_default(world: World) -> None:
    two_repos(world)

    assert (await latest(IDENTIFIER)).source.docker == OTHER
    assert (await from_tag(IDENTIFIER, "2.0.0")).source.docker == OTHER


async def test_tag_only_in_the_second_source_uses_that_source(world: World) -> None:
    two_repos(world)

    extension = await from_tag(IDENTIFIER, "1.0.0")
    assert extension.source.docker == GOOD
    assert extension.source.tag == "1.0.0"


async def test_installed_repo_is_kept_against_a_higher_priority_repo(world: World) -> None:
    two_repos(world)
    world.install(GOOD)

    assert (await latest(IDENTIFIER)).source.docker == GOOD
    assert (await from_tag(IDENTIFIER, "1.5.0")).source.docker == GOOD


async def test_installed_repo_missing_from_all_candidates_raises(world: World) -> None:
    two_repos(world)
    world.install("old/ext")

    with pytest.raises(ExtensionSourceConflict) as error:
        await latest(IDENTIFIER)
    assert error.value.installed_docker == "old/ext"
    assert [(item["manifest"], item["docker"]) for item in error.value.candidates] == [("m1", OTHER), ("m2", GOOD)]


async def test_single_source_that_moved_the_repo_is_not_a_conflict(world: World) -> None:
    world.add_source("m1", [entry(OTHER, ["2.0.0"])])
    world.install("old/ext")

    assert (await latest(IDENTIFIER)).source.docker == OTHER


async def test_manifest_identifier_forces_the_source(world: World) -> None:
    two_repos(world)
    world.install("old/ext")

    extension = await latest(IDENTIFIER, manifest_id="m2")
    assert extension.source.docker == GOOD
    assert extension.source.tag == "1.5.0"
    assert (await from_tag(IDENTIFIER, "1.0.0", manifest_id="m2")).source.docker == GOOD


async def test_manifest_identifier_can_switch_the_installed_repo(world: World) -> None:
    two_repos(world)
    world.install(GOOD)

    assert (await latest(IDENTIFIER, manifest_id="m1")).source.docker == OTHER


async def test_unknown_manifest_identifier_is_not_found(world: World) -> None:
    two_repos(world)

    with pytest.raises(ManifestNotFound):
        await latest(IDENTIFIER, manifest_id="nope")


async def test_manifest_without_the_extension_is_not_found(world: World) -> None:
    two_repos(world)
    world.add_source("m3", [entry("a/b", ["1.0.0"], identifier="other.ext")])

    with pytest.raises(ExtensionNotFound):
        await latest(IDENTIFIER, manifest_id="m3")


async def test_disabled_source_is_not_a_candidate(world: World) -> None:
    world.add_source("m1", [entry(OTHER, ["2.0.0"])], enabled=False)
    world.add_source("m2", [entry(GOOD, ["1.0.0"])])
    world.install("old/ext")

    assert (await latest(IDENTIFIER)).source.docker == GOOD


async def test_consolidated_lists_the_other_repos_as_conflicts(world: World) -> None:
    two_repos(world)
    world.add_source("m3", [entry(GOOD, ["1.0.0"]), entry("a/b", ["1.0.0"], identifier="plain.ext")])

    consolidated = await ManifestManager.instance().fetch_consolidated()

    by_id = {item.identifier: item for item in consolidated}
    assert len(consolidated) == 2
    assert by_id[IDENTIFIER].docker == OTHER
    assert (by_id[IDENTIFIER].manifest, by_id[IDENTIFIER].manifest_name) == ("m1", "Source m1")
    assert (by_id["plain.ext"].manifest, by_id["plain.ext"].manifest_name) == ("m3", "Source m3")
    assert [(c.manifest, c.manifest_name, c.docker, c.name, c.tags) for c in by_id[IDENTIFIER].conflicts] == [
        ("m2", "Source m2", GOOD, f"Ext {GOOD}", ["1.0.0", "1.5.0"])
    ]
    assert by_id["plain.ext"].conflicts == []


@pytest.fixture(autouse=True)
def no_real_install(world: World) -> None:
    async def fake_install(*_args: Any, **_kwargs: Any) -> Any:
        yield b"done"

    world.monkeypatch.setattr(Extension, "install", fake_install)
    world.monkeypatch.setattr(Extension, "update", fake_install)


# The routes are called as functions: Starlette's TestClient needs httpx, which kraken does not depend on.
ROUTES: Dict[str, Callable[..., Awaitable[Any]]] = {
    "v2 install latest": lambda **kw: v2_extension.install_by_identifier(IDENTIFIER, **kw),
    "v2 install tag": lambda **kw: v2_extension.install_by_identifier_and_tag(IDENTIFIER, "1.5.0", **kw),
    "v2 update latest": lambda **kw: v2_extension.update_to_latest(IDENTIFIER, **kw),
    "v2 update tag": lambda **kw: v2_extension.update_to_tag(IDENTIFIER, "1.5.0", **kw),
    "v1 update_to_version": lambda **kw: v1_extension.update_extension(IDENTIFIER, "1.5.0", **kw),
}


@pytest.mark.parametrize("route", ROUTES)
async def test_routes_answer_409_with_the_candidates(world: World, route: str) -> None:
    two_repos(world)
    world.install("old/ext")

    with pytest.raises(HTTPException) as error:
        await ROUTES[route]()

    assert error.value.status_code == 409
    detail: Any = error.value.detail
    assert detail["code"] == "extension_source_conflict"
    assert detail["identifier"] == IDENTIFIER
    assert detail["installed_docker"] == "old/ext"
    assert {c["manifest"] for c in detail["candidates"]} == {"m1", "m2"}
    assert set(detail["candidates"][0]) == {"manifest", "manifest_name", "docker", "name", "tags"}


@pytest.mark.parametrize("route", ROUTES)
async def test_routes_accept_manifest_identifier(world: World, route: str) -> None:
    two_repos(world)
    world.install("old/ext")

    await ROUTES[route](manifest_identifier="m2")
    with pytest.raises(HTTPException) as error:
        await ROUTES[route](manifest_identifier="nope")
    assert error.value.status_code == 404


async def test_routes_without_conflict_keep_working(world: World) -> None:
    two_repos(world)
    world.install(GOOD)

    await v2_extension.update_to_latest(IDENTIFIER)
    await v2_extension.install_by_identifier(IDENTIFIER)


async def test_consolidated_routes_expose_conflicts(world: World) -> None:
    two_repos(world)

    for route in (v2_manifest.fetch_consolidated, v1_index.fetch_manifest):
        body = await route()
        assert len(body) == 1
        assert [c.manifest for c in body[0].conflicts] == ["m2"]
        assert (body[0].docker, body[0].manifest, body[0].manifest_name) == (OTHER, "m1", "Source m1")
