#!/usr/bin/env python3
"""Verify catalog journey presence on BlueOS tags (one-off, not part of catalog build)."""

import json
import re
import subprocess
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
RUST = REPO / "catalog/crates/catalog-git/src/feature_presence.rs"
MAP = REPO / "catalog/feature_presence_map.json"
BASELINE = REPO / "catalog/requirements-baselines/1.5.0-beta.40.json"
INVENTORY = (
    REPO / "catalog/extras/features-1.4.5-since-1.4.3/inventory.json"
)
OUT_DIR = Path(__file__).resolve().parent
OUT = OUT_DIR / "presence.json"

TAGS = ("1.4.2", "1.4.3")


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


def write_features_md(
    tag_shas: dict[str, str],
    journeys_out: list[dict],
    added: list[str],
    removed: list[str],
    by_id: dict[str, dict],
    inv_by_journey: dict[str, dict],
) -> None:
    present_142 = [r for r in journeys_out if r["present_1_4_2"]]
    present_143 = [r for r in journeys_out if r["present_1_4_3"]]
    unchanged = [r for r in journeys_out if r["present_1_4_2"] and r["present_1_4_3"]]
    def is_1_5(row: dict) -> bool:
        inv = inv_by_journey.get(row["journey"], {})
        first = str(inv.get("first_tag") or row.get("first_tag") or "")
        return first.startswith("1.5")

    absent_143 = [r for r in journeys_out if not r["present_1_4_3"] and is_1_5(r)]

    lines = [
        "# Catalog features: 1.4.3 since 1.4.2",
        "",
        "## Scope",
        "",
        "Tags and SHAs (`presence.json` `tag_shas`):",
        "",
        "| Tag | SHA |",
        "|-----|-----|",
        f"| 1.4.2 | `{tag_shas['1.4.2']}` |",
        f"| 1.4.3 | `{tag_shas['1.4.3']}` |",
        "",
        "Method: catalog journeys. A journey is present on a tag when `intro_commit` is an ancestor of the tag commit, or the journey `source_path` exists on that tag.",
        "",
        f"Inventory: {len(journeys_out)} catalog journeys total.",
        "",
        "Note: the catalog is incomplete. Git tree, commits, and shipped binaries are required for a full product inventory.",
        "",
        "## Added in 1.4.3 since 1.4.2",
        "",
    ]
    if not added:
        lines.append("None. Catalog journey presence is unchanged between 1.4.2 and 1.4.3.")
        lines.append("")
    else:
        for jid in added:
            r = by_id[jid]
            inv = inv_by_journey.get(jid, {})
            summary = inv.get("summary", jid)
            intro_short = (r.get("intro_commit") or "")[:12]
            why = f"{r['why_1_4_3']} / {r['why_1_4_2']}"
            lines.append(
                f"- **{summary}** (`{jid}`, {r['module']}, {intro_short}, {why})"
            )
            lines.append("")

    lines += [
        "## Unchanged count",
        "",
        f"{len(unchanged)} journeys were already present on 1.4.2 and remain present on 1.4.3.",
        "",
        "## Not in 1.4.3 (catalog-only / 1.5+)",
        "",
        f"These {len(absent_143)} inventory journeys are absent on 1.4.3. They are not part of the 1.4.3 feature set.",
        "",
    ]
    by_mod: dict[str, list[dict]] = defaultdict(list)
    for r in absent_143:
        by_mod[r["module"]].append(r)
    for mod in sorted(by_mod):
        lines.append(f"### {mod}")
        lines.append("")
        for r in sorted(by_mod[mod], key=lambda x: x["journey"]):
            inv = inv_by_journey.get(r["journey"], {})
            first = inv.get("first_tag") or r.get("first_tag") or "?"
            lines.append(f"- `{r['journey']}` (first_tag: {first})")
        lines.append("")

    lines += [
        "## Cross-check",
        "",
        f"Removed journeys: {removed if removed else 'none'}.",
        f"present_1.4.2={len(present_142)}; present_1.4.3={len(present_143)}; added={len(added)}.",
        "",
    ]
    (OUT_DIR / "FEATURES.md").write_text("\n".join(lines))


