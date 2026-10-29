#!/usr/bin/env bash
#
# check-anchored-placement.sh  (backlog row 477, rule P)
#
# An anchored panel is placed in document coordinates: a top-layer popover with `position: absolute`, moved by `transform`
# alone, so the page carries it when it scrolls and no `visualViewport` offset is ever added to a rect. The scripts that
# place a panel (`primitives/src/top_layer/{anchor_fallback,scroll_clamp}.rs`, the item-aligned Select in
# `primitives/src/select/components/list.rs`) must not write `top`, `left`, `bottom` or `right` other than the constant
# `0` / `auto` that anchors the transform, must not make a panel `position: fixed`, and must not read
# `visualViewport.offsetTop`/`offsetLeft` outside the shared helper `visible()` (the one place that turns the visible
# region into document coordinates, and the one place that makes a `position: fixed` probe, which is not a panel).
#
# THE CLASS: a panel written as `top = rect.top + vv.offsetTop` is correct in Chromium (rects are layout-viewport
# relative) and wrong in iOS WebKit (visual-viewport relative); with the keyboard up it drifted by the whole pan.
# Document coordinates minus the root element's own origin are right in both, and a source scan keeps the position
# writes from coming back. Comment lines are ignored; `playwright/anchored-placement.spec.ts` is the behavioural half.
#
# Exit status: 0 clean, 1 a violation, 2 a broken self-test.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

exec python3 -I - "$@" <<'PYEOF'
import re
import sys

FILES = [
    "primitives/src/top_layer/anchor_fallback.rs",
    "primitives/src/top_layer/scroll_clamp.rs",
    "primitives/src/select/components/list.rs",
]
# (pattern, message). Applied to code lines (a `//` comment line is skipped).
RULES = [
    (re.compile(r"""\.style\.(?:top|left|bottom|right)\s*=(?!=)"""), "writes an inset property directly"),
    (re.compile(r"""(?:put|setProperty)\(\s*['"](?:top|left|bottom|right)['"]\s*,(?!\s*['"](?:0|auto)['"])"""),
     "writes an inset property other than the constant 0 / auto"),
    (re.compile(r"""['"]position['"]\s*,\s*['"]fixed['"]|position\s*:\s*fixed(?!-)"""), "makes a panel position: fixed"),
    (re.compile(r"""\.style\.position\s*=\s*['"]fixed"""), "makes a panel position: fixed"),
]
VV_OFFSET = re.compile(r"""\boffset(?:Top|Left)\b""")


def scan(path, text):
    out = []
    in_helper = False
    for n, line in enumerate(text.splitlines(), 1):
        t = line.strip()
        if t.startswith("//") or t.startswith("///") or t.startswith("//!"):
            continue
        if "function visible()" in line:
            in_helper = True
        elif in_helper and line.startswith("}"):
            in_helper = False
        for rx, msg in RULES:
            if not in_helper and rx.search(line):
                out.append(f"{path}:{n}: {msg}: {t}")
        if VV_OFFSET.search(line) and "vv." in line and not in_helper:
            out.append(f"{path}:{n}: reads visualViewport.offset* outside visible(): {t}")
    return out


def self_test():
    ok = "  put('top', '0');\n  put('transform', 'translate(1px, 2px)');\n"
    bad = [
        "el.style.top = y + 'px';",
        "put('left', rnd(x) + 'px');",
        "content.style.position = 'fixed';",
        "put('position', 'fixed');",
        "const y = a.t + vv.offsetTop;",
    ]
    if scan("t.rs", ok):
        return "a clean script was flagged"
    for b in bad:
        if not scan("t.rs", b):
            return f"not flagged: {b}"
    if scan("t.rs", "// el.style.top = 1;"):
        return "a comment was flagged"
    return None


def main():
    failure = self_test()
    if failure:
        print(f"check-anchored-placement: self-test failed: {failure}", file=sys.stderr)
        return 2
    problems = []
    for path in FILES:
        with open(path, encoding="utf-8") as f:
            problems += scan(path, f.read())
    if problems:
        print("check-anchored-placement: FAIL -- anchored panels are placed by `transform` in document coordinates (row 477):")
        for p in problems:
            print("  " + p)
        return 1
    print(f"check-anchored-placement: OK -- {len(FILES)} placement files write no inset or fixed position")
    return 0


sys.exit(main())
PYEOF
