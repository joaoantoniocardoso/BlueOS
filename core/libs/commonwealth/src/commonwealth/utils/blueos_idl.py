"""Runtime ROS 2 `.msg` parsing and CDR codec for the versioned BlueOS zenoh API (D-05, D-06, D-17).

Key helpers mirror `core/libs/api/src/lib.rs` (single source of truth for key layout).
"""

from __future__ import annotations

import os
import re
import struct
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Literal

API_VERSION = "v1"
KEY_PREFIX = "blueos/v1"

ENCODING_APPLICATION_CDR = "application/cdr"
TYPE_HASH_ATTACHMENT_KEY = "blueos.type_hash"

ENCODING_APPLICATION_JSON = "application/json"

ENCAPSULATION_CDR_LE = bytes((0x00, 0x01, 0x00, 0x00))

PRIMITIVE_TYPES = frozenset(
    {
        "bool",
        "byte",
        "char",
        "float32",
        "float64",
        "int8",
        "int16",
        "int32",
        "int64",
        "uint8",
        "uint16",
        "uint32",
        "uint64",
        "string",
    }
)


class IdlCodecError(ValueError):
    pass


@dataclass(frozen=True)
class FieldType:
    kind: Literal["primitive", "string", "message", "vector", "array"]
    primitive: str | None = None
    message_type: str | None = None
    element: FieldType | None = None
    array_size: int | None = None


@dataclass
class MessageDef:
    schema_name: str
    fields: list[tuple[str, FieldType]]


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


_MESSAGES: dict[str, MessageDef] = {}


def ensure_idl_loaded(interfaces_root: Path | None = None) -> None:
    if _MESSAGES:
        return
    root = interfaces_root or default_idl_interfaces_root()
    _load_interfaces(root)


def _load_interfaces(root: Path) -> None:
    raw_text: dict[str, str] = {}
    for path in sorted(root.rglob("*.msg")):
        relative = path.relative_to(root).with_suffix("").as_posix()
        raw_text[relative] = path.read_text(encoding="utf-8")

    messages: dict[str, MessageDef] = {}
    for schema_name, text in raw_text.items():
        messages[schema_name] = _parse_message(schema_name, text)

    for message_def in messages.values():
        for _name, field_type in message_def.fields:
            _validate_field_type(field_type, messages)

    _MESSAGES.clear()
    _MESSAGES.update(messages)


def _validate_field_type(field_type: FieldType, messages: dict[str, MessageDef]) -> None:
    if field_type.kind == "message":
        assert field_type.message_type is not None
        if field_type.message_type not in messages:
            raise IdlCodecError(f"Unknown nested message type: {field_type.message_type}")
    if field_type.element is not None:
        _validate_field_type(field_type.element, messages)


def _package_prefix(schema_name: str) -> str:
    return schema_name.split("/msg/", maxsplit=1)[0]


def _normalize_message_type(type_reference: str, owning_schema: str) -> str:
    if type_reference in PRIMITIVE_TYPES or type_reference.startswith("string"):
        return type_reference
    if "/msg/" in type_reference:
        return type_reference
    if "/" in type_reference:
        package, name = type_reference.split("/", maxsplit=1)
        return f"{package}/msg/{name}"
    package = _package_prefix(owning_schema)
    return f"{package}/msg/{type_reference}"


def _parse_field_type(type_token: str, owning_schema: str) -> FieldType:
    bounded_match = re.fullmatch(r"string<=([0-9]+)", type_token)
    if bounded_match or type_token == "string":
        return FieldType(kind="string")

    if type_token.endswith("[]"):
        element_reference = type_token[: -len("[]")]
        element = _parse_field_type(element_reference, owning_schema)
        if element.kind in {"vector", "array"}:
            raise IdlCodecError(f"Nested vector/array not supported: {type_token}")
        return FieldType(kind="vector", element=element)

    fixed_match = re.fullmatch(r"(.+)\[([0-9]+)\]", type_token)
    if fixed_match:
        element_reference, size_text = fixed_match.groups()
        element = _parse_field_type(element_reference, owning_schema)
        if element.kind in {"vector", "array"}:
            raise IdlCodecError(f"Nested vector/array not supported: {type_token}")
        return FieldType(kind="array", element=element, array_size=int(size_text))

    if type_token in PRIMITIVE_TYPES:
        return FieldType(kind="primitive", primitive=type_token)

    message_type = _normalize_message_type(type_token, owning_schema)
    return FieldType(kind="message", message_type=message_type)


