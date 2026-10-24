#!/usr/bin/env bash
#
# check-component-fonts.sh
#
# A component stylesheet never picks a typeface: in every `preview/src/components/*/style.css` a `font-family` is
# exactly `inherit` (for the elements the UA does not inherit for: button, input, textarea, select) or a
# `var(--dx-font-<name>)` theme token, the `font` shorthand may only be `font: inherit`, and an `@font-face` is
# flagged too. The app picks the font, so an overlay can never fall back to a hard-coded system stack.
#
#   font-family: inherit;                  -- for the elements the UA does NOT inherit for (button, input,
#                                             textarea, select, optgroup), which would otherwise fall back to
#                                             the system UI font
#   font-family: var(--dx-font-<name>);    -- a theme token the host can retune (optionally with an
#                                             `inherit` fallback: `var(--dx-font-mono, inherit)`)
#
# and the `font` shorthand (which resets the family too) may only be `font: inherit`.
#
# THE DEFECT (dev-docs/research/round3-2026-10-05.md, row 162): a hard-coded
# `font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif` in a
# component sheet. A top-layer `<dialog>` still inherits from its DOM parent, so none of that is needed -- it
# only makes every overlay (and the calendar) render in the system UI font while the page around it is Geist,
# and ignore the app font of any consumer. Five stylesheets (dialog, alert-dialog, sheet, drawer, calendar)
# carried the same stack: a class, not five typos, so the gate makes the class unable to recur: a literal
# family is simply not expressible in a component stylesheet.
#
# Comments are stripped before scanning, so prose that mentions a font is fine; a declaration that spans
# several lines is matched as one. `@font-face` descriptors are flagged on
# purpose: a component must not ship a face either.
#
# If you are here because it failed: delete the declaration (the element inherits already) or use
# `font-family: inherit` on a form control. If you really need a distinct face (monospace for `kbd`/`code`),
# add a `--dx-font-<name>` token to `preview/assets/dx-components-theme.css` and reference that.
#
# Usage: scripts/check-component-fonts.sh [FILE ...]   (default: preview/src/components/*/style.css; explicit
# files are for self-tests)
# Exit status: 0 if every scanned stylesheet is clean.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import glob
import re
import sys

files = sys.argv[1:] or sorted(glob.glob("preview/src/components/*/style.css"))
if not files:
    print("check-component-fonts: no stylesheets found (run from a checkout)", file=sys.stderr)
    sys.exit(2)

# Allowed `font-family` values (compared case-insensitively, whitespace-collapsed, `!important` dropped).
FAMILY_OK = re.compile(r"^(inherit|var\(\s*--dx-font-[a-z0-9-]+\s*(,\s*inherit\s*)?\))$", re.I)
SHORTHAND_OK = re.compile(r"^inherit$", re.I)

DECL = re.compile(r"(?<![\w-])(font-family|font)\s*:\s*([^;{}]*)(?=[;}])", re.I)
COMMENT = re.compile(r"/\*.*?\*/", re.S)

def blank_comments(text):
    # Replace comments with same-length whitespace (keeping newlines) so line numbers survive.
    return COMMENT.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)

errors = []
for path in files:
    with open(path, encoding="utf-8") as fh:
        raw = fh.read()
    text = blank_comments(raw)
    for m in DECL.finditer(text):
        prop = m.group(1).lower()
        value = re.sub(r"\s+", " ", m.group(2)).strip()
        value = re.sub(r"\s*!important$", "", value, flags=re.I)
        ok = (FAMILY_OK if prop == "font-family" else SHORTHAND_OK).match(value)
        if not ok:
            line = text.count("\n", 0, m.start()) + 1
            errors.append((path, line, prop, value))

if errors:
    for path, line, prop, value in errors:
        print(f"{path}:{line}: `{prop}: {value}` -- a component stylesheet may only inherit the font "
              f"(`inherit`, or `var(--dx-font-<name>)` for `font-family`)")
    print(f"\ncheck-component-fonts: {len(errors)} hard-coded font declaration(s). The app/docs site chooses the "
          f"font; delete the declaration or use `inherit`. See the header of scripts/check-component-fonts.sh.",
          file=sys.stderr)
    sys.exit(1)

print(f"check-component-fonts: OK ({len(files)} stylesheet(s), no hard-coded font family)")
PYEOF
