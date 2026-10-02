#!/usr/bin/env python3
"""Fail when documentation names a repository path that is not in git.

Checked files are Markdown under ``docs/`` except ``docs/architecture/draft-1/``
(a frozen historical record), every ``README.md``, ``AGENTS.md``, and
``GLOSSARY.md``. ``node_modules``, ``target``, and submodule contents are not
documentation in this repository, so they are not scanned.

A reference is an inline code span or a Markdown link destination (including
images). It is a repository path when it starts with a top-level directory
(``core/``, ``docs/``, ``.hooks/``, ...) or, for a link, when it is relative to
the Markdown file's directory.

An inline span that contains ``/`` is also a path when its first segment is a
directory in any candidate base: the Markdown file's folder, each ancestor up
to the repository root, ``core/``, ``core/frontend/src/``, and each
``core/services/<service>/`` directory. The span must exist under at least one
such base. ``services/tank/...`` written from ``docs/architecture/`` is
``core/services/tank/...``. ``app/src/lib.rs`` in a service README is that
service's ``app/``. Each service directory is a base because ``core/app`` is
the multicall crate, not a service's ``app/``: ``app/endpoints.toml`` in the
glossary is ``core/services/example/app/endpoints.toml``, and binding it only
to ``core/app`` would report a file that is not supposed to be there. A span
whose first segment matches no base stays prose (``lib.rs``,
``service="tank"``). The D-26 manifest line is a TOML assignment inside a
fenced sample, and even as inline code it does not name a directory.

Inline ``./`` and ``../`` stay prose: the first segment is ``.`` or ``..``,
not a directory in a base. ``./.hooks/pre-push`` in
``docs/architecture/rust-style.md`` is a command run from the repository root
(the same sentence is copied into ``AGENTS.md``). Cookbook entry text such as
``../../frontend/tests/...`` is relative to the example service root, which
``question_index`` checks; the Markdown link beside it is the file-relative
path.

Skipped on purpose: URLs, pure anchors, placeholders containing ``<>{}*``, and
inline code that contains spaces (a command). Link destinations may contain
spaces. A trailing ``:line``, ``:line:column``, or GitHub ``#L`` suffix is
removed before the path is checked. Fenced samples are not scanned, so a path
that appears only as an illustration of file contents is not a reference.
"""

from __future__ import annotations

import posixpath
import re
import subprocess
import sys
from pathlib import Path

PLACEHOLDER_CHARACTERS = set("<>{}*")
DRAFT_RECORD_PREFIX = "docs/architecture/draft-1/"
SKIPPED_DIRECTORY_NAMES = {"node_modules", "target"}
# Docs name trees under the workspace and under the frontend sources without the prefix.
EXTRA_RELATIVE_BASES = ("core", "core/frontend/src")
LINE_SUFFIX = re.compile(r"(?::\d+(?:-\d+)?)+$")
HASH_LINE_SUFFIX = re.compile(r"#L\d+(?:-L\d+)?$")
URI_SCHEME = re.compile(r"^([A-Za-z][A-Za-z0-9+.-]*):(.*)$")
FENCE_LINE = re.compile(r"^(`{3,}|~{3,})(.*)$")
GITMODULES_PATH = re.compile(r"^path\s*=\s*(\S+)\s*$")


def main() -> int:
    if len(sys.argv) > 2:
        print("Usage: check_docs_paths.py [repository_root]", file=sys.stderr)
        return 2
    if len(sys.argv) == 2:
        repository_root = Path(sys.argv[1])
    else:
        repository_root = Path(
            subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True).strip()
        )
    findings = collect_missing_paths(repository_root)
    for markdown_path, line_number, resolved_path, reference in findings:
        print(
            f"{markdown_path}:{line_number}: missing repository path '{resolved_path}' "
            f"referenced as '{reference}'"
        )
    return 1 if findings else 0


