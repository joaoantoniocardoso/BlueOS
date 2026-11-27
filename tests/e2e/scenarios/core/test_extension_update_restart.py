import json
import threading
import time
import urllib.request
from collections.abc import Iterator
from pathlib import Path

import pytest
from blueos.core import BlueOS
from blueos.http import request as blueos_request
from targets.sil import docker
from targets.sil.target import SilTarget
from targets.target import Target
from utils.retry import wait_until

PROXY = Path(__file__).resolve().parents[2] / "blueos" / "slow_registry_proxy.py"
REGISTRY_PORT, PROXY_PORT = 5000, 5001
LAYER_BYTES = 12 * 1024 * 1024
BYTES_PER_SECOND = 100 * 1024
IDENTIFIER = "e2e.update"
DOCKER = f"localhost:{PROXY_PORT}/update"
OLD_CONTAINER = f"extension-localhost{PROXY_PORT}update1"


def expect(condition: bool, message: str = "condition not met") -> None:
    if not condition:
        raise AssertionError(message)


@pytest.fixture(name="registry", scope="module")
def registry_fixture(target: Target, blueos: BlueOS) -> Iterator[None]:
    assert isinstance(target, SilTarget)
    docker.load(target.ssh, "registry:2")
    target.shell(f"sudo docker run -d --name e2e-registry -p 127.0.0.1:{REGISTRY_PORT}:5000 registry:2")
    wait_until(
        lambda: target.shell(f"curl -sf http://127.0.0.1:{REGISTRY_PORT}/v2/"),
        60,
        errors=(RuntimeError,),
        description="the registry",
    )
    local = f"localhost:{REGISTRY_PORT}"
    # Tag 1 adds no layer, so its pull through the slow proxy ends at once. Tag 2 adds one slow layer
    target.shell(
        f"sudo docker create --name e2e-seed-1 --entrypoint sleep registry:2 infinity "
        f"&& sudo docker commit e2e-seed-1 {local}/update:1 && sudo docker rm e2e-seed-1 "
        f"&& sudo docker run --name e2e-seed-2 --entrypoint sh registry:2 -c 'head -c {LAYER_BYTES} /dev/urandom > /blob' "
        f"&& sudo docker commit -c 'ENTRYPOINT [\"sleep\"]' -c 'CMD [\"infinity\"]' e2e-seed-2 {local}/update:2 "
        f"&& sudo docker rm e2e-seed-2 "
        f"&& sudo docker push -q {local}/update:1 && sudo docker push -q {local}/update:2 "
        f"&& sudo docker rmi {local}/update:2"
    )
    target.shell(f"sudo docker tag {local}/update:1 {DOCKER}:1")
    target.ssh.put(PROXY, "/opt/e2e/slow_registry_proxy.py")
    target.shell("sudo docker cp /opt/e2e/slow_registry_proxy.py blueos-core:/tmp/slow_registry_proxy.py")
    target.shell(
        f"sudo docker exec -d blueos-core python3 /tmp/slow_registry_proxy.py {PROXY_PORT} {REGISTRY_PORT} "
        f"{BYTES_PER_SECOND}"
    )
    wait_until(
        lambda: target.shell(f"curl -sf http://127.0.0.1:{PROXY_PORT}/v2/"),
        60,
        errors=(RuntimeError,),
        description="the proxy",
    )
    yield
    target.shell("sudo docker rm -f e2e-registry")


def install_body(tag: str) -> dict[str, object]:
    return {"identifier": IDENTIFIER, "name": "e2e", "docker": DOCKER, "tag": tag, "enabled": True,
            "permissions": "{}"}  # fmt: skip


def install(blueos: BlueOS, tag: str) -> None:
    http_request = urllib.request.Request(
        f"{blueos.base_url}/kraken/v2.0/extension/",
        data=json.dumps(install_body(tag)).encode(),
        method="POST",
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(http_request, timeout=300) as response:
        response.read()


def enabled_tags(blueos: BlueOS) -> list[str]:
    extensions = blueos_request("GET", f"{blueos.base_url}/kraken/v2.0/extension/")
    return sorted(ext["tag"] for ext in extensions if ext["identifier"] == IDENTIFIER and ext["enabled"])


def running(target: Target, container: str) -> None:
    expect(target.shell(f"sudo docker inspect -f '{{{{.State.Running}}}}' {container}") == "true", container)


def test_kraken_restart_during_an_update_keeps_the_old_tag(target: Target, blueos: BlueOS, registry: None) -> None:
    del registry
    install(blueos, "1")
    wait_until(lambda: running(target, OLD_CONTAINER), 120, errors=(AssertionError, RuntimeError), description="tag 1")

    # The slow layer takes about two minutes, the restart comes while it is still downloading
    updater = threading.Thread(target=lambda: install_ignoring_errors(blueos, "2"), daemon=True)
    updater.start()
    time.sleep(15)

    target.shell("sudo docker restart blueos-core")
    blueos.wait_ready()
    wait_until(
        lambda: blueos_request("GET", f"{blueos.base_url}/kraken/v2.0/extension/"),
        180,
        errors=(OSError, RuntimeError),
        description="kraken",
    )

    assert enabled_tags(blueos) == ["1"]
    wait_until(lambda: running(target, OLD_CONTAINER), 60, errors=(AssertionError, RuntimeError), description="tag 1")


def install_ignoring_errors(blueos: BlueOS, tag: str) -> None:
    try:
        install(blueos, tag)
    except Exception:  # pylint: disable=broad-exception-caught
        pass
