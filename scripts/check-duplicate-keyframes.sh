#!/usr/bin/env bash
#
# check-duplicate-keyframes.sh
#
# Two `@keyframes` blocks with a byte-identical body (comments and whitespace stripped, `0%` read as `from`,
# `100%` as `to`) are one animation written twice. Fails when a stylesheet defines a body that another
# stylesheet (or the same one, under another name) already defines, unless the duplicate is allowlisted below
# with a reason. Invisible to cargo and clippy, and with CI frozen (dev-docs/backlog.md row 33) nothing else
# would notice.
#
# THE CLASS (row 145, lane F): at the 2026-10-13 recount the preview had 72 `@keyframes`, 52 of them members
# of 21 byte-identical clusters. A menu's pop-in was pasted into five sheets under five names (the select and
# combobox copies even shared one name, with a comment asking the next editor to "keep the two in step"), the
# pure opacity fade into eight, the colour picker's 4px drop into two, and one stylesheet carried two
# directional slides that differed only in which one a `[dir="rtl"]` rule named. Each copy was right on the day
# it was pasted and the set drifts from then on (a duration or an offset tuned in one place). The construction
# is one shared set in the theme -- `dx-fade-in/out`, `dx-pop-in/out`, `dx-drop-in/out`, selected through the
# `--dx-floating-enter/-exit` slots of the floating-surface recipe in `dx-components-theme.css` -- and this
# gate keeps a new component from pasting its own copy: use the shared name, or, for motion that really is
# new, write a body that differs.
#
# SCOPE: every `*.css` under `preview/src` and `preview/assets` (node_modules and build output skipped). The
# parse also fails if it sees more `@keyframes` tokens than it parsed, so an exotic spelling cannot slip past.
#
# ALLOW: a cluster is allowed when at most ONE of its members is NOT covered by an entry (the canonical
# definition stays; the covered ones are the debt). Entries are `path-glob::name-glob`, each with a reason, and
# an entry that covers nothing fails the run as stale, so the list shrinks and cannot become blanket
# permission. A pattern is matched against `<stylesheet path>::<keyframes name>`.
#
# Usage: scripts/check-duplicate-keyframes.sh [FILE ...]   (default: every stylesheet; explicit files are for
# self-tests, and skip the stale-entry check)
# Exit 0: clean. Exit 1: a duplicate, with file:line detail on stderr.

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

python3 - "$@" <<'PYEOF'
import collections
import fnmatch
import re
import sys
from pathlib import Path

# (path-glob::name-glob, reason). Each covers the DUPLICATE side of a cluster; the canonical one stays out.
ALLOW = [
    ("preview/src/components/alert_dialog/style.css::dx-alert-animate-*",
     "pure opacity fade, identical to the theme's dx-fade-in/out; owned by the modal-panel lane (row 145 / audit row 2), which moves it to the shared pair"),
    ("preview/src/components/command/style.css::dx-command-dialog-backdrop-*",
     "pure opacity fade, identical to the theme's dx-fade-in/out; modal-panel lane"),
    ("preview/src/components/dialog/style.css::dx-dialog-backdrop-animate-*",
     "pure opacity fade, identical to the theme's dx-fade-in/out; modal-panel lane"),
    ("preview/src/components/drawer/style.css::dx-drawer-root-*",
     "pure opacity fade, identical to the theme's dx-fade-in/out; modal-panel lane"),
    ("preview/src/components/sheet/style.css::dx-sheet-root-*",
     "pure opacity fade, identical to the theme's dx-fade-in/out; modal-panel lane"),
    ("preview/src/components/drawer/style.css::dx-drawer-slide-*",
     "edge slides identical to the sheet's dx-slide-*; modal-panel lane moves both to one shared set"),
    ("preview/src/components/toast/style.css::dx-slide-up-*",
     "two names on purpose: the toast swaps `-even`/`-odd` to restart the same animation on a re-triggered toast"),
]

explicit = sys.argv[1:]
if explicit:
    files = [Path(f) for f in explicit]
else:
    files = []
    for root in ("preview/src", "preview/assets"):
        for p in sorted(Path(root).rglob("*.css")):
            if "node_modules" in p.parts or "target" in p.parts:
                continue
            files.append(p)
if not files:
    print("check-duplicate-keyframes: no stylesheets found (run from a checkout)", file=sys.stderr)
    sys.exit(2)

COMMENT = re.compile(r"/\*.*?\*/", re.S)
START = re.compile(r"@(?:-webkit-)?keyframes\s+([\w-]+)\s*\{")

def blank_comments(text):
    return COMMENT.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)

def normalise(body):
    b = re.sub(r"\s+", "", body)
    b = re.sub(r"(^|[{};,])0%", r"\1from", b)
    b = re.sub(r"(^|[{};,])100%", r"\1to", b)
    return b

clusters = collections.defaultdict(list)  # body -> [(path, line, name)]
parsed = 0
errors = []
for path in files:
    raw = path.read_text(encoding="utf-8")
    text = blank_comments(raw)
    tokens = len(re.findall(r"@(?:-webkit-)?keyframes\b", text))
    n = 0
    for m in START.finditer(text):
        i, depth = m.end(), 1
        while depth and i < len(text):
            depth += (text[i] == "{") - (text[i] == "}")
            i += 1
        body = normalise(text[m.end():i - 1])
        clusters[body].append((str(path), text.count("\n", 0, m.start()) + 1, m.group(1)))
        n += 1
    if n != tokens:
        errors.append(f"{path}: parsed {n} @keyframes but the file has {tokens}; the parse needs to learn this spelling")
    parsed += n

used = set()

def covered(member):
    path, _, name = member
    key = f"{path}::{name}"
    for idx, (pat, _) in enumerate(ALLOW):
        if fnmatch.fnmatch(key, pat):
            used.add(idx)
            return True
    return False

dups = 0
for body, members in clusters.items():
    if len(members) < 2:
        continue
    uncovered = [m for m in members if not covered(m)]
    if len(uncovered) > 1:
        dups += 1
        where = ", ".join(f"{p}:{l} {n}" for p, l, n in uncovered)
        errors.append(f"byte-identical @keyframes in {len(uncovered)} places: {where} -- use one shared "
                      f"definition (the theme's dx-fade-*, dx-pop-*, dx-drop-* for floating surfaces), "
                      f"or make the new body differ")

if not explicit:
    for idx, (pat, reason) in enumerate(ALLOW):
        if idx not in used:
            errors.append(f"stale allowlist entry `{pat}` ({reason}): it no longer covers a duplicate; delete it")

if errors:
    for e in errors:
        print(e, file=sys.stderr)
    print(f"\ncheck-duplicate-keyframes: {len(errors)} problem(s). See the header of scripts/check-duplicate-keyframes.sh.",
          file=sys.stderr)
    sys.exit(1)

print(f"check-duplicate-keyframes: OK ({parsed} @keyframes in {len(files)} stylesheet(s), "
      f"{sum(1 for m in clusters.values() if len(m) > 1)} duplicate cluster(s), all allowlisted)")
PYEOF
