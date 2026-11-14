#!/usr/bin/env python3
"""Close backlog rows without breaking what still names them, and check that nothing names a closed row.

A row that is finished moves from dev-docs/backlog/ to dev-docs/backlog/done/. Three things keep naming a row by id after
it moved and then break a gate later (row 451 was named by scripts/check-hot-events.allow.tsv):

  owner column   a `scripts/**/*.tsv` table whose header comment declares a `row` column (`# path<TAB>reason<TAB>row. ...`,
                 the allow and callers tables of check-hot-events and check-bridge-behaviours): the id in that column must be an
                 OPEN row, because a closed row owns nothing. The tables are derived from that header; none is listed here.
  depends on     an open row's `- **Depends on:** ...` bullet that names the row (`[row N](...)` or `row N`) and does not say it
                 is settled: a link into `done/`, or `(landed)`, `(merged)`, `(closed)`, `(done)`, `(withdrawn)`, `(superseded)`,
                 `(rejected)` or `(declined)` right after the reference.
  js-grant       an open row with a non-empty `js-grant:` is read by the ratchet (scripts/lib/rows.py); a closed row is not, so
                 closing it withdraws the grant of a lane whose raise is not merged yet. Only the closing command checks this
                 (a grant already spent cannot be told from one still waiting, so a gate cannot); `--grant-spent` says the
                 grant is spent or no longer needed.

Modes (run with `python3 -I`; scripts/backlog-close.sh wraps the first):
  <id>... [--status TEXT] [--grant-spent]
      refuses, naming file and line, while an open reference exists (the rows closed together do not count against each
      other); otherwise `git mv`s each row file to done/, repairs the relative links of the moved file and every link to it in
      the tracked docs, scripts and tables, writes the closed status (the row's own `## Status / trigger` when it already
      starts with CLOSED, LANDED, DONE, MERGED, WITHDRAWN, REJECTED, DECLINED or SUPERSEDED, else `CLOSED <date>. ` in front of
      it, or --status TEXT, which must start with one of those words) and `updated:`, and runs scripts/backlog-index.sh.
  --check
      the gate (scripts/check-backlog-index.sh runs it): every owner-column id is an open row and no open row's Depends on
      names a closed row unsettled. Exit 1 lists each as `file:line: message`.
  --self-test
      scratch trees with each kind of reference, the refusal, the move and the link repair.
"""
import datetime
import os
import re
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "lib"))
from rows import CLOSED_STATUS, front_matter, parse_columns  # noqa: E402

DONE_WORDS = r"(?:landed|merged|closed|done|withdrawn|superseded|rejected|declined)"
SETTLED = re.compile(rf"\s*\(\s*{DONE_WORDS}\b", re.I)
REF = re.compile(r"\[[^\]]*\]\(([^)\s]*)\)|\brows?\s+(\d+)\b", re.I)
SKIP_DIRS = {"node_modules", "target", ".git", "archive"}
LINK = re.compile(r"\]\(([^)#\s]+)(#[^)]*)?\)")


def row_files(root):
    """{id: (path relative to root, closed)} of every row file."""
    out = {}
    for sub, closed in (("dev-docs/backlog", False), ("dev-docs/backlog/done", True)):
        d = os.path.join(root, sub)
        for name in sorted(os.listdir(d)) if os.path.isdir(d) else []:
            m = re.match(r"(\d+)-.*\.md$", name)
            if m:
                out[int(m.group(1))] = (f"{sub}/{name}", closed)
    return out


def read(root, rel):
    with open(os.path.join(root, rel), encoding="utf-8") as f:
        return f.read()


def owner_tables(root):
    """[(path, column index)] of the `.tsv` tables under scripts/ whose first comment line names a `row` column."""
    out = []
    base = os.path.join(root, "scripts")
    for dp, dirs, files in os.walk(base):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        for name in sorted(files):
            if not name.endswith(".tsv"):
                continue
            rel = os.path.relpath(os.path.join(dp, name), root)
            first = next((ln for ln in read(root, rel).split("\n") if ln.strip()), "")
            if not first.startswith("#"):
                continue
            cols = [c.strip("# ") for c in re.split(r"<TAB>|\t", first)]
            idx = [i for i, c in enumerate(cols) if re.match(r"row\b", c)]
            if idx and len(cols) > 1:
                out.append((rel, idx[0]))
    return sorted(out)


