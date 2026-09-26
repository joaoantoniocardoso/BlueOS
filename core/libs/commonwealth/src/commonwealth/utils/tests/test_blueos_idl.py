import json
from pathlib import Path
from typing import Any

import pytest
from commonwealth.utils import blueos_idl

_VECTORS_PATH = Path(__file__).resolve().parents[5] / "idl" / "tests" / "vectors" / "cdr.json"
_CDR_VECTORS: dict[str, Any] = json.loads(_VECTORS_PATH.read_text(encoding="utf-8"))
_IDL_ROOT = Path(__file__).resolve().parents[5] / "idl" / "interfaces"
blueos_idl.ensure_idl_loaded(_IDL_ROOT)


_VECTOR_IDS = [f"{vector['category']}:{vector['schema_name']}" for vector in _CDR_VECTORS["vectors"]]


@pytest.fixture(name="idl_root", autouse=True)
def fixture_idl_root() -> Path:
    return _IDL_ROOT


def test_log_key_matches_rust_api() -> None:
    assert blueos_idl.log_key("kraken") == "blueos/v1/kraken/log"


def test_foxglove_log_round_trip() -> None:
    for vector in _CDR_VECTORS["vectors"]:
        if vector["schema_name"] == "foxglove_msgs/msg/Log" and vector["category"] == "example":
            payload = bytes.fromhex(vector["hex"])
            assert blueos_idl.decode(vector["schema_name"], payload) == vector["decoded"]
            assert blueos_idl.encode(vector["schema_name"], vector["decoded"]).hex() == vector["hex"]
            return
    pytest.fail("foxglove Log example vector missing")


def test_decode_rust_produced_log_payload() -> None:
    for vector in _CDR_VECTORS["vectors"]:
        if vector["schema_name"] == "foxglove_msgs/msg/Log" and vector["category"] == "example":
            payload = bytes.fromhex(vector["hex"])
            decoded = blueos_idl.decode(vector["schema_name"], payload)
            assert decoded["message"] == "hello"
            assert decoded["line"] == 42
            return
    pytest.fail("foxglove Log example vector missing")


def test_command_ack_decode_old_writer_new_reader() -> None:
    for vector in _CDR_VECTORS["vectors"]:
        if vector["schema_name"] == "blueos_msgs/msg/CommandAck" and vector["category"] == "old_writer":
            decoded = blueos_idl.decode(vector["schema_name"], bytes.fromhex(vector["hex"]))
            assert decoded == vector["decoded"]
            return
    pytest.fail("CommandAck old_writer vector missing")


@pytest.mark.parametrize("vector", _CDR_VECTORS["vectors"], ids=_VECTOR_IDS)
def test_decode_rust_vector(vector: dict[str, Any]) -> None:
    assert blueos_idl.decode(vector["schema_name"], bytes.fromhex(vector["hex"])) == vector["decoded"]


@pytest.mark.parametrize("vector", _CDR_VECTORS["vectors"], ids=_VECTOR_IDS)
def test_encode_matches_rust_vector(vector: dict[str, Any]) -> None:
    if vector.get("skip_encode_round_trip"):
        pytest.skip("old writer payload omits trailing fields")
    assert blueos_idl.encode(vector["schema_name"], vector["decoded"]).hex() == vector["hex"]


def test_decode_tolerates_trailing_bytes() -> None:
    hex_payload = next(
        vector["hex"]
        for vector in _CDR_VECTORS["vectors"]
        if vector["schema_name"] == "builtin_interfaces/msg/Time" and vector["category"] == "default"
    )
    payload = bytes.fromhex(hex_payload) + b"\xff\x00\x01"
    decoded = blueos_idl.decode("builtin_interfaces/msg/Time", payload)
    assert decoded == {"sec": 0, "nanosec": 0}


def test_decode_corrupt_payload_raises() -> None:
    hex_payload = next(
        vector["hex"]
        for vector in _CDR_VECTORS["vectors"]
        if vector["schema_name"] == "blueos_example_msgs/msg/PumpState" and vector["category"] == "default"
    )
    payload = bytes.fromhex(hex_payload)
    truncated = payload[:2]
    with pytest.raises(blueos_idl.IdlCodecError):
        blueos_idl.decode("blueos_example_msgs/msg/PumpState", truncated)


@pytest.mark.parametrize(("primitive", "value"), [("byte", -1), ("char", 200)])
def test_byte_is_signed_and_char_is_unsigned(primitive: str, value: int) -> None:
    # pylint: disable=protected-access
    writer = blueos_idl.CdrWriter()
    blueos_idl._write_primitive(writer, primitive, value)
    reader = blueos_idl.CdrReader.from_payload(writer.finish_with_encapsulation())
    assert blueos_idl._read_primitive(reader, primitive) == value


def test_invalid_encapsulation_raises() -> None:
    with pytest.raises(blueos_idl.IdlCodecError):
        blueos_idl.decode("blueos_msgs/msg/CommandAck", b"\xff\xff\xff\xff")
