#!/usr/bin/env bash
#
# check-press-under.sh  (backlog row 295)
#
# A pressed `dx-` control gets the theme's press wash (`dx-components-theme.css`, "Press feedback"), a
# `background-image` layer that out-ranks every component rule so the wash paints on a control whose sheet
# writes a `background` shorthand. That rule REPLACES the element's own `background-image`: a pressable card
# with a gradient (Navigation Menu's featured card) loses it while pressed. The wash therefore ends with
# `var(--dx-press-under, none)`, and a control that paints an image or gradient hands it over by declaring it once
# in that slot:  `--dx-press-under: <image>; background-image: var(--dx-press-under);`.
#
# By construction, then: in a component sheet (preview/src/components/*/style.css), a rule that declares an image
# or gradient in `background-image`, or in a `background` shorthand containing `gradient(` or `url(`, must
# declare `--dx-press-under` as well. An element that can never be pressed (not a button, link, summary or
# interactive-role element) goes on ALLOW below with its reason; an entry that matches nothing fails as stale
# (scripts/lib/allow.py), so the list cannot rot. The stylesheet is read by scripts/lib/css.py (comments kept as
# newlines, strings, `url(...)` and nested at-rules understood), the one reader every CSS gate shares.
#
# Usage: scripts/check-press-under.sh [FILE ...]   (explicit files are for self-tests; no stale report)
# Exit status: 0 when clean.

set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import glob, re, sys

sys.path.insert(0, "scripts/lib")
from allow import Allowlist  # noqa: E402
from css import rules as css_rules  # noqa: E402

explicit = sys.argv[1:]
files = explicit or sorted(glob.glob("preview/src/components/*/style.css"))
if not files:
    print("check-press-under: no stylesheets found", file=sys.stderr)
    sys.exit(2)

# (file, selector substring, reason)
ALLOW = Allowlist("check-press-under", [
    ("preview/src/components/color_picker/style.css", ".dx-color-picker-swatch",
     "a `div role=img` showing the current colour over a checkerboard: no press handler, not in the press list"),
    ("preview/src/components/color_picker/style.css", ".dx-color-picker-slider-track",
     "a `div` drag surface (the thumb is the control, and sliders are outside the press list)"),
    ("preview/src/components/color_picker/style.css", ".dx-color-picker-area-track",
     "a `div` drag surface (the thumb is the control, and sliders are outside the press list)"),
    ("preview/src/components/progress/style.css", ".dx-progress-indicator",
     "a `div` inside a `role=progressbar`: output only, never pressed"),
], fields=("file", "selector"))

def rules(text):
    """(selector, body) for every innermost style rule, nested at-rules flattened (the shared reader)."""
    return [(r.prelude, r.body) for r in css_rules(text) if not r.nested]

img = re.compile(r"(?:^|;)\s*(background-image|background)\s*:\s*([^;]*)", re.I)
bad = []
for f in files:
    text = open(f, encoding="utf-8").read()
    for sel, body in rules(text):
        has_image = False
        for m in img.finditer(body):
            prop, val = m.group(1).lower(), m.group(2)
            if prop == "background-image" and val.strip() not in ("none", "initial", "unset", "inherit") or \
               prop == "background" and re.search(r"gradient\(|url\(", val):
                if "var(--dx-press-under" in val:
                    continue
                has_image = True
        if not has_image or "--dx-press-under" in body:
            continue
        allowed = ALLOW.find_all(lambda e: (not explicit and f == e.file and e.selector in sel)
                                 or (explicit and e.selector in sel))
        if not allowed:
            bad.append((f, " ".join(sel.split())))

if bad:
    print("check-press-under: FAIL -- a component rule paints an image or gradient without the press slot:", file=sys.stderr)
    for f, sel in bad:
        print(f"  {f}: {sel}", file=sys.stderr)
    print("Declare it once as `--dx-press-under: <image>; background-image: var(--dx-press-under);` so the theme's press\n"
          "wash layers over it, or (an element that is never pressed) add it to ALLOW in this script with a reason.", file=sys.stderr)
    sys.exit(1)
problems = ALLOW.problems(stale=not explicit)
if problems:
    for p in problems:
        print(f"check-press-under: {p}", file=sys.stderr)
    sys.exit(1)
print(f"check-press-under: OK -- {len(files)} stylesheet(s)")
PYEOF
