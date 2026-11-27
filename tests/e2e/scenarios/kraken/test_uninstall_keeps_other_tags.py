import json
from collections.abc import Iterator

import pytest
from blueos.core import BlueOS
from blueos.http import request
from targets.target import Target
from utils.retry import wait_until

IDENTIFIER = "sil.sharedimage"
REPOSITORY = "silext/sharedimage"
CONTAINER_PREFIX = "extension-silextsharedimage"
SHARED_TAG = "shared"


def image_exists(target: Target, reference: str) -> bool:
    try:
        target.shell(f"sudo docker image inspect {reference}")
    except Exception:  # pylint: disable=broad-except
        return False
    return True


def build_image(target: Target, base: str, reference: str, label: str) -> None:
    target.shell(f"printf 'FROM {base}\\nLABEL sil={label}\\n' | sudo docker build -q -t {reference} -")


def require_containers(target: Target, present: bool) -> list[str]:
    containers = target.shell(f"sudo docker ps -a --filter name={CONTAINER_PREFIX} --format '{{{{.Names}}}}'").split()
    if bool(containers) != present:
        raise RuntimeError(f"containers {CONTAINER_PREFIX}: {containers}")
    return containers


@pytest.fixture(name="extension_with_moved_tag")
def extension_with_moved_tag_fixture(blueos: BlueOS, target: Target) -> Iterator[None]:
    core_image = target.shell("sudo docker inspect blueos-core --format '{{.Config.Image}}'").strip()
    # A new image id that no other container uses, so that Docker can delete it
    build_image(target, core_image, f"{REPOSITORY}:installed", "original")
    target.shell(f"sudo docker tag {REPOSITORY}:installed {REPOSITORY}:{SHARED_TAG}")
    # The pull fails because the repository does not exist, but the image is already local, so the install holds
    request(
        "POST",
        f"{blueos.base_url}/kraken/v2.0/extension/",
        {
            "identifier": IDENTIFIER,
            "name": "Shared image",
            "docker": REPOSITORY,
            "tag": "installed",
            "permissions": json.dumps({"Entrypoint": ["sleep", "infinity"]}),
            "enabled": True,
            "user_permissions": "",
        },
    )
    wait_until(lambda: require_containers(target, True), 120, description="extension container")
    # Docker then reports the id of the old image for the container, as after a pull that moves the tag
    build_image(target, core_image, f"{REPOSITORY}:installed", "moved")
    yield
    for tag in ("installed", SHARED_TAG):
        target.shell(f"sudo docker rmi -f {REPOSITORY}:{tag} || true")


@pytest.mark.usefixtures("extension_with_moved_tag")
def test_uninstall_keeps_other_tags_of_the_same_image(blueos: BlueOS, target: Target) -> None:
    request("DELETE", f"{blueos.base_url}/kraken/v2.0/extension/{IDENTIFIER}/installed")

    wait_until(lambda: require_containers(target, False), 60, description="container removal")
    assert image_exists(target, f"{REPOSITORY}:{SHARED_TAG}")
