# isort: skip_file
import asyncio
import json
import time
from types import SimpleNamespace
from typing import Any, Dict, List, Optional
from unittest import mock

import pytest

from test_extension_install_rollback import (
    DOCKER,
    make_extension,
    make_settings,
)
from aiodocker.exceptions import DockerError  # noqa: E402
from commonwealth.utils.streaming import streamer  # noqa: E402
from extension.exceptions import ExtensionCleanupFailed  # noqa: E402
from extension.extension import Extension  # noqa: E402
from extension.models import PullPolicy  # noqa: E402
from kraken import Kraken  # noqa: E402

pytestmark = pytest.mark.asyncio

REMOVAL_SECONDS = 0.1


class FakeContainer(dict):  # type: ignore
    def __init__(self, engine: "FakeEngine", info: Dict[str, Any]) -> None:
        super().__init__(info)
        self.engine = engine

    async def show(self, **_: Any) -> Dict[str, Any]:
        return {"Config": {"Image": self["Image"]}}

    async def stop(self, **_: Any) -> None:
        self["State"]["Running"] = False

    async def wait(self, **_: Any) -> None:
        return None

    async def start(self) -> None:
        return None

    async def delete(self) -> None:
        name = self["Names"][0][1:]
        if name not in self.engine.containers:
            raise DockerError(404, {"message": f"No such container: {name}"})  # type: ignore
        if name in self.engine.removing:
            self.engine.conflicts.append(name)
            raise DockerError(409, {"message": f"removal of container {name} is already in progress"})  # type: ignore
        if self.engine.delete_error is not None:
            raise self.engine.delete_error
        self.engine.removing.add(name)
        await asyncio.sleep(REMOVAL_SECONDS)
        del self.engine.containers[name]
        self.engine.removing.discard(name)


class FakeEngine:
    """Docker with the semantics that matter here: removing a container takes time, and a second removal gets 409."""

    def __init__(self) -> None:
        self.containers: Dict[str, Dict[str, Any]] = {}
        self.removing: set[str] = set()
        self.conflicts: List[str] = []
        self.delete_error: Optional[DockerError] = None
        self.client = mock.AsyncMock()
        self.client.containers.list = self._list
        self.client.containers.get = self._get
        self.client.containers.create = self._create
        self.client.images.inspect.return_value = {"Id": "id"}

    def add(self, tag: str, running: bool) -> str:
        name = make_settings(tag).container_name()
        self.containers[name] = {
            "Names": [f"/{name}"], "Image": f"{DOCKER}:{tag}", "ImageID": "id", "Status": "Up",
            "Config": {"Labels": {}}, "State": {"Running": running},
        }  # fmt: skip
        return name

    async def _list(self, filters: Dict[str, Any], **options: Any) -> List[FakeContainer]:
        await asyncio.sleep(0)
        found = list(self.containers.items())
        if not options.get("all"):
            found = [(name, info) for name, info in found if info["State"]["Running"]]
        if "name" in filters:
            wanted = next(iter(filters["name"]))
            found = [(name, info) for name, info in found if wanted in name]
        if "status" in filters:
            found = [(name, info) for name, info in found if info["State"]["Running"]]
        return [FakeContainer(self, info) for _, info in found]

    async def _get(self, name: str) -> FakeContainer:
        if name not in self.containers:
            raise DockerError(404, {"message": "no such container"})  # type: ignore
        return FakeContainer(self, self.containers[name])

    async def _create(self, config: Dict[str, Any], name: str) -> FakeContainer:
        self.add_new(name, config)
        return FakeContainer(self, self.containers[name])

    def add_new(self, name: str, config: Dict[str, Any]) -> None:
        self.containers[name] = {
            "Names": [f"/{name}"], "Image": config["Image"], "ImageID": "id", "Status": "Up",
            "Config": {"Labels": config["Labels"]}, "State": {"Running": False},
        }  # fmt: skip


@pytest.fixture(name="world")
def fixture_world() -> Any:
    engine = FakeEngine()
    settings = SimpleNamespace(extensions=[])
    ctx = mock.MagicMock()
    ctx.return_value.__aenter__ = mock.AsyncMock(return_value=engine.client)
    ctx.return_value.__aexit__ = mock.AsyncMock(return_value=False)
    kraken = Kraken.__new__(Kraken)
    kraken.manifest = mock.Mock(fetch_extension_version=mock.AsyncMock(return_value=None))
    with (
        mock.patch("extension.extension.DockerCtx", ctx),
        mock.patch("harbor.container.DockerCtx", ctx),
        mock.patch.object(Extension, "_settings", settings),
        mock.patch.object(Extension, "_manager", mock.Mock()),
        mock.patch.object(Extension, "locked_entries", {}),
        mock.patch.object(Extension, "start_attempts", {}),
        mock.patch.object(Extension, "container_locks", {}, create=True),
    ):
        yield SimpleNamespace(engine=engine, settings=settings, kraken=kraken)


