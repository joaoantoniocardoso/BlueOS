from typing import Any

import pytest
from loguru import logger

from extension.extension import Extension


@pytest.fixture(name="logs")
def fixture_logs() -> Any:
    messages: list[str] = []
    sink_id = logger.add(lambda message: messages.append(str(message)), level="WARNING")
    yield messages
    logger.remove(sink_id)


@pytest.mark.parametrize("raw, reason", [("{bad", "not valid JSON"), ('"text"', "got str"), ("5", "got int")])
def test_malformed_label_logs_warning_and_returns_fallback(logs: list[str], raw: str, reason: str) -> None:
    result = Extension._load_optional_json_label({"links": raw}, "links", {})

    assert result == {}
    assert len(logs) == 1
    assert "links" in logs[0]
    assert reason in logs[0]


def test_valid_and_missing_labels_do_not_log(logs: list[str]) -> None:
    assert Extension._load_optional_json_label({"authors": '[{"name": "a"}]'}, "authors", []) == [{"name": "a"}]
    assert Extension._load_optional_json_label({}, "authors", []) is None
    assert not logs
