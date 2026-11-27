# isort: skip_file
import sys
from pathlib import Path
from typing import List, Set
from unittest import mock

import pytest

_KRAKEN_DIR = str(Path(__file__).resolve().parent)
sys.path.insert(0, _KRAKEN_DIR)
_saved = {module_name: sys.modules.get(module_name) for module_name in ("config", "settings", "utils")}
for module_name in _saved:
    sys.modules.pop(module_name, None)

from extension.extension import Extension  # noqa: E402

for module_name, module in _saved.items():
    if module is not None:
        sys.modules[module_name] = module
    else:
        sys.modules.pop(module_name, None)

pytestmark = pytest.mark.asyncio


async def cleaned_references(repo_tags: List[str], active: Set[str]) -> List[str]:
    client = mock.MagicMock()
    client.images.list = mock.AsyncMock(return_value=[{"RepoTags": repo_tags}])
    client.images.delete = mock.AsyncMock()
    ctx = mock.MagicMock()
    ctx.__aenter__ = mock.AsyncMock(return_value=client)
    ctx.__aexit__ = mock.AsyncMock(return_value=False)
    with mock.patch("extension.extension.DockerCtx", return_value=ctx):
        await Extension._cleanup_orphan_temp_tags(active)
    return [call.args[0] for call in client.images.delete.call_args_list]


async def test_orphan_kraken_temp_tag_is_removed() -> None:
    assert await cleaned_references(["acme/ext:temp-0123abcd"], set()) == ["acme/ext:temp-0123abcd"]


@pytest.mark.parametrize("tag", ["temp-1", "temp-release", "temp-0123ABCD", "temp-0123abcdef"])
async def test_tags_not_made_by_kraken_survive(tag: str) -> None:
    assert await cleaned_references([f"acme/ext:{tag}"], set()) == []


async def test_tracked_temp_tag_survives() -> None:
    assert await cleaned_references(["acme/ext:temp-0123abcd"], {"acme/ext:temp-0123abcd"}) == []


async def test_installed_extension_with_temp_shaped_tag_survives() -> None:
    settings = [mock.MagicMock(identifier="acme.ext", docker="acme/ext", tag="temp-0123abcd")]
    with mock.patch.object(Extension, "_fetch_settings", return_value=settings), mock.patch.object(
        Extension, "_cleanup_orphan_temp_tags", new_callable=mock.AsyncMock
    ) as cleanup:
        await Extension.cleanup_temporary_extensions()
    assert "acme/ext:temp-0123abcd" in cleanup.call_args.args[0]
