#!/usr/bin/env bash
#
# check-hover-mix-space.sh  (backlog rows 243, 260)
#
# shadcn base-nova writes its hover step as `color-mix(in oklch, var(--secondary), var(--foreground) 5%)`.
# The theme carries that space once as `--dx-hover-mix-space` (dx-components-theme.css), and every hover /
# pressed / active fill that steps a colour toward the ink or the page reads it:
# `color-mix(in var(--dx-hover-mix-space), <fill>, var(--dx-foreground) N%)`. Hand-writing the space
# (`in oklab` is 3 levels off on a grey) is how components drift from Button (rows 243, 260).
#
# This gate fails on a LITERAL interpolation space (`color-mix(in oklab|oklch|srgb|...`) in a component
# sheet (preview/src/components/*/style.css) or a preview/assets/*.css sheet when the mix is a STEP, i.e.
# no argument is `transparent`. A `color-mix(<c> N%, transparent)` is an alpha tint (shadcn's `/50`
# shorthand), not a step, and is out of scope. Comments are stripped. Genuine non-hover steps are in ALLOW
# below, each with its reason; an entry that matches nothing is reported (not failed) so it can be deleted.
#
# Usage: scripts/check-hover-mix-space.sh [FILE ...]   (explicit files are for self-tests; no stale report)
# Exit status: 0 when clean.

set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import glob, re, sys

sys.path.insert(0, "scripts/lib")
from allow import Allowlist  # noqa: E402
from css import blank_comments  # noqa: E402

explicit = sys.argv[1:]
files = explicit or sorted(glob.glob("preview/src/components/*/style.css") + glob.glob("preview/assets/*.css"))
if not files:
    print("check-hover-mix-space: no stylesheets found", file=sys.stderr)
    sys.exit(2)

# (file, substring of the mix body, reason)
ALLOW = Allowlist("check-hover-mix-space", [
    ("preview/src/components/button/style.css", "oklch(from var(--dx-primary-foreground)",
     "accent-preset primary hover steps toward the computed black/white contrast colour, not toward --dx-foreground; "
     "shadcn's primary hover is `/80` alpha and the preset-only recipe is guarded by theme-preset-contrast.spec.ts"),
    ("preview/assets/dx-effects.css", "--dx-internal-shimmer-highlight",
     "text shimmer gradient stop between two shimmer colours; not a hover or state fill"),
    ("preview/assets/theme-presets.css", "var(--foreground) 26%",
     "preset border emphasis toward the preset's own background; a preset-scoped token, not a hover fill"),
], fields=("file", "substring"))
violations = []
for path in files:
    src = open(path, encoding="utf-8").read()
    src = blank_comments(src)
    for m in re.finditer(r"color-mix\(", src):
        i, depth = m.end(), 1
        while i < len(src) and depth:
            depth += {"(": 1, ")": -1}.get(src[i], 0)
            i += 1
        body = src[m.end():i - 1]
        if not re.match(r"\s*in\s+(?!var\()[a-z0-9-]+", body):
            continue
        if re.search(r"\btransparent\b", body):
            continue
        line = src.count("\n", 0, m.start()) + 1
        if not explicit and ALLOW.find(lambda e: e.file == path and e.substring in body):
            continue
        violations.append((path, line, " ".join(body.split())))

if violations:
    print("check-hover-mix-space: FAIL -- a literal interpolation space in a colour STEP; write "
          "`color-mix(in var(--dx-hover-mix-space), ...)` (row 260) or add an ALLOW entry with a reason:")
    for p, l, b in violations:
        print(f"  {p}:{l}: color-mix({b[:110]})")
    sys.exit(1)
problems = ALLOW.problems(stale=not explicit)
if problems:
    for p in problems:
        print(f"check-hover-mix-space: {p}")
    sys.exit(1)
print(f"check-hover-mix-space: PASS ({len(files)} stylesheets)")
PYEOF
