#!/usr/bin/env python3
"""Rebuild the two backlog INDEX files from the row files in dev-docs/backlog/, or check them.

Layout (see the header of dev-docs/backlog.md):
  dev-docs/backlog/<NNN>-<slug>.md        open row   -> listed in dev-docs/backlog.md (the open index)
  dev-docs/backlog/done/<NNN>-<slug>.md   done row   -> listed in dev-docs/backlog-done.md (the done index)
Only the block between the BEGIN/END GENERATED markers of each index file is generated; the header above it
is hand-written. A citation "dev-docs/backlog.md row N" resolves by id in either index.

Modes (run with `python3 -I`; scripts/backlog-index.sh wraps this):
  (default)  sync the derived front matter fields (`status`, `size`) of every row file from its sections, then
             rewrite the generated block of dev-docs/backlog.md.
  --check    write nothing; exit 1 listing every problem: missing front matter fields, id / file name mismatch,
             duplicate ids, a row present in both open and done, missing or reordered sections, `status`/`size`
             out of sync with their sections, a stale index (open or done).
"""
import html
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "dev-docs"
BACKLOG = DOCS / "backlog"
INDEX = DOCS / "backlog.md"
DONE_INDEX = DOCS / "backlog-done.md"
BEGIN = "<!-- BEGIN GENERATED: scripts/backlog-index.sh rewrites everything up to END; edit the row files, not this block -->"
END = "<!-- END GENERATED -->"
SECTIONS = ["Item", "Status / trigger", "Size", "Detail"]
REQUIRED = ["id", "title", "status", "size", "area", "updated"]
NAME_RE = re.compile(r"^(\d{3,})-[a-z0-9][a-z0-9-]*\.md$")


# ---- derived fields
def strip_md(s):
    s = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", s)
    s = s.replace("**", "").replace("`", "")
    return re.sub(r"\s+", " ", s).strip()


def clip(s, n):
    if len(s) <= n:
        return s
    cut = s[:n].rsplit(" ", 1)[0] if " " in s[:n] else s[:n]
    return cut.rstrip(" ,;:-") + "…"


def derive_status(cell):
    return clip(strip_md(cell), 100)


def derive_size(cell):
    return clip(strip_md(cell), 40)


# ---- parsing
def split_front_matter(text):
    """-> (dict | None, body, fm_lines | None)"""
    lines = text.split("\n")
    if not lines or lines[0] != "---":
        return None, text, None
    try:
        end = lines.index("---", 1)
    except ValueError:
        return None, text, None
    fm = {}
    for ln in lines[1:end]:
        if not ln.strip():
            continue
        k, sep, v = ln.partition(":")
        if not sep:
            return None, text, None
        v = v.strip()
        if v.startswith('"'):
            try:
                v = json.loads(v)
            except ValueError:
                v = "\0bad-json"
        fm[k.strip()] = v
    return fm, "\n".join(lines[end + 1 :]), lines[1:end]


def parse_sections(body):
    """-> dict heading -> text for the four cell sections (exact `## ` headings), plus the order seen."""
    out, order, cur, buf = {}, [], None, []
    for ln in body.split("\n"):
        if ln.startswith("## "):
            if cur is not None:
                out[cur] = "\n".join(buf).strip("\n")
            cur, buf = ln[3:].strip(), []
            order.append(cur)
        elif cur is not None:
            buf.append(ln)
    if cur is not None:
        out[cur] = "\n".join(buf).strip("\n")
    return out, order


def load_rows():
    rows, problems = [], []
    for state, d in (("open", BACKLOG), ("done", BACKLOG / "done")):
        if not d.is_dir():
            problems.append(f"missing directory {d.relative_to(ROOT)}")
            continue
        for p in sorted(d.glob("*.md")):
            rel = p.relative_to(ROOT).as_posix()
            text = p.read_text(encoding="utf-8")
            fm, body, _ = split_front_matter(text)
            if fm is None:
                problems.append(f"{rel}: no front matter")
                continue
            miss = [k for k in REQUIRED if k not in fm or fm[k] == ""]
            if miss:
                problems.append(f"{rel}: front matter missing/empty: {', '.join(miss)}")
            if any(v == "\0bad-json" for v in fm.values()):
                problems.append(f"{rel}: a quoted front matter value is not valid JSON")
            m = NAME_RE.match(p.name)
            if not m:
                problems.append(f"{rel}: file name must be <NNN>-<slug>.md")
            try:
                rid = int(fm.get("id", ""))
            except ValueError:
                problems.append(f"{rel}: id is not an integer")
                continue
            if m and int(m.group(1)) != rid:
                problems.append(f"{rel}: id {rid} does not match the file name prefix {m.group(1)}")
            if m and m.group(1) != f"{rid:03d}" and rid < 1000:
                problems.append(f"{rel}: prefix must be the zero-padded id {rid:03d}")
            if not re.fullmatch(r"\d{4}-\d\d-\d\d|unknown", str(fm.get("updated", ""))):
                problems.append(f"{rel}: updated must be YYYY-MM-DD or unknown")
            secs, order = parse_sections(body)
            if [h for h in order if h in SECTIONS] != SECTIONS:
                problems.append(f"{rel}: sections must be exactly, in order: {', '.join('## ' + s for s in SECTIONS)}")
            else:
                if fm.get("status") != derive_status(secs["Status / trigger"]):
                    problems.append(f"{rel}: front matter `status` is out of sync with '## Status / trigger' (run scripts/backlog-index.sh)")
                if fm.get("size") != derive_size(secs["Size"]):
                    problems.append(f"{rel}: front matter `size` is out of sync with '## Size' (run scripts/backlog-index.sh)")
            rows.append({"id": rid, "state": state, "path": p, "rel": rel, "fm": fm, "secs": secs, "text": text})
    seen = {}
    for r in rows:
        if r["id"] in seen:
            a, b = seen[r["id"]], r
            where = "both open and done" if a["state"] != b["state"] else "twice"
            problems.append(f"row {r['id']} exists {where}: {a['rel']} and {b['rel']}")
        seen[r["id"]] = r
    return rows, problems


