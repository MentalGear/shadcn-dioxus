#!/usr/bin/env python3
"""Preservation proof for the backlog split: nothing from the original monolithic file was lost.

Parses the ORIGINAL dev-docs/backlog.md (default: `git show de7059ba:dev-docs/backlog.md`, or a path / `-` for
stdin) independently of the splitter and checks against dev-docs/backlog/:
  1. every table row id has exactly one row file (open or done), and no row file has an id the original lacks;
  2. each row's four cells (Item, Trigger, Effort, Notes) equal the file's sections "## Item", "## Status / trigger",
     "## Size", "## Detail" -- verbatim, except that relative links were rebased one or two levels deeper
     (`](./x)` -> `](../x)` / `](../../x)`); cells that match only through that rebase are counted separately;
  3. every non-blank narrative line (everything that is not a table row) occurs in dev-docs/backlog/notes/*.md at least
     as many times as in the original (multiset containment); the lines that notes add are listed.
Exit 0 and a summary line when nothing is missing; exit 1 listing every missing cell / line otherwise.

Run: python3 -I dev-docs/archive/scripts/backlog-split-verify.py [original-backlog.md | -]
"""
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BACKLOG = ROOT / "dev-docs" / "backlog"
BASE_COMMIT = "de7059ba"
SECTION_OF = ["Item", "Status / trigger", "Size", "Detail"]  # old columns: Item, Trigger, Effort, Notes


def derebase(s):
    return s.replace("](../../", "](./").replace("](../", "](./")


def row_sections(text):
    lines = text.split("\n")
    idx = {}
    for i, ln in enumerate(lines):
        if ln.startswith("## ") and ln[3:].strip() in SECTION_OF + ["Related notes"] and ln[3:].strip() not in idx:
            idx[ln[3:].strip()] = i
    out = {}
    marks = sorted(idx.items(), key=lambda kv: kv[1])
    for k, (name, i) in enumerate(marks):
        j = marks[k + 1][1] if k + 1 < len(marks) else len(lines)
        out[name] = "\n".join(lines[i + 1 : j]).strip("\n")
    return out


def main():
    arg = sys.argv[1] if len(sys.argv) > 1 else None
    if arg == "-":
        src = sys.stdin.read()
    elif arg:
        src = Path(arg).read_text(encoding="utf-8")
    else:
        src = subprocess.check_output(["git", "show", f"{BASE_COMMIT}:dev-docs/backlog.md"], cwd=ROOT).decode("utf-8")

    rows, narrative = {}, []
    for ln in src.split("\n"):
        m = re.match(r"^\| (\d+) \| (.*?) \| (.*?) \| (.*?) \| (.*) \|$", ln)
        if m:
            rows[int(m.group(1))] = list(m.groups()[1:])
        elif ln.strip():
            narrative.append(ln)

    files = {}
    for d in (BACKLOG, BACKLOG / "done"):
        for p in d.glob("*.md"):
            m = re.match(r"(\d+)-", p.name)
            if m:
                files.setdefault(int(m.group(1)), []).append(p)

    missing, cells_ok, cells_rebased = [], 0, 0
    for rid, cells in sorted(rows.items()):
        fs = files.get(rid, [])
        if len(fs) != 1:
            missing.append(f"row {rid}: {len(fs)} row files")
            continue
        secs = row_sections(fs[0].read_text(encoding="utf-8"))
        for name, orig in zip(SECTION_OF, cells):
            new = secs.get(name)
            if new == orig:
                cells_ok += 1
            elif new is not None and derebase(new) == orig:
                cells_ok += 1
                cells_rebased += 1
            else:
                missing.append(f"row {rid}: section '{name}' differs from the original cell")
    extra_ids = sorted(set(files) - set(rows))
    for rid in extra_ids:
        missing.append(f"row file {rid} has no row in the original")

    notes_lines = Counter()
    n_notes = 0
    for p in sorted((BACKLOG / "notes").glob("*.md")):
        n_notes += 1
        for ln in derebase(p.read_text(encoding="utf-8")).split("\n"):
            if ln.strip():
                notes_lines[ln] += 1
    orig_lines = Counter(narrative)
    lines_ok = 0
    for ln, n in orig_lines.items():
        have = notes_lines.get(ln, 0)
        lines_ok += min(n, have)
        if have < n:
            missing.append(f"narrative line missing x{n - have}: {ln[:100]}")
    added = notes_lines - orig_lines

    print(
        f"backlog-split-verify: rows in original {len(rows)}, row files {sum(len(v) for v in files.values())}; "
        f"cells matched {cells_ok}/{len(rows) * 4} ({cells_rebased} only after the link rebase); "
        f"narrative lines matched {lines_ok}/{len(narrative)} in {n_notes} notes files; "
        f"lines added by notes {sum(added.values())} (front matter / provenance); missing {len(missing)}"
    )
    for ln in sorted(added):
        if not ln.startswith(("title:", "date:", "kind:", "---")):
            print("  note-only line: " + ln[:110])
    if missing:
        for m in missing:
            print("  MISSING " + m)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
