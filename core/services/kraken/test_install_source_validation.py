# isort: skip_file
import json
import sys
from pathlib import Path
from typing import Any, Dict, List, Tuple
from unittest import mock

import pytest

_KRAKEN_DIR = str(Path(__file__).resolve().parent)
sys.path.insert(0, _KRAKEN_DIR)
# Root pytest also collects the other services, whose top-level `config` and
# `settings` modules would otherwise be the ones imported here.
_saved = {module_name: sys.modules.get(module_name) for module_name in ("config", "settings", "utils")}
for module_name in _saved:
    sys.modules.pop(module_name, None)

from extension.extension import Extension  # noqa: E402
from extension.models import ExtensionInstallSource, ExtensionSource  # noqa: E402
from pydantic import ValidationError  # noqa: E402

for module_name, module in _saved.items():
    if module is not None:
        sys.modules[module_name] = module
    else:
        sys.modules.pop(module_name, None)

VALID: Dict[str, Any] = {
    "identifier": "ab.cd",
    "tag": "1.0.0",
    "name": "Name",
    "docker": "bluerobotics/blueos-thing",
    "enabled": True,
    "permissions": json.dumps({"ExposedPorts": {"80/tcp": {}}}),
    "user_permissions": "",
}


def build_app() -> Any:
    # The root environment does not carry kraken's own dependencies; the route tests need them.
    pytest.importorskip("fastapi_versioning")
    # pylint: disable=import-outside-toplevel
    from api.v1.routers.extension import extension_router_v1
    from api.v2.routers.extension import extension_router_v2
    from fastapi import FastAPI

    app = FastAPI()
    app.include_router(extension_router_v1)
    app.include_router(extension_router_v2)
    return app


async def post(path: str, body: Dict[str, Any]) -> int:
    app = build_app()
    payload = json.dumps(body).encode()
    sent: List[Dict[str, Any]] = []
    headers: List[Tuple[bytes, bytes]] = [(b"content-type", b"application/json")]
    path_only, _, query = path.partition("?")
    scope = {
        "type": "http",
        "asgi": {"version": "3.0"},
        "http_version": "1.1",
        "method": "POST",
        "path": path_only,
        "raw_path": path_only.encode(),
        "query_string": query.encode(),
        "headers": headers,
        "server": ("test", 80),
        "client": ("test", 1),
        "scheme": "http",
        "root_path": "",
    }
    pending = [{"type": "http.request", "body": payload, "more_body": False}]

    async def receive() -> Dict[str, Any]:
        return pending.pop(0) if pending else {"type": "http.disconnect"}

    async def send(message: Dict[str, Any]) -> None:
        sent.append(message)

    await app(scope, receive, send)
    return int(next(message["status"] for message in sent if message["type"] == "http.response.start"))


ROUTES = [
    "/extension/install",
    "/extension/",
    "/extension/upload/finalize?temp_tag=temp-x",
]

BAD_FIELDS: List[Tuple[str, Any]] = [
    ("tag", ""),
    ("tag", " 1"),
    ("docker", ""),
    ("docker", "has space"),
    ("docker", "Bluerobotics/Upper"),
    ("docker", "repo:1.0"),
    ("docker", "repo//x"),
    ("name", ""),
    ("name", "   "),
    ("identifier", ""),
    ("identifier", " ab.cd"),
    ("identifier", "ab.cd/evil"),
    ("identifier", "ab.cd?x"),
    ("identifier", "ab.cd#f"),
    ("identifier", "ab.cd.."),
    ("identifier", "a%2Fb"),
    ("identifier", "nodot"),
    ("permissions", "not json"),
    ("permissions", ""),
    ("permissions", "[]"),
    ("permissions", "{bad"),
    ("user_permissions", "not json"),
    ("user_permissions", "[]"),
]


@pytest.fixture(autouse=True)
def fake_install() -> Any:
    async def fake(*_args: Any, **_kwargs: Any) -> Any:
        yield "ok"

    with mock.patch.object(Extension, "install", fake), mock.patch.object(
        Extension, "from_settings", mock.AsyncMock(return_value=[])
    ):
        yield


@pytest.mark.parametrize("field,value", BAD_FIELDS)
def test_model_rejects_invalid_value(field: str, value: Any) -> None:
    with pytest.raises(ValidationError):
        ExtensionInstallSource(**{**VALID, field: value})


@pytest.mark.asyncio
@pytest.mark.parametrize("route", ROUTES)
@pytest.mark.parametrize("field,value", BAD_FIELDS)
async def test_invalid_body_is_rejected_with_422(route: str, field: str, value: Any) -> None:
    assert await post(route, {**VALID, field: value}) == 422


@pytest.mark.asyncio
@pytest.mark.parametrize("route", ROUTES)
@pytest.mark.parametrize(
    "overrides",
    [
        {},
        {"identifier": "Blue-Robotics.Ping_viewer2"},
        {"docker": "localhost:5000/team/img"},
        {"docker": "registry.example.com/a/b.c_d__e-f"},
        {"docker": "nginx"},
        {"permissions": "{}"},
        {"user_permissions": "{}"},
        {"name": " Padded name "},
    ],
)
async def test_valid_body_is_accepted(route: str, overrides: Dict[str, Any]) -> None:
    status = await post(route, {**VALID, **overrides})
    # The finalize route finds no temporary extension (404): it got past validation.
    assert status in (200, 201, 404), status


def test_null_permissions_from_the_catalog_become_an_empty_object() -> None:
    # The catalog entry williangalvani.dashcam v0.0.3 has "permissions": null.
    source = ExtensionInstallSource(**{**VALID, "permissions": "null"})
    assert source.permissions == "{}"


def test_values_already_on_disk_still_load() -> None:
    # Settings written by an older version go through ExtensionSource, which must stay lenient.
    source = ExtensionSource(**{**VALID, "identifier": "ab.cd/evil", "docker": "Upper", "permissions": "not json"})
    assert source.identifier == "ab.cd/evil"
