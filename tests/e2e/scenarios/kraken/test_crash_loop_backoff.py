import json
import time
import urllib.request

from blueos.core import BlueOS
from images import CORE
from targets.target import Target

CONTAINER = "extension-test_2fcrashy_3a1_2e0"
OBSERVE_SECONDS = 60
# Attempts 1 to 3 wait 12 s, 14 s and 18 s: starts at about 0, 15, 35 and 55 s. Without backoff there is one every 5 s.
MAX_STARTS = 5


def test_crash_loop_backs_off(blueos: BlueOS, target: Target) -> None:
    # The core image has a shell and is already in the VM, so the extension needs no registry
    target.shell(f"sudo docker tag {CORE} test/crashy:1.0")
    body = {
        "identifier": "test.crashy",
        "name": "crashy",
        "docker": "test/crashy",
        "tag": "1.0",
        "enabled": True,
        "pull": "if-missing",
        "permissions": json.dumps({"Entrypoint": ["sh", "-c", "sleep 1"]}),
    }
    install = urllib.request.Request(
        f"{blueos.base_url}/kraken/v2.0/extension/",
        data=json.dumps(body).encode(),
        method="POST",
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(install, timeout=120) as response:
        response.read()

    started = int(time.time())
    time.sleep(OBSERVE_SECONDS)
    events = target.shell(
        f"sudo docker events --since {started} --until {started + OBSERVE_SECONDS} "
        f"--filter container={CONTAINER} --filter event=start"
    )
    starts = len(events.splitlines())
    assert 1 <= starts <= MAX_STARTS, f"{starts} starts in {OBSERVE_SECONDS} s"