def write_functions(
    tag_shas: dict[str, str],
    journeys_out: list[dict],
    inv_by_journey: dict[str, dict],
) -> dict:
    journey_presence = {j["journey"]: j for j in journeys_out}
    warnings: list[str] = []
    baseline = json.loads(BASELINE.read_text())
    funcs = [r for r in baseline["requirements"] if r.get("kind") == "functional"]

    def present_on(tag_key: str, verifying: list[str]) -> bool:
        if not verifying:
            return False
        for jid in verifying:
            row = journey_presence.get(jid)
            if row is None:
                w = f"verifying_journey missing from presence.json: {jid}"
                if w not in warnings:
                    warnings.append(w)
                continue
            if row[tag_key]:
                return True
        return False

    def why_for(verifying: list[str]) -> str:
        parts = []
        for jid in verifying:
            row = journey_presence.get(jid)
            if row is None:
                parts.append(f"{jid}: missing")
                continue
            intro_short = (row.get("intro_commit") or "")[:12]
            parts.append(
                f"{jid}: {row['why_1_4_3']} ({intro_short}, {row['module']}, {row.get('source_path')}; 1.4.2={row['why_1_4_2']})"
            )
        return "; ".join(parts)

    records = []
    for f in funcs:
        vj = f.get("verifying_journeys") or []
        p142 = present_on("present_1_4_2", vj)
        p143 = present_on("present_1_4_3", vj)
        records.append(
            {
                "function_id": f.get("function_id") or f.get("id"),
                "statement_text": (f.get("statement") or {}).get("text") or "",
                "feature_id": f.get("feature_id"),
                "verifying_journeys": vj,
                "present_1_4_2": p142,
                "present_1_4_3": p143,
                "why": why_for(vj),
            }
        )

    added = sorted(
        (r for r in records if r["present_1_4_3"] and not r["present_1_4_2"]),
        key=lambda r: r["function_id"] or "",
    )
    unchanged = [r for r in records if r["present_1_4_2"] and r["present_1_4_3"]]
    def first_tag_for(r: dict) -> str:
        for jid in r["verifying_journeys"]:
            inv = inv_by_journey.get(jid, {})
            if inv.get("first_tag"):
                return str(inv["first_tag"])
            row = next((j for j in journeys_out if j["journey"] == jid), None)
            if row and row.get("first_tag"):
                return str(row["first_tag"])
        return ""

    not_in = sorted(
        (
            r
            for r in records
            if not r["present_1_4_3"] and first_tag_for(r).startswith("1.5")
        ),
        key=lambda r: r["function_id"] or "",
    )

    out = {
        "counts": {
            "total_functions": len(records),
            "present_1_4_2": sum(r["present_1_4_2"] for r in records),
            "present_1_4_3": sum(r["present_1_4_3"] for r in records),
            "added": len(added),
            "unchanged": len(unchanged),
            "not_in_1_4_3": len(not_in),
        },
        "added": [
            {
                "function_id": r["function_id"],
                "statement_text": r["statement_text"],
                "feature_id": r["feature_id"],
                "verifying_journeys": r["verifying_journeys"],
                "why": r["why"],
            }
            for r in added
        ],
        "not_in_1_4_3": [
            {
                "function_id": r["function_id"],
                "verifying_journeys": r["verifying_journeys"],
            }
            for r in not_in
        ],
        "warnings": warnings,
    }
    (OUT_DIR / "functions.json").write_text(json.dumps(out, indent=2) + "\n")

    lines = [
        "# Catalog functions: 1.4.3 since 1.4.2",
        "",
        "## Scope",
        "",
        "Functions are ActionCatalog Action requirements (kind==functional) from",
        "catalog/requirements-baselines/1.5.0-beta.40.json (108 total).",
        "",
        "A function is present on a tag when ANY verifying_journey is present on that tag.",
        "Journey presence comes from live git verification in presence.json",
        "(intro_commit ancestor and/or source_path on tag SHA).",
        "",
        "Tags and SHAs (presence.json tag_shas):",
        "",
        "| Tag | SHA |",
        "|-----|-----|",
        f"| 1.4.2 | `{tag_shas['1.4.2']}` |",
        f"| 1.4.3 | `{tag_shas['1.4.3']}` |",
        "",
        "Note: the catalog is incomplete. Git tree, commits, and shipped binaries are required for a full product inventory.",
        "",
        "## Added in 1.4.3 since 1.4.2",
        "",
    ]
    if not added:
        lines.append("None. Catalog function presence is unchanged between 1.4.2 and 1.4.3.")
        lines.append("")
    else:
        for r in added:
            vj = ", ".join(r["verifying_journeys"]) or "-"
            j0 = r["verifying_journeys"][0] if r["verifying_journeys"] else None
            jrow = journey_presence.get(j0 or "")
            intro_short = ((jrow or {}).get("intro_commit") or "")[:12]
            module = (jrow or {}).get("module", "?")
            why_short = (jrow or {}).get("why_1_4_3", "?")
            lines.append(
                f"- **{r['statement_text']}** (`{r['function_id']}`, {r['feature_id']}, {vj}, {intro_short}, {module}, {why_short}; 1.4.2=absent)"
            )
            lines.append("")

    lines += [
        "## Unchanged count",
        "",
        f"{len(unchanged)} functions were already present on 1.4.2 and remain present on 1.4.3.",
        "",
        "## Not in 1.4.3 (catalog-only / 1.5+)",
        "",
        f"These {len(not_in)} catalog functions are absent on 1.4.3.",
        "They are not part of the 1.4.3 function set.",
        "",
    ]
    by_mod: dict[str, list[dict]] = defaultdict(list)
    for r in not_in:
        mods = []
        for jid in r["verifying_journeys"]:
            row = journey_presence.get(jid)
            if row:
                mods.append(row["module"])
        mod = mods[0] if mods else "?"
        by_mod[mod].append(r)
    for mod in sorted(by_mod):
        lines.append(f"### {mod}")
        lines.append("")
        for r in by_mod[mod]:
            vj = ", ".join(r["verifying_journeys"])
            firsts = []
            for jid in r["verifying_journeys"]:
                inv = inv_by_journey.get(jid, {})
                if inv.get("first_tag"):
                    firsts.append(inv["first_tag"])
            first = firsts[0] if firsts else "?"
            lines.append(
                f"- `{r['function_id']}` ({vj}, first_tag: {first})"
            )
        lines.append("")

    (OUT_DIR / "FUNCTIONS.md").write_text("\n".join(lines))
    return out


