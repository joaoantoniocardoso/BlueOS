import base64
import time
import urllib.request
from concurrent.futures import ThreadPoolExecutor

import pytest
from blueos.core import BlueOS
from blueos.http import request
from targets.target import Target

SILENT_PORT = 9199
CALLS = 3
CLIENT_TIMEOUT = 200
ANSWER_LIMIT = 150

# Accepts connections, logs one line per connection and never answers
SILENT_SERVER = f"""
import socket, threading
server = socket.socket()
server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
server.bind(("127.0.0.1", {SILENT_PORT}))
server.listen()
clients = []
while True:
    client, _ = server.accept()
    clients.append(client)
    with open("/tmp/e2e-silent/connections", "a") as log:
        log.write("x\\n")
"""


@pytest.fixture(name="api", scope="module")
def api_fixture(target: Target, blueos: BlueOS) -> str:
    encoded = base64.b64encode(SILENT_SERVER.encode()).decode()
    target.shell(
        "sudo docker exec blueos-core mkdir -p /tmp/e2e-silent && "
        f"echo {encoded} | base64 -d | sudo docker exec -i blueos-core sh -c 'cat > /tmp/e2e-silent/server.py'"
    )
    target.shell(
        "sudo docker exec -d blueos-core sh -c 'cd /tmp/e2e-silent && exec /usr/blueos/venv/bin/python server.py'"
    )
    api = f"{blueos.base_url}/kraken/v2.0/manifest"
    source = {"name": "e2e-silent", "url": f"http://127.0.0.1:{SILENT_PORT}/manifest.json", "enabled": True}
    request("POST", f"{api}/?validate_url=false", source)
    return api


def consolidated(api: str) -> float:
    start = time.monotonic()
    with urllib.request.urlopen(f"{api}/consolidated", timeout=CLIENT_TIMEOUT) as response:
        response.read()
    return time.monotonic() - start


def test_silent_catalog_host_is_cut_off_and_downloaded_once(api: str, target: Target) -> None:
    with ThreadPoolExecutor(CALLS) as pool:
        elapsed = list(pool.map(lambda _: consolidated(api), range(CALLS)))

    log = target.shell("sudo docker exec blueos-core cat /tmp/e2e-silent/connections")
    connections = len(log.split())
    assert max(elapsed) < ANSWER_LIMIT, f"calls took {elapsed} s"
    assert connections == 1, f"{connections} downloads for {CALLS} calls"
