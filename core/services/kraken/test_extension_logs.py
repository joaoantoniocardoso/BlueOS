from pathlib import Path
from unittest.mock import MagicMock

from commonwealth.utils import blueos_idl
from commonwealth.utils.logs import LOG_SCHEMA, log_publisher_options
from extension_logs import ExtensionLogPublisher

LIBS_ROOT = Path(__file__).resolve().parents[2] / "libs"
IDL_INTERFACES_ROOT = LIBS_ROOT / "idl" / "interfaces"


def test_extension_log_topic_uses_shared_key_helper() -> None:
    extension = MagicMock()
    extension.identifier = "my/extension"
    extension.name = "ignored"
    extension.container_name.return_value = "container"

    assert ExtensionLogPublisher._topic_for(extension) == "blueos/v1/kraken/log/extension/my_extension"


def test_extension_log_payload_is_cdr_foxglove_log() -> None:
    blueos_idl.ensure_idl_loaded(IDL_INTERFACES_ROOT)
    payload = ExtensionLogPublisher._format_log_payload("my-container", "INFO: hello")

    decoded = blueos_idl.decode(LOG_SCHEMA, payload)
    assert decoded["message"] == "hello"
    assert decoded["level"] == 2
    assert decoded["name"] == "my-container"


def test_extension_log_publisher_declares_cdr_publisher_on_extension_key() -> None:
    extension = MagicMock()
    extension.identifier = "demo"
    extension.name = "demo"
    extension.container_name.return_value = "demo-container"

    zenoh_publisher = MagicMock()
    router = MagicMock()
    router.add_publisher.return_value = zenoh_publisher

    publisher = ExtensionLogPublisher.__new__(ExtensionLogPublisher)
    publisher._publishers = {}
    publisher._zenoh_router = router

    declared = publisher._declare_publisher("demo-container", extension)

    assert declared is zenoh_publisher
    router.add_publisher.assert_called_once_with(
        "blueos/v1/kraken/log/extension/demo",
        absolute=True,
        publisher_options=log_publisher_options(),
    )


def test_extension_log_publish_puts_encoded_bytes() -> None:
    zenoh_publisher = MagicMock()
    publisher = ExtensionLogPublisher.__new__(ExtensionLogPublisher)
    payload = b"\x00\x01\x00\x00"

    publisher._publish(zenoh_publisher, "blueos/v1/kraken/log/extension/demo", payload)

    zenoh_publisher.put.assert_called_once_with(payload)