def collect_missing_paths(
    repository_root: Path,
) -> list[tuple[str, int, str, str]]:
    existing_paths, directories, top_level_directories, submodule_prefixes = load_repository(
        repository_root
    )
    findings: list[tuple[str, int, str, str]] = []
    for relative_path in sorted(existing_paths):
        if not markdown_in_scope(relative_path, submodule_prefixes):
            continue
        document = repository_root / relative_path
        if not document.is_file():
            continue
        text = document.read_text(encoding="utf-8", errors="replace")
        markdown_directory = posixpath.dirname(relative_path)
        findings.extend(
            missing_in_document(
                relative_path,
                markdown_directory,
                text,
                existing_paths,
                directories,
                top_level_directories,
            )
        )
    findings.sort()
    return findings


def load_repository(repository_root: Path) -> tuple[set[str], set[str], set[str], set[str]]:
    completed = subprocess.run(
        ["git", "ls-files", "-s", "-z"],
        cwd=repository_root,
        check=True,
        capture_output=True,
    )
    existing_paths: set[str] = {"."}
    directories: set[str] = set()
    top_level_directories: set[str] = set()
    submodule_prefixes: set[str] = set(gitmodules_paths(repository_root))
    for record in completed.stdout.split(b"\0"):
        if not record:
            continue
        metadata, path_bytes = record.split(b"\t", 1)
        mode = metadata.decode("ascii").split(" ", 1)[0]
        relative_path = path_bytes.decode("utf-8", "surrogateescape")
        existing_paths.add(relative_path)
        if mode == "160000":
            submodule_prefixes.add(relative_path)
            record_directory(existing_paths, directories, top_level_directories, relative_path)
            continue
        parent = relative_path
        while "/" in parent:
            parent = parent.rsplit("/", 1)[0]
            record_directory(existing_paths, directories, top_level_directories, parent)
    for submodule_prefix in submodule_prefixes:
        record_directory(existing_paths, directories, top_level_directories, submodule_prefix)
    return existing_paths, directories, top_level_directories, submodule_prefixes


def record_directory(
    existing_paths: set[str],
    directories: set[str],
    top_level_directories: set[str],
    directory_path: str,
) -> None:
    existing_paths.add(directory_path)
    directories.add(directory_path)
    top_level_directories.add(directory_path.split("/", 1)[0])


def gitmodules_paths(repository_root: Path) -> set[str]:
    gitmodules = repository_root / ".gitmodules"
    if not gitmodules.is_file():
        return set()
    paths: set[str] = set()
    for line in gitmodules.read_text(encoding="utf-8", errors="replace").splitlines():
        match = GITMODULES_PATH.match(line.strip())
        if match:
            paths.add(match.group(1).rstrip("/"))
    return paths


def markdown_in_scope(relative_path: str, submodule_prefixes: set[str]) -> bool:
    if not relative_path.endswith(".md"):
        return False
    parts = relative_path.split("/")
    if any(part in SKIPPED_DIRECTORY_NAMES for part in parts):
        return False
    if relative_path.startswith(DRAFT_RECORD_PREFIX):
        return False
    for submodule_prefix in submodule_prefixes:
        if relative_path == submodule_prefix or relative_path.startswith(submodule_prefix + "/"):
            return False
    if relative_path in {"AGENTS.md", "GLOSSARY.md"}:
        return True
    if relative_path.startswith("docs/"):
        return True
    return parts[-1] == "README.md"


def missing_in_document(
    markdown_path: str,
    markdown_directory: str,
    text: str,
    existing_paths: set[str],
    directories: set[str],
    top_level_directories: set[str],
) -> list[tuple[str, int, str, str]]:
    findings: list[tuple[str, int, str, str]] = []
    fence_character = ""
    fence_length = 0
    for line_number, line in enumerate(text.splitlines(), start=1):
        # A fence is a sample of file contents (the D-26 manifest's `service = "tank"`
        # is TOML, not a repository path), so nothing inside it is a reference.
        opener = fence_opener(line)
        if fence_character:
            if (
                opener is not None
                and opener[0] == fence_character
                and opener[1] >= fence_length
                and opener[2].strip() == ""
            ):
                fence_character = ""
                fence_length = 0
            continue
        if opener is not None:
            fence_character, fence_length = opener[0], opener[1]
            continue
        spans = code_span_ranges(line)
        for _start, _end, content in spans:
            reference = prepare_code_span(content)
            if reference is None:
                continue
            for resolved in missing_code_span(
                reference,
                markdown_directory,
                existing_paths,
                directories,
                top_level_directories,
            ):
                findings.append((markdown_path, line_number, resolved, content.strip()))
        for destination in link_destinations(line, spans):
            reference = prepare_link_destination(destination, top_level_directories)
            if reference is None:
                continue
            resolved = resolve_reference(
                reference,
                markdown_directory,
                top_level_directories,
                allow_bare_relative=True,
            )
            if resolved is not None and resolved not in existing_paths:
                findings.append((markdown_path, line_number, resolved, destination.strip()))
    return findings


