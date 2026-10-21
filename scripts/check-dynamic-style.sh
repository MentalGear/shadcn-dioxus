#!/usr/bin/env bash
#
# check-dynamic-style.sh
#
# A `style:` attribute in rsx must never be an `if`/`match` expression choosing between whole style strings.
#
# THE DEFECT (found 2026-10-20): the questionnaire `progress` demo painted its checkpoint segments with
# `style: if index < current { "...; background: var(--dx-primary);" } else { "...; background: var(--dx-muted);" }`.
# Dioxus web patches a changed `style` of that shape property by property and lost properties: after two steps
# the segments that had changed carried only `height` and `flex` (no background, no radius), so the bar showed one
# bright segment, two invisible ones and one dark one. The collapsible `file_tree` and bubble `collapsible` chevrons
# (`style: if open() { "rotate: ..." } else { "" }`) stayed rotated after closing for the same reason. Three
# instances were a class, so it is no longer expressible: state-dependent paint goes through a class or a
# `data-state` attribute with a rule in the component's `style.css`, or the two states are two `if`/`else`
# branches of static elements (a static `style` literal is never diffed).
#
# Usage: scripts/check-dynamic-style.sh [FILE ...]   (default: every .rs under preview/src and primitives/src)
# Exit status: 0 if no file has such an attribute.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import glob
import re
import sys

files = sys.argv[1:] or sorted(glob.glob("preview/src/**/*.rs", recursive=True) + glob.glob("primitives/src/**/*.rs", recursive=True))
if not files:
    print("check-dynamic-style: no sources found (run from a checkout)", file=sys.stderr)
    sys.exit(2)

# `style: if ...`, `style: match ...`, `style: { if ...` -- but not `font_style:`, `list_style:` etc.
PAT = re.compile(r"(?<![\w-])style\s*:\s*(?:\{\s*)?(?:if|match)\b")
# (path, text the line contains) -> why it is allowed. Each is the same shape as the defect but has not been seen to
# misrender (its value never drops a property the previous one had, or no spec watches the change); convert them
# the next time their file is touched. Stale entries fail.
ALLOW = {
    ("preview/src/main.rs", "style: match member.2 {"): "docs-site member badge colour, one property in every arm",
    ("primitives/src/chart/components/series/radar.rs", "style: if ring_fill {"): "set once at render, a ring fill never toggles live",
    ("primitives/src/combobox/components/input.rs", "style: if ctx.is_multi()"): "is_multi is fixed for a combobox's lifetime",
}
used = set()
errors = []
for path in files:
    with open(path, encoding="utf-8") as fh:
        for n, line in enumerate(fh, 1):
            code = line.split("//", 1)[0]
            if PAT.search(code):
                key = next((k for k in ALLOW if k[0] == path.removeprefix("./") and k[1] in code), None)
                if key:
                    used.add(key)
                else:
                    errors.append((path, n, line.strip()))
if not sys.argv[1:]:
    for key in ALLOW:
        if key not in used:
            errors.append((key[0], 0, f"stale ALLOW entry {key[1]!r}"))

if errors:
    for path, n, text in errors:
        print(f"{path}:{n}: {text}\n    a `style:` chosen by if/match is patched property by property and loses properties")
    print(f"\ncheck-dynamic-style: {len(errors)} dynamic style attribute(s). Use a class or `data-state` + a rule in "
          f"style.css, or two static if/else branches. See the header of scripts/check-dynamic-style.sh.", file=sys.stderr)
    sys.exit(1)

print(f"check-dynamic-style: OK ({len(files)} source file(s), no if/match style attribute)")
PYEOF