def _is_constant_line(line: str) -> bool:
    return bool(re.match(r"^[A-Za-z_/][\w/]*\s+\w+=", line))


def _parse_message(schema_name: str, text: str) -> MessageDef:
    fields: list[tuple[str, FieldType]] = []
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if _is_constant_line(stripped):
            continue
        parts = stripped.split()
        if len(parts) < 2:
            continue
        type_token, field_name = parts[0], parts[1]
        fields.append((field_name, _parse_field_type(type_token, schema_name)))

    return MessageDef(schema_name=schema_name, fields=fields)


def _alignment_for_primitive(primitive: str) -> int:
    if primitive in {"bool", "byte", "char", "int8", "uint8"}:
        return 1
    if primitive in {"int16", "uint16"}:
        return 2
    if primitive in {"int32", "uint32", "float32"}:
        return 4
    if primitive in {"int64", "uint64", "float64"}:
        return 8
    return 1


def _alignment_for_field(field_type: FieldType) -> int:
    if field_type.kind == "primitive":
        assert field_type.primitive is not None
        return _alignment_for_primitive(field_type.primitive)
    if field_type.kind == "string":
        return 4
    if field_type.kind == "message":
        assert field_type.message_type is not None
        message_def = _MESSAGES[field_type.message_type]
        if not message_def.fields:
            return 1
        return _alignment_for_field(message_def.fields[0][1])
    if field_type.kind in {"vector", "array"}:
        return 4
    return 1


class CdrWriter:
    def __init__(self) -> None:
        self._buffer = bytearray()

    @property
    def _position(self) -> int:
        return len(self._buffer)

    def _align(self, alignment: int) -> None:
        padding = (alignment - (self._position % alignment)) % alignment
        self._buffer.extend(b"\x00" * padding)

    def write_bool(self, value: bool) -> None:
        self._buffer.append(1 if value else 0)

    def write_u8(self, value: int) -> None:
        self._buffer.append(value & 0xFF)

    def write_i8(self, value: int) -> None:
        self.write_u8(value)

    def write_u16(self, value: int) -> None:
        self._align(2)
        self._buffer.extend(struct.pack("<H", value & 0xFFFF))

    def write_i16(self, value: int) -> None:
        self.write_u16(value)

    def write_u32(self, value: int) -> None:
        self._align(4)
        self._buffer.extend(struct.pack("<I", value & 0xFFFFFFFF))

    def write_i32(self, value: int) -> None:
        self.write_u32(value)

    def write_u64(self, value: int) -> None:
        self._align(8)
        self._buffer.extend(struct.pack("<Q", value & 0xFFFFFFFFFFFFFFFF))

    def write_i64(self, value: int) -> None:
        self.write_u64(value)

    def write_f32(self, value: float) -> None:
        self.write_u32(struct.unpack("<I", struct.pack("<f", value))[0])

    def write_f64(self, value: float) -> None:
        self.write_u64(struct.unpack("<Q", struct.pack("<d", value))[0])

    def write_string(self, value: str) -> None:
        encoded = value.encode("utf-8")
        length = len(encoded) + 1
        self.write_u32(length)
        self._buffer.extend(encoded)
        self._buffer.append(0)
        total = 4 + len(encoded) + 1
        padding = (4 - (total % 4)) % 4
        self._buffer.extend(b"\x00" * padding)

    def finish_with_encapsulation(self) -> bytes:
        return ENCAPSULATION_CDR_LE + bytes(self._buffer)


