#!/usr/bin/env bash
#
# check-unawaited-focus.sh
#
# Every `.set_focus(true)` / `.set_focus(false)` on a mounted element is awaited. `MountedData::set_focus` returns a
# boxed future; dropping it unpolled runs on dioxus-web only because that renderer focuses before it returns the
# future, while dioxus-desktop sends its query when the future is created and a custom renderer may defer all of it to
# the first poll. `_ = el.set_focus(true).await;` costs nothing where the focus is eager and is the only form that is
# right everywhere. The collection's own `set_focus(Some(index))` and `set_focus(None)` are synchronous signal writes
# and are not matched (the argument must be exactly `true` or `false`).
#
# Comment lines are skipped; a call inside a test module (after the first `#[cfg(test)]`) is a fixture. Scope:
# primitives/src and preview/src (*.rs). Usage: scripts/check-unawaited-focus.sh [FILE ...] (explicit files are scanned
# as given, for proving the gate red). Exit 0: clean. Exit 1: file:line on stdout.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 -I - "$@" <<'PYEOF'
import glob
import re
import sys

CALL = re.compile(r"\.set_focus\(\s*(?:true|false)\s*\)(?!\s*\.await)")

files = [a for a in sys.argv[1:] if a] or sorted(
    glob.glob("primitives/src/**/*.rs", recursive=True) + glob.glob("preview/src/**/*.rs", recursive=True))
errors = []
awaited = 0
for path in files:
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
        if CALL.search(line):
            errors.append(f"{path}:{i + 1}: `set_focus(..)` is not awaited; write `_ = el.set_focus(true).await;`")
        elif re.search(r"\.set_focus\(\s*(?:true|false)\s*\)\s*\.await", line):
            awaited += 1

if errors:
    for e in errors:
        print(e)
    print(f"\ncheck-unawaited-focus: {len(errors)} problem(s). See the header of scripts/check-unawaited-focus.sh.", file=sys.stderr)
    sys.exit(1)
print(f"check-unawaited-focus: OK ({awaited} awaited set_focus call(s))")
PYEOF
