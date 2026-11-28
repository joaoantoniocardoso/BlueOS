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
LAYER_BYTES = 4 * 1024 * 1024
BYTES_PER_SECOND = 100 * 1024
IDENTIFIER = "e2e.concurrency"
TAGS = ("1", "2", "3")


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
    images = " && ".join(
        f"head -c {LAYER_BYTES} /dev/urandom > /tmp/blob && tar -cf /tmp/blob.tar -C /tmp blob "
        f"&& sudo docker import /tmp/blob.tar {local}/slow:{tag} && sudo docker push -q {local}/slow:{tag} "
        f"&& sudo docker rmi {local}/slow:{tag}"
        for tag in TAGS
    )
    target.shell(images)
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


def install(blueos: BlueOS, docker_image: str, tag: str) -> bytes:
    body = {"identifier": IDENTIFIER, "name": "e2e", "docker": docker_image, "tag": tag, "enabled": False,
            "permissions": "{}"}  # fmt: skip
    request = urllib.request.Request(
        f"{blueos.base_url}/kraken/v2.0/extension/",
        data=json.dumps(body).encode(),
        method="POST",
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(request, timeout=300) as response:
        return b"".join(response)


def installed_tags(blueos: BlueOS) -> list[str]:
    extensions = blueos_request("GET", f"{blueos.base_url}/kraken/v2.0/extension/")
    return sorted(ext["tag"] for ext in extensions if ext["identifier"] == IDENTIFIER)


def test_two_installs_of_one_extension_keep_the_extension(blueos: BlueOS, registry: str) -> None:
    image = f"{registry}/slow"
    first = install(blueos, image, "1")
    assert installed_tags(blueos) == ["1"], first[-500:]

    outputs: dict[str, bytes] = {}
    update = threading.Thread(target=lambda: outputs.update({"2": install(blueos, image, "2")}))
    update.start()
    time.sleep(5)
    outputs["3"] = install(blueos, image, "3")
    update.join(timeout=300)

    assert b"already being installed" in outputs["3"], outputs["3"][-500:]
    assert installed_tags(blueos) == ["2"]
