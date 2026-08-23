#!/usr/bin/env python3
"""Insert anchor lines into vertical Evidence blocks (no file/line changes)."""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
DOCS = REPO.parent / "BlueOS-docs"
ANCHOR_MAX = 60


def extract_anchor(line: str) -> str:
    ascii_text = "".join(ch for ch in line if ord(ch) < 128)
    collapsed = " ".join(ascii_text.split())
    if not any(c.isascii() and not c.isspace() for c in collapsed):
        return ""
    return collapsed[:ANCHOR_MAX]


def escape_rust(value: str) -> str:
    return value.replace("\\", "\\\\").replace('"', '\\"')


def resolve_path(expr: str, aliases: dict[str, str]) -> tuple[str | None, bool]:
    expr = expr.strip()
    if expr.startswith('"') and expr.endswith('"'):
        path = expr[1:-1]
        return path, path.startswith("content/") or path.endswith(".md")
    if expr in aliases:
        path = aliases[expr]
        return path, path.startswith("content/") or path.endswith(".md")
    return None, False


def collect_aliases(text: str) -> dict[str, str]:
    aliases: dict[str, str] = {}
    for name, path in re.findall(
        r"(?:pub\s+)?const\s+(\w+)\s*:\s*&str\s*=\s*\"([^\"]+)\";", text
    ):
        aliases[name] = path
    return aliases


def read_line(path: Path, line: int) -> str | None:
    if not path.is_file():
        return None
    lines = path.read_text().splitlines()
    if line < 1 or line > len(lines):
        return None
    return lines[line - 1]


def extract_block(text: str, start: int) -> tuple[int, str] | None:
    open_brace = text.find("{", start)
    if open_brace == -1:
        return None
    depth = 0
    in_string = False
    escape = False
    quote = ""
    for idx in range(open_brace, len(text)):
        ch = text[idx]
        if in_string:
            if escape:
                escape = False
            elif ch == "\\":
                escape = True
            elif ch == quote:
                in_string = False
            continue
        if ch in "\"'":
            in_string = True
            quote = ch
            continue
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                end = idx + 1
                if end < len(text) and text[end] == ",":
                    end += 1
                return end, text[start:end]
    return None


def insert_anchor(block: str, anchor: str) -> str:
    lines = block.splitlines()
    close_idx = max(
        i
        for i, line in enumerate(lines)
        if line.strip() in ("}", "},")
    )
    field_indent = next(
        line[: line.find("file:")] for line in lines if "file:" in line
    )
    out = lines[:close_idx]
    out.append(f'{field_indent}anchor: "{anchor}",')
    out.extend(lines[close_idx:])
    return "\n".join(out)


def evidence_literal_starts(text: str) -> list[int]:
    return [
        match.start()
        for match in re.finditer(r"Evidence \{\n\s*file:", text)
    ]


def migrate_text(text: str, repo: Path) -> tuple[str, int]:
    aliases = collect_aliases(text)
    out: list[str] = []
    idx = 0
    touched = 0
    for start in evidence_literal_starts(text):
        if start < idx:
            continue
        out.append(text[idx:start])
        parsed = extract_block(text, start)
        if parsed is None:
            out.append(text[start:])
            idx = len(text)
            break
        end, block = parsed
        if "anchor:" in block:
            out.append(block)
        else:
            file_m = re.search(r"file:\s*([^,\n]+)", block)
            line_m = re.search(r"line:\s*(\d+)", block)
            if not (file_m and line_m):
                out.append(block)
            else:
                resolved, is_doc = resolve_path(file_m.group(1), aliases)
                anchor = ""
                if resolved:
                    target = DOCS / resolved if is_doc else repo / resolved
                    raw = read_line(target, int(line_m.group(1)))
                    if raw is not None:
                        anchor = escape_rust(extract_anchor(raw))
                out.append(insert_anchor(block, anchor))
                touched += 1
        idx = end
    out.append(text[idx:])
    return "".join(out), touched


def main() -> int:
    targets = [
        REPO / "catalog" / "src" / "services",
        REPO / "catalog" / "src" / "pages",
        REPO / "catalog" / "src" / "page.rs",
    ]
    total = 0
    files = 0
    for target in targets:
        paths = [target] if target.is_file() else sorted(target.rglob("*.rs"))
        for path in paths:
            original = path.read_text()
            updated, touched = migrate_text(original, REPO)
            if updated != original:
                path.write_text(updated)
                files += 1
                total += touched
    print(f"insert_evidence_anchors: files={files} blocks={total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
