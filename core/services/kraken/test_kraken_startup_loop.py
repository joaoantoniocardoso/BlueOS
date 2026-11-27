# isort: skip_file
import sys
from pathlib import Path
from typing import Generator, List
from unittest import mock

import pytest

_KRAKEN_DIR = str(Path(__file__).resolve().parent)
sys.path.insert(0, _KRAKEN_DIR)
# Root pytest also collects the other services, whose top-level `config` and
# `settings` modules would otherwise be the ones imported here.
_saved = {module_name: sys.modules.get(module_name) for module_name in ("config", "settings", "utils")}
for module_name in _saved:
    sys.modules.pop(module_name, None)

from loguru import logger  # noqa: E402
from extension.exceptions import IncompatibleExtension  # noqa: E402
from extension.extension import Extension  # noqa: E402
from jobs import JobsManager  # noqa: E402
from jobs.exceptions import JobNotFound  # noqa: E402
from jobs.models import Job, JobMethod  # noqa: E402
from kraken import Kraken  # noqa: E402
from settings import ExtensionSettings  # noqa: E402

for module_name, module in _saved.items():
    if module is not None:
        sys.modules[module_name] = module
    else:
        sys.modules.pop(module_name, None)

pytestmark = pytest.mark.asyncio


@pytest.fixture(name="warnings")
def fixture_warnings() -> Generator[List[str], None, None]:
    messages: List[str] = []
    handler_id = logger.add(lambda message: messages.append(str(message)), level="WARNING")
    yield messages
    logger.remove(handler_id)


async def test_incompatible_dead_extension_logs_the_reason(warnings: List[str]) -> None:
    ext = ExtensionSettings(
        identifier="ext", name="Ext", docker="bluerobotics/blueos-ext", tag="1.0.0", permissions="{}",
        enabled=True, user_permissions="",
    )  # fmt: skip
    kraken = mock.Mock()
    kraken._extension_start_try_valid.return_value = True
    kraken.manifest.fetch_extension_version = mock.AsyncMock(side_effect=IncompatibleExtension("needs core 1.5"))
    start = mock.AsyncMock()
    with (
        mock.patch("kraken.ContainerManager.get_running_containers", mock.AsyncMock(return_value=[])),
        mock.patch.object(Extension, "_fetch_settings", return_value=[ext]),
        mock.patch.object(Extension, "start", start),
    ):
        await Kraken.init_dead_extensions(kraken)

    assert any("not compatible anymore" in message and "needs core 1.5" in message for message in warnings)
    start.assert_awaited_once()  # an incompatible extension is still started, from the tag already in the settings


@pytest.mark.parametrize("error", [JobNotFound("missing"), RuntimeError("jobs broken")])
def test_default_install_job_lookup(error: Exception) -> None:
    with mock.patch.object(JobsManager, "get_by_identifier", side_effect=error):
        if isinstance(error, JobNotFound):
            assert Kraken.is_install_default_ext_job_created(mock.Mock(), "job") is False
        else:
            with pytest.raises(RuntimeError, match="jobs broken"):
                Kraken.is_install_default_ext_job_created(mock.Mock(), "job")


def test_default_install_job_found() -> None:
    job = Job(id="job", route="v2.0/extension", method=JobMethod.POST, body={}, retries=1)
    with mock.patch.object(JobsManager, "get_by_identifier", return_value=job):
        assert Kraken.is_install_default_ext_job_created(mock.Mock(), "job") is True
