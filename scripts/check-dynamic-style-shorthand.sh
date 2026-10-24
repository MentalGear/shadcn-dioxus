#!/usr/bin/env bash
#
# check-dynamic-style-shorthand.sh
#
# An rsx `style:` string with an interpolation may not declare a shorthand whose value holds a `var()` (`border`,
# `background`, `border-radius`, ...), and a `style:` chosen by `if`/`match` is banned: Dioxus's web renderer drops a
# `var()`-valued shorthand from an inline `style` the first time the string is re-set (the server markup is right, the
# hydrated page loses the border) and keeps stale longhands when one string is swapped for another. Declare the
# shorthand in a stylesheet or `style {}` rule keyed by a class or data attribute and keep only computed numbers
# inline, spell the longhands, or paint by `data-state`. Reasoned allowlists, stale entries fail.
#
# Dioxus's web renderer (dioxus-interpreter-js 0.7.x, `setAttributeInner`, case "style") re-sets an inline
# `style` like this: remember every longhand the element's style declares, `setAttribute("style", value)`,
# then put back each remembered longhand whose value reads empty. A shorthand whose value holds a `var()`
# (`border: 2px solid var(--dx-primary)`, `background: var(--c)`, `border-radius: var(--r)`) is stored as
# pending-substitution longhands that all read "", so the "put back" writes `setProperty(longhand, "")`, which
# REMOVES them -- and the whole shorthand with them. The string you wrote is right and the markup the server
# sent is right (the SSG page shows the border); the first time the string is re-set on the client the
# declaration silently disappears. Examples: the carousel's active dot loses its colour and radius after the
# first click (documented in carousel/docs.md), and the photo wall's minimap box loses its border and fill as
# soon as the wall mounts its first tiles (`first-paint-styles` reports "border 2px -> 0 on
# /component/virtual_grid/": the JS-off page and the hydrated page compute different styles).
#
# The same re-set also makes the opposite swap wrong: a string swapped for another string or for "" leaves
# the remembered longhands in place, because they were remembered before the swap and are restored only if
# they read empty. A checkpoint bar painted with
# `style: if done { "...; background: var(--dx-primary);" } else { "...; background: var(--dx-muted);" }`: after
# two steps the segments that changed keep only `height` and `flex` (computed background transparent), so
# the bar shows one bright segment, two invisible ones and one dark one; a chevron
# (`style: if open() { "rotate: ..." } else { "" }`) stays rotated after closing (the questionnaire `progress`
# demo and the `file_tree` / bubble `collapsible` chevrons were the instances).
# Both are the one interpreter behaviour, so the gate has two arms.
#
# A static `style: "..."` is never re-set, so only a string with an interpolation, or a `style:` whose value is
# chosen at run time by `if`/`match`, can hit it. The class is made unable to recur by construction:
#
#   ARM 2. A rsx `style:` attribute must not be an `if` or `match` expression choosing between whole style
#   strings. State-dependent paint goes through a class or a `data-state` attribute with a rule in the
#   component's `style.css`, or the two states are two static `if`/`else` elements (a static literal is never
#   re-set). Reasoned exceptions are in ALLOW_CHOICE below.
#
#   ARM 1.
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

# ARM 2 exceptions: (file suffix, substring of the line, reason). Each is the same shape as the defect, but none
# misrenders on the built site, because the choice is made once and never swapped live.
ALLOW_CHOICE = [
    ("preview/src/main.rs", "style: match member.2 {", "docs-site member status dot: the status is fixed per row and never changes after render, and every arm declares the same longhands (background-color is a longhand)"),
    ("primitives/src/chart/components/series/radar.rs", "style: if ring_fill {", "sets only a custom property, or nothing; ring_fill is fixed for a chart's lifetime, so the string is never swapped"),
    ("primitives/src/combobox/components/input.rs", "style: if ctx.is_multi()", "is_multi is fixed for a combobox's lifetime, so the string is never swapped for \"\""),
]
used_choice = set()
CHOICE = re.compile(r"(?<![\w-])\"?style\"?\s*:\s*(?:\{\s*)?(?:if|match)\b")

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
    for n, line_text in enumerate(text.split("\n"), 1):
        if CHOICE.search(line_text):
            hit = next((i for i, (suffix, sub, _w) in enumerate(ALLOW_CHOICE) if path.endswith(suffix) and sub in line_text), None)
            if hit is not None:
                used_choice.add(hit)
            else:
                problems.append(f"{path}:{n}: `style:` chosen by if/match: `{line_text.strip()[:90]}`")
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
    stale = [f"ALLOW_CHOICE entry {i} ({s}, {sub!r}) matches nothing: remove it" for i, (s, sub, _) in enumerate(ALLOW_CHOICE) if i not in used_choice]
    stale += [f"ALLOW entry {i} ({s}, {sub!r}) matches nothing: remove it" for i, (s, sub, _) in enumerate(ALLOW) if i not in used]

if problems or stale:
    for p in problems:
        print(p)
    for s in stale:
        print(s)
    if problems:
        print(
            "\ncheck-dynamic-style-shorthand: Dioxus re-sets an inline `style` by remembering and restoring longhands: it drops "
            "a `var()` shorthand and keeps stale longhands when a string is swapped (a `style:` chosen by if/match). Move the declaration to a stylesheet / `style {}` rule keyed by a class or data attribute "
            "(keep only the computed numbers inline), or write the longhands. See this script's header."
        )
    sys.exit(1)

print(f"check-dynamic-style-shorthand: OK ({scanned} file(s) scanned)")
PYEOF
