#!/usr/bin/env bash
#
# check-press-under.sh  (backlog row 295)
#
# A pressed `dx-` control gets the theme's press wash (`dx-components-theme.css`, "Press feedback"), a
# `background-image` layer that out-ranks every component rule so the wash paints on a control whose sheet
# writes a `background` shorthand. That rule REPLACES the element's own `background-image`: a pressable card
# with a gradient (Navigation Menu's featured card) lost it while pressed. The wash therefore ends with
# `var(--dx-press-under, none)`, and a control that paints an image or gradient hands it over by declaring it once
# in that slot:  `--dx-press-under: <image>; background-image: var(--dx-press-under);`.
#
# By construction, then: in a component sheet (preview/src/components/*/style.css), a rule that declares an image
# or gradient in `background-image`, or in a `background` shorthand containing `gradient(` or `url(`, must
# declare `--dx-press-under` as well. An element that can never be pressed (not a button, link, summary or
# interactive-role element) goes on ALLOW below with its reason; an entry that matches nothing is reported, not
# failed, so it can be deleted. Comments are stripped.
#
# Usage: scripts/check-press-under.sh [FILE ...]   (explicit files are for self-tests; no stale report)
# Exit status: 0 when clean.

set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import glob, re, sys

explicit = sys.argv[1:]
files = explicit or sorted(glob.glob("preview/src/components/*/style.css"))
if not files:
    print("check-press-under: no stylesheets found", file=sys.stderr)
    sys.exit(2)

# (file, selector substring, reason)
ALLOW = [
    ("preview/src/components/color_picker/style.css", ".dx-color-picker-swatch",
     "a `div role=img` showing the current colour over a checkerboard: no press handler, not in the press list"),
    ("preview/src/components/color_picker/style.css", ".dx-color-picker-slider-track",
     "a `div` drag surface (the thumb is the control, and sliders are outside the press list)"),
    ("preview/src/components/color_picker/style.css", ".dx-color-picker-area-track",
     "a `div` drag surface (the thumb is the control, and sliders are outside the press list)"),
    ("preview/src/components/progress/style.css", ".dx-progress-indicator",
     "a `div` inside a `role=progressbar`: output only, never pressed"),
]

def strip_comments(t):
    return re.sub(r"/\*.*?\*/", "", t, flags=re.S)

def rules(text):
    """Yield (selector, body) for every innermost block, nested at-rules flattened."""
    out, stack, start = [], [], 0
    for i, ch in enumerate(text):
        if ch == "{":
            stack.append((text[start:i].strip(), i + 1))
            start = i + 1
        elif ch == "}":
            if stack:
                sel, b = stack.pop()
                body = text[b:i]
                if "{" not in body:
                    out.append((sel, body))
            start = i + 1
    return out

img = re.compile(r"(?:^|;)\s*(background-image|background)\s*:\s*([^;]*)", re.I)
bad, used = [], set()
for f in files:
    text = strip_comments(open(f, encoding="utf-8").read())
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
        allowed = False
        for k, (af, asel, _why) in enumerate(ALLOW):
            if (not explicit and f == af and asel in sel) or (explicit and asel in sel):
                used.add(k)
                allowed = True
        if not allowed:
            bad.append((f, " ".join(sel.split())))

if bad:
    print("check-press-under: FAIL -- a component rule paints an image or gradient without the press slot:", file=sys.stderr)
    for f, sel in bad:
        print(f"  {f}: {sel}", file=sys.stderr)
    print("Declare it once as `--dx-press-under: <image>; background-image: var(--dx-press-under);` so the theme's press\n"
          "wash layers over it, or (an element that is never pressed) add it to ALLOW in this script with a reason.", file=sys.stderr)
    sys.exit(1)
if not explicit:
    for k, (af, asel, why) in enumerate(ALLOW):
        if k not in used:
            print(f"check-press-under: stale ALLOW entry (matches nothing): {af} {asel}")
print(f"check-press-under: OK -- {len(files)} stylesheet(s)")
PYEOF
