# isort: skip_file
from typing import Dict, Tuple
from unittest import mock

import pytest
from aiodocker.exceptions import DockerError

from test_extension_install_rollback import (
    DOCKER,
    FakeDocker,
    fixture_env,
    use_docker,
)  # noqa: F401  # pylint: disable=unused-import
from extension.exceptions import ExtensionPullFailed
from extension.extension import Extension

pytestmark = pytest.mark.asyncio


async def test_failing_temp_tag_fails_the_upload_and_leaves_no_entry(env):  # type: ignore
    fake = FakeDocker({"loaded:1": "sha256:a"})
    fake.client.images.tag.side_effect = DockerError(500, {"message": "tag boom"})  # type: ignore
    patch_ctx = use_docker(fake)
    activity: Dict[str, Tuple[int, int]] = {}

    with patch_ctx, mock.patch.object(Extension, "temp_extension_activity", activity):
        with pytest.raises(ExtensionPullFailed, match="tag boom"):
            await Extension.create_temporary_extension("loaded:1", {"docker": DOCKER})

    assert not env.settings.extensions
    assert not activity
    Extension._manager.save.assert_not_called()  # type: ignore
