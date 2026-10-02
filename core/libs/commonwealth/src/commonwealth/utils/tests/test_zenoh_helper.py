from unittest.mock import MagicMock

import pytest
from commonwealth.utils import blueos_idl
from commonwealth.utils.zenoh_helper import ZenohSession


def test_register_standard_service_keys_skips_when_idl_missing(monkeypatch: pytest.MonkeyPatch) -> None:
    zenoh_session = object.__new__(ZenohSession)
    mock_session = MagicMock()
    zenoh_session.session = mock_session
    zenoh_session._liveliness_token = None

    def raise_missing() -> None:
        raise FileNotFoundError("IDL interfaces directory not found")

    monkeypatch.setattr(
        "commonwealth.utils.zenoh_helper.blueos_idl.ensure_idl_loaded",
        raise_missing,
    )

    zenoh_session._register_standard_service_keys("kraken")

    assert zenoh_session._liveliness_token is None
    mock_session.liveliness.assert_not_called()
    mock_session.declare_queryable.assert_not_called()


def test_register_standard_service_keys_declares_liveliness_and_info(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    zenoh_session = object.__new__(ZenohSession)
    mock_session = MagicMock()
    mock_liveliness = MagicMock()
    mock_session.liveliness.return_value = mock_liveliness
    zenoh_session.session = mock_session
    zenoh_session._liveliness_token = None

    monkeypatch.setattr("commonwealth.utils.zenoh_helper.blueos_idl.ensure_idl_loaded", lambda: None)
    monkeypatch.setattr(
        "commonwealth.utils.zenoh_helper.blueos_idl.encode",
        lambda schema_name, value: b"service-info",
    )

    zenoh_session._register_standard_service_keys("wifi-manager")

    mock_liveliness.declare_token.assert_called_once_with(blueos_idl.service_liveliness_key("wifi-manager"))
    mock_session.declare_queryable.assert_called_once()
    info_key, info_handler = mock_session.declare_queryable.call_args[0]
    assert info_key == blueos_idl.info_query_key("wifi-manager")
    mock_query = MagicMock()
    info_handler(mock_query)
    mock_query.reply.assert_called_once()
    reply_key, reply_payload = mock_query.reply.call_args[0]
    assert reply_key == blueos_idl.info_query_key("wifi-manager")
    assert reply_payload == b"service-info"
