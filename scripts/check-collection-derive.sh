#!/usr/bin/env bash
#
# check-collection-derive.sh
#
# Every view of rows derives from one model, `primitives/src/data_view/`: in `preview/src/components/**/*.rs` there is
# no `.to_lowercase().contains(` (call `contains_query`), no `HashSet<usize>`, `Vec<bool>` or `[bool; N]` as a
# selection (selection is a `HashSet<String>` of row keys in `ViewState`) and no ad-hoc `.sort_by(` / `.sort_by_key(`
# (a `Field::sortable` through `derive`). Reasoned allowlist (ToggleGroup's deprecated index props, the catalogue
# sort); a stale entry fails. Backlog row 360.
#
#   In `preview/src/components/**/*.rs` (comments stripped) none of these may appear:
#     - `.to_lowercase().contains(`          -> call `dioxus_primitives::data_view::contains_query`
#     - `HashSet<usize>` / `Vec<bool>` / `[bool; N]` as a selection -> a `HashSet<String>` of row keys
#     - `.sort_by(`, `.sort_by_key(`, `.sort_unstable_by*(` -> a `Field::sortable` + `derive`
#   The stages themselves live in `primitives/src/data_view/`, which is out of scope: the data_table folder
#   only re-exports them.
#
# EXCEPTIONS are in ALLOW below, each with a reason; an entry that no longer matches anything is itself an
# error (a fixed site must leave the list). ToggleGroup's pressed set is a `HashSet<String>` of item values
# since row 364; only its deprecated index props stay allowlisted.
#
# Usage: scripts/check-collection-derive.sh [FILE ...]   (default: every .rs under preview/src/components;
# explicit files are for self-tests, and skip the stale-entry check)
# Exit status: 0 when clean, 1 on a violation, 2 when nothing could be scanned.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import glob
import re
import sys

explicit = sys.argv[1:]
files = explicit or sorted(glob.glob("preview/src/components/**/*.rs", recursive=True))
if not files:
    print("check-collection-derive: no sources found (run from a checkout)", file=sys.stderr)
    sys.exit(2)

# (file suffix, rule id, reason). A rule id is one of the RULES keys.
TOGGLE = "the deprecated index API of ToggleGroup (`pressed` / `default_pressed` / `on_pressed_change`), kept so old callers compile; the value API is `HashSet<String>`"
ALLOW = [
    ("toggle_group/component.rs", "set-usize", TOGGLE),
    ("components/mod.rs", "sort-by", "orders the site's component catalogue (demo names), not rows of a collection"),
]
used = set()

RULES = {
    "lowercase-contains": (
        re.compile(r"\.to_lowercase\(\)\s*\.contains\("),
        "use dioxus_primitives::data_view::contains_query (one matcher)",
    ),
    "set-usize": (
        re.compile(r"HashSet<usize>"),
        "selection is a HashSet<String> of row keys (ViewState.selected), never positions",
    ),
    "bool-array": (
        re.compile(r"\[bool;\s*[\w:]+\]|Vec<bool>"),
        "selection is a HashSet<String> of row keys (ViewState.selected), never a flag per position",
    ),
    "sort-by": (
        re.compile(r"\.sort_(?:unstable_)?by(?:_key|_cached_key)?\s*\("),
        "sort through a data_view Field::sortable and derive, not an ad-hoc comparator",
    ),
}

BLOCK_COMMENT = re.compile(r"/\*.*?\*/", re.S)

def blank(text):
    text = BLOCK_COMMENT.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)
    # whole-line `//` comments only (a `//` inside a string, such as a URL, is code)
    return re.sub(r"(?m)^(\s*)//.*$", lambda m: m.group(1), text)

def allowed(path, rule):
    for entry in ALLOW:
        suffix, rid, _reason = entry
        if rid == rule and path.endswith(suffix):
            used.add(id(entry))
            return True
    return False

bad = []
for path in files:
    try:
        text = blank(open(path, encoding="utf-8").read())
    except OSError as err:
        print(f"check-collection-derive: cannot read {path}: {err}", file=sys.stderr)
        sys.exit(2)
    for rule, (rx, hint) in RULES.items():
        for m in rx.finditer(text):
            if allowed(path, rule):
                continue
            line = text.count("\n", 0, m.start()) + 1
            bad.append(f"{path}:{line}: `{m.group(0).strip()}` -- {hint}")

if not explicit:
    for entry in ALLOW:
        if id(entry) not in used:
            bad.append(f"stale allowlist entry ({entry[0]}, {entry[1]}): no longer matches anything -- remove it")

if bad:
    print("check-collection-derive: FAIL")
    for b in bad:
        print("  " + b)
    sys.exit(1)
print(f"check-collection-derive: OK ({len(files)} files)")
PYEOF
