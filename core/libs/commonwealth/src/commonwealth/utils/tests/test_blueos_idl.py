import json
import logging
from collections.abc import Callable, Generator
from pathlib import Path
from typing import Any, cast

import pytest
from commonwealth.utils import blueos_idl

LIBS_ROOT = Path(__file__).resolve().parents[5]
CDR_VECTORS_PATH = LIBS_ROOT / "idl" / "tests" / "vectors" / "cdr.json"
KEYS_VECTORS_PATH = LIBS_ROOT / "api" / "tests" / "vectors" / "keys.json"
IDL_INTERFACES_ROOT = LIBS_ROOT / "idl" / "interfaces"
INTERFACE_KINDS_ROOT = LIBS_ROOT / "idl" / "codegen" / "tests" / "fixtures" / "interface_kinds" / "interfaces"


@pytest.fixture(autouse=True)
def reset_idl_cache() -> Generator[None, None, None]:
    blueos_idl.reset_runtime_state()
    yield
    blueos_idl.reset_runtime_state()


@pytest.fixture(autouse=True)
def idl_interfaces(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("BLUEOS_IDL_INTERFACES", str(IDL_INTERFACES_ROOT))
    blueos_idl.ensure_idl_loaded(IDL_INTERFACES_ROOT)


def load_json(path: Path) -> dict[str, Any]:
    return cast(dict[str, Any], json.loads(path.read_text(encoding="utf-8")))


def decode_hex(hex_payload: str) -> bytes:
    return bytes.fromhex(hex_payload)


def encode_hex(payload: bytes) -> str:
    return payload.hex()


def _assert_service_cases(
    cases: list[dict[str, str]],
    builder: Callable[[str], str],
) -> None:
    for case in cases:
        assert builder(case["service"]) == case["expected"]


def _assert_named_cases(
    cases: list[dict[str, str]],
    builder: Callable[[str, str], str],
) -> None:
    for case in cases:
        assert builder(case["service"], case["name"]) == case["expected"]


def test_keys_vectors_match_python_helpers() -> None:
    vectors = load_json(KEYS_VECTORS_PATH)
    constants = vectors["constants"]
    assert blueos_idl.API_VERSION == constants["api_version"]
    assert blueos_idl.KEY_PREFIX == constants["key_prefix"]
    assert blueos_idl.ENCODING_APPLICATION_CDR == constants["encoding_application_cdr"]
    assert blueos_idl.TYPE_HASH_ATTACHMENT_KEY == constants["type_hash_attachment_key"]

    _assert_service_cases(vectors["service_liveliness_key"], blueos_idl.service_liveliness_key)
    _assert_named_cases(vectors["command_key"], blueos_idl.command_key)
    _assert_named_cases(vectors["state_key"], blueos_idl.state_key)
    _assert_named_cases(vectors["event_key"], blueos_idl.event_key)
    _assert_named_cases(vectors["query_key"], blueos_idl.query_key)
    _assert_service_cases(vectors["jobs_key"], blueos_idl.jobs_key)
    _assert_service_cases(vectors["settings_key"], blueos_idl.settings_key)
    _assert_service_cases(vectors["log_key"], blueos_idl.log_key)
    for case in vectors["extension_log_key"]:
        assert blueos_idl.extension_log_key(case["service"], case["extension_identifier"]) == case["expected"]
    _assert_service_cases(vectors["http_gateway_prefix"], blueos_idl.http_gateway_prefix)
    _assert_service_cases(vectors["status_state_key"], blueos_idl.status_state_key)
    _assert_service_cases(vectors["info_query_key"], blueos_idl.info_query_key)
    for case in vectors["cdr_encoding"]:
        assert blueos_idl.cdr_encoding(case["schema_name"]) == case["expected"]


def test_cdr_vectors_match_python_codec() -> None:
    vectors = load_json(CDR_VECTORS_PATH)["vectors"]
    for vector in vectors:
        payload = decode_hex(vector["hex"])
        decoded = blueos_idl.decode(vector["schema_name"], payload)
        assert decoded == vector["decoded"], vector["schema_name"]
        if vector.get("skip_encode_round_trip"):
            continue
        if vector["category"] == "python_producer":
            continue
        if vector["category"] == "default":
            reencoded = blueos_idl.encode(vector["schema_name"], {})
            assert encode_hex(reencoded) == vector["hex"], vector["schema_name"]
        else:
            reencoded = blueos_idl.encode(vector["schema_name"], vector["decoded"])
            assert encode_hex(reencoded) == vector["hex"], vector["schema_name"]


@pytest.mark.parametrize("hostile_sequence", ["capabilities", "endpoints"])
def test_service_info_rejects_hostile_sequence_length(hostile_sequence: str) -> None:
    writer = blueos_idl.CdrWriter()
    for text in ("recorder", "1.0.0", "dev"):
        writer.write_string(text)
    if hostile_sequence == "endpoints":
        writer.write_u32(0)
    writer.write_u32(0xFFFF_FFFF)
    with pytest.raises(blueos_idl.IdlCodecError):
        blueos_idl.decode("blueos_msgs/msg/ServiceInfo", writer.finish_with_encapsulation())


def test_image_idl_interfaces_root_matches_copy_libs() -> None:
    assert blueos_idl.IMAGE_IDL_INTERFACES_ROOT == Path("/home/pi/libs/idl/interfaces")


def test_missing_interfaces_log_once(caplog: pytest.LogCaptureFixture, tmp_path: Path) -> None:
    missing_root = tmp_path / "missing"
    caplog.set_level(logging.WARNING)
    blueos_idl.reset_runtime_state()
    for _ in range(3):
        with pytest.raises(FileNotFoundError):
            blueos_idl.ensure_idl_loaded(missing_root)
    warning_records = [record for record in caplog.records if record.levelno == logging.WARNING]
    assert len(warning_records) == 1


def test_missing_msg_definitions_log_once(caplog: pytest.LogCaptureFixture, tmp_path: Path) -> None:
    interfaces_root = tmp_path / "interfaces"
    package_root = interfaces_root / "broken_msgs" / "msg"
    package_root.mkdir(parents=True)
    (package_root / "NeedsMissing.msg").write_text(
        "missing_pkg/msg/Missing nested\n",
        encoding="utf-8",
    )
    (package_root / "AlsoNeedsMissing.msg").write_text(
        "missing_pkg/msg/Missing other\n",
        encoding="utf-8",
    )
    caplog.set_level(logging.WARNING)
    blueos_idl.reset_runtime_state()
    for _ in range(2):
        with pytest.raises(blueos_idl.IdlCodecError):
            blueos_idl.ensure_idl_loaded(interfaces_root)
    warning_records = [record for record in caplog.records if record.levelno == logging.WARNING]
    assert len(warning_records) == 1
    assert "missing_pkg/msg/Missing" in warning_records[0].message


def test_srv_and_action_parts_are_messages() -> None:
    blueos_idl.reset_runtime_state()
    blueos_idl.ensure_idl_loaded(INTERFACE_KINDS_ROOT)

    assert not blueos_idl.decode("fixture_msgs/action/Drain_Goal", decode_hex("0001000000"))
    goal = blueos_idl.encode("fixture_msgs/action/Fill_Goal", {"level": 1.0, "rate": 2.0})
    assert encode_hex(goal) == "000100000000803f00000040"
    response = {"level": 0.5, "progress": {"done": 1, "total": 2}}
    payload = blueos_idl.encode("fixture_msgs/srv/Measure_Response", response)
    assert blueos_idl.decode("fixture_msgs/srv/Measure_Response", payload) == response
