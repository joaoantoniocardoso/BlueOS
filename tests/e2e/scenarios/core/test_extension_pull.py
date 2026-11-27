import json
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
IDLE_LIMIT_SECONDS = 120


@pytest.fixture(name="registry", scope="module")
def registry_fixture(target: Target, blueos: BlueOS) -> Iterator[str]:
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
    target.shell(
        f"head -c {LAYER_BYTES} /dev/urandom > /tmp/blob && tar -cf /tmp/blob.tar -C /tmp blob "
        f"&& sudo docker import /tmp/blob.tar {local}/slow:1 && sudo docker tag {local}/slow:1 {local}/stall:1 "
        f"&& sudo docker push -q {local}/slow:1 && sudo docker push -q {local}/stall:1 "
        f"&& sudo docker rmi {local}/slow:1 {local}/stall:1"
    )
    cmd = '--change \'CMD ["/bin/registry", "--version"]\''
    target.shell(
        "sudo docker export e2e-registry > /tmp/hung.tar && cp /tmp/hung.tar /tmp/good.tar "
        "&& echo good > /tmp/good.txt && tar -rf /tmp/good.tar -C /tmp good.txt "
        f"&& sudo docker import {cmd} /tmp/hung.tar {local}/hung:1 && sudo docker import {cmd} /tmp/good.tar {local}/good:1 "
        f"&& sudo docker push -q {local}/hung:1 && sudo docker push -q {local}/good:1 "
        f"&& sudo docker rmi {local}/hung:1 {local}/good:1"
    )
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
    wait_until(
        lambda: blueos_request("GET", f"{blueos.base_url}/kraken/v2.0/extension/"),
        120,
        errors=(RuntimeError,),
        description="kraken",
    )
    yield f"localhost:{PROXY_PORT}"
    target.shell("sudo docker rm -f e2e-registry")


def install_stream(
    blueos: BlueOS, docker_image: str, timeout: float, enabled: bool = False
) -> tuple[float, list[bytes]]:
    body = {"identifier": f"e2e.{docker_image.rpartition('/')[2]}", "name": "e2e", "docker": docker_image, "tag": "1", "enabled": enabled,
            "permissions": "{}"}  # fmt: skip
    request = urllib.request.Request(
        f"{blueos.base_url}/kraken/v2.0/extension/",
        data=json.dumps(body).encode(),
        method="POST",
        headers={"Content-Type": "application/json"},
    )
    start = time.monotonic()
    with urllib.request.urlopen(request, timeout=timeout) as response:
        lines = list(response)
    return time.monotonic() - start, lines


def test_stalled_pull_fails_after_the_idle_limit(blueos: BlueOS, registry: str) -> None:
    elapsed, lines = install_stream(blueos, f"{registry}/stall", timeout=IDLE_LIMIT_SECONDS + 60)
    assert IDLE_LIMIT_SECONDS <= elapsed < IDLE_LIMIT_SECONDS + 40, b"".join(lines)[-500:]
    assert any(b'"fragment": -1' in line for line in lines), "no heartbeat while Docker was silent"
    assert b"Failed to pull extension" in b"".join(lines)


def container_exists(target: Target, name: str) -> None:
    if not target.shell(f"sudo docker ps -a --filter name={name} --format {{{{.Names}}}}"):
        raise RuntimeError(f"no container {name}")


def test_hung_pull_does_not_stop_other_extensions_from_starting(blueos: BlueOS, registry: str, target: Target) -> None:
    for name in ("hung", "good"):  # kraken starts the dead extensions in the order they were installed
        install_stream(blueos, f"{registry}/{name}", timeout=120, enabled=True)
    names = [f"extension-{registry}/{name}1".replace(":", "").replace("/", "") for name in ("hung", "good")]
    target.shell(f"sudo docker rm -f {' '.join(names)}; sudo docker rmi -f {registry}/hung:1 {registry}/good:1; true")
    target.shell("sudo docker exec blueos-core touch /tmp/hung")
    wait_until(
        lambda: container_exists(target, names[1]),
        IDLE_LIMIT_SECONDS + 90,
        errors=(RuntimeError,),
        description="the healthy extension to start while the pull of the other hangs",
    )
