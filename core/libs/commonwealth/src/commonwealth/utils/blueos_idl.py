"""Runtime ROS 2 `.msg` parsing and CDR codec for the versioned BlueOS zenoh API (D-05, D-06, D-17).

Key helpers mirror `core/libs/api/src/lib.rs` (single source of truth for key layout).
"""

from __future__ import annotations

import os
import re
import struct
from pathlib import Path
from typing import Any

from rosbags.interfaces import Nodetype
from rosbags.serde.errors import SerdeError
from rosbags.typesys import Stores, get_types_from_msg, get_typestore

API_VERSION = "v1"
KEY_PREFIX = "blueos/v1"

ENCODING_APPLICATION_CDR = "application/cdr"
TYPE_HASH_ATTACHMENT_KEY = "blueos.type_hash"

ENCODING_APPLICATION_JSON = "application/json"

# REST-over-zenoh queryables use JSON payloads (D-18 gateway), not IDL CDR.


def service_liveliness_key(service: str) -> str:
    return f"{KEY_PREFIX}/services/{service}"


def service_info_key(service: str) -> str:
    return f"{KEY_PREFIX}/services/{service}/info"


def command_key(service: str, name: str) -> str:
    return f"{KEY_PREFIX}/{service}/command/{name}"


def state_key(service: str, name: str) -> str:
    return f"{KEY_PREFIX}/{service}/state/{name}"


def event_key(service: str, name: str) -> str:
    return f"{KEY_PREFIX}/{service}/event/{name}"


def query_key(service: str, name: str) -> str:
    return f"{KEY_PREFIX}/{service}/query/{name}"


def jobs_key(service: str) -> str:
    return f"{KEY_PREFIX}/{service}/jobs"


def settings_key(service: str) -> str:
    return f"{KEY_PREFIX}/{service}/settings"


def log_key(service: str) -> str:
    return f"{KEY_PREFIX}/{service}/log"


def status_state_key(service: str) -> str:
    return state_key(service, "status")


def info_query_key(service: str) -> str:
    return query_key(service, "info")


def http_gateway_prefix(service: str) -> str:
    return f"{KEY_PREFIX}/{service}/http"


def extension_log_key(service: str, extension_identifier: str) -> str:
    safe_identifier = extension_identifier.replace("/", "_").replace(" ", "_")
    return f"{log_key(service)}/extension/{safe_identifier}"


def cdr_encoding(schema_name: str) -> str:
    return f"{ENCODING_APPLICATION_CDR};{schema_name}"


def default_idl_interfaces_root() -> Path:
    env_path = os.environ.get("BLUEOS_IDL_INTERFACES")
    if env_path:
        return Path(env_path)
    repo_candidate = Path(__file__).resolve().parents[4] / "idl" / "interfaces"
    if repo_candidate.is_dir():
        return repo_candidate
    image_candidate = Path("/home/pi/libs/idl/interfaces")
    if image_candidate.is_dir():
        return image_candidate
    raise FileNotFoundError(
        "BlueOS IDL interfaces directory not found; set BLUEOS_IDL_INTERFACES or install libs/idl"
    )


_MSG_TEXT: dict[str, str] = {}
_STORE = get_typestore(Stores.EMPTY)
_TRUNCATED_STORES: dict[tuple[str, int], Any] = {}


def _load_interfaces(root: Path) -> None:
    global _MSG_TEXT, _STORE, _TRUNCATED_STORES
    types: dict[str, Any] = {}
    messages: dict[str, str] = {}
    for path in sorted(root.rglob("*.msg")):
        relative = path.relative_to(root).with_suffix("").as_posix()
        text = path.read_text(encoding="utf-8")
        messages[relative] = text
        types.update(get_types_from_msg(text, relative))
    store = get_typestore(Stores.EMPTY)
    store.register(types)
    _MSG_TEXT = messages
    _STORE = store
    _TRUNCATED_STORES = {}


def ensure_idl_loaded(interfaces_root: Path | None = None) -> None:
    if _MSG_TEXT:
        return
    root = interfaces_root or default_idl_interfaces_root()
    _load_interfaces(root)


def _message_fielddefs(schema_name: str) -> list[tuple[str, Any]]:
    constants, fields = _STORE.fielddefs[schema_name]
    _ = constants
    return list(fields)


def _field_names(schema_name: str) -> list[str]:
    return [name for name, _desc in _message_fielddefs(schema_name)]


def _truncate_msg_text(schema_name: str, field_count: int) -> str:
    original = _MSG_TEXT[schema_name]
    allowed_names = set(_field_names(schema_name)[:field_count])
    output_lines: list[str] = []
    for line in original.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            output_lines.append(line)
            continue
        if _is_constant_line(stripped):
            output_lines.append(line)
            continue
        field_name = stripped.split()[-1]
        if field_name in allowed_names:
            output_lines.append(line)
    return "\n".join(output_lines) + "\n"