def table_refs(root, ids=None):
    """[(path, line, id, text)] of the owner-column ids of every derived table (only `ids` when given)."""
    out = []
    for rel, i in owner_tables(root):
        for n, ln in enumerate(read(root, rel).split("\n"), 1):
            if not ln.strip() or ln.startswith("#"):
                continue
            cells = ln.split("\t")
            if i < len(cells):
                for m in re.finditer(r"\b\d+\b", cells[i]):
                    if ids is None or int(m.group(0)) in ids:
                        out.append((rel, n, int(m.group(0)), cells[0][:60]))
    return out


def depends_refs(root, rows, skip=frozenset()):
    """[(path, line, id, settled)] of the references in the Depends on bullets of the open rows not in `skip`."""
    out = []
    for rid, (rel, closed) in sorted(rows.items()):
        if closed or rid in skip or CLOSED_STATUS.match(front_matter(read(root, rel)).get("status", "")):
            continue
        lines = read(root, rel).split("\n")
        i = 0
        while i < len(lines):
            if re.match(r"\s*[-*]?\s*\**Depends on:?\**", lines[i], re.I):
                j = i
                while j + 1 < len(lines) and re.match(r"\s{2,}\S", lines[j + 1]) and not re.match(r"\s*[-*] ", lines[j + 1]):
                    j += 1
                text = " ".join(lines[i:j + 1])
                for m in REF.finditer(text):
                    target, num = m.group(1), m.group(2)
                    n = None
                    if target is not None:
                        mm = re.search(r"(?:^|/)(\d+)-[^/]*\.md$", target)
                        n = int(mm.group(1)) if mm else None
                        if n is None:
                            mt = re.search(r"\brows?\s+(\d+)", m.group(0), re.I)
                            n = int(mt.group(1)) if mt else None
                    elif num:
                        n = int(num)
                    if n is None:
                        continue
                    settled = (target is not None and "/done/" in "/" + target) or bool(SETTLED.match(text[m.end():]))
                    out.append((rel, i + 1, n, settled))
                i = j
            i += 1
    return out


def check_tree(root):
    """The gate's findings: `file:line: message` for each owner-column id that is not an open row and each unsettled
    Depends on reference to a closed row."""
    rows = row_files(root)
    bad = []
    for rel, n, rid, who in table_refs(root):
        if rid not in rows:
            bad.append(f"{rel}:{n}: `{who}` names row {rid}, which does not exist")
        elif rows[rid][1]:
            bad.append(f"{rel}:{n}: `{who}` names row {rid}, which is closed ({rows[rid][0]}); a closed row owns nothing")
    for rel, n, rid, settled in depends_refs(root, rows):
        if rid in rows and rows[rid][1] and not settled:
            bad.append(f"{rel}:{n}: Depends on names row {rid}, which is closed ({rows[rid][0]}); link it under done/ or write `(landed)` after it")
    return bad


def close_findings(root, ids, grant_spent):
    """Why the rows `ids` cannot be closed now: `file:line: message` for each open reference."""
    rows = row_files(root)
    bad = []
    for rid in ids:
        if rid not in rows:
            bad.append(f"row {rid}: no row file under dev-docs/backlog/")
        elif rows[rid][1]:
            bad.append(f"{rows[rid][0]}: row {rid} is already closed")
    if bad:
        return bad
    for rel, n, rid, who in table_refs(root, set(ids)):
        bad.append(f"{rel}:{n}: `{who}` names row {rid} as its owner row; delete or re-own the entry first")
    for rel, n, rid, settled in depends_refs(root, rows, skip=set(ids)):
        if rid in ids and not settled:
            bad.append(f"{rel}:{n}: Depends on names row {rid}; link it under done/ or write `(landed)` after it, or drop the dependency")
    if not grant_spent:
        for rid in ids:
            rel = rows[rid][0]
            fm = front_matter(read(root, rel))
            if parse_columns(fm.get("js-grant", "")):
                ln = next((k for k, t in enumerate(read(root, rel).split("\n"), 1) if t.startswith("js-grant:")), 1)
                bad.append(f"{rel}:{ln}: row {rid} still carries a `js-grant:` the ratchet reads while the row is open; "
                           f"closing it withdraws the grant of a raise that is not merged yet (--grant-spent when it is spent or not needed)")
    return bad