class CdrReader:
    def __init__(self, body: bytes) -> None:
        self._buffer = body
        self._position = 0

    @classmethod
    def from_payload(cls, payload: bytes) -> CdrReader:
        if len(payload) < 4 or payload[0:4] != ENCAPSULATION_CDR_LE:
            raise IdlCodecError("Invalid CDR encapsulation header")
        return cls(payload[4:])

    def is_exhausted(self) -> bool:
        return self._position >= len(self._buffer)

    def _align(self, alignment: int) -> None:
        offset = self._position % alignment
        if offset == 0:
            return
        padding = alignment - offset
        if self._position + padding > len(self._buffer):
            raise IdlCodecError("Unexpected end of CDR payload during alignment")
        self._position += padding

    def _read_exact(self, count: int) -> bytes:
        if self._position + count > len(self._buffer):
            raise IdlCodecError("Unexpected end of CDR payload")
        slice_end = self._position + count
        data = self._buffer[self._position : slice_end]
        self._position = slice_end
        return bytes(data)

    def read_bool(self) -> bool:
        value = self._read_exact(1)[0]
        if value == 0:
            return False
        if value == 1:
            return True
        raise IdlCodecError("Invalid bool value in CDR payload")

    def read_u8(self) -> int:
        return int(self._read_exact(1)[0])

    def read_i8(self) -> int:
        return int(struct.unpack("b", self.read_u8().to_bytes(1, "little"))[0])

    def read_u16(self) -> int:
        self._align(2)
        return int(struct.unpack("<H", self._read_exact(2))[0])

    def read_i16(self) -> int:
        return int(struct.unpack("<h", struct.pack("<H", self.read_u16() & 0xFFFF))[0])

    def read_u32(self) -> int:
        self._align(4)
        return int(struct.unpack("<I", self._read_exact(4))[0])

    def read_i32(self) -> int:
        return int(struct.unpack("<i", struct.pack("<I", self.read_u32() & 0xFFFFFFFF))[0])

    def read_u64(self) -> int:
        self._align(8)
        return int(struct.unpack("<Q", self._read_exact(8))[0])

    def read_i64(self) -> int:
        return int(struct.unpack("<q", struct.pack("<Q", self.read_u64() & 0xFFFFFFFFFFFFFFFF))[0])

    def read_f32(self) -> float:
        return float(struct.unpack("<f", struct.pack("<I", self.read_u32() & 0xFFFFFFFF))[0])

    def read_f64(self) -> float:
        return float(struct.unpack("<d", struct.pack("<Q", self.read_u64() & 0xFFFFFFFFFFFFFFFF))[0])

    def read_string(self) -> str:
        length = self.read_u32()
        if length == 0:
            return ""
        data = self._read_exact(length)
        if data[-1] != 0:
            raise IdlCodecError("Invalid string encoding in CDR payload")
        text = data[:-1].decode("utf-8")
        total = 4 + length
        padding = (4 - (total % 4)) % 4
        if padding:
            self._read_exact(padding)
        return text


def _default_for_field(field_type: FieldType) -> Any:
    if field_type.kind == "primitive":
        assert field_type.primitive is not None
        if field_type.primitive == "bool":
            return False
        if field_type.primitive in {"float32", "float64"}:
            return 0.0
        return 0
    if field_type.kind == "string":
        return ""
    if field_type.kind == "vector":
        return []
    if field_type.kind == "array":
        assert field_type.element is not None and field_type.array_size is not None
        return [_default_for_field(field_type.element) for _ in range(field_type.array_size)]
    if field_type.kind == "message":
        assert field_type.message_type is not None
        return _defaults_for_message(field_type.message_type)
    raise IdlCodecError(f"Unsupported field type: {field_type}")


def _defaults_for_message(schema_name: str) -> dict[str, Any]:
    message_def = _MESSAGES[schema_name]
    return {name: _default_for_field(field_type) for name, field_type in message_def.fields}


def _write_primitive(writer: CdrWriter, primitive: str, value: Any) -> None:
    if primitive == "bool":
        writer.write_bool(bool(value))
    elif primitive == "byte" or primitive == "uint8":
        writer.write_u8(int(value))
    elif primitive == "char" or primitive == "int8":
        writer.write_i8(int(value))
    elif primitive == "uint16":
        writer.write_u16(int(value))
    elif primitive == "int16":
        writer.write_i16(int(value))
    elif primitive == "uint32":
        writer.write_u32(int(value))
    elif primitive == "int32":
        writer.write_i32(int(value))
    elif primitive == "uint64":
        writer.write_u64(int(value))
    elif primitive == "int64":
        writer.write_i64(int(value))
    elif primitive == "float32":
        writer.write_f32(float(value))
    elif primitive == "float64":
        writer.write_f64(float(value))
    else:
        raise IdlCodecError(f"Unsupported primitive: {primitive}")


