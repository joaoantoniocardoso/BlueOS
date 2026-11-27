import json
import urllib.request
from collections.abc import Iterator

import pytest
from blueos.core import BlueOS
from blueos.http import request as blueos_request
from targets.sil import docker
from targets.sil.target import SilTarget
from targets.target import Target
from utils.retry import wait_until

REGISTRY = "localhost:5000"
DOCKER = f"{REGISTRY}/e2e/ext"
IDENTIFIER = "e2e.enable"
EXTENSIONS = "/kraken/v2.0/extension"


@pytest.fixture(name="registry", scope="module")
def registry_fixture(target: Target, blueos: BlueOS) -> Iterator[None]:
    assert isinstance(target, SilTarget)
    docker.load(target.ssh, "registry:2")
    docker.load(target.ssh, "busybox:1.36")
    target.shell("sudo docker run -d --name e2e-registry -p 127.0.0.1:5000:5000 registry:2")
    wait_until(
        lambda: target.shell("curl -sf http://127.0.0.1:5000/v2/"),
        60,
        errors=(RuntimeError,),
        description="the registry",
    )
    for tag in ("1", "2"):
        target.shell(f"sudo docker tag busybox:1.36 {DOCKER}:{tag} && sudo docker push -q {DOCKER}:{tag}")
    wait_until(
        lambda: blueos_request("GET", f"{blueos.base_url}{EXTENSIONS}/"),
        120,
        errors=(RuntimeError,),
        description="kraken",
    )
    yield
    target.shell("sudo docker rm -f e2e-registry")


def manifest(digest: str) -> str:
    versions = {
        tag: {
            "type": "other",
            "images": [{"expanded_size": 1, "platform": {"architecture": "amd64"}, "digest": digest}],
            "authors": [{"name": "e2e", "email": "e2e@example.com"}],
            "filter_tags": [],
            "extra_links": {},
            "tag": tag,
            "permissions": {"Cmd": ["sleep", "3600"]},
        }
        for tag in ("1", "2")
    }
    entry = {
        "identifier": IDENTIFIER,
        "name": "e2e",
        "website": "https://example.com",
        "docker": DOCKER,
        "description": "e2e",
        "versions": versions,
    }
    return json.dumps([entry])


def install(blueos: BlueOS, tag: str, purge: bool) -> None:
    request = urllib.request.Request(
        f"{blueos.base_url}{EXTENSIONS}/{IDENTIFIER}/{tag}?purge={str(purge).lower()}", method="PUT"
    )
    with urllib.request.urlopen(request, timeout=300) as response:
        response.read()


def enabled_tags(blueos: BlueOS) -> set[str]:
    extensions = blueos_request("GET", f"{blueos.base_url}{EXTENSIONS}/")
    return {ext["tag"] for ext in extensions if ext["identifier"] == IDENTIFIER and ext["enabled"]}


def container_names(target: Target) -> set[str]:
    return set(target.shell("sudo docker ps -a --format '{{.Names}}' --filter name=extension-").split())


def require(condition: bool) -> None:
    if not condition:
        raise RuntimeError("not yet")


def container_name(tag: str) -> str:
    return f"extension-localhost5000e2eext{tag}"


@pytest.mark.usefixtures("registry")
def test_enable_disables_the_other_tag(target: Target, blueos: BlueOS) -> None:
    digest = (
        target.shell(f"sudo docker inspect --format '{{{{index .RepoDigests 0}}}}' {DOCKER}:1").strip().split("@")[1]
    )
    target.shell(
        f"sudo docker exec blueos-core mkdir -p /tmp/e2e-manifest && echo '{manifest(digest)}' "
        "| sudo docker exec -i blueos-core tee /tmp/e2e-manifest/manifest.json >/dev/null"
    )
    target.shell("sudo docker exec -d blueos-core python3 -m http.server 8099 --bind 127.0.0.1 -d /tmp/e2e-manifest")
    wait_until(lambda: target.shell("curl -sf http://127.0.0.1:8099/manifest.json"), 30, description="manifest server")
    blueos_request(
        "POST",
        f"{blueos.base_url}/kraken/v2.0/manifest/",
        {"name": "e2e", "url": "http://127.0.0.1:8099/manifest.json", "enabled": True},
    )
    install(blueos, "1", purge=True)
    install(blueos, "2", purge=False)
    wait_until(lambda: require(container_name("2") in container_names(target)), 120, description="container of tag 2")

    blueos_request("POST", f"{blueos.base_url}{EXTENSIONS}/{IDENTIFIER}/1/enable")

    assert enabled_tags(blueos) == {"1"}
    assert container_name("2") not in container_names(target)
