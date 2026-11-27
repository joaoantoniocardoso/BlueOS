import json
import re

from blueos.core import BlueOS
from targets.target import Target
from utils.retry import wait_until

SOURCE_IMAGE = "williangalvani/blueos-example1:v1.0.1"
DOCKER = "e2etest/sub"
IDENTIFIER = "e2etest.sub"


def expect(condition: bool) -> None:
    if not condition:
        raise AssertionError


def container_name(tag: str) -> str:
    return "extension-" + re.sub(r"[^a-zA-Z0-9]", lambda m: f"_{ord(m.group()):02x}", f"{DOCKER}:{tag}")


def legacy_container_name(tag: str) -> str:
    return "extension-" + re.sub(r"[^a-zA-Z0-9]", "", f"{DOCKER}{tag}")


def running_container(target: Target, tag: str) -> tuple[str, str]:
    """(ID, name) of the running container made from DOCKER:tag, whatever its name scheme is"""
    for line in target.shell("sudo docker ps --format '{{.ID}} {{.Names}} {{.Image}}'").splitlines():
        container_id, name, image = line.split()
        if image == f"{DOCKER}:{tag}":
            return container_id, name
    raise AssertionError(f"no running container for {DOCKER}:{tag}")


def inspect(target: Target, name: str, field: str) -> str:
    return target.shell(f"sudo docker inspect -f '{field}' {name}").strip()


def images(target: Target) -> list[str]:
    return target.shell("sudo docker images --format '{{.Repository}}:{{.Tag}}'").split()


def install(target: Target, tag: str) -> None:
    body = {
        "identifier": IDENTIFIER,
        "name": "Sub",
        "docker": DOCKER,
        "tag": tag,
        "enabled": True,
        "permissions": "{}",
    }
    # The pull of the fake repository fails and is ignored: the image is already in the VM
    target.shell(
        "curl -s -X POST -H 'Content-Type: application/json' "
        f"-d '{json.dumps(body)}' http://localhost/kraken/v2.0/extension/ > /dev/null"
    )


def prepare_images(target: Target, *tags: str) -> None:
    target.shell(f"sudo docker pull {SOURCE_IMAGE}")
    for tag in tags:
        target.shell(f"sudo docker tag {SOURCE_IMAGE} {DOCKER}:{tag}")


def test_uninstalling_a_tag_keeps_the_container_of_a_longer_tag(target: Target) -> None:
    prepare_images(target, "1.0", "1.0.1")
    install(target, "1.0")
    wait_until(lambda: running_container(target, "1.0"), 300, description="tag 1.0 running")
    install(target, "1.0.1")
    before = wait_until(lambda: running_container(target, "1.0.1"), 300, description="tag 1.0.1 running")

    target.shell(f"curl -sf -X DELETE http://localhost/kraken/v2.0/extension/{IDENTIFIER}/1.0")

    wait_until(lambda: expect(f"{DOCKER}:1.0" not in images(target)), 60, description="image of tag 1.0 removed")
    assert running_container(target, "1.0.1") == before
    assert f"{DOCKER}:1.0.1" in images(target)


def test_a_legacy_named_container_is_adopted_without_a_restart(target: Target, blueos: BlueOS) -> None:
    prepare_images(target, "2.0")
    install(target, "2.0")
    container_id, name = wait_until(lambda: running_container(target, "2.0"), 300, description="tag 2.0 running")
    # What an older Kraken left behind
    if name != legacy_container_name("2.0"):
        target.shell(f"sudo docker rename {name} {legacy_container_name('2.0')}")
    started = inspect(target, container_id, "{{.State.StartedAt}}")

    target.shell("sudo docker restart blueos-core")
    blueos.wait_ready()

    wait_until(
        lambda: expect(running_container(target, "2.0") == (container_id, container_name("2.0"))),
        180,
        description="container renamed under its new name",
    )
    assert inspect(target, container_id, "{{.State.StartedAt}}") == started
