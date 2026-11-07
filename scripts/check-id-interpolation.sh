#!/usr/bin/env bash
#
# check-id-interpolation.sh
#
# A caller-supplied id never reaches CSS or a page script by hand-formatting, and is never read from `attributes` a
# second way. An id is arbitrary text (`a.b`, `x:y`, `it's`): in `anchor-name: --dxa-{id}` the dot ends the identifier,
# in `getElementById('{id}')` the apostrophe ends the string, in a `[style*="..."]` selector a quote ends the value. Each
# context has one helper in `primitives/src/escape.rs` (`css_ident`, `css_string`, `anchor_selector`, `js_string`) and
# one reader of the caller's id in `primitives/src/caller_id.rs` (an empty id is no id). In `primitives/src` (*.rs, test
# modules after the first `#[cfg(test)]` and comments excluded; the two helper files are exempt) this fails on
#
#   1. `getElementById('{..}')` or `getElementById("{..}")`: a quoted placeholder; bind `escape::js_string(&id)` and
#      write `getElementById({id})`;
#   2. `--dxa-{` on a line without `css_ident(`, and `--dxa-' +` / `--dxa-" +` (a script that concatenates an id into
#      the selector): the Rust side formats `css_ident(id)`, a script receives `escape::anchor_selector(id)` as data;
#   3. a `[style*=` selector written by hand;
#   4. `.name == "id"`: read the caller's id with `caller_id::caller_id`.
#
# Usage: scripts/check-id-interpolation.sh [FILE=repo/name ...] (explicit files are scanned as given, for proving the
# gate red). Exit 0: clean. Exit 1: file:line on stdout.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 -I - "$@" <<'PYEOF'
import glob
import re
import sys

EXEMPT = {"primitives/src/escape.rs", "primitives/src/caller_id.rs"}
RULES = [
    (re.compile(r"""getElementById\(\s*['"]\{"""),
     "a quoted `{..}` placeholder in getElementById; bind `escape::js_string(&id)` and write `getElementById({id})`"),
    (re.compile(r"""--dxa-['"]\s*\+"""),
     "a script concatenates an id into the anchor selector; pass `escape::anchor_selector(id)` as data"),
    (re.compile(r"""\[style\*="""),
     "a hand-written `[style*=` selector; use `escape::anchor_selector(id)`"),
    (re.compile(r"""\.name\s*==\s*"id\""""),
     "a second reader of the caller's id; use `caller_id::caller_id(attributes)`"),
]
DXA = re.compile(r"--dxa-\{")

explicit = [a for a in sys.argv[1:] if a]
files = {}
if explicit:
    for spec in explicit:
        path, _, name = spec.partition("=")
        files[name or path] = path
else:
    for path in sorted(glob.glob("primitives/src/**/*.rs", recursive=True)):
        files[path] = path

errors = []
for name, path in files.items():
    if name in EXEMPT or re.search(r"(^|/)(tests?|[a-z_]+_tests)\.rs$", name):
        continue
    with open(path, encoding="utf-8") as fh:
        lines = fh.read().split("\n")
    # test code starts at the first inline `#[cfg(..test..)] mod x {` (a `#[cfg(test)] use` or `mod x;` is not it)
    end = next((i for i, l in enumerate(lines)
                if re.match(r"\s*#\[cfg\((all\()?test", l) and i + 1 < len(lines)
                and re.match(r"\s*(pub(\(crate\))?\s+)?mod\s+\w+\s*\{", lines[i + 1])), len(lines))
    for i in range(end):
        line = lines[i]
        if re.match(r"\s*//", line):
            continue
        for rx, why in RULES:
            if rx.search(line):
                errors.append(f"{name}:{i + 1}: {why}")
        if DXA.search(line) and "css_ident(" not in line:
            errors.append(f"{name}:{i + 1}: `--dxa-{{` formats an id by hand; format `escape::css_ident(id)`")

if errors:
    for e in errors:
        print(e)
    print(f"\ncheck-id-interpolation: {len(errors)} problem(s). See the header of scripts/check-id-interpolation.sh.", file=sys.stderr)
    sys.exit(1)
print(f"check-id-interpolation: OK ({len(files)} source(s) scanned)")
PYEOF
