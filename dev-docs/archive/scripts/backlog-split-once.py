#!/usr/bin/env python3
"""One-off, historical: split the monolithic dev-docs/backlog.md into dev-docs/backlog/.

Input: the original table file (default: `git show de7059ba:dev-docs/backlog.md`, or the path given as argv[1]).
Output (under dev-docs/backlog/): <NNN>-<slug>.md per open row, done/<NNN>-<slug>.md per done row,
notes/<slug>.md per narrative section. It does NOT write dev-docs/backlog.md (scripts/backlog-index.py does).

Kept for provenance and so the proof in dev-docs/archive/scripts/backlog-split-verify.py can be re-run against the same input.
After the split the row files are the source of truth; do not re-run this over edited files.

Run: python3 -I dev-docs/archive/scripts/backlog-split-once.py [original-backlog.md]
"""
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "dev-docs" / "backlog"
BASE_COMMIT = "de7059ba"

ROW_RE = re.compile(r"\| (\d+) \|")
DONE_RE = re.compile(r"(LANDED|FIXED|RESOLVED|CLOSED|SUPERSEDED|DONE)\b", re.I)
# a done-looking status that still names open work stays open
OPEN_OVERRIDE = re.compile(
    r"PARTLY|PARTIAL|residue below|stays OPEN|remaining check|open follow-ups at the end", re.I
)

AREA_RULES = [
    ("ci", r"\bCI\b|3-browser|workflow|github actions"),
    ("process", r"lane|worktree|session process|gate procedure|run-gates|definition-of-done|disk-full|CARGO_TARGET_DIR|cargo build-directory"),
    ("build", r"\bSSG\b|dx build|cargo|deploy|stylelint|installer|\bCLI\b|registry|starter template|build\.rs|dependency"),
    ("tests", r"playwright|\bspec\b|\bspecs\b|oracle|\baxe\b|flaky|test-suite|tier-?[12]|mutation|fixtures?|harness|\.spec\.ts|unit test"),
    ("docs", r"\bdocs?\b|documentation|favicon|website|fonts|demo parity|demo upgrades|props reference"),
    ("theme", r"theme|token|\bcss\b|stylesheet|nova|dark|hover|motion|animation|shadcn parity|unification"),
    ("charts", r"chart|recharts"),
    ("carousel", r"carousel"),
    ("virtual", r"virtual|message[_ ]scroller|windowed|find-in-page"),
    ("menus", r"menu|menubar|navbar|sidebar|\bnav\b|navigation"),
    ("overlays", r"popover|tooltip|hovercard|dialog|drawer|sheet|toast|top[-_ ]layer|overlay|anchor|combobox|select\b"),
    ("inputs", r"input|otp|calendar|date[ _-]?picker|color[ _-]?picker|slider|toggle|tabs|form"),
    ("upstream", r"upstream|dioxus-cli|arborium|framework issues|fork"),
    ("a11y", r"aria|\bapg\b|keyboard|focus|\brtl\b|role="),
]


def strip_md(s: str) -> str:
    s = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", s)
    s = s.replace("**", "").replace("`", "")
    return re.sub(r"\s+", " ", s).strip()


def clip(s: str, n: int) -> str:
    if len(s) <= n:
        return s
    cut = s[:n].rsplit(" ", 1)[0] if " " in s[:n] else s[:n]
    return cut.rstrip(" ,;:-") + "…"


def derive_status(cell: str) -> str:
    return clip(strip_md(cell), 100)


def derive_size(cell: str) -> str:
    return clip(strip_md(cell), 40)


def make_title(item: str) -> str:
    t = strip_md(item)
    if "-- was:" in t and DONE_RE.match(t):
        t = t.split("-- was:", 1)[1].strip()
    elif DONE_RE.match(t):
        # "LANDED 2026-09-04: rest", "LANDED 2026-09-04, verified (X). rest"
        m = re.match(r"[^:.—]*?[:.—]\s*(.+)", t)
        if m and len(m.group(1)) > 12:
            t = m.group(1)
    return clip(t, 110)


def slugify(t: str) -> str:
    s = re.sub(r"[^a-z0-9]+", "-", t.lower()).strip("-")
    if len(s) > 48:
        s = s[:48].rsplit("-", 1)[0] if "-" in s[:48] else s[:48]
    return s.strip("-") or "row"


def area_of(item: str, detail: str) -> str:
    text = strip_md(item) + " " + strip_md(detail)[:300]
    for name, rx in AREA_RULES:
        if re.search(rx, strip_md(item), re.I):
            return name
    for name, rx in AREA_RULES:
        if re.search(rx, text, re.I):
            return name
    return "misc"