def fence_opener(line: str) -> tuple[str, int, str] | None:
    # Indented fences (lists) still introduce a sample, so only the marker matters.
    match = FENCE_LINE.match(line.lstrip())
    if not match:
        return None
    token = match.group(1)
    info = match.group(2)
    if token[0] == "`" and "`" in info:
        return None
    return token[0], len(token), info


def code_span_ranges(line: str) -> list[tuple[int, int, str]]:
    ranges: list[tuple[int, int, str]] = []
    index = 0
    length = len(line)
    while index < length:
        if line[index] != "`":
            index += 1
            continue
        marker_start = index
        while index < length and line[index] == "`":
            index += 1
        marker = line[marker_start:index]
        closer = line.find(marker, index)
        if closer < 0:
            break
        ranges.append((marker_start, closer + len(marker), line[index:closer]))
        index = closer + len(marker)
    return ranges


def link_destinations(line: str, spans: list[tuple[int, int, str]]) -> list[str]:
    destinations: list[str] = []
    search_from = 0
    while True:
        found = line.find("](", search_from)
        if found < 0:
            break
        if inside_span(found, spans):
            search_from = found + 2
            continue
        parsed = parse_link_destination(line, found + 2)
        if parsed is None:
            search_from = found + 2
            continue
        destination, end = parsed
        destinations.append(destination)
        search_from = end
    return destinations


def inside_span(index: int, spans: list[tuple[int, int, str]]) -> bool:
    return any(start <= index < end for start, end, _content in spans)


def parse_link_destination(line: str, start: int) -> tuple[str, int] | None:
    if start >= len(line):
        return None
    if line[start] == "<":
        end = line.find(">", start + 1)
        if end < 0:
            return None
        destination = line[start + 1 : end]
        cursor = end + 1
    else:
        depth = 0
        cursor = start
        while cursor < len(line):
            character = line[cursor]
            if character == "\\" and cursor + 1 < len(line):
                cursor += 2
                continue
            if character == "(":
                depth += 1
            elif character == ")":
                if depth == 0:
                    break
                depth -= 1
            elif character.isspace() and depth == 0:
                break
            cursor += 1
        destination = line[start:cursor]
    while cursor < len(line) and line[cursor].isspace():
        cursor += 1
    if cursor < len(line) and line[cursor] in "\"'":
        quote = line[cursor]
        end_quote = line.find(quote, cursor + 1)
        if end_quote < 0:
            return None
        cursor = end_quote + 1
        while cursor < len(line) and line[cursor].isspace():
            cursor += 1
    if cursor >= len(line) or line[cursor] != ")":
        return None
    return destination, cursor + 1


def prepare_code_span(content: str) -> str | None:
    candidate = content.strip()
    if not candidate or any(character.isspace() for character in candidate):
        return None
    if any(character in candidate for character in PLACEHOLDER_CHARACTERS):
        return None
    if candidate.startswith("#") or is_url(candidate):
        return None
    if "#" in candidate:
        hash_suffix = HASH_LINE_SUFFIX.search(candidate)
        if hash_suffix is None or "#" in candidate[: hash_suffix.start()]:
            return None
        candidate = candidate[: hash_suffix.start()]
    candidate = strip_line_suffix(candidate)
    if len(candidate) > 1:
        candidate = candidate.rstrip("/")
    if not candidate:
        return None
    return candidate


