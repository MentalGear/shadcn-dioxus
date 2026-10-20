#!/usr/bin/env bash
#
# check-dynamic-style-shorthand.sh
#
# Dioxus's web renderer (dioxus-interpreter-js 0.7.x, `setAttributeInner`, case "style") re-sets an inline
# `style` like this: remember every longhand the element's style declares, `setAttribute("style", value)`,
# then put back each remembered longhand whose value reads empty. A shorthand whose value holds a `var()`
# (`border: 2px solid var(--dx-primary)`, `background: var(--c)`, `border-radius: var(--r)`) is stored as
# pending-substitution longhands that all read "", so the "put back" writes `setProperty(longhand, "")`, which
# REMOVES them -- and the whole shorthand with them. The string you wrote is right and the markup the server
# sent is right (the SSG page shows the border); the first time the string is re-set on the client the
# declaration silently disappears. Seen twice: the carousel's active dot lost its colour and radius after the
# first click (documented in carousel/docs.md), and the photo wall's minimap box lost its border and fill as
# soon as the wall mounted its first tiles (`first-paint-styles` found it as "border 2px -> 0 on
# /component/virtual_grid/": the JS-off page and the hydrated page computed different styles).
#
# A static `style: "..."` is never re-set, so only a string with an interpolation can hit it. The class is
# made unable to recur by construction:
#
#   An inline `style:` string (rsx `style: "..."` / `"style": "..."`) that has an interpolation `{...}` must not
#   declare a SHORTHAND property (border*, background, margin, padding, font, outline, flex, gap, inset, overflow,
#   transition, animation, mask, ...) with a `var()` in its value. Put the declaration in a stylesheet or a
#   `style {}` rule keyed by a class or `data-*` attribute (which the string never touches), keep only the
#   computed numbers inline, or spell the longhands (`border-top-width`, `background-color`: a longhand with a
#   `var()` keeps its text and survives).
#
# Not covered: a style string assembled elsewhere and passed as `style: "{style}"` (the string literal has no
# declaration to read); the same rule applies to it by hand.
#
# Scope: preview/src/**, primitives/src/**, templates/** (`*.rs`). Comments are blanked first.
# EXCEPTIONS are in ALLOW below, each with its reason; an entry that no longer matches is an error (a fixed
# site must leave the list). The list starts empty.
#
# Usage: scripts/check-dynamic-style-shorthand.sh [FILE ...]   (default: every *.rs under the scopes; explicit
# files are for self-tests, and skip the allowlist stale-entry check)
# Exit status: 0 when clean, 1 on a violation, 2 when nothing could be scanned.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import glob
import re
import sys

explicit = sys.argv[1:]
files = explicit or sorted(
    glob.glob("preview/src/**/*.rs", recursive=True)
    + glob.glob("primitives/src/**/*.rs", recursive=True)
    + glob.glob("templates/**/*.rs", recursive=True)
)
if not files:
    print("check-dynamic-style-shorthand: no sources found (run from a checkout)", file=sys.stderr)
    sys.exit(2)

# (file suffix, substring of the declaration, reason). Empty on purpose: see the header.
ALLOW = [
]
used = set()

SHORTHANDS = {
    "all", "animation", "background", "border", "border-block", "border-block-end", "border-block-start",
    "border-bottom", "border-color", "border-image", "border-inline", "border-inline-end", "border-inline-start",
    "border-left", "border-radius", "border-right", "border-style", "border-top", "border-width", "columns",
    "flex", "flex-flow", "font", "gap", "grid", "grid-area", "grid-column", "grid-row", "grid-template",
    "inset", "inset-block", "inset-inline", "list-style", "margin", "margin-block", "margin-inline", "mask",
    "outline", "overflow", "padding", "padding-block", "padding-inline", "place-content", "place-items",
    "place-self", "scroll-margin", "scroll-padding", "text-decoration", "text-emphasis", "transition",
    "white-space", "-webkit-mask", "-webkit-text-stroke",
}

BLOCK_COMMENT = re.compile(r"/\*.*?\*/", re.S)
# `style: "..."` or `"style": "..."`, the literal optionally inside `format!(`; the literal may span lines.
STYLE_LITERAL = re.compile(r'(?<![\w-])"?style"?\s*:\s*(?:format!\(\s*)?"((?:[^"\\]|\\.)*)"', re.S)
DECL = re.compile(r"(?:^|;)\s*(-?[a-z][a-z-]*)\s*:\s*([^;]*)", re.I)

def blank(text):
    # whole-line `//` comments only (a `//` inside a string, such as a URL, is code)
    return re.sub(r"(?m)^(\s*)//.*$", lambda m: m.group(1), text)

def has_interpolation(literal):
    return "{" in literal.replace("{{", "").replace("}}", "")

problems = []
scanned = 0
for path in files:
    try:
        text = blank(open(path, encoding="utf-8").read())
    except OSError as err:
        print(f"check-dynamic-style-shorthand: cannot read {path}: {err}", file=sys.stderr)
        sys.exit(2)
    scanned += 1
    for m in STYLE_LITERAL.finditer(text):
        literal = m.group(1)
        if not has_interpolation(literal):
            continue
        line = text.count("\n", 0, m.start()) + 1
        for d in DECL.finditer(literal):
            prop, value = d.group(1).lower(), d.group(2)
            if prop in SHORTHANDS and "var(" in value:
                decl = f"{prop}: {value.strip()}"
                allowed = next(
                    (i for i, (suffix, sub, _why) in enumerate(ALLOW) if path.endswith(suffix) and sub in decl),
                    None,
                )
                if allowed is not None:
                    used.add(allowed)
                    continue
                problems.append(f"{path}:{line}: dynamic inline style declares the shorthand `{decl}`")

stale = []
if not explicit:
    stale = [f"ALLOW entry {i} ({s}, {sub!r}) matches nothing: remove it" for i, (s, sub, _) in enumerate(ALLOW) if i not in used]

if problems or stale:
    for p in problems:
        print(p)
    for s in stale:
        print(s)
    if problems:
        print(
            "\ncheck-dynamic-style-shorthand: Dioxus drops a `var()` shorthand from an inline `style` the first time the "
            "string is re-set. Move the declaration to a stylesheet / `style {}` rule keyed by a class or data attribute "
            "(keep only the computed numbers inline), or write the longhands. See this script's header."
        )
    sys.exit(1)

print(f"check-dynamic-style-shorthand: OK ({scanned} file(s) scanned)")
PYEOF
