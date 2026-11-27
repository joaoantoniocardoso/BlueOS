import json
import urllib.error
import urllib.request
from collections.abc import Iterator
from pathlib import Path
from typing import Any

import pytest
from blueos.core import BlueOS
from blueos.http import request as blueos_request
from targets.sil import docker
from targets.sil.target import SilTarget
from targets.target import Target
from utils.retry import wait_until

REGISTRY_PORT, MANIFEST_PORT = 5000, 8101
REGISTRY = f"localhost:{REGISTRY_PORT}"
IDENTIFIER = "e2e.conflict"
PRIORITY_REPO, OTHER_REPO, THIRD_REPO = (f"{REGISTRY}/e2e-{name}/ext" for name in ("first", "second", "third"))
IMAGES = {PRIORITY_REPO: ("2.0.0",), OTHER_REPO: ("1.0.0", "1.5.0"), THIRD_REPO: ("1",)}


def kraken(blueos: BlueOS, path: str) -> str:
    return f"{blueos.base_url}/kraken/v2.0/{path}"


def entry(repo: str, digests: dict[str, str]) -> dict[str, Any]:
    image = {"expanded_size": 1, "platform": {"architecture": "amd64", "os": "linux"}}
    version = {"type": "tool", "authors": [], "filter_tags": [], "extra_links": {}}
    return {
        "identifier": IDENTIFIER,
        "name": f"Conflict {repo}",
        "website": "https://example.com",
        "docker": repo,
        "description": "e2e",
        "versions": {
            tag: {**version, "tag": tag, "images": [{**image, "digest": digests[f"{repo}:{tag}"]}]}
            for tag in IMAGES[repo]
        },
    }


@pytest.fixture(name="catalog", scope="module")
def catalog_fixture(
    target: Target, blueos: BlueOS, tmp_path_factory: pytest.TempPathFactory
) -> Iterator[dict[str, str]]:
    """Two manifest sources that list one identifier with two repos on a local registry.

    Returns the identifier of each source: the first one has the higher priority.
    """
    assert isinstance(target, SilTarget)
    docker.load(target.ssh, "registry:2")
    target.shell(f"sudo docker run -d --name e2e-registry -p 127.0.0.1:{REGISTRY_PORT}:5000 registry:2")
    wait_until(
        lambda: target.shell(f"curl -sf http://127.0.0.1:{REGISTRY_PORT}/v2/"),
        60,
        errors=(RuntimeError,),
        description="the registry",
    )
    target.shell(
        "sudo docker rm -f e2e-base; "
        "sudo docker commit -c 'ENTRYPOINT []' -c 'CMD [\"sleep\", \"infinity\"]' "
        "$(sudo docker create --name e2e-base registry:2) e2e-ext:base"
    )
    digests: dict[str, str] = {}
    for repo, tags in IMAGES.items():
        for tag in tags:
            target.shell(f"sudo docker tag e2e-ext:base {repo}:{tag} && sudo docker push -q {repo}:{tag}")
            digests[f"{repo}:{tag}"] = target.shell(
                f"sudo docker inspect --format '{{{{index .RepoDigests 0}}}}' {repo}:{tag}"
            ).rpartition("@")[2]

    directory = tmp_path_factory.mktemp("manifests")
    target.shell("sudo docker exec blueos-core mkdir -p /tmp/e2e-manifests")
    for name, repo in (("first", PRIORITY_REPO), ("second", OTHER_REPO)):
        path: Path = directory / f"{name}.json"
        path.write_text(json.dumps([entry(repo, digests)]))
        target.ssh.put(path, f"/opt/e2e/{name}.json")
        target.shell(f"sudo docker cp /opt/e2e/{name}.json blueos-core:/tmp/e2e-manifests/{name}.json")
    target.shell(
        f"sudo docker exec -d blueos-core python3 -m http.server {MANIFEST_PORT} --directory /tmp/e2e-manifests"
    )
    wait_until(
        lambda: target.shell(f"curl -sf http://127.0.0.1:{MANIFEST_PORT}/first.json"),
        60,
        errors=(RuntimeError,),
        description="the manifest host",
    )
    wait_until(lambda: blueos_request("GET", kraken(blueos, "extension/")), 120, description="kraken")
    sources = {}
    for name in ("first", "second"):
        body = {"name": f"e2e {name}", "url": f"http://127.0.0.1:{MANIFEST_PORT}/{name}.json", "enabled": True}
        sources[name] = blueos_request("POST", kraken(blueos, "manifest/"), body)["identifier"]
    yield sources
    for identifier in sources.values():
        blueos_request("DELETE", kraken(blueos, f"manifest/{identifier}"))
    target.shell("sudo docker rm -f e2e-registry e2e-base")