def _is_constant_line(line: str) -> bool:
    return bool(re.match(r"^[A-Za-z_/][\w/]*\s+\w+=", line))


def _store_for_field_count(schema_name: str, field_count: int) -> Any:
    cache_key = (schema_name, field_count)
    cached = _TRUNCATED_STORES.get(cache_key)
    if cached is not None:
        return cached
    full_count = len(_field_names(schema_name))
    if field_count >= full_count:
        return _STORE
    types: dict[str, Any] = {}
    for name, text in _MSG_TEXT.items():
        if name == schema_name:
            types.update(get_types_from_msg(_truncate_msg_text(name, field_count), name))
        else:
            types.update(get_types_from_msg(text, name))
    store = get_typestore(Stores.EMPTY)
    store.register(types)
    _TRUNCATED_STORES[cache_key] = store
    return store


def _nested_type_name(descriptor: Any) -> str:
    if isinstance(descriptor, str):
        return descriptor
    return str(descriptor[1])


def _base_type_name(descriptor: Any) -> str:
    if isinstance(descriptor, tuple):
        return str(descriptor[0])
    return str(descriptor)


def _default_value(field_desc: Any) -> Any:
    node_type, descriptor = field_desc
    if node_type == Nodetype.NAME:
        nested_type = _nested_type_name(descriptor)
        nested_fields = _message_fielddefs(nested_type)
        return _dict_from_defaults(nested_type, nested_fields)
    if node_type == Nodetype.BASE:
        base_type = _base_type_name(descriptor)
        if base_type == "string":
            return ""
        if base_type == "bool":
            return False
        if base_type in {"float", "float32", "float64", "double"}:
            return 0.0
        return 0
    if node_type == Nodetype.ARRAY:
        return []
    return 0


def _dict_from_defaults(schema_name: str, fielddefs: list[tuple[str, Any]]) -> dict[str, Any]:
    values: dict[str, Any] = {}
    for name, desc in fielddefs:
        values[name] = _default_value(desc)
    return values


def _message_to_dict(message: Any) -> dict[str, Any]:
    schema_name = message.__msgtype__
    result: dict[str, Any] = {}
    for name, desc in _message_fielddefs(schema_name):
        if not hasattr(message, name):
            continue
        value = getattr(message, name)
        node_type, descriptor = desc
        if node_type == Nodetype.NAME:
            result[name] = _message_to_dict(value)
        elif node_type == Nodetype.ARRAY:
            nested = descriptor[0]
            if nested[0] == Nodetype.NAME:
                result[name] = [_message_to_dict(entry) for entry in value]
            else:
                result[name] = list(value)
        else:
            result[name] = value
    return result


def _dict_to_message(schema_name: str, value: dict[str, Any]) -> Any:
    message_type = _STORE.types[schema_name]
    kwargs: dict[str, Any] = {}
    for name, desc in _message_fielddefs(schema_name):
        if name not in value:
            kwargs[name] = _default_value(desc)
            continue
        field_value = value[name]
        node_type, descriptor = desc
        if node_type == Nodetype.NAME:
            nested_type = _nested_type_name(descriptor)
            kwargs[name] = _dict_to_message(nested_type, field_value)
        elif node_type == Nodetype.ARRAY:
            nested = descriptor[0]
            if nested[0] == Nodetype.NAME:
                nested_type = nested[1][1]
                kwargs[name] = [_dict_to_message(nested_type, entry) for entry in field_value]
            else:
                kwargs[name] = list(field_value)
        else:
            kwargs[name] = field_value
    return message_type(**kwargs)


def encode(schema_name: str, value: dict[str, Any]) -> bytes:
    ensure_idl_loaded()
    message = _dict_to_message(schema_name, value)
    payload = _STORE.serialize_cdr(message, schema_name)
    return bytes(payload)


def decode(schema_name: str, payload: bytes) -> dict[str, Any]:
    ensure_idl_loaded()
    field_count = len(_field_names(schema_name))
    last_error: Exception | None = None
    for count in range(field_count, 0, -1):
        store = _store_for_field_count(schema_name, count)
        try:
            message = store.deserialize_cdr(payload, schema_name)
        except (SerdeError, AssertionError, ValueError, struct.error) as error:
            last_error = error
            continue
        decoded = _message_to_dict(message)
        if count < field_count:
            defaults = _dict_from_defaults(schema_name, _message_fielddefs(schema_name))
            for field_name in _field_names(schema_name):
                if field_name not in decoded:
                    decoded[field_name] = defaults[field_name]
        return decoded
    if last_error is not None:
        raise last_error
    raise ValueError(f"Failed to decode CDR for schema {schema_name}")
