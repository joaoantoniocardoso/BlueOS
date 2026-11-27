from collections.abc import Iterator

import pytest
from blueos.core import BlueOS
from blueos.http import request
from targets.target import Target
from utils.retry import wait_until

BLOCK_DNS_RULES = ("-p udp --dport 53 -j REJECT", "-p tcp --dport 53 -j REJECT")
OFFLINE_ROUTES = (
    ("GET", "manifest/consolidated"),
    ("POST", "extension/org.ext/install"),
    ("PUT", "extension/org.ext"),
)
KRAKEN_MAIN = "python3 /home/pi/services/kraken/main.py"


def kraken_pid(target: Target) -> str:
    return target.shell(f"sudo docker exec blueos-core pgrep -f '{KRAKEN_MAIN}'").strip()


def restart_kraken(target: Target, blueos: BlueOS) -> None:
    # run-service restarts kraken after a kill, not after a clean exit, and the new process has no manifest cache
    old_pid = kraken_pid(target)
    target.shell(f"sudo docker exec blueos-core kill -9 {old_pid}")

    def kraken_is_new_and_ready() -> None:
        if kraken_pid(target) == old_pid:
            raise RuntimeError("kraken is still the old process")
        request("GET", f"{blueos.base_url}/kraken/v2.0/manifest/?data=false")

    wait_until(
        kraken_is_new_and_ready,
        120,
        errors=(OSError, RuntimeError),
        description="kraken restart",
    )


@pytest.fixture(name="dns_blocked")
def dns_blocked_fixture(target: Target) -> Iterator[None]:
    for rule in BLOCK_DNS_RULES:
        target.shell(f"sudo iptables -I OUTPUT {rule}")
    yield
    for rule in BLOCK_DNS_RULES:
        target.shell(f"sudo iptables -D OUTPUT {rule}")


@pytest.mark.usefixtures("dns_blocked")
def test_catalog_install_and_update_answer_502_when_every_source_is_unreachable(target: Target, blueos: BlueOS) -> None:
    restart_kraken(target, blueos)

    for method, path in OFFLINE_ROUTES:
        with pytest.raises(RuntimeError, match=r"failed \(502\).*backend is offline"):
            request(method, f"{blueos.base_url}/kraken/v2.0/{path}")


def test_catalog_recovers_when_the_network_is_back(target: Target, blueos: BlueOS) -> None:
    restart_kraken(target, blueos)

    assert request("GET", f"{blueos.base_url}/kraken/v2.0/manifest/consolidated")
