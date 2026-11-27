import importlib
from collections.abc import Callable, Iterator

import pytest
from targets.sil import target as sil_target


@pytest.fixture(name="reload_target")
def reload_target_fixture(monkeypatch: pytest.MonkeyPatch) -> Iterator[Callable[..., type[sil_target.SilTarget]]]:
    def reload(**env: str) -> type[sil_target.SilTarget]:
        for name in ("E2E_SIL_SSH_PORT", "E2E_SIL_HTTP_PORT"):
            monkeypatch.delenv(name, raising=False)
        for name, value in env.items():
            monkeypatch.setenv(name, value)
        importlib.reload(sil_target)
        return sil_target.SilTarget

    yield reload
    monkeypatch.undo()
    importlib.reload(sil_target)


def test_default_ports(reload_target: Callable[..., type[sil_target.SilTarget]]) -> None:
    target_class = reload_target()
    assert (target_class.vm_ssh_forward_port, target_class.vm_http_forward_port) == (2222, 8080)


def test_env_ports(reload_target: Callable[..., type[sil_target.SilTarget]]) -> None:
    target_class = reload_target(E2E_SIL_SSH_PORT="2221", E2E_SIL_HTTP_PORT="8081")
    assert (target_class.vm_ssh_forward_port, target_class.vm_http_forward_port) == (2221, 8081)
