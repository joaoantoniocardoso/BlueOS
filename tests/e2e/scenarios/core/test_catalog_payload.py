import base64
import json
import urllib.request

import pytest
from blueos.core import BlueOS
from blueos.http import request
from targets.target import Target
from utils.retry import wait_until

IDENTIFIER = "e2e.big"
MANIFEST_PORT = 9198
README_BYTES = 1_000_000
TAGS = ("1", "2", "3")


def manifest() -> list[dict[str, object]]:
    version = {
        "type": "other",
        "images": [{"expanded_size": 1, "platform": {"architecture": "amd64", "os": "linux"}, "digest": None}],
        "authors": [{"name": "e2e", "email": "e2e@example.com"}],
        "filter_tags": [],
        "extra_links": {},
        "permissions": {},
    }
    return [
        {
            "identifier": IDENTIFIER,
            "name": "E2E big readme",
            "website": "http://example.com",
            "docker": "localhost/e2e-big",
            "description": "Extension with a large readme per version",
            "versions": {tag: {**version, "tag": tag} for tag in TAGS},
        }
    ]


@pytest.fixture(name="api", scope="module")
def api_fixture(target: Target, blueos: BlueOS) -> str:
    # The readmes are made inside the VM: 3 MB would not fit in a command line
    script = (
        "import json,sys;m=json.load(sys.stdin);"
        f"[v.update(readme=t*{README_BYTES}) for v in m[0]['versions'].values() for t in [v['tag']]];"
        "json.dump(m,open('/tmp/e2e-big/manifest.json','w'))"
    )
    encoded = base64.b64encode(json.dumps(manifest()).encode()).decode()
    target.shell(
        "sudo docker exec blueos-core mkdir -p /tmp/e2e-big && "
        f'echo {encoded} | base64 -d | sudo docker exec -i blueos-core /usr/blueos/venv/bin/python -c "{script}"'
    )
    target.shell(
        "sudo docker exec -d blueos-core sh -c "
        f"'cd /tmp/e2e-big && exec /usr/blueos/venv/bin/python -m http.server {MANIFEST_PORT} --bind 127.0.0.1'"
    )
    api = f"{blueos.base_url}/kraken/v2.0/manifest"
    source = {"name": "e2e-big", "url": f"http://127.0.0.1:{MANIFEST_PORT}/manifest.json", "enabled": True}
    wait_until(lambda: request("POST", f"{api}/", source), 60, description="manifest source added")
    wait_until(lambda: catalog_has_extension(api), 60, description="catalog served")
    return api


def raw(url: str) -> bytes:
    with urllib.request.urlopen(url, timeout=60) as response:
        body: bytes = response.read()
    return body


def catalog_has_extension(api: str) -> None:
    if IDENTIFIER.encode() not in raw(f"{api}/consolidated"):
        raise RuntimeError(f"{IDENTIFIER} is not in the catalog yet")


def test_catalog_list_omits_readmes(api: str) -> None:
    full = raw(f"{api}/consolidated")
    slim = raw(f"{api}/consolidated?readme=false")

    assert len(full) > README_BYTES * len(TAGS)
    assert len(slim) < len(full) // 10
    entry = next(entry for entry in json.loads(slim) if entry["identifier"] == IDENTIFIER)
    assert all(version["readme"] is None for version in entry["versions"].values())


def test_readme_of_one_version(api: str) -> None:
    body = raw(f"{api}/consolidated/{IDENTIFIER}/2/readme")

    assert body.decode() == "2" * README_BYTES