def updated_of(status: str, whole: str) -> str:
    for text in (status, whole):
        ds = re.findall(r"\b20\d\d-\d\d-\d\d\b", text)
        if ds:
            return max(ds)
    return "unknown"


def rebase(text: str, up: str) -> str:
    # relative links in the old backlog.md pointed at siblings in dev-docs/ as ](./x); the new files sit deeper
    return text.replace("](./", "](" + up)


def jstr(s: str) -> str:
    return json.dumps(s, ensure_ascii=False)


def main():
    if len(sys.argv) > 1:
        src = Path(sys.argv[1]).read_text(encoding="utf-8")
    else:
        src = subprocess.check_output(
            ["git", "show", f"{BASE_COMMIT}:dev-docs/backlog.md"], cwd=ROOT
        ).decode("utf-8")
    lines = src.split("\n")

    rows, notes = [], []  # notes: list of [heading_line_or_None, [lines]]
    orig_header = []
    cur = None
    for ln in lines:
        if ROW_RE.match(ln):
            c = ln[2:-2].split(" | ")
            assert len(c) == 5, ln[:60]
            rows.append(c)
            continue
        if ln.startswith("| # | Item |") or ln.startswith("|---|---|---|---|---|"):
            orig_header.append(ln)
            continue
        if re.match(r"#{1,3} ", ln):
            cur = [ln, []]
            notes.append(cur)
            continue
        if cur is None:
            cur = [None, []]
            notes.append(cur)
        cur[1].append(ln)

    # ---- rows
    (OUT / "done").mkdir(parents=True, exist_ok=True)
    (OUT / "notes").mkdir(parents=True, exist_ok=True)
    related = {"116": ["row-116-before-after-2026-10-04"]}
    n_open = n_done = 0
    for rid, item, status, size, detail in rows:
        title = make_title(item)
        st0 = re.sub(r"^[\s*~_]+", "", status)
        done = bool(DONE_RE.match(st0)) and not OPEN_OVERRIDE.search(status)
        num = int(rid)
        fname = f"{num:03d}-{slugify(title)}.md"
        up = "../../" if done else "../"
        fm = [
            "---",
            f"id: {num}",
            f"title: {jstr(title)}",
            f"status: {jstr(derive_status(status))}",
            f"size: {jstr(derive_size(size))}",
            f"area: {area_of(item, detail)}",
            f"updated: {updated_of(status, ' '.join([item, status, size, detail]))}",
            "---",
            "",
            f"# Row {num}: {title}",
            "",
            "## Item",
            "",
            rebase(item, up),
            "",
            "## Status / trigger",
            "",
            rebase(status, up),
            "",
            "## Size",
            "",
            rebase(size, up),
            "",
            "## Detail",
            "",
            rebase(detail, up),
            "",
        ]
        for slug in related.get(rid, []):
            fm += ["## Related notes", "", f"- [{slug}]({'../notes/' if done else 'notes/'}{slug}.md)", ""]
        path = OUT / ("done" if done else "") / fname
        path.write_text("\n".join(fm), encoding="utf-8")
        n_done += done
        n_open += not done

    # ---- notes
    used = set()
    for head, body in notes:
        title = head.lstrip("#").strip()
        m = re.search(r"(\d{4}-\d\d-\d\d)", title)
        date = m.group(1) if m else ""
        intro = head.startswith("# ")
        if intro:
            slug = "ranking-intro"
        elif title.startswith("Fresh-session order"):
            slug = "fresh-session-order-" + date + ("-next-session" if "next session" in title else "")
        elif title.startswith("What a fresh session should pick up next"):
            slug = "pickup-next-" + date
        elif title.startswith("Row 116"):
            slug = "row-116-before-after-" + date
        elif "queue evaluation" in title:
            slug = "queue-evaluation-" + date
        else:
            slug = slugify(title)
        assert slug not in used, slug
        used.add(slug)
        text = ["---", f"title: {jstr(title)}", f"date: {date or 'n/a'}", "kind: note", "---", "", head] + body
        if intro:
            text += [
                "",
                "## Original table columns",
                "",
                "The old file's table used these columns; the row files name them Item, Status / trigger "
                "(old: Trigger), Size (old: Effort) and Detail (old: Notes):",
                "",
            ] + orig_header
        txt = "\n".join(text).rstrip("\n") + "\n"
        (OUT / "notes" / f"{slug}.md").write_text(rebase(txt, "../../"), encoding="utf-8")

    print(f"rows: {len(rows)} (open {n_open}, done {n_done}); notes: {len(notes)}")


if __name__ == "__main__":
    main()
