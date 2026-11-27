# isort: skip_file
import sys
from pathlib import Path
from types import SimpleNamespace
from typing import Any, Dict, List, Optional
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
from harbor import ContainerManager  # noqa: E402
from harbor.exceptions import ContainerNotFound  # noqa: E402
import kraken as kraken_module  # noqa: E402
from settings import ExtensionSettings  # noqa: E402

for module_name, module in _saved.items():
    if module is not None:
        sys.modules[module_name] = module
    else:
        sys.modules.pop(module_name, None)

pytestmark = pytest.mark.asyncio


def legacy_name(docker: str, tag: str) -> str:
    return "extension-" + "".join(char for char in f"{docker}{tag}" if char.isascii() and char.isalnum())


def ext_settings(docker: str, tag: str) -> ExtensionSettings:
    return ExtensionSettings(identifier="id", name="n", docker=docker, tag=tag, permissions="{}", enabled=True)


class FakeContainer:
    def __init__(self, name: str, image: str, running: bool = True) -> None:
        self.name = name
        self.image = image
        self.running = running
        self.deleted = False
        self.id = f"id-{name}"

    def __getitem__(self, key: str) -> Any:
        return {"Names": [f"/{self.name}"], "Image": self.image}[key]

    async def show(self) -> Dict[str, Any]:
        return {"Id": self.id, "Config": {"Image": self.image}, "State": {"Running": self.running}}

    async def rename(self, newname: str) -> None:
        self.name = newname

    async def stop(self, **_: Any) -> None:
        self.running = False

    async def wait(self, **_: Any) -> None:
        return None

    async def delete(self, **_: Any) -> None:
        self.deleted = True


class FakeClient:
    """Like Docker: the `name` filter matches substrings, and `all` includes stopped containers."""

    def __init__(self, *containers: FakeContainer) -> None:
        self.all_containers = list(containers)
        self.containers = mock.Mock()
        self.containers.list = self._list
        self.images = mock.Mock()
        self.images.delete = mock.AsyncMock()

    # `all` is the keyword that aiodocker passes on to Docker
    async def _list(  # pylint: disable=redefined-builtin
        self, all: bool = False, filters: Optional[Dict[str, Any]] = None
    ) -> List[FakeContainer]:
        result = [c for c in self.all_containers if not c.deleted and (all or c.running)]
        for name in (filters or {}).get("name", {}):
            result = [c for c in result if name in c.name]
        return result


class FakeCtx:
    def __init__(self, client: FakeClient) -> None:
        self.client = client

    async def __aenter__(self) -> FakeClient:
        return self.client

    async def __aexit__(self, *_: Any) -> None:
        return None


def test_container_names_are_unique_per_image_and_tag() -> None:
    pairs = [
        ("org/ext", "1.0"),
        ("org/ext", "1.0.1"),
        ("org/ext", "1.10"),
        ("org/ext", "11.0"),
        ("org/ext", "v1.2"),
        ("org/ext", "v1-2"),
        ("org/ext1", ".0"),
        ("org/ext", "1.0"),
    ]
    names = [ext_settings(*pair).container_name() for pair in pairs[:-1]]
    assert len(set(names)) == len(names)
    assert ext_settings(*pairs[0]).container_name() == ext_settings(*pairs[-1]).container_name()
    assert all(name.startswith("extension-") for name in names)


async def test_get_raw_container_by_name_ignores_longer_names() -> None:
    client = FakeClient(FakeContainer("extension-ab", "i:1"), FakeContainer("extension-abc", "i:2"))
    container = await ContainerManager.get_raw_container_by_name(client, "extension-ab")  # type: ignore
    assert container["Names"] == ["/extension-ab"]
    with pytest.raises(ContainerNotFound):
        await ContainerManager.get_raw_container_by_name(client, "extension-a")  # type: ignore


async def test_stop_all_by_name_only_stops_the_exact_name() -> None:
    short, long = FakeContainer("extension-ab", "i:1"), FakeContainer("extension-abc", "i:2")
    await ContainerManager.stop_all_by_name(FakeClient(short, long), "extension-ab")  # type: ignore
    assert not short.running
    assert long.running


async def test_remove_leaves_the_container_and_image_of_another_tag() -> None:
    old = ext_settings("org/ext", "1.0")
    new = ext_settings("org/ext", "1.0.1")
    old_container = FakeContainer(old.container_name(), "sha256:old")
    new_container = FakeContainer(new.container_name(), "sha256:new")
    client = FakeClient(new_container, old_container)  # the longer name is listed first on purpose

    with mock.patch("extension.extension.DockerCtx", return_value=FakeCtx(client)):
        await Extension.remove(old.container_name())

    assert old_container.deleted
    assert not new_container.deleted
    assert new_container.running
    client.images.delete.assert_awaited_once_with("sha256:old", force=True, noprune=False)


