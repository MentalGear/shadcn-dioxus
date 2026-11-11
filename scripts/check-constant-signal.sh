#!/usr/bin/env bash
#
# check-constant-signal.sh
#
# No `ReadSignal::new(Signal::new(..))` in render code: a render body that writes it allocates a signal on every render,
# and as a prop it compares unequal each time, so the child re-renders whenever the parent does (`Slider`'s `min_steps`
# defeated `SliderImpl`'s memoisation). A constant `ReadSignal` is `use_constant_signal(value)`
# (`primitives/src/constant_signal.rs`), which builds one per component instance. The raw form stays legal where it is
# evaluated once: a `#[props(default = ..)]` attribute, a function that returns a default (`-> ReadSignal<..>` on the
# line above), the body of a `use_hook(..)` closure (it opens on the line or one of the three above), the helper
# module itself, and test code (after the first `#[cfg(test)]`). A site with another reason carries
# `constant-signal-ok: <reason>` on the line or the one above; a marker with nothing to excuse is stale and fails.
#
# Scope: primitives/src and preview/src (*.rs). Usage: scripts/check-constant-signal.sh [FILE ...] (explicit files are
# scanned as given, for proving the gate red). Exit 0: clean. Exit 1: file:line on stdout.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 -I - "$@" <<'PYEOF'
import glob
import re
import sys

HELPER = "primitives/src/constant_signal.rs"
RAW = re.compile(r"ReadSignal::new\(\s*Signal::new\(")
PROPS = re.compile(r"^\s*#\[props\(")
MARKER = re.compile(r"constant-signal-ok:\s*(\S.*)")

files = [a for a in sys.argv[1:] if a] or sorted(
    glob.glob("primitives/src/**/*.rs", recursive=True) + glob.glob("preview/src/**/*.rs", recursive=True))
errors = []
sites = 0
for path in files:
    if path == HELPER:
        continue
    with open(path, encoding="utf-8") as fh:
        lines = fh.read().split("\n")
    # test code starts at the first inline `#[cfg(..test..)] mod x {` (a `#[cfg(test)] use` or `mod x;` is not it)
    end = next((i for i, l in enumerate(lines)
                if re.match(r"\s*#\[cfg\((all\()?test", l) and i + 1 < len(lines)
                and re.match(r"\s*(pub(\(crate\))?\s+)?mod\s+\w+\s*\{", lines[i + 1])), len(lines))
    used = set()
    for i in range(end):
        line = lines[i]
        if re.match(r"\s*//", line) or not RAW.search(line):
            continue
        sites += 1
        above = lines[max(0, i - 3):i]
        if PROPS.match(line):
            continue
        if any(re.search(r"->\s*ReadSignal<", l) for l in lines[max(0, i - 2):i + 1]):
            continue
        if any("use_hook(" in l for l in above + [line]):
            continue
        marked = [j for j in (i, i - 1) if j >= 0 and MARKER.search(lines[j])]
        if marked:
            used.add(marked[0])
            continue
        errors.append(f"{path}:{i + 1}: `ReadSignal::new(Signal::new(..))` outside a props default, a default fn or a "
                      f"`use_hook` closure builds a signal per render; use `use_constant_signal(..)` "
                      f"(primitives/src/constant_signal.rs)")
    for i in range(end):
        if MARKER.search(lines[i]) and i not in used:
            errors.append(f"{path}:{i + 1}: stale `constant-signal-ok` marker: no raw constant signal on this line or the next")

if errors:
    for e in errors:
        print(e)
    print(f"\ncheck-constant-signal: {len(errors)} problem(s). See the header of scripts/check-constant-signal.sh.", file=sys.stderr)
    sys.exit(1)
print(f"check-constant-signal: OK ({sites} raw constant-signal site(s), all evaluated once)")
PYEOF
