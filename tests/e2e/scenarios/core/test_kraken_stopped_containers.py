import json

from targets.target import Target
from utils.retry import wait_until

EXAMPLE = "williangalvani/blueos-example1"
ORPHAN = "extension-e2eorphan1"


def expect(condition: bool) -> None:
    if not condition:
        raise AssertionError


def containers(target: Target) -> list[str]:
    return target.shell("sudo docker ps -a --format '{{.Names}}' --filter name=extension-").split()


def test_cleaner_deletes_exited_containers_of_unknown_extensions(target: Target) -> None:
    body = {
        "identifier": "williangalvani.example1",
        "name": "Example 1",
        "docker": EXAMPLE,
        "tag": "v1.0.1",
        "enabled": True,
        "permissions": "{}",
    }
    target.shell(
        "curl -sf -X POST -H 'Content-Type: application/json' "
        f"-d '{json.dumps(body)}' http://localhost/kraken/v2.0/extension/ > /dev/null"
    )
    wait_until(lambda: expect(len(containers(target)) == 1), 300, description="extension container")

    target.shell(f"sudo docker run --name {ORPHAN} --entrypoint false {EXAMPLE}:v1.0.1 || true")
    assert ORPHAN in containers(target)

    wait_until(lambda: expect(ORPHAN not in containers(target)), 150, description="orphan container deleted")
    assert len(containers(target)) == 1