def crashed_before(settings: Any, seconds_ago: int = 30) -> None:
    """The start of this extension failed `seconds_ago` seconds ago, so the starter backs off from then."""
    Extension.start_attempts[f"{settings.identifier}{settings.tag}"] = (1, int(time.monotonic()) - seconds_ago)


async def test_starter_does_not_delete_the_container_an_install_is_removing(world: Any) -> None:
    """Upgrade or downgrade: the old tag is dead and the starter comes back while the install removes it."""
    old = make_settings("1")
    world.settings.extensions = [old]
    world.engine.add("1", running=False)
    world.engine.add("2", running=False)
    crashed_before(old)

    async def slow_manifest(*_: Any) -> None:
        await asyncio.sleep(0.02)

    world.kraken.manifest.fetch_extension_version = slow_manifest

    starter = asyncio.create_task(world.kraken.init_dead_extensions())
    await asyncio.sleep(0)
    install = make_extension("2").install(clear_remaining_tags=False, pull=PullPolicy.NEVER)
    await asyncio.gather(starter, anext_all(install))

    assert world.engine.conflicts == []


async def anext_all(generator: Any) -> None:
    async for _ in generator:
        pass


async def test_starter_does_not_delete_the_container_the_cleaner_is_removing(world: Any) -> None:
    """A dangling container is being removed by the cleaner while the same tag is installed again."""
    name = world.engine.add("1", running=True)

    cleaner = asyncio.create_task(world.kraken.kill_dangling_containers())
    await asyncio.sleep(0.02)
    world.settings.extensions = [make_settings("1")]
    world.engine.containers[name]["State"]["Running"] = False
    await asyncio.gather(cleaner, world.kraken.init_dead_extensions())

    assert world.engine.conflicts == []


async def test_second_removal_of_one_container_does_not_reach_docker(world: Any) -> None:
    """Two uninstall requests for one extension overlap (a double click, or a retry of the job)."""
    world.settings.extensions = [make_settings("1")]
    world.engine.add("1", running=True)

    first = asyncio.create_task(make_extension("1").uninstall())
    await asyncio.sleep(0.02)
    world.settings.extensions = [make_settings("1")]
    await make_extension("1").uninstall()
    await first

    assert world.engine.conflicts == []
    assert not world.engine.containers


async def test_uninstall_raises_when_docker_fails_to_remove_the_container(world: Any) -> None:
    world.settings.extensions = [make_settings("1")]
    world.engine.add("1", running=True)
    world.engine.delete_error = DockerError(500, {"message": "device or resource busy"})  # type: ignore

    with pytest.raises(DockerError, match="device or resource busy"):
        await make_extension("1").uninstall()


@pytest.mark.parametrize("status, raises", [(404, False), (500, True)])
async def test_uninstall_of_a_missing_container_only_ignores_a_missing_image(
    world: Any, status: int, raises: bool
) -> None:
    world.settings.extensions = [make_settings("1")]
    world.engine.client.images.delete.side_effect = DockerError(status, {"message": "image delete failed"})  # type: ignore

    if raises:
        with pytest.raises(DockerError):
            await make_extension("1").uninstall()
    else:
        await make_extension("1").uninstall()


async def test_failed_removal_of_an_old_tag_is_reported_as_installed_with_a_failed_cleanup(world: Any) -> None:
    world.settings.extensions = [make_settings("1", enabled=False), make_settings("2")]
    world.engine.add("1", running=False)
    world.engine.delete_error = DockerError(500, {"message": "device or resource busy"})  # type: ignore

    with pytest.raises(ExtensionCleanupFailed, match="^Installed, but could not remove old tags: 1: .*busy"):
        await make_extension("2")._clear_remaining_tags()


async def test_install_stream_ends_with_the_cleanup_error_after_the_new_tag_is_installed(world: Any) -> None:
    world.settings.extensions = [make_settings("1", enabled=False)]
    world.engine.add("1", running=False)
    world.engine.delete_error = DockerError(500, {"message": "device or resource busy"})  # type: ignore

    with mock.patch.object(Extension, "_local_image_id", mock.AsyncMock(return_value="id")):
        lines = [line async for line in streamer(make_extension("2").install(pull=PullPolicy.NEVER))]

    last = json.loads(lines[-1].removesuffix("|\n\n|"))
    assert last["status"] == 500
    assert last["error"].startswith("Installed, but could not remove old tags: 1: ")
    assert {ext.tag: ext.enabled for ext in world.settings.extensions}["2"] is True