def git(root, *args):
    return subprocess.run(["git", "-C", root, *args], capture_output=True, text=True, check=True).stdout


def move_rows(root, ids, status, today):
    rows = row_files(root)
    moved = {}
    for rid in ids:
        old = rows[rid][0]
        new = "dev-docs/backlog/done/" + os.path.basename(old)
        git(root, "mv", old, new)
        moved[old] = new
        text = read(root, new)
        m = re.search(r"(?ms)^## Status / trigger\s*\n\s*\n?(.*?)(?=^## )", text)
        body = m.group(1) if m else ""
        first = body.strip()
        if m and not CLOSED_STATUS.match(first):
            text = text[:m.start(1)] + f"{status or 'CLOSED ' + today + '.'} " + text[m.start(1):]
        text = re.sub(r"(?m)^updated: .*$", f"updated: {today}", text, count=1)
        with open(os.path.join(root, new), "w", encoding="utf-8") as f:
            f.write(text)
    fix_links(root, moved)
    return moved


def fix_links(root, moved):
    """Repair relative links after the moves: the moved files' own links, and every link (and bare path) naming a moved file."""
    olds = {o: n for o, n in moved.items()}
    gone = set(olds)

    def rel_to(src_dir, target):
        r = os.path.relpath(target, src_dir)
        return r if r.startswith(".") else "./" + r

    def fix_file(rel, text, was_at):
        def sub(m):
            t = m.group(1)
            if re.match(r"[a-z][a-z0-9+.-]*:", t, re.I) or t.startswith("/"):
                return m.group(0)
            old_target = os.path.normpath(os.path.join(os.path.dirname(was_at), t))
            new_target = olds.get(old_target, old_target)
            if old_target not in gone and was_at == rel:
                return m.group(0)
            if not os.path.exists(os.path.join(root, new_target)):
                return m.group(0)
            return "](" + rel_to(os.path.dirname(rel), new_target) + (m.group(2) or "") + ")"
        return LINK.sub(sub, text)

    files = [p for p in git(root, "ls-files", "*.md", "*.sh", "*.py", "*.mjs", "*.ts", "*.tsv").split("\n") if p]
    for rel in files:
        if rel.startswith("docs/") or rel in ("dev-docs/backlog.md", "dev-docs/backlog-done.md"):
            continue
        path = os.path.join(root, rel)
        if not os.path.isfile(path):
            continue
        was = next((o for o, n in moved.items() if n == rel), rel)
        try:
            text = read(root, rel)
        except (OSError, UnicodeDecodeError):
            continue
        new = fix_file(rel, text, was)
        for o, n in moved.items():
            new = new.replace(o, n)          # a repo-relative path written out in prose or a comment
        if new != text:
            with open(path, "w", encoding="utf-8") as f:
                f.write(new)


def run_close(root, ids, status, grant_spent):
    if status and not CLOSED_STATUS.match(status):
        print("backlog-close: --status must start with CLOSED, LANDED, DONE, MERGED, WITHDRAWN, REJECTED, DECLINED or SUPERSEDED", file=sys.stderr)
        return 2
    bad = close_findings(root, ids, grant_spent)
    if bad:
        print("backlog-close: REFUSED, nothing moved:", file=sys.stderr)
        for b in bad:
            print("  " + b, file=sys.stderr)
        return 1
    today = datetime.date.today().isoformat()
    moved = move_rows(root, ids, status, today)
    r = subprocess.run([os.path.join(root, "scripts/backlog-index.sh")], capture_output=True, text=True, cwd=root)
    sys.stdout.write(r.stdout)
    sys.stderr.write(r.stderr)
    for o, n in moved.items():
        print(f"backlog-close: {o} -> {n}")
    return r.returncode


