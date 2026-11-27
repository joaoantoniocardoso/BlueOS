import json
from typing import Any, cast

from blueos.core import BlueOS
from blueos.http import request
from targets.target import Target
from utils.retry import wait_until

SETTINGS_FILE = "/root/.config/kraken/settings-2.json"
EXTENSIONS = {
    "williangalvani.example1": ("Example 1", "williangalvani/blueos-example1"),
    "williangalvani.example2": ("Example 2", "williangalvani/blueos-example2"),
}
MANIFEST = {"name": "custom source", "url": "http://example.invalid/manifest.json", "enabled": True}


def expect(condition: bool) -> None:
    if not condition:
        raise AssertionError


def installed(blueos: BlueOS) -> set[str]:
    extensions = request("GET", f"{blueos.base_url}/kraken/v2.0/extension/")
    return {extension["identifier"] for extension in extensions}


def container_ids(target: Target) -> list[str]:
    return sorted(target.shell("sudo docker ps -aq --filter name=extension-").split())


def saved_settings(target: Target) -> dict[str, Any]:
    return cast(dict[str, Any], json.loads(target.shell(f"sudo docker exec blueos-core cat {SETTINGS_FILE}")))


def saved_identifiers(target: Target) -> set[str]:
    return {extension["identifier"] for extension in saved_settings(target)["extensions"]}


def install(target: Target, blueos: BlueOS, identifier: str) -> None:
    if identifier not in installed(blueos):
        name, docker = EXTENSIONS[identifier]
        body = {
            "identifier": identifier,
            "name": name,
            "docker": docker,
            "tag": "v1.0.1",
            "enabled": True,
            "permissions": "{}",
        }
        target.shell(
            "curl -sf -X POST -H 'Content-Type: application/json' "
            f"-d '{json.dumps(body)}' http://localhost/kraken/v2.0/extension/ > /dev/null"
        )
    wait_until(
        lambda: expect(len(container_ids(target)) == len(installed(blueos))), 300, description="extension containers"
    )


def test_saves_from_different_parts_of_kraken_do_not_erase_each_other(target: Target, blueos: BlueOS) -> None:
    first, second = EXTENSIONS
    install(target, blueos, first)

    target.shell(
        "curl -sf -X POST -H 'Content-Type: application/json' "
        f"-d '{json.dumps(MANIFEST)}' 'http://localhost/kraken/v2.0/manifest/?validate_url=false' > /dev/null"
    )
    assert first in installed(blueos)
    assert saved_identifiers(target) == installed(blueos)

    install(target, blueos, second)
    assert {first, second} <= installed(blueos)
    assert saved_identifiers(target) == installed(blueos)
    assert MANIFEST["name"] in [manifest["name"] for manifest in saved_settings(target)["manifests"]]

    ids, before = installed(blueos), container_ids(target)
    target.shell("sudo docker restart blueos-core")
    blueos.wait_ready()
    wait_until(lambda: expect(installed(blueos) == ids), 180, description="extension list after restart")
    assert container_ids(target) == before
