import shlex

from blueos.core import BlueOS
from targets.target import Target
from utils.retry import wait_until

SETTINGS_FOLDER = "/root/.config/kraken"
SETTINGS_FILE = f"{SETTINGS_FOLDER}/settings-2.json"
TRUNCATED = '{"VERSION": 2, "extensions": [{"identifier": "foo'


def in_core(target: Target, command: str) -> str:
    return target.shell(f"sudo docker exec blueos-core sh -c {shlex.quote(command)}")


def expect(condition: bool) -> None:
    if not condition:
        raise AssertionError


def test_truncated_settings_are_kept_aside_and_logged(target: Target, blueos: BlueOS) -> None:
    wait_until(lambda: in_core(target, f"test -s {SETTINGS_FILE} && echo ok"), 120, description="kraken settings file")

    in_core(target, f"printf %s {shlex.quote(TRUNCATED)} > {SETTINGS_FILE}")
    target.shell("sudo docker restart blueos-core")
    blueos.wait_ready()

    wait_until(
        lambda: expect(in_core(target, f"ls {SETTINGS_FOLDER}/settings-2.json.invalid-* | wc -l").strip() == "1"),
        180,
        description="settings copy kept aside",
    )
    kept = in_core(target, f"cat {SETTINGS_FOLDER}/settings-2.json.invalid-*")
    assert kept == TRUNCATED
    in_core(target, f"python3 -c \"import json; json.load(open('{SETTINGS_FILE}'))\"")

    log = in_core(target, "grep -rh 'was kept as' /var/logs/blueos/services/kraken || true")
    assert f"Invalid settings file {SETTINGS_FILE}" in log
    assert "settings-2.json.invalid-" in log