def call(blueos: BlueOS, method: str, path: str, body: dict[str, Any] | None = None) -> tuple[int, Any]:
    data = json.dumps(body).encode() if body else None
    http_request = urllib.request.Request(
        kraken(blueos, path), data=data, method=method, headers={"Content-Type": "application/json"}
    )
    try:
        with urllib.request.urlopen(http_request, timeout=300) as response:
            content = b"".join(response)
            return response.status, content
    except urllib.error.HTTPError as error:
        return error.code, json.loads(error.read() or "null")


def installed(blueos: BlueOS) -> list[tuple[str, str]]:
    extensions = blueos_request("GET", kraken(blueos, "extension/"))
    return sorted((ext["docker"], ext["tag"]) for ext in extensions if ext["identifier"] == IDENTIFIER)


def test_catalog_lists_the_other_repo_as_conflict(blueos: BlueOS, catalog: dict[str, str]) -> None:
    entries = [
        item
        for item in blueos_request("GET", kraken(blueos, "manifest/consolidated"))
        if item["identifier"] == IDENTIFIER
    ]

    assert len(entries) == 1
    assert entries[0]["docker"] == PRIORITY_REPO
    conflicts = entries[0]["conflicts"]
    assert [(c["manifest"], c["docker"], c["tags"]) for c in conflicts] == [
        (catalog["second"], OTHER_REPO, ["1.0.0", "1.5.0"])
    ]


def test_fresh_install_follows_priority(blueos: BlueOS, catalog: dict[str, str]) -> None:
    status, output = call(blueos, "POST", f"extension/{IDENTIFIER}/install")

    assert status == 200, output
    assert installed(blueos) == [(PRIORITY_REPO, "2.0.0")]


def test_explicit_source_switches_the_repo(blueos: BlueOS, catalog: dict[str, str]) -> None:
    status, output = call(blueos, "PUT", f"extension/{IDENTIFIER}?manifest_identifier={catalog['second']}")

    assert status == 200, output
    assert installed(blueos) == [(OTHER_REPO, "1.5.0")]


def test_update_keeps_the_installed_repo(blueos: BlueOS, catalog: dict[str, str]) -> None:
    status, output = call(blueos, "PUT", f"extension/{IDENTIFIER}")

    assert status == 200, output
    assert installed(blueos) == [(OTHER_REPO, "1.5.0")]


def test_installed_repo_in_no_source_needs_a_choice(blueos: BlueOS, catalog: dict[str, str]) -> None:
    body = {
        "identifier": IDENTIFIER,
        "name": "e2e",
        "docker": THIRD_REPO,
        "tag": "1",
        "enabled": False,
        "permissions": "",
    }
    status, output = call(blueos, "POST", "extension/", body)
    assert status == 200, output
    assert (THIRD_REPO, "1") in installed(blueos)

    status, detail = call(blueos, "PUT", f"extension/{IDENTIFIER}")

    assert status == 409
    assert detail["detail"]["code"] == "extension_source_conflict"
    assert detail["detail"]["installed_docker"] == THIRD_REPO
    assert {c["docker"] for c in detail["detail"]["candidates"]} == {PRIORITY_REPO, OTHER_REPO}
    assert (THIRD_REPO, "1") in installed(blueos)
