#!/usr/bin/env bash
#
# check-destructive-text.sh
#
# Destructive text is the theme's one clamped role `--dx-destructive-text` (at least 4.5:1 on every surface in both
# modes): no `color`, `caret-color`, `text-decoration-color` or `*-ink`/`*-color` custom property in `preview/src` or
# `primitives/src` (`*.css`, `*.rs`, inline `style:` strings) may read the fill-grade `var(--dx-destructive)`, bare or
# inside `color-mix`. Borders, backgrounds, fills, rings and shadows keep it. Reasoned allowlist, stale entries fail.
#
#   A declaration that paints TEXT -- `color`, `caret-color`, `text-decoration-color`,
#   `-webkit-text-fill-color`, `text-emphasis-color`, or a custom property that is a text ink
#   (`--*-ink`, `--*-color` unless the name says it is an edge, ring, fill, background or shadow) --
#   in a stylesheet or an inline `style:` string must not mention `var(--dx-destructive)`, bare or as a
#   `color-mix(` operand. Read `var(--dx-destructive-text)`.
#
#   `border*`, `outline*`, `background*`, `fill`, `stroke`, `box-shadow`, `accent-color` stay legal: they
#   are non-text paint, where the fill-grade role is right (the destructive Button's solid fill, an
#   invalid ring). The `-fill`, `-foreground` and `-text` siblings never match: only the exact
#   `var(--dx-destructive)` reference does.
#
# Scope: preview/src/** and primitives/src/** (`*.css`, `*.rs`). The theme sheet (preview/assets) is where
# the role is defined and is out of scope. Comments are stripped first, so prose that names the token is fine.
#
# EXCEPTIONS are in ALLOW below, each with its reason; an entry that no longer matches anything is itself
# an error (a fixed site must leave the list). The list is empty: a known site is fixed, not allowlisted.
#
# Usage: scripts/check-destructive-text.sh [FILE ...]   (default: every *.css and *.rs under the scopes;
# explicit files are for self-tests, and skip the allowlist stale-entry check)
# Exit status: 0 when clean, 1 on a violation, 2 when nothing could be scanned.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import glob
import re
import sys

sys.path.insert(0, "scripts/lib")
from allow import Allowlist  # noqa: E402
from css import blank_comments  # noqa: E402

explicit = sys.argv[1:]
files = explicit or sorted(
    glob.glob("preview/src/**/*.css", recursive=True)
    + glob.glob("preview/src/**/*.rs", recursive=True)
    + glob.glob("primitives/src/**/*.css", recursive=True)
    + glob.glob("primitives/src/**/*.rs", recursive=True)
)
if not files:
    print("check-destructive-text: no sources found (run from a checkout)", file=sys.stderr)
    sys.exit(2)

# (file suffix, substring of the declaration, reason). Empty on purpose: see the header.
ALLOW = Allowlist("check-destructive-text", [], fields=("suffix", "needle"))

def blank(text, path):
    text = blank_comments(text, strings=not path.endswith(".rs"))
    if path.endswith(".rs"):
        # whole-line `//` comments only (a `//` inside a string, such as a URL, is code)
        text = re.sub(r"(?m)^(\s*)//.*$", lambda m: m.group(1), text)
    return text

TEXT_PROPS = {"color", "caret-color", "text-decoration-color", "-webkit-text-fill-color", "text-emphasis-color"}
NOT_TEXT_NAME = re.compile(r"border|ring|edge|outline|fill|stroke|background|bg|shadow|scrim|overlay", re.I)
# The value sits in a lookahead so an inline `style: "color: ..."` still yields the inner `color` declaration
# (a consuming match would swallow it as the value of `style`); it stops at `;`, a brace or a quote.
DECL = re.compile(r"(?<![\w-])((?:--)?[a-z][\w-]*)\s*:\s*(?=([^;{}\"]*))", re.I)
FILL_GRADE = re.compile(r"var\(\s*--dx-destructive\s*[,)]")

def is_text_paint(name):
    n = name.lower()
    if n in TEXT_PROPS:
        return True
    if n.startswith("--") and (n.endswith("-ink") or n.endswith("-color")):
        return not NOT_TEXT_NAME.search(n)
    return False

def allowed(path, decl):
    return bool(ALLOW.find_all(lambda e: path.endswith(e.suffix) and e.needle in decl))

errors = []
for path in files:
    with open(path, encoding="utf-8") as fh:
        text = blank(fh.read(), path)
    for m in DECL.finditer(text):
        name, value = m.group(1), m.group(2)
        if not is_text_paint(name) or not FILL_GRADE.search(value):
            continue
        decl = f"{name}: {' '.join(value.split())}"
        if allowed(path, decl):
            continue
        line = text.count("\n", 0, m.start(1)) + 1
        errors.append(f"{path}:{line}: `{decl}` paints text with the fill-grade `--dx-destructive` "
                      f"(2.1-2.8:1 on dark surfaces); read `var(--dx-destructive-text)`")

if errors:
    for e in errors:
        print(e)
    print(f"\ncheck-destructive-text: {len(errors)} problem(s). See the header of scripts/check-destructive-text.sh "
          f"and `--dx-destructive-text` in preview/assets/dx-components-theme.css.", file=sys.stderr)
    sys.exit(1)

problems = ALLOW.problems(stale=not explicit)
if problems:
    for p in problems:
        print(p)
    print("\ncheck-destructive-text: bad allowlist (a site that no longer violates must leave it)", file=sys.stderr)
    sys.exit(1)
n_used = sum(1 for e in ALLOW if ALLOW.used(e))
print(f"check-destructive-text: OK ({len(files)} file(s): no text paint reads the fill-grade --dx-destructive; "
      f"{n_used} allowlist entr{'y' if n_used == 1 else 'ies'} in use)")
PYEOF
