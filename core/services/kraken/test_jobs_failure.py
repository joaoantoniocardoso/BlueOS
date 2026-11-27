# isort: skip_file
import sys
from pathlib import Path
from typing import Tuple

import pytest

_KRAKEN_DIR = str(Path(__file__).resolve().parent)
sys.path.insert(0, _KRAKEN_DIR)

from aiohttp import web  # noqa: E402
from loguru import logger  # noqa: E402
from jobs import JobsManager  # noqa: E402
from jobs.models import Job, JobMethod, JobStatus  # noqa: E402
from kraken import Kraken  # noqa: E402

pytestmark = pytest.mark.asyncio


@pytest.fixture(autouse=True)
def clean_manager(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(JobsManager, "_jobs", [])
    monkeypatch.setattr(JobsManager, "_failed", [])
    monkeypatch.setattr("jobs.jobs.asyncio.sleep", _no_sleep)


async def _no_sleep(_: float) -> None:
    return None


def make_manager(base_host: str) -> JobsManager:
    manager = JobsManager()
    manager.set_base_host(base_host)
    return manager


async def start_server(status: int, body: str) -> Tuple[web.AppRunner, str]:
    async def handler(_: web.Request) -> web.Response:
        return web.Response(status=status, text=body)

    app = web.Application()
    app.router.add_route("*", "/{tail:.*}", handler)
    runner = web.AppRunner(app)
    await runner.setup()
    site = web.TCPSite(runner, "127.0.0.1", 0)
    await site.start()
    port = site._server.sockets[0].getsockname()[1]  # type: ignore
    return runner, f"http://127.0.0.1:{port}"


async def test_failed_job_logs_reason_and_keeps_error() -> None:
    runner, host = await start_server(500, "disk exploded")
    messages = []
    sink = logger.add(lambda message: messages.append(message.record["message"]), level="WARNING")
    try:
        job = Job(id="j1", route="x", method=JobMethod.POST, body={}, retries=2)
        await make_manager(host).execute_job(job)
    finally:
        logger.remove(sink)
        await runner.cleanup()

    assert any("attempt 1/2" in m and "500" in m and "disk exploded" in m for m in messages), messages
    assert any("failed to be executed" in m and "disk exploded" in m for m in messages), messages
    assert job.status == JobStatus.FAILED
    assert job.error
    assert "disk exploded" in str(job.error)
    assert [j.id for j in JobsManager.get()] == ["j1"]
    assert JobsManager.get_by_identifier("j1").error == job.error


async def test_unreachable_target_fails_with_reason() -> None:
    job = Job(id="j2", route="x", method=JobMethod.GET, body=None, retries=1)
    await make_manager("http://127.0.0.1:1").execute_job(job)
    assert job.status == JobStatus.FAILED
    assert job.error


async def test_successful_job_is_not_kept() -> None:
    runner, host = await start_server(200, "ok")
    try:
        job = Job(id="j3", route="x", method=JobMethod.POST, body={}, retries=1)
        await make_manager(host).execute_job(job)
    finally:
        await runner.cleanup()
    assert job.status == JobStatus.PENDING
    assert job.error is None
    assert not JobsManager.get()


async def test_failed_job_can_be_deleted_and_is_replaced_by_same_id() -> None:
    old = Job(id="j4", route="x", method=JobMethod.GET, body=None, retries=1)
    await make_manager("http://127.0.0.1:1").execute_job(old)
    new = Job(id="j4", route="x", method=JobMethod.GET, body=None, retries=1)
    JobsManager.add(new)
    assert JobsManager.get() == [new]
    JobsManager.delete("j4")
    assert not JobsManager.get()


async def test_failed_default_install_job_counts_as_not_created() -> None:
    failed = Job(id="__default_install_x", route="x", method=JobMethod.POST, body={}, retries=1)
    await make_manager("http://127.0.0.1:1").execute_job(failed)
    kraken = Kraken.__new__(Kraken)
    assert not kraken.is_install_default_ext_job_created("__default_install_x")
    JobsManager.add(Job(id="__default_install_y", route="x", method=JobMethod.POST, body={}))
    assert kraken.is_install_default_ext_job_created("__default_install_y")
