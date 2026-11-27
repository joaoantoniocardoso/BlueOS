import json

import pytest
from blueos.core import BlueOS
from blueos.http import request
from targets.target import Target
from utils.retry import wait_until

SETTINGS_FILE = "/root/.config/kraken/settings-2.json"
EXTENSIONS = {
    "williangalvani.example1": ("Example 1", "williangalvani/blueos-example1"),
    "williangalvani.example2": ("Example 2", "williangalvani/blueos-example2"),
}


def expect(condition: bool) -> None:
    if not condition:
        raise AssertionError


def installed(blueos: BlueOS) -> set[str]:
    extensions = request("GET", f"{blueos.base_url}/kraken/v2.0/extension/")
    return {extension["identifier"] for extension in extensions}


def container_ids(target: Target) -> list[str]:
    return sorted(target.shell("sudo docker ps -aq --filter name=extension-").split())


def install_extensions(target: Target, blueos: BlueOS) -> None:
    for identifier, (name, docker) in EXTENSIONS.items():
        if identifier in installed(blueos):
            continue
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
    wait_until(lambda: expect(len(container_ids(target)) == 2), 300, description="both extension containers")


@pytest.mark.parametrize("loss", ["rm", "truncate -s 0"])
def test_extensions_survive_a_lost_settings_file(target: Target, blueos: BlueOS, loss: str) -> None:
    install_extensions(target, blueos)
    before = container_ids(target)
    assert installed(blueos) == set(EXTENSIONS)

    target.shell(f"sudo docker exec blueos-core sh -c '{loss} {SETTINGS_FILE}'")
    target.shell("sudo docker restart blueos-core")
    blueos.wait_ready()

    wait_until(lambda: expect(installed(blueos) == set(EXTENSIONS)), 180, description="recovered extension list")
    assert container_ids(target) == before