# ---- self-test
ROW = """---
id: {n}
title: "Row {n}"
status: "{status}"
size: "S"
area: tooling
updated: 2026-10-01
{extra}---

# Row {n}: row {n}

## Item

Item. See [the sibling](./{sib}).

## Status / trigger

{status}

## Size

S

## Detail

{detail}
"""


def _write(root, rel, text):
    p = os.path.join(root, rel)
    os.makedirs(os.path.dirname(p), exist_ok=True)
    with open(p, "w", encoding="utf-8") as f:
        f.write(text)


def self_test():
    def need(cond, what):
        if not cond:
            raise SystemExit(f"backlog-close: self-test FAILED: {what}")

    with tempfile.TemporaryDirectory() as root:
        git(root, "init", "-q")
        git(root, "config", "user.email", "t@example.invalid")
        git(root, "config", "user.name", "t")
        for src in ("scripts/backlog-index.py", "scripts/backlog-index.sh"):
            _write(root, src, open(os.path.join(HERE, "..", src), encoding="utf-8").read())
        os.chmod(os.path.join(root, "scripts/backlog-index.sh"), 0o755)
        for name in ("backlog.md", "backlog-done.md"):
            _write(root, f"dev-docs/{name}", "# idx\n\n<!-- BEGIN GENERATED: scripts/backlog-index.sh rewrites everything up to END; edit the row files, not this block -->\n<!-- END GENERATED -->\n")
        fx = lambda n, status="BACKLOG. open", extra="", sib="002-b.md", detail="none": ROW.format(n=n, status=status, extra=extra, sib=sib, detail=detail)
        _write(root, "dev-docs/backlog/001-a.md", fx(1, sib="002-b.md", detail="see [two](./002-b.md)"))
        _write(root, "dev-docs/backlog/002-b.md", fx(2, sib="001-a.md"))
        _write(root, "dev-docs/backlog/003-c.md", fx(3, sib="001-a.md", detail="- **Depends on:** [row 1](./001-a.md), row 2 (landed)"))
        _write(root, "dev-docs/backlog/004-d.md", fx(4, sib="001-a.md", extra="js-grant: js_file_bytes=10\n"))
        _write(root, "dev-docs/backlog/005-e.md", fx(5, sib="001-a.md", status="LANDED in abc"))
        _write(root, "dev-docs/backlog/done/006-f.md", fx(6, sib="../001-a.md", status="LANDED earlier", detail="- **Depends on:** [row 5](../005-e.md)"))
        _write(root, "scripts/x.allow.tsv", "# path<TAB>reason<TAB>row. Allowlist.\nfile.rs\treason\t1\nother.rs\treason\t99\n")
        _write(root, "scripts/y.baseline.tsv", "# path\tbytes\nfile.rs\t5\n")
        _write(root, "scripts/z.allow.tsv", "# path<TAB>why (row 1)\nf.rs\tbecause row 1\n")
        _write(root, "scripts/doc.md", "see [row](../dev-docs/backlog/005-e.md) and dev-docs/backlog/005-e.md\n")
        git(root, "add", "-A")
        git(root, "commit", "-qm", "base")
        need(owner_tables(root) == [("scripts/x.allow.tsv", 2)], f"derived tables {owner_tables(root)}")
        found = check_tree(root)
        need(any("x.allow.tsv:3" in f and "row 99" in f and "does not exist" in f for f in found), f"missing row not found: {found}")
        need(any("003-c.md" in f and "row 5" not in f for f in found) is False, f"a settled reference was flagged: {found}")
        need(any("006" not in f and "Depends" in f for f in found) is False, "closed row's own Depends on was flagged")
        # refusals, each naming file and line
        bad = close_findings(root, [1], False)
        need(any("x.allow.tsv:2" in b for b in bad), f"owner table not named: {bad}")
        need(any("003-c.md:" in b and "Depends on" in b for b in bad), f"Depends on not named: {bad}")
        need(not any("row 2" in b and "003-c" in b for b in bad), "a settled reference refused")
        bad = close_findings(root, [4], False)
        need(len(bad) == 1 and "004-d.md:" in bad[0] and "js-grant" in bad[0], f"js-grant not refused: {bad}")
        need(close_findings(root, [4], True) == [], "--grant-spent still refused")
        need(close_findings(root, [2], False) == [], "a row only settled-referenced was refused")
        need(close_findings(root, [1, 2, 3], True) != [] and all("003" not in b for b in close_findings(root, [1, 2, 3], True) if "Depends" in b), "rows closed together count against each other")
        need(run_close(root, [1], None, False) == 1 and os.path.exists(os.path.join(root, "dev-docs/backlog/001-a.md")), "a refused close moved the row")
        # a real close: row 2 (settled ref), with link repair
        need(run_close(root, [2], None, False) == 0, "close of row 2 failed")
        need(os.path.exists(os.path.join(root, "dev-docs/backlog/done/002-b.md")) and not os.path.exists(os.path.join(root, "dev-docs/backlog/002-b.md")), "row 2 not moved")
        moved = read(root, "dev-docs/backlog/done/002-b.md")
        need(moved.startswith("---") and 'status: "CLOSED ' in moved and "## Status / trigger\n\nCLOSED " in moved, "status not set to CLOSED")
        need("](../001-a.md)" in moved, "the moved row's own link is not repaired")
        need("[two](./done/002-b.md)" in read(root, "dev-docs/backlog/001-a.md"), "a link to the moved row is not repaired")
        # a row whose status is already closed keeps its text; --status is used otherwise
        need(run_close(root, [5], None, False) == 0 and 'status: "LANDED in abc"' in read(root, "dev-docs/backlog/done/005-e.md"), "an already LANDED status was rewritten")
        need("](../dev-docs/backlog/done/005-e.md)" in read(root, "scripts/doc.md") and "\ndev-docs/backlog/done/005-e.md" not in read(root, "scripts/doc.md") and " dev-docs/backlog/done/005-e.md" in read(root, "scripts/doc.md"), "links and paths to row 5 are not repaired")
        need(run_close(root, [4], "bogus", True) == 2, "a --status without a closed word was accepted")
        need(run_close(root, [4], "LANDED in def", True) == 0 and 'status: "LANDED in def BACKLOG. open"' in read(root, "dev-docs/backlog/done/004-d.md"), "--status not applied")
        need(subprocess.run(["python3", "-I", os.path.join(root, "scripts/backlog-index.py"), "--check"], capture_output=True, text=True, cwd=root).returncode == 0, "the index is not current after a close")
        # the gate: a hand move of a row that a table names is caught
        git(root, "add", "-A")
        git(root, "commit", "-qm", "closed")
        _write(root, "scripts/x.allow.tsv", "# path<TAB>reason<TAB>row. Allowlist.\nfile.rs\treason\t5\n")
        found = check_tree(root)
        need(len(found) == 1 and "x.allow.tsv:2" in found[0] and "closed" in found[0], f"gate missed a table row naming a closed row: {found}")
        _write(root, "dev-docs/backlog/003-c.md", read(root, "dev-docs/backlog/003-c.md").replace("row 2 (landed)", "row 2"))
        found = check_tree(root)
        need(any("003-c.md" in f and "Depends on names row 2" in f for f in found), f"gate missed an unsettled Depends on: {found}")
    print("backlog-close: self-test OK")
    return 0


def main(argv):
    root = os.path.dirname(HERE)
    args = list(argv)
    if "--self-test" in args:
        return self_test()
    if "--root" in args:
        i = args.index("--root")
        root = os.path.abspath(args[i + 1])
        del args[i:i + 2]
    if "--check" in args:
        bad = check_tree(root)
        if bad:
            print("backlog-close: OPEN REFERENCES TO CLOSED ROWS (fix the reference, or reopen the row)")
            for b in bad:
                print("  - " + b)
            return 1
        return 0
    status = None
    if "--status" in args:
        i = args.index("--status")
        status = args[i + 1]
        del args[i:i + 2]
    spent = "--grant-spent" in args
    args = [a for a in args if a != "--grant-spent"]
    if not args or not all(a.isdigit() for a in args):
        print(__doc__)
        return 2
    return run_close(root, [int(a) for a in args], status, spent)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
