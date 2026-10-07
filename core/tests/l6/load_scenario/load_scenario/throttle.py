import re

_THROTTLED_PATTERN = re.compile(r"throttled=(0x[0-9a-fA-F]+)")


def parse_vcgencmd_throttled(output: str) -> int:
    match = _THROTTLED_PATTERN.search(output.strip())
    if match is None:
        return 0
    return int(match.group(1), 16)


def run_was_throttled(throttle_before: int, throttle_after: int) -> bool:
    current_state_mask = 0x5
    if throttle_after & current_state_mask:
        return True
    return (throttle_after & ~throttle_before) != 0
