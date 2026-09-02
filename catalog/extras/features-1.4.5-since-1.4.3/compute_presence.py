#!/usr/bin/env python3
"""Verify catalog journey presence on BlueOS tags (one-off, not part of catalog build)."""

import json
import re
import subprocess
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
RUST = REPO / "catalog/crates/catalog-git/src/feature_presence.rs"
MAP = REPO / "catalog/feature_presence_map.json"
OUT = Path(__file__).resolve().parent / "presence.json"

TAGS = ("1.4.3", "1.4.4", "1.4.4-beta.23", "1.4.5")


def parse_rust_constants() -> tuple[list[tuple[str, str]], dict[str, str], dict[str, str]]:
    text = RUST.read_text()
    ov_block = re.search(r"pub\(crate\) const OVERRIDES:.*?= &\[(.*?)\];", text, re.S).group(1)
    overrides: list[tuple[str, str]] = []
    for m in re.finditer(
        r'\(\s*"([^"]+)"\s*,\s*Override::(?:Path|Pickaxe)\(\s*(?:"[^"]*"\s*,\s*)?\s*"([^"]+)"',
        ov_block,
        re.S,
    ):
        overrides.append((m.group(1), m.group(2)))
    md_block = re.search(
        r"pub\(crate\) const MODULE_DEFAULT_PATH:.*?= &\[(.*?)\];", text, re.S
    ).group(1)
    module_default = dict(re.findall(r'\("([^"]+)",\s*"([^"]+)"\)', md_block))
    ms_block = re.search(r"pub\(crate\) const MODULE_S:.*?= &\[(.*?)\];", text, re.S).group(1)
    module_s = {
        mod: path
        for mod, _term, path in re.findall(
            r'\(\s*"([^"]+)",\s*"([^"]+)",\s*"([^"]+)"\s*\)', ms_block
        )
    }
    return overrides, module_default, module_s


OVERRIDES, MODULE_DEFAULT_PATH, MODULE_S = parse_rust_constants()


def snake_to_pascal(s: str) -> str:
    return "".join(part.capitalize() for part in s.split("_"))


def overrides_lookup_keys(journey: str, module: str) -> list[str]:
    keys: list[str] = []
    for key in (snake_to_pascal(journey), module):
        if key not in keys:
            keys.append(key)
    return keys


def source_path_for(journey: str, module: str) -> str | None:
    for key in overrides_lookup_keys(journey, module):
        for ov_key, path in OVERRIDES:
            if ov_key == key:
                return path
    if module in MODULE_S:
        return MODULE_S[module]
    return MODULE_DEFAULT_PATH.get(module)


def git_ok(args: list[str]) -> bool:
    return subprocess.run(args, cwd=REPO, capture_output=True).returncode == 0


def rev_parse(tag: str) -> str:
    return subprocess.check_output(["git", "rev-parse", tag], cwd=REPO, text=True).strip()


def presence_on(tag: str, intro_commit: str, source_path: str | None) -> tuple[bool, str]:
    anc = bool(intro_commit) and git_ok(
        ["git", "merge-base", "--is-ancestor", intro_commit, tag]
    )
    pth = bool(source_path) and git_ok(["git", "cat-file", "-e", f"{tag}:{source_path}"])
    if anc and pth:
        why = "ancestor+path"
    elif anc:
        why = "ancestor"
    elif pth:
        why = "path"
    else:
        why = "absent"
    return anc or pth, why


def main() -> None:
    tag_shas = {tag: rev_parse(tag) for tag in TAGS}
    journeys_out = []
    for j in json.loads(MAP.read_text())["journeys"]:
        journey, module = j["journey"], j["module"]
        intro = j.get("intro_commit") or ""
        sp = source_path_for(journey, module)
        p143, w143 = presence_on("1.4.3", intro, sp)
        p144, _ = presence_on("1.4.4", intro, sp)
        pb23, _ = presence_on("1.4.4-beta.23", intro, sp)
        p145, w145 = presence_on("1.4.5", intro, sp)
        journeys_out.append(
            {
                "journey": journey,
                "module": module,
                "intro_commit": intro,
                "source_path": sp,
                "present_1_4_3": p143,
                "present_1_4_4": p144,
                "present_beta23": pb23,
                "present_1_4_5": p145,
                "why_1_4_5": w145,
                "why_1_4_3": w143,
            }
        )

    def added_since(base: str, newer: str) -> list[str]:
        return sorted(r["journey"] for r in journeys_out if r[newer] and not r[base])

    added = added_since("present_1_4_3", "present_1_4_5")
    removed = sorted(
        r["journey"] for r in journeys_out if r["present_1_4_3"] and not r["present_1_4_5"]
    )
    added_on_1_4_4 = added_since("present_1_4_3", "present_1_4_4")
    added_on_beta23 = added_since("present_1_4_3", "present_beta23")
    sets_equal = added == added_on_1_4_4 == added_on_beta23

    out = {
        "tag_shas": tag_shas,
        "added": added,
        "removed": removed,
        "added_on_1_4_4": added_on_1_4_4,
        "added_on_beta23": added_on_beta23,
        "sets_equal": sets_equal,
        "counts": {
            "total": len(journeys_out),
            "present_1_4_3": sum(r["present_1_4_3"] for r in journeys_out),
            "present_1_4_5": sum(r["present_1_4_5"] for r in journeys_out),
            "added": len(added),
        },
        "journeys": journeys_out,
    }
    OUT.write_text(json.dumps(out, indent=2) + "\n")
    print(
        json.dumps(
            {
                "tag_shas": tag_shas,
                "counts": out["counts"],
                "added": added,
                "removed": removed,
                "sets_equal": sets_equal,
                "path": str(OUT),
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
