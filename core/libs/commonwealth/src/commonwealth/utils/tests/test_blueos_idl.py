from pathlib import Path

import pytest

from commonwealth.utils import blueos_idl

RUST_LOG_FIXTURE_HEX = (
    "000100000100000002000000020000000600000068656c6c6f000000"
    "05000000746573740000000005000000662e7079000000002a000000"
)
RUST_COMMAND_ACK_FIXTURE_HEX = (
    "0001000001000000000000002a00000000000000070000007175657565640000"
)
RUST_COMMAND_ACK_OLD_WRITER_HEX = "000100000100000000000000070000000000000000"


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
