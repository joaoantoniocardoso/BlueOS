import json
from pathlib import Path

import pytest

from commonwealth.utils import blueos_idl

RUST_LOG_FIXTURE_HEX = (
    "000100000100000002000000020000000600000068656c6c6f000000"
    "05000000746573740000000005000000662e7079000000002a000000"
)
RUST_COMMAND_ACK_OLD_WRITER_HEX = "0001000001000000000000000700000000000000"

_FIXTURES_PATH = Path(__file__).with_name("rust_default_cdr.json")
RUST_DEFAULT_PAYLOADS: dict[str, str] = json.loads(_FIXTURES_PATH.read_text(encoding="utf-8"))


@pytest.fixture(name="idl_root", autouse=True)
def fixture_idl_root() -> Path:
    root = Path(__file__).resolve().parents[5] / "idl" / "interfaces"
    blueos_idl.ensure_idl_loaded(root)
    return root


def test_log_key_matches_rust_api() -> None:
    assert blueos_idl.log_key("kraken") == "blueos/v1/kraken/log"


def test_foxglove_log_round_trip() -> None:
    message = {
        "timestamp": {"sec": 1, "nanosec": 2},
        "level": 2,
        "message": "hello",
        "name": "test",
        "file": "f.py",
        "line": 42,
    }
    payload = blueos_idl.encode("foxglove_msgs/msg/Log", message)
    assert payload.hex() == RUST_LOG_FIXTURE_HEX
    assert blueos_idl.decode("foxglove_msgs/msg/Log", payload) == message


def test_decode_rust_produced_log_payload() -> None:
    payload = bytes.fromhex(RUST_LOG_FIXTURE_HEX)
    decoded = blueos_idl.decode("foxglove_msgs/msg/Log", payload)
    assert decoded["message"] == "hello"
    assert decoded["line"] == 42


def test_command_ack_round_trip_matches_rust() -> None:
    message = {"accepted": True, "job_id": 42, "reason": "queued"}
    payload = blueos_idl.encode("blueos_msgs/msg/CommandAck", message)
    assert blueos_idl.decode("blueos_msgs/msg/CommandAck", payload) == message


def test_command_ack_decode_old_writer_new_reader() -> None:
    payload = bytes.fromhex(RUST_COMMAND_ACK_OLD_WRITER_HEX)
    decoded = blueos_idl.decode("blueos_msgs/msg/CommandAck", payload)
    assert decoded["accepted"] is True
    assert decoded["job_id"] == 7
    assert decoded["reason"] == ""


@pytest.mark.parametrize("schema_name", sorted(RUST_DEFAULT_PAYLOADS.keys()))
def test_decode_rust_default_payload(schema_name: str) -> None:
    payload = bytes.fromhex(RUST_DEFAULT_PAYLOADS[schema_name])
    decoded = blueos_idl.decode(schema_name, payload)
    round_trip = blueos_idl.encode(schema_name, decoded)
    assert blueos_idl.decode(schema_name, round_trip) == decoded


@pytest.mark.parametrize("schema_name", sorted(RUST_DEFAULT_PAYLOADS.keys()))
def test_python_default_round_trip_matches_rust(schema_name: str) -> None:
    defaults = blueos_idl.decode(schema_name, bytes.fromhex(RUST_DEFAULT_PAYLOADS[schema_name]))
    payload = blueos_idl.encode(schema_name, defaults)
    assert payload.hex() == RUST_DEFAULT_PAYLOADS[schema_name]


def test_decode_tolerates_trailing_bytes() -> None:
    payload = bytes.fromhex(RUST_DEFAULT_PAYLOADS["builtin_interfaces/msg/Time"]) + b"\xff\x00\x01"
    decoded = blueos_idl.decode("builtin_interfaces/msg/Time", payload)
    assert decoded == {"sec": 0, "nanosec": 0}


def test_decode_corrupt_payload_raises() -> None:
    payload = bytes.fromhex(RUST_DEFAULT_PAYLOADS["blueos_msgs/msg/CommandAck"])
    truncated = payload[:10]
    with pytest.raises(blueos_idl.IdlCodecError):
        blueos_idl.decode("blueos_msgs/msg/CommandAck", truncated)


def test_invalid_encapsulation_raises() -> None:
    with pytest.raises(blueos_idl.IdlCodecError):
        blueos_idl.decode("blueos_msgs/msg/CommandAck", b"\xff\xff\xff\xff")
