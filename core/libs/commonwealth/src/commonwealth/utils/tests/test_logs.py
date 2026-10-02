import json
from pathlib import Path
from typing import Any, cast

from commonwealth.utils import blueos_idl
from commonwealth.utils.logs import LOG_SCHEMA, log_publisher_options

LIBS_ROOT = Path(__file__).resolve().parents[5]
CDR_VECTORS_PATH = LIBS_ROOT / "idl" / "tests" / "vectors" / "cdr.json"
IDL_INTERFACES_ROOT = LIBS_ROOT / "idl" / "interfaces"

PYTHON_PRODUCER_LOG_DECODED: dict[str, Any] = {
    "timestamp": {"sec": 9, "nanosec": 8},
    "level": 3,
    "message": "python commonwealth log",
    "name": "wifi-manager",
    "file": "main.py",
    "line": 17,
}

PYTHON_PRODUCER_LOG_HEX = (
    "0001000009000000080000000300000018000000707974686f6e20636f6d6d6f6e7765616c7468206c6f67000d000000776966692d6d616e6167657200000000080000006d61696e2e70790011000000"
)


def test_log_publisher_uses_cdr_foxglove_schema() -> None:
    encoding = log_publisher_options()["encoding"]
    assert str(encoding) == f"application/cdr;{LOG_SCHEMA}"


def test_python_producer_log_vector_in_shared_cdr_json() -> None:
    vectors = cast(dict[str, Any], json.loads(CDR_VECTORS_PATH.read_text(encoding="utf-8")))
    matching = [
        vector
        for vector in vectors["vectors"]
        if vector["schema_name"] == LOG_SCHEMA and vector.get("category") == "python_producer"
    ]
    assert len(matching) == 1
    vector = matching[0]
    assert vector["hex"] == PYTHON_PRODUCER_LOG_HEX
    assert vector["decoded"] == PYTHON_PRODUCER_LOG_DECODED
    assert vector.get("skip_encode_round_trip") is True


def test_python_producer_log_encodes_shared_vector() -> None:
    blueos_idl.ensure_idl_loaded(IDL_INTERFACES_ROOT)
    payload = blueos_idl.encode(LOG_SCHEMA, PYTHON_PRODUCER_LOG_DECODED)
    assert payload.hex() == PYTHON_PRODUCER_LOG_HEX
    assert blueos_idl.decode(LOG_SCHEMA, payload) == PYTHON_PRODUCER_LOG_DECODED


def test_log_key_not_legacy_services_path() -> None:
    assert blueos_idl.log_key("wifi-manager") == "blueos/v1/wifi-manager/log"
    assert not blueos_idl.log_key("wifi-manager").startswith("services/")