def prepare_link_destination(destination: str, top_level_directories: set[str]) -> str | None:
    candidate = destination.strip()
    if not candidate or candidate.startswith("#") or is_url(candidate):
        return None
    if any(character in candidate for character in PLACEHOLDER_CHARACTERS):
        return None
    candidate = candidate.split("#", 1)[0].strip()
    if not candidate:
        return None
    candidate = strip_line_suffix(candidate).strip()
    if len(candidate) > 1:
        candidate = candidate.rstrip("/")
    if not candidate:
        return None
    if candidate.startswith("/"):
        without_slash = candidate.lstrip("/")
        if not is_repository_root_path(without_slash, top_level_directories):
            return None
        candidate = without_slash
    return candidate


def is_url(candidate: str) -> bool:
    if "://" in candidate or candidate.startswith("//"):
        return True
    match = URI_SCHEME.match(candidate)
    if not match:
        return False
    scheme = match.group(1)
    remainder = match.group(2)
    if "." in scheme:
        return False
    if re.fullmatch(r"\d+(?:-\d+)?(?::\d+(?:-\d+)?)*", remainder):
        return False
    return True


def strip_line_suffix(candidate: str) -> str:
    return LINE_SUFFIX.sub("", candidate)


def missing_code_span(
    candidate: str,
    markdown_directory: str,
    existing_paths: set[str],
    directories: set[str],
    top_level_directories: set[str],
) -> list[str]:
    if is_repository_root_path(candidate, top_level_directories):
        resolved = posixpath.normpath(candidate)
        if resolved not in existing_paths:
            return [resolved]
        return []
    resolutions = resolutions_from_bases(candidate, markdown_directory, directories)
    if resolutions is None:
        return []
    if any(resolved in existing_paths for resolved in resolutions):
        return []
    return resolutions


def candidate_bases(markdown_directory: str, directories: set[str]) -> list[str]:
    bases: list[str] = []
    directory = markdown_directory
    while True:
        if directory not in bases:
            bases.append(directory)
        if directory == "":
            break
        directory = posixpath.dirname(directory)
    for extra in EXTRA_RELATIVE_BASES:
        if extra not in bases:
            bases.append(extra)
    # `core/app` is the multicall crate. Service layout spans (`app/endpoints.toml`)
    # name `core/services/<service>/app/`, so each service directory is a base.
    service_prefix = "core/services/"
    for service_directory in sorted(directories):
        if not service_directory.startswith(service_prefix):
            continue
        service_name = service_directory[len(service_prefix) :]
        if not service_name or "/" in service_name:
            continue
        if service_directory not in bases:
            bases.append(service_directory)
    return bases


def resolutions_from_bases(
    candidate: str, markdown_directory: str, directories: set[str]
) -> list[str] | None:
    # `./.hooks/pre-push` and `../../frontend/...` are not paths from a base:
    # `.` and `..` are not directory names in the repository.
    if "/" not in candidate:
        return None
    first_segment, _separator, _remainder = candidate.partition("/")
    if first_segment in {".", ".."}:
        return None
    resolutions: list[str] = []
    seen: set[str] = set()
    for base in candidate_bases(markdown_directory, directories):
        directory = posixpath.join(base, first_segment) if base else first_segment
        if directory not in directories:
            continue
        resolved = posixpath.normpath(posixpath.join(base, candidate) if base else candidate)
        if resolved in seen:
            continue
        seen.add(resolved)
        resolutions.append(resolved)
    if not resolutions:
        return None
    return resolutions


def is_repository_root_path(candidate: str, top_level_directories: set[str]) -> bool:
    if candidate.startswith(("./", "../")):
        return False
    first_component, _separator, _remainder = candidate.partition("/")
    return first_component in top_level_directories


def resolve_reference(
    candidate: str,
    markdown_directory: str,
    top_level_directories: set[str],
    *,
    allow_bare_relative: bool,
) -> str | None:
    if is_repository_root_path(candidate, top_level_directories):
        return posixpath.normpath(candidate)
    # Only link destinations are relative to the Markdown file. Inline `./` and
    # `../` name some other base (the repo root, or the example service root).
    if not allow_bare_relative:
        return None
    if markdown_directory:
        joined = posixpath.join(markdown_directory, candidate)
    else:
        joined = candidate
    return posixpath.normpath(joined)


if __name__ == "__main__":
    sys.exit(main())