def _read_primitive(reader: CdrReader, primitive: str) -> Any:
    if primitive == "bool":
        return reader.read_bool()
    if primitive == "byte" or primitive == "uint8" or primitive == "char":
        return reader.read_u8()
    if primitive == "int8":
        return reader.read_i8()
    if primitive == "uint16":
        return reader.read_u16()
    if primitive == "int16":
        return reader.read_i16()
    if primitive == "uint32":
        return reader.read_u32()
    if primitive == "int32":
        return reader.read_i32()
    if primitive == "uint64":
        return reader.read_u64()
    if primitive == "int64":
        return reader.read_i64()
    if primitive == "float32":
        return reader.read_f32()
    if primitive == "float64":
        return reader.read_f64()
    raise IdlCodecError(f"Unsupported primitive: {primitive}")


def _encode_field(writer: CdrWriter, field_type: FieldType, value: Any) -> None:
    if field_type.kind == "primitive":
        assert field_type.primitive is not None
        _write_primitive(writer, field_type.primitive, value)
    elif field_type.kind == "string":
        writer.write_string(str(value))
    elif field_type.kind == "message":
        assert field_type.message_type is not None
        nested = value if isinstance(value, dict) else {}
        _encode_message_fields(writer, field_type.message_type, nested)
    elif field_type.kind == "vector":
        assert field_type.element is not None
        items = list(value or [])
        writer.write_u32(len(items))
        for item in items:
            _encode_field(writer, field_type.element, item)
    elif field_type.kind == "array":
        assert field_type.element is not None and field_type.array_size is not None
        items = list(value or [])
        if len(items) < field_type.array_size:
            items = items + [_default_for_field(field_type.element) for _ in range(field_type.array_size - len(items))]
        for index in range(field_type.array_size):
            _encode_field(writer, field_type.element, items[index])
    else:
        raise IdlCodecError(f"Unsupported field type: {field_type}")


def _encode_message_fields(writer: CdrWriter, schema_name: str, value: dict[str, Any]) -> None:
    defaults = _defaults_for_message(schema_name)
    message_def = _MESSAGES[schema_name]
    for name, field_type in message_def.fields:
        field_value = value.get(name, defaults[name])
        _encode_field(writer, field_type, field_value)


def _decode_field(reader: CdrReader, field_type: FieldType) -> Any:
    if reader.is_exhausted():
        return _default_for_field(field_type)

    if field_type.kind == "vector":
        if reader.is_exhausted():
            return []
        count = reader.read_u32()
        assert field_type.element is not None
        return [_decode_field_value(reader, field_type.element) for _ in range(count)]

    if field_type.kind == "array":
        assert field_type.element is not None and field_type.array_size is not None
        if reader.is_exhausted():
            return [_default_for_field(field_type.element) for _ in range(field_type.array_size)]
        return [
            _decode_field_value(reader, field_type.element) for _ in range(field_type.array_size)
        ]

    return _decode_field_value(reader, field_type)


def _decode_field_value(reader: CdrReader, field_type: FieldType) -> Any:
    if field_type.kind == "primitive":
        assert field_type.primitive is not None
        return _read_primitive(reader, field_type.primitive)
    if field_type.kind == "string":
        return reader.read_string()
    if field_type.kind == "message":
        assert field_type.message_type is not None
        return _decode_message_fields(reader, field_type.message_type)
    raise IdlCodecError(f"Unsupported field type: {field_type}")


def _decode_message_fields(reader: CdrReader, schema_name: str) -> dict[str, Any]:
    message_def = _MESSAGES[schema_name]
    decoded: dict[str, Any] = {}
    for name, field_type in message_def.fields:
        if reader.is_exhausted():
            decoded[name] = _default_for_field(field_type)
        else:
            decoded[name] = _decode_field(reader, field_type)
    return decoded


def encode(schema_name: str, value: dict[str, Any]) -> bytes:
    ensure_idl_loaded()
    if schema_name not in _MESSAGES:
        raise IdlCodecError(f"Unknown schema: {schema_name}")
    writer = CdrWriter()
    _encode_message_fields(writer, schema_name, value)
    return writer.finish_with_encapsulation()


def decode(schema_name: str, payload: bytes) -> dict[str, Any]:
    ensure_idl_loaded()
    if schema_name not in _MESSAGES:
        raise IdlCodecError(f"Unknown schema: {schema_name}")
    reader = CdrReader.from_payload(payload)
    return _decode_message_fields(reader, schema_name)
