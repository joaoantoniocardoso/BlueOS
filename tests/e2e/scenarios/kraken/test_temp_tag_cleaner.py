from collections.abc import Iterator

import pytest
from targets.target import Target
from utils.retry import wait_until

FOREIGN_TAG = "e2etest/image:temp-1"
KRAKEN_TEMP_TAG = "e2etest/image:temp-0123abcd"


def assert_cleaned(target: Target) -> None:
    assert KRAKEN_TEMP_TAG not in image_tags(target)


def image_tags(target: Target) -> list[str]:
    return target.shell("sudo docker images --format '{{.Repository}}:{{.Tag}}'").split()


@pytest.fixture(name="tagged_images")
def tagged_images_fixture(target: Target) -> Iterator[None]:
    image_id = target.shell("sudo docker images -q | head -1").strip()
    for tag in (FOREIGN_TAG, KRAKEN_TEMP_TAG):
        target.shell(f"sudo docker tag {image_id} {tag}")
    try:
        yield
    finally:
        target.shell(f"sudo docker rmi {FOREIGN_TAG} {KRAKEN_TEMP_TAG} || true")


@pytest.mark.usefixtures("tagged_images")
def test_cleaner_removes_only_kraken_temp_tags(target: Target) -> None:
    # The cleaner runs every 60 s: once the orphan temp tag is gone, a pass has run
    wait_until(
        lambda: assert_cleaned(target),
        timeout=150,
        errors=(AssertionError,),
        description="cleaner to remove the orphan kraken temp tag",
    )
    assert FOREIGN_TAG in image_tags(target)