# ---- rendering
def cell(s):
    return html.escape(s, quote=False).replace("|", "\\|")


def render_open(rows):
    open_rows = sorted((r for r in rows if r["state"] == "open"), key=lambda r: r["id"])
    out = [BEGIN, "", f"## Open rows ({len(open_rows)})", "",
           "| # | Item | Status / trigger | Size | Area |", "|---|---|---|---|---|"]
    for r in open_rows:
        f = r["fm"]
        out.append(
            f"| [{r['id']}](backlog/{r['path'].name}) | {cell(f['title'])} | {cell(f['status'])} | {cell(f['size'])} | {cell(f['area'])} |"
        )
    out += ["", END]
    return "\n".join(out)


def render_done(rows):
    done_rows = sorted((r for r in rows if r["state"] == "done"), key=lambda r: r["id"])
    out = [BEGIN, "", f"## Done rows ({len(done_rows)})", "", "| # | Item | Updated |", "|---|---|---|"]
    for r in done_rows:
        f = r["fm"]
        out.append(f"| [{r['id']}](backlog/done/{r['path'].name}) | {cell(f['title'])} | {cell(f['updated'])} |")
    out += ["", END]
    return "\n".join(out)


def splice(path, index_text, block):
    a, b = index_text.find(BEGIN), index_text.find(END)
    if a < 0 or b < 0 or b < a:
        raise SystemExit(f"{path.relative_to(ROOT)}: BEGIN/END GENERATED markers not found")
    return index_text[:a] + block + index_text[b + len(END) :]


def sync_derived(rows):
    """Rewrite status/size front matter lines from the sections; returns the number of files changed."""
    changed = 0
    for r in rows:
        secs = r["secs"]
        if not all(s in secs for s in SECTIONS):
            continue
        want = {"status": derive_status(secs["Status / trigger"]), "size": derive_size(secs["Size"])}
        text = r["text"]
        for k, v in want.items():
            line = f"{k}: {json.dumps(v, ensure_ascii=False)}"
            text = re.sub(rf"(?m)^{k}: .*$", lambda _m: line, text, count=1)
        if text != r["text"]:
            r["path"].write_text(text, encoding="utf-8")
            r["text"] = text
            r["fm"].update(want)
            changed += 1
    return changed


def main():
    check = "--check" in sys.argv[1:]
    unknown = [a for a in sys.argv[1:] if a != "--check"]
    if unknown:
        print(__doc__)
        return 2
    rows, problems = load_rows()
    if not check:
        n = sync_derived(rows)
        rows, problems = load_rows()
        if n:
            print(f"backlog-index: synced status/size front matter in {n} row file(s)")
    if problems:
        print("backlog-index: PROBLEMS")
        for p in problems:
            print("  - " + p)
        return 1
    targets = [(INDEX, render_open(rows)), (DONE_INDEX, render_done(rows))]
    new = [(p, splice(p, p.read_text(encoding="utf-8"), block)) for p, block in targets]
    o = sum(r["state"] == "open" for r in rows)
    if check:
        stale = [p for p, text in new if text != p.read_text(encoding="utf-8")]
        if stale:
            names = ", ".join(p.relative_to(ROOT).as_posix() for p in stale)
            print(f"backlog-index: {names} stale; run scripts/backlog-index.sh and commit the result")
            return 1
        print(f"backlog-index: OK -- {o} open + {len(rows) - o} done rows, both indexes current")
        return 0
    for p, text in new:
        p.write_text(text, encoding="utf-8")
    print(f"backlog-index: wrote {INDEX.relative_to(ROOT)} ({o} open) and {DONE_INDEX.relative_to(ROOT)} ({len(rows) - o} done)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
