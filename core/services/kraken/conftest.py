import importlib.machinery
import importlib.util
import os
import sys
import tempfile
import types
from pathlib import Path
from unittest import mock

import fastapi.dependencies.utils as fastapi_utils
from fastapi.routing import APIRoute

sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "libs" / "commonwealth"))
# Root pytest also collects the other services, whose top-level `config`, `settings` and `utils` modules would
# otherwise be the ones kraken imports. Kraken's modules are imported once, here, and the tests reuse them.
_saved = {name: sys.modules.pop(name, None) for name in ("config", "settings", "utils")}
_saved_xdg = os.environ.get("XDG_CONFIG_HOME")
# The settings managers are created at import, keep them away from the real configuration folder
os.environ["XDG_CONFIG_HOME"] = tempfile.mkdtemp()

# The root test environment carries neither fastapi-versioning nor python-multipart, which the routers need to load.
# Stub them once for the whole session, so no test module has to do it, and tests that load routers do not clash.
if importlib.util.find_spec("fastapi_versioning") is None:
    _stub = types.ModuleType("fastapi_versioning")
    _stub.__spec__ = importlib.machinery.ModuleSpec("fastapi_versioning", None)
    _stub.versioned_api_route = lambda *_args, **_kwargs: APIRoute  # type: ignore[attr-defined]
    _stub.VersionedFastAPI = mock.MagicMock()  # type: ignore[attr-defined]
    sys.modules["fastapi_versioning"] = _stub
if importlib.util.find_spec("multipart") is None:
    fastapi_utils.check_file_field = lambda _field: None

import kraken  # noqa: E402,F401  # pylint: disable=wrong-import-position,unused-import

for _name, _module in _saved.items():
    if _module is not None:
        sys.modules[_name] = _module
    else:
        sys.modules.pop(_name, None)
if _saved_xdg is None:
    os.environ.pop("XDG_CONFIG_HOME", None)
else:
    os.environ["XDG_CONFIG_HOME"] = _saved_xdg