def main() -> None:
    tag_shas = {tag: rev_parse(tag) for tag in TAGS}
    map_json = json.loads(MAP.read_text())
    inv_by_journey = {}
    if INVENTORY.exists():
        inv_by_journey = {j["journey"]: j for j in json.loads(INVENTORY.read_text())["journeys"]}
    for j in map_json["journeys"]:
        inv_by_journey.setdefault(j["journey"], {}).setdefault("first_tag", j.get("first_tag"))
        inv_by_journey.setdefault(j["journey"], {}).setdefault("module", j.get("module"))

    journeys_out = []
    for j in map_json["journeys"]:
        journey, module = j["journey"], j["module"]
        intro = j.get("intro_commit") or ""
        sp = source_path_for(journey, module)
        p142, w142 = presence_on("1.4.2", intro, sp)
        p143, w143 = presence_on("1.4.3", intro, sp)
        journeys_out.append(
            {
                "journey": journey,
                "module": module,
                "intro_commit": intro,
                "source_path": sp,
                "first_tag": j.get("first_tag"),
                "present_1_4_2": p142,
                "present_1_4_3": p143,
                "why_1_4_2": w142,
                "why_1_4_3": w143,
            }
        )

    added = sorted(r["journey"] for r in journeys_out if r["present_1_4_3"] and not r["present_1_4_2"])
    removed = sorted(r["journey"] for r in journeys_out if r["present_1_4_2"] and not r["present_1_4_3"])
    by_id = {r["journey"]: r for r in journeys_out}

    out = {
        "tag_shas": tag_shas,
        "added": added,
        "removed": removed,
        "counts": {
            "total": len(journeys_out),
            "present_1_4_2": sum(r["present_1_4_2"] for r in journeys_out),
            "present_1_4_3": sum(r["present_1_4_3"] for r in journeys_out),
            "added": len(added),
            "removed": len(removed),
        },
        "journeys": journeys_out,
    }
    OUT.write_text(json.dumps(out, indent=2) + "\n")
    write_features_md(tag_shas, journeys_out, added, removed, by_id, inv_by_journey)
    fn = write_functions(tag_shas, journeys_out, inv_by_journey)
    print(
        json.dumps(
            {
                "tag_shas": tag_shas,
                "counts": out["counts"],
                "added": added,
                "removed": removed,
                "function_counts": fn["counts"],
                "added_function_ids": [r["function_id"] for r in fn["added"]],
                "files": [
                    str(OUT),
                    str(OUT_DIR / "functions.json"),
                    str(OUT_DIR / "FEATURES.md"),
                    str(OUT_DIR / "FUNCTIONS.md"),
                ],
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
