"""Reader and validator of the platform-rule-row TSV (row 444, phase 0.1).

A platform rule row is the second, closed admission ground for a behaviour effect that no probe can observe
(dev-docs/research/sync-js-root-2026-10-26.md, section 2.2, "The platform rule row"). The rows live in
`primitives/conformance/platform-rules.tsv`, one per `effect<TAB>site<TAB>category<TAB>citation<TAB>observable_on<TAB>
check_line<TAB>row`, behind a header line naming those columns. `#` lines and blank lines are skipped. A row is an owner
decision: this module never writes one.

What the reader checks (every defect is one message string):
  - the header names exactly the required columns, in order, and every row has that many cells, none blank;
  - `category` is one of the closed table below (the root doc's (a), (b), (c)) and `effect` is a member of it;
  - `observable_on` is `none` or a comma-separated list of Playwright project names that exist in
    `playwright/playwright.config.ts` (uncommented `name:` entries of `projects`);
  - no two rows share an `(effect, site)` pair, and no row repeats another cell for cell (via `allow.Allowlist`,
    whose reason is the `row` column, the backlog row that records the owner decision).
Not checked here: that a `site` still exists in the tree (the `site!` scan is item 0.3's artefact), pass a
`site_exists` callable to `problems` once it does. A row whose `observable_on` engine is launched by a gate-run
config is stale in the root doc's sense: `launched_engines_stale` takes the launched project names.

API (standard library plus the sibling `allow` module only):

    sys.path.insert(0, "scripts/lib")
    import platform_rules as pr
    table = pr.load(root)                    # PlatformTable(rows, errors); root defaults to the repo root
    table.errors                             # structural defects found while reading (header, cells, category)
    table.rows                               # list of Row namedtuples, one per well-formed data line
    pr.problems(table, root=root, site_exists=None, launched=None)   # all defects, as a list of str
    pr.projects(root)                        # the set of Playwright project names
    pr.CATEGORIES                            # {"a": (description, frozenset(effects)), ...}
    pr.COLUMNS                               # the header tuple
    pr.check_lines(table)                    # the distinct `check_line` values (for the desktop-pass.md count)
    pr.find(table, effect, site)             # the row admitting that effect at that site, or None
    pr.parse(text, projects, source)         # the same from a string and an explicit project set (self-test)

CLI: `platform_rules.py` validates the tree's TSV (exit 1 on a defect); `--self-test` runs the fixtures
(red: unknown category, unknown project, missing column, duplicate; green: a valid row).
"""
import os
import re
import sys
from collections import namedtuple
from typing import Callable, Iterable, List, Optional

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from allow import Allowlist  # noqa: E402

__all__ = ["CATEGORIES", "COLUMNS", "Row", "PlatformTable", "load", "parse", "problems", "projects", "check_lines",
           "find", "launched_engines_stale"]

TSV = "primitives/conformance/platform-rules.tsv"
PW_CONFIG = "playwright/playwright.config.ts"
COLUMNS = ("effect", "site", "category", "citation", "observable_on", "check_line", "row")
Row = namedtuple("Row", COLUMNS + ("line",))
PlatformTable = namedtuple("PlatformTable", "rows errors source")

# The closed category table of the root doc (section 2.2, rule 4). Edited only through an owner decision row.
# (b) has no row while no library code uses the clipboard.
CATEGORIES = {
    "a": ("prevent of a UA-reserved shortcut", frozenset({"prevent"})),
    "b": ("an activation-gated API effect, by effect name",
          frozenset({"clipboardWrite", "requestFullscreen", "showPicker", "share"})),
    "c": ("an engine-specific default action from a cited table", frozenset({"dataTransfer"})),
}


def _root(root: Optional[str]) -> str:
    return root or os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def projects(root: Optional[str] = None) -> frozenset:
    """Names of the uncommented Playwright projects in the config."""
    text = open(os.path.join(_root(root), PW_CONFIG), encoding="utf-8").read()
    names = set()
    for ln in text.splitlines():
        if ln.lstrip().startswith("//"):
            continue
        m = re.match(r"\s*name:\s*[\"']([^\"']+)[\"']", ln)
        if m:
            names.add(m.group(1))
    return frozenset(names)


def parse(text: str, known_projects: Iterable[str], source: str = TSV) -> PlatformTable:
    known = frozenset(known_projects)
    errors: List[str] = []
    rows: List[Row] = []
    header_seen = False
    for no, raw in enumerate(text.splitlines(), 1):
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        cells = raw.split("\t")
        where = f"{source}:{no}"
        if not header_seen:
            header_seen = True
            if tuple(cells) != COLUMNS:
                missing = [c for c in COLUMNS if c not in cells]
                errors.append(f"{where}: header must be {'<TAB>'.join(COLUMNS)}"
                              + (f" (missing column: {', '.join(missing)})" if missing else ""))
            continue
        if len(cells) != len(COLUMNS):
            errors.append(f"{where}: {len(cells)} cells, expected {len(COLUMNS)} ({', '.join(COLUMNS)})")
            continue
        blank = [c for c, v in zip(COLUMNS, cells) if not v.strip()]
        if blank:
            errors.append(f"{where}: blank cell in column {', '.join(blank)}")
            continue
        r = Row(*cells, no)
        cat = CATEGORIES.get(r.category)
        if cat is None:
            errors.append(f"{where}: unknown category {r.category!r} (closed table: {', '.join(sorted(CATEGORIES))}); "
                          "a new category is an owner decision")
        elif r.effect not in cat[1]:
            errors.append(f"{where}: effect {r.effect!r} is not a member of category ({r.category}) "
                          f"({', '.join(sorted(cat[1]))})")
        if r.observable_on != "none":
            for p in (x.strip() for x in r.observable_on.split(",")):
                if p not in known:
                    errors.append(f"{where}: observable_on {p!r} is not a Playwright project "
                                  f"({', '.join(sorted(known))}) -- stale or misspelt")
        rows.append(r)
    if not header_seen:
        errors.append(f"{source}: no header line")
    return PlatformTable(rows, errors, source)


