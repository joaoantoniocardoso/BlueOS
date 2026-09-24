from unittest.mock import MagicMock

import pytest

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
