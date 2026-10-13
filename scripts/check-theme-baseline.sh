#!/usr/bin/env bash
#
# check-theme-baseline.sh
#
# The theme baseline (docs/backlog.md row 145, lane T) moved two things that every component
# stylesheet used to paste into ONE zero-specificity rule in `preview/assets/dx-components-theme.css`,
# keyed on the `dx-` class prefix (the row-111 border-box precedent). This gate makes the copies
# inexpressible again, so the class cannot grow back one stylesheet at a time:
#
#  1. `touch-action: manipulation` (no double-tap-to-zoom on a control). It was declared 50 times in 33
#     component stylesheets, with a 400-character comment copied 44 times, and kept in step by hand
#     with `main.css`'s `!important` backstop, which hid every sheet that forgot it. The theme rule
#     covers every `dx-` element that is a button, link, form control, `summary`, focusable by
#     `tabindex`, or an interactive ARIA widget role. A component sheet may still declare a DIFFERENT
#     value where the element is a drag surface or a scroller (`none`, `pan-x`, `pan-y`, ...); it may
#     not re-declare `manipulation`. If you are here because an element is clickable but not in the
#     theme's list, give it a role or `tabindex` (which it needs for the keyboard anyway) or extend
#     the theme rule's list -- do not paste the declaration.
#
#  2. The modal scrim: a `::backdrop` rule that paints (`background`, `backdrop-filter`, `transition`).
#     Dialog, AlertDialog, Sheet, Drawer, CommandDialog and the modal Popover carried the same
#     three-rule block each; it is now ONE rule on any `<dialog>` with a `dx-` class, driven by the
#     `--dx-overlay-*` tokens. A component sheet has no `::backdrop` rule to write; to retune the
#     scrim change the tokens, to remove it use `overlay: false` (`data-dx-overlay="off"`).
#
#  3. The two theme rules themselves are still there (a deleted rule would otherwise pass 1 and 2
#     and silently drop the behaviour from every component at once).
#
# Comments are stripped before scanning, so prose that mentions either is fine.
#
# Usage: scripts/check-theme-baseline.sh [FILE ...]   (default: preview/src/components/*/style.css;
# explicit files are for self-tests, and skip arm 3)
# Exit status: 0 when clean.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import glob
import re
import sys

THEME = "preview/assets/dx-components-theme.css"
explicit = sys.argv[1:]
files = explicit or sorted(glob.glob("preview/src/components/*/style.css"))
if not files:
    print("check-theme-baseline: no stylesheets found (run from a checkout)", file=sys.stderr)
    sys.exit(2)

COMMENT = re.compile(r"/\*.*?\*/", re.S)

def blank_comments(text):
    # Same-length whitespace, newlines kept, so reported line numbers are the file's.
    return COMMENT.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)

TOUCH = re.compile(r"(?<![\w-])touch-action\s*:\s*manipulation\b", re.I)
# A rule whose selector list mentions `::backdrop`: capture selector and body (no nested blocks inside).
BACKDROP_RULE = re.compile(r"([^{}]*::backdrop[^{}]*)\{([^{}]*)\}", re.I)
PAINTS = re.compile(r"(?<![\w-])(background(-color|-image)?|backdrop-filter|transition(-[a-z]+)?)\s*:", re.I)

errors = []
for path in files:
    with open(path, encoding="utf-8") as fh:
        text = blank_comments(fh.read())
    for m in TOUCH.finditer(text):
        line = text.count("\n", 0, m.start()) + 1
        errors.append(f"{path}:{line}: `touch-action: manipulation` is declared once, in {THEME} "
                      f"(the shared `dx-` control rule); delete it here")
    for m in BACKDROP_RULE.finditer(text):
        selector = " ".join(m.group(1).split())
        # The overlay-off switch is a theme rule, never a component one; a component has none to write.
        if PAINTS.search(m.group(2)):
            line = text.count("\n", 0, m.start(2)) + 1
            errors.append(f"{path}:{line}: `{selector}` paints a `::backdrop`; the modal scrim is the one "
                          f"shared rule in {THEME} (\"The modal scrim, once\"), retuned by the "
                          f"--dx-overlay-* tokens")

if not explicit:
    with open(THEME, encoding="utf-8") as fh:
        theme = blank_comments(fh.read())
    # Arm 3: the theme still has both rules.
    touch_rule = re.search(r"([^{}]*)\{\s*touch-action\s*:\s*manipulation\s*;?\s*\}", theme)
    if not touch_rule:
        errors.append(f"{THEME}: the shared `touch-action: manipulation` rule is gone")
    else:
        sel = touch_rule.group(1)
        for needed in ['button[class^="dx-"]', '[role="menuitem"][class^="dx-"]', 'a[href][class^="dx-"]',
                       '[role="spinbutton"][class^="dx-"]', '[tabindex]:not([tabindex="-1"])[class^="dx-"]']:
            if needed not in sel:
                errors.append(f"{THEME}: the shared touch-action rule no longer lists `{needed}`")
        if re.search(r"(^|,)\s*(button|a\[href\]|input|\[role=[^\]]*\])\s*(,|$)", sel):
            errors.append(f"{THEME}: the shared touch-action rule has a bare (non-`dx-`) selector; it must stay "
                          f"scoped to the `dx-` class namespace like the box-model rule")
    scrim = re.search(r"dialog\[class\^=\"dx-\"\][^{}]*::backdrop[^{}]*\{[^{}]*backdrop-filter\s*:\s*none", theme)
    scrim_open = re.search(r"\[open\]\[data-state=\"open\"\][^{}]*::backdrop[^{}]*\{[^{}]*--dx-overlay-scrim", theme)
    if not scrim or not scrim_open:
        errors.append(f"{THEME}: the shared modal-scrim `::backdrop` rules (resting + `[open][data-state=\"open\"]`) are gone")

if errors:
    for e in errors:
        print(e)
    print(f"\ncheck-theme-baseline: {len(errors)} problem(s). See the header of scripts/check-theme-baseline.sh.",
          file=sys.stderr)
    sys.exit(1)

print(f"check-theme-baseline: OK ({len(files)} stylesheet(s): no per-component touch-action: manipulation "
      f"or ::backdrop scrim; theme rules present)")
PYEOF