def load(root: Optional[str] = None) -> PlatformTable:
    root = _root(root)
    text = open(os.path.join(root, TSV), encoding="utf-8").read()
    return parse(text, projects(root), TSV)


def launched_engines_stale(table: PlatformTable, launched: Iterable[str]) -> List[str]:
    """Rows whose `observable_on` engine a gate-run config launches: a witness could now see them."""
    launched = set(launched)
    out = []
    for r in table.rows:
        hit = sorted(launched & {x.strip() for x in r.observable_on.split(",")})
        if hit:
            out.append(f"{table.source}:{r.line}: ({r.effect}, {r.site}) observable_on {', '.join(hit)} is now launched "
                       "by a gate-run config: the row is stale, the witness decides")
    return out


def problems(table: PlatformTable, root: Optional[str] = None, site_exists: Optional[Callable[[str], bool]] = None,
             launched: Optional[Iterable[str]] = None) -> List[str]:
    out = list(table.errors)
    allow = Allowlist("platform-rules", [r[:6] + (r.row,) for r in table.rows], fields=COLUMNS[:6], source=table.source)
    out += [f"{table.source}: {p}" for p in allow.problems(stale=False)]
    seen = {}
    for r in table.rows:
        k = (r.effect, r.site)
        if k in seen and tuple(r[:6]) != tuple(table.rows[seen[k]][:6]):
            out.append(f"{table.source}:{r.line}: duplicate row for ({r.effect}, {r.site}) (first at line "
                       f"{table.rows[seen[k]].line})")
        seen.setdefault(k, table.rows.index(r))
        if site_exists is not None and not site_exists(r.site):
            out.append(f"{table.source}:{r.line}: site {r.site!r} no longer exists -- remove the row")
    if launched is not None:
        out += launched_engines_stale(table, launched)
    return out


def check_lines(table: PlatformTable) -> List[str]:
    return sorted({r.check_line for r in table.rows})


def find(table: PlatformTable, effect: str, site: str) -> Optional[Row]:
    for r in table.rows:
        if r.effect == effect and r.site == site:
            return r
    return None


def _self_test() -> int:
    hdr = "\t".join(COLUMNS) + "\n"
    good = "prevent\thotkey@toast F6\ta\thttps://example.test/f6\tnone\tdesktop-pass.md#f6\t444\n"
    pj = {"chromium", "firefox", "webkit"}
    cases = [
        ("green: valid row", hdr + good, True, "", {}),
        ("green: header only", hdr, True, "", {}),
        ("red: unknown category", hdr + good.replace("\ta\t", "\tz\t"), False, "unknown category", {}),
        ("red: effect outside category", hdr + good.replace("\ta\t", "\tc\t"), False, "not a member", {}),
        ("red: unknown project", hdr + good.replace("none", "chromium,safari"), False, "'safari'", {}),
        ("red: missing column", "\t".join(COLUMNS[:-1]) + "\n" + good, False, "missing column: row", {}),
        ("red: short row", hdr + "prevent\thotkey\ta\n", False, "cells", {}),
        ("red: blank cell", hdr + good.replace("444", " "), False, "blank cell", {}),
        ("red: duplicate", hdr + good + good, False, "duplicate", {}),
        ("red: duplicate effect+site", hdr + good + good.replace("none", "chromium"), False, "duplicate row", {}),
        ("red: site gone", hdr + good, False, "no longer exists", {"site_exists": lambda s: False}),
        ("red: engine launched", hdr + good.replace("none", "webkit"), False, "now launched", {"launched": ["webkit"]}),
    ]
    bad = 0
    for name, text, ok, needle, kw in cases:
        probs = problems(parse(text, pj, "fixture"), **kw)
        passed = (not probs) if ok else any(needle in p for p in probs)
        print(f"{'ok  ' if passed else 'FAIL'} {name}" + ("" if passed else f": {probs}"))
        bad += not passed
    t = load()
    live = problems(t)
    print(f"{'ok  ' if not live else 'FAIL'} tree TSV ({len(t.rows)} rows, projects {sorted(projects())})")
    bad += bool(live)
    return 1 if bad else 0


def main(argv) -> int:
    if "--self-test" in argv:
        return _self_test()
    t = load()
    probs = problems(t)
    for p in probs:
        print(p, file=sys.stderr)
    print(f"platform-rules: {len(t.rows)} row(s), {len(probs)} problem(s)")
    return 1 if probs else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
