from collections.abc import Iterator

import pytest
from blueos.core import BlueOS
from targets.target import Target

CLOCK_STEP_HOURS = 13


@pytest.fixture(name="stepped_clock")
def stepped_clock_fixture(target: Target) -> Iterator[None]:
    # A Pi without a real-time clock steps its clock at NTP sync, so stop the NTP client to keep the step
    target.shell("sudo systemctl stop systemd-timesyncd")
    target.shell(f"sudo date -s '+{CLOCK_STEP_HOURS} hours'")
    try:
        yield
    finally:
        target.shell(f"sudo date -s '-{CLOCK_STEP_HOURS} hours'")
        target.shell("sudo systemctl start systemd-timesyncd")


@pytest.mark.usefixtures("stepped_clock")
def test_container_uptime_ignores_clock_step(blueos: BlueOS, target: Target) -> None:
    host_uptime = float(target.shell("cut -d' ' -f1 /proc/uptime"))
    containers = blueos.containers()
    assert containers
    for container in containers:
        assert 0 <= container["uptime_seconds"] <= host_uptime + 5, container