async def test_remove_finds_stopped_containers_by_exact_name_only() -> None:
    old = ext_settings("org/ext", "1.0")
    client = FakeClient(FakeContainer(old.container_name() + "x", "sha256:other"))
    with mock.patch("extension.extension.DockerCtx", return_value=FakeCtx(client)):
        with pytest.raises(ContainerNotFound):
            await Extension.remove(old.container_name())


async def test_adopt_renames_a_running_legacy_container_without_stopping_it() -> None:
    entry = ext_settings("org/ext", "1.0")
    legacy = FakeContainer(legacy_name("org/ext", "1.0"), "org/ext:1.0")
    client = FakeClient(legacy)

    assert await ContainerManager.adopt_legacy_container(
        client, entry.legacy_container_name(), entry.container_name(), entry.fullname()  # type: ignore
    )

    assert legacy.name == entry.container_name()
    assert legacy.running
    assert not legacy.deleted
    assert legacy.id == "id-" + legacy_name("org/ext", "1.0")


async def test_adopt_leaves_an_unrelated_container_with_the_legacy_name_alone() -> None:
    entry = ext_settings("org/ext", "1.0")
    stranger = FakeContainer(legacy_name("org/ext", "1.0"), "someone/else:9")
    client = FakeClient(stranger)

    assert not await ContainerManager.adopt_legacy_container(
        client, entry.legacy_container_name(), entry.container_name(), entry.fullname()  # type: ignore
    )

    assert stranger.name == legacy_name("org/ext", "1.0")


async def test_adopt_does_not_touch_a_longer_legacy_name() -> None:
    entry = ext_settings("org/ext", "1.0")
    other = FakeContainer(legacy_name("org/ext", "1.0.1"), "org/ext:1.0.1")
    client = FakeClient(other)

    assert not await ContainerManager.adopt_legacy_container(
        client, entry.legacy_container_name(), entry.container_name(), entry.fullname()  # type: ignore
    )

    assert other.name == legacy_name("org/ext", "1.0.1")


async def test_adopt_gives_each_colliding_legacy_container_to_the_tag_it_runs() -> None:
    first, second = ext_settings("org/ext", "v1.2"), ext_settings("org/ext", "v1-2")
    assert first.legacy_container_name() == second.legacy_container_name()
    legacy = FakeContainer(first.legacy_container_name(), second.fullname())
    client = FakeClient(legacy)

    assert not await ContainerManager.adopt_legacy_container(
        client, first.legacy_container_name(), first.container_name(), first.fullname()  # type: ignore
    )
    assert await ContainerManager.adopt_legacy_container(
        client, second.legacy_container_name(), second.container_name(), second.fullname()  # type: ignore
    )
    assert legacy.name == second.container_name()


async def test_adopt_keeps_a_container_already_under_the_new_name() -> None:
    entry = ext_settings("org/ext", "1.0")
    current = FakeContainer(entry.container_name(), entry.fullname())
    legacy = FakeContainer(entry.legacy_container_name(), entry.fullname(), running=False)
    client = FakeClient(current, legacy)

    assert not await ContainerManager.adopt_legacy_container(
        client, entry.legacy_container_name(), entry.container_name(), entry.fullname()  # type: ignore
    )
    assert legacy.name == entry.legacy_container_name()


async def test_cleaner_adopts_a_legacy_container_instead_of_killing_it() -> None:
    entry = ext_settings("org/ext", "1.0")
    legacy = FakeContainer(entry.legacy_container_name(), entry.fullname())
    client = FakeClient(legacy)

    async def running() -> List[Any]:
        return [SimpleNamespace(name=f"/{legacy.name}")]

    instance = kraken_module.Kraken.__new__(kraken_module.Kraken)
    with (
        mock.patch.object(Extension, "_fetch_settings", return_value=[entry]),
        mock.patch.object(Extension, "remove", new_callable=mock.AsyncMock) as remove,
        mock.patch.object(ContainerManager, "get_running_containers", side_effect=running),
        mock.patch("kraken.DockerCtx", return_value=FakeCtx(client)),
    ):
        await instance.kill_dangling_containers()

    remove.assert_not_awaited()
    assert legacy.name == entry.container_name()
    assert legacy.running
