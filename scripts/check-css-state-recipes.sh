#!/usr/bin/env bash
#
# check-css-state-recipes.sh
#
# Focus, invalid and disabled are shared tokens in the theme's "State recipes" section, so a component sheet may say
# where a ring paints but not what it is: a disabled `opacity` other than `var(--dx-opacity-disabled)`, a hand-written
# focus `box-shadow`, `outline` or `border-color`, a focus ring on a text-entry control that is transparent outside
# keyboard modality, the 2px `--dx-ring`, a hand-mixed destructive invalid ring, or a `cursor: not-allowed`
# beside `pointer-events: none` fails, with a reasoned allowlist. New control states compose through the tokens and
# slot variables.
#
#   FOCUS     --dx-ring-field (inset-edge fields), --dx-ring-focus (+ `border-color: var(--dx-ring-color)`
#             on a bordered control), --dx-ring-focus-inset (a container that clips an outer halo)
#   INVALID   --dx-ring-invalid-color (edge), --dx-ring-invalid (halo), --dx-ring-invalid-field (both)
#   DISABLED  opacity: var(--dx-opacity-disabled), and pointer-events OR cursor, never both
#
# Without them every sheet grows its own focus-ring family (a 2px `--dx-ring`, an `outline: 2px solid`, an inset
# 2px, a literal 3px `color-mix`, a hard-coded blue, ...), its own invalid vocabulary and its own hand-written
# `opacity: 0.5` for disabled, each correct on the day it is pasted and drifting from then on (row 145). A
# component stylesheet may say WHERE the ring paints (which element, which pseudo-element); it may not
# re-derive WHAT it is. This gate fails, per rule (comments stripped, `:not(...)` groups ignored):
#
#   disabled-opacity   a rule selecting a disabled state (`:disabled`, `[disabled]`, `[data-disabled]`,
#                      `[aria-disabled]`) declares an `opacity` that is not `var(--dx-opacity-disabled)`
#                      (directly, or as the fallback of a slot variable). `opacity: 1` (a reset) is fine.
#   dead-cursor        one rule declares both `cursor: not-allowed` and `pointer-events: none`: the cursor
#                      of an element that takes no pointer events is never shown. Pick one.
#   focus-shadow       a `:focus-visible` / `:focus-within` rule declares a `box-shadow` that does not
#                      reference a focus or invalid recipe token (or is `none`).
#   focus-outline      such a rule declares an `outline` other than `none`/`0`. The ring is a token, not an
#                      outline (the forced-colors outline is the theme's one).
#   focus-border       such a rule declares a `border-color` that is not `--dx-ring-color`/the invalid edge.
#   text-entry-ring    a `:focus-visible` / `:focus-within` rule on a TEXT-ENTRY control (a selector naming
#                      `input`, `textarea`, `[contenteditable]`, or a `.dx-...-input|-textarea|-segment|-search`
#                      class: Input, Textarea, Input Group, Input OTP, Combobox/Command input, Date Picker
#                      segment, ...) declares a `box-shadow` that reads `--dx-ring-focus` or
#                      `--dx-ring-focus-inset`. Those two are zeroed outside keyboard modality
#                      (`:root[data-dx-modality]:not([data-dx-modality="keyboard"])` in the theme), but a text
#                      field the user clicked or tapped is still focused and must still ring, as in shadcn:
#                      a field takes `--dx-ring-field` (inset edge + halo) or, when its edge is a real
#                      border (Textarea outline), `--dx-ring-focus-halo` beside `border-color`. Without it the
#                      outline Textarea loses its halo after a click.
#   legacy-ring        `var(--dx-ring)` (the 2px solid ring) anywhere: use `--dx-ring-focus`, or
#                      `--dx-ring-focus-inset`. (The token stays defined for `[data-dx-interactive]`.)
#   literal-ring       `color-mix(... var(--dx-ring-color) 50% ...)` written out: that is `--dx-ring-focus`.
#   invalid-shadow     a rule selecting `[aria-invalid]` / `[data-invalid]` declares a `box-shadow` that does
#                      not reference `--dx-ring-invalid*` (or the `--dx-input-invalid-shadow` slot), or a
#                      `border-color` that is not `--dx-ring-invalid-color`: the destructive edge and halo
#                      are one recipe, not a per-control mix of `--dx-destructive` percentages.
#
# EXCEPTIONS are in ALLOW below, each with its reason; an entry that no longer matches anything fails the run as
# stale (scripts/lib/allow.py). The sheets are read by scripts/lib/css.py, the one CSS reader of the gates.
#
# What this does NOT catch: whether a control that sets `outline: none` has a ring AT ALL (text controls
# cannot be identified from CSS; `playwright/focus-ring.spec.ts` holds the field controls to the token),
# or whether a control has an invalid style (the control decides whether it can be invalid).
#
# Usage: scripts/check-css-state-recipes.sh [FILE ...]   (default: preview/src/components/*/style.css;
# explicit files are for self-tests, and skip the allowlist stale-entry report)
# Exit status: 0 when clean.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import glob
import re
import sys

sys.path.insert(0, "scripts/lib")
from allow import Allowlist  # noqa: E402
from css import blank_comments, rules as css_rules  # noqa: E402

explicit = sys.argv[1:]
files = explicit or sorted(glob.glob("preview/src/components/*/style.css"))
if not files:
    print("check-css-state-recipes: no stylesheets found (run from a checkout)", file=sys.stderr)
    sys.exit(2)

# (check, file suffix, substring of the selector or declaration value, reason)
ALLOW = Allowlist("check-css-state-recipes", [
    ("focus-shadow", "sidebar/style.css", "--dx-sidebar-ring",
     "shadcn's sidebar buttons ring with `focus-visible:ring-2 ring-sidebar-ring` (2px, the sidebar's own ring colour)"),
    ("focus-outline", "tabs/style.css", "1px solid var(--dx-ring-color)",
     "Nova's tabs trigger is literally `focus-visible:outline-1 focus-visible:outline-ring` on top of the ring"),
    ("focus-border", "button/style.css", "--dx-destructive",
     "the destructive Button's focus border is destructive/40 (Nova `focus-visible:border-destructive/40`)"),
    ("focus-border", "alert_dialog/style.css", "--dx-destructive",
     "the alert dialog's action is a destructive Button; it takes the destructive Button's focus border"),
], fields=("check", "suffix", "needle"))

def strip_not(sel):
    """Drop every `:not( ... )` group (balanced), so a negated state is not read as the state."""
    out, i = [], 0
    while i < len(sel):
        if sel.startswith(":not(", i):
            depth, j = 0, i + 4
            while j < len(sel):
                if sel[j] == "(":
                    depth += 1
                elif sel[j] == ")":
                    depth -= 1
                    if depth == 0:
                        break
                j += 1
            i = j + 1
        else:
            out.append(sel[i])
            i += 1
    return "".join(out)

DECL = re.compile(r"(?<![\w-])([a-z-]+)\s*:\s*([^;]+)")
DISABLED = re.compile(r":disabled|\[disabled|\[data-disabled|\[aria-disabled")
FOCUS = re.compile(r":focus-visible|:focus-within")
INVALID = re.compile(r"\[aria-invalid|\[data-invalid")
FOCUS_TOKENS = ("--dx-ring-field", "--dx-ring-focus", "--dx-ring-invalid")
# A selector that names a text-entry control (see the text-entry-ring rule in the header).
TEXT_ENTRY = re.compile(
    r"(?<![\w.#-])(?:input|textarea)(?![\w-])|\[contenteditable"
    r"|\.dx-(?:[a-z0-9]+-)*(?:input|textarea|segment|search)(?:-[a-z0-9]+)*(?![\w-])"
)
# `--dx-ring-focus` / `--dx-ring-focus-inset` as a var() argument, but not `--dx-ring-focus-halo`.
MODALITY_RING = re.compile(r"var\(\s*--dx-ring-focus(?:-inset)?\s*[,)]")

def allowed(check, path, selector, value):
    return bool(ALLOW.find_all(lambda e: e.check == check and path.endswith(e.suffix)
                               and (e.needle == "" or e.needle in selector or e.needle in value)))

errors = []

def fail(check, path, line, selector, value, msg):
    if allowed(check, path, selector, value):
        return
    errors.append(f"{path}:{line}: [{check}] `{' '.join(selector.split())[:80]}`: {msg}")

for path in files:
    with open(path, encoding="utf-8") as fh:
        raw = fh.read()
    text = blank_comments(raw)
    for m in re.finditer(r"var\(--dx-ring\)", text):
        line = text.count("\n", 0, m.start()) + 1
        fail("legacy-ring", path, line, "var(--dx-ring)", "--dx-ring",
             "the legacy 2px ring; use var(--dx-ring-focus) (or --dx-ring-focus-inset in a clipping container)")
    for r in css_rules(raw):
        selector, body = r.prelude, r.body
        if not re.search(r"[.\[:#]", selector):
            continue  # @keyframes steps (`from`, `50%`)
        line = r.brace_line
        bare = strip_not(selector)
        decls = {k: " ".join(v.split()) for k, v in DECL.findall(body)}

        for k, v in decls.items():
            if re.search(r"color-mix\([^;]*var\(--dx-ring-color\)\s*50%", v):
                fail("literal-ring", path, line, selector, v,
                     "a hand-written ring/50 halo is --dx-ring-focus; use the token")

        if DISABLED.search(bare) and "opacity" in decls:
            v = decls["opacity"]
            if "--dx-opacity-disabled" not in v and v not in ("1", "inherit", "unset"):
                fail("disabled-opacity", path, line, selector, v,
                     f"disabled `opacity: {v}`; use var(--dx-opacity-disabled)")

        if "not-allowed" in decls.get("cursor", "") and decls.get("pointer-events") == "none":
            fail("dead-cursor", path, line, selector, "", "`cursor: not-allowed` with `pointer-events: none`: the cursor never shows; keep one")

        if FOCUS.search(bare):
            v = decls.get("box-shadow")
            if v is not None and v != "none" and not any(t in v for t in FOCUS_TOKENS):
                fail("focus-shadow", path, line, selector, v,
                     f"focus `box-shadow: {v}` is not a recipe token (--dx-ring-field / --dx-ring-focus / --dx-ring-focus-inset)")
            if v is not None and TEXT_ENTRY.search(bare) and MODALITY_RING.search(v):
                fail("text-entry-ring", path, line, selector, v,
                     f"text-entry focus `box-shadow: {v}` is zeroed outside keyboard modality, so a clicked or tapped "
                     "field loses its ring; use var(--dx-ring-field), or var(--dx-ring-focus-halo) beside the border-color")
            v = decls.get("outline")
            if v is not None and v not in ("none", "0") and not v.startswith("0 "):
                fail("focus-outline", path, line, selector, v,
                     f"focus `outline: {v}`; the ring is a token on box-shadow (outline: none)")
            v = decls.get("border-color")
            if v is not None and not any(t in v for t in ("--dx-ring-color", "--dx-ring-invalid-color")) \
                    and v not in ("transparent", "inherit", "currentColor"):
                fail("focus-border", path, line, selector, v,
                     f"focus `border-color: {v}`; the focused edge is var(--dx-ring-color)")

        if INVALID.search(bare):
            v = decls.get("box-shadow")
            if v is not None and v != "none" and "--dx-ring-invalid" not in v and "--dx-input-invalid-shadow" not in v:
                fail("invalid-shadow", path, line, selector, v,
                     f"invalid `box-shadow: {v}` is not the recipe; use var(--dx-ring-invalid-field) / var(--dx-ring-invalid)")
            v = decls.get("border-color")
            if v is not None and "--dx-ring-invalid-color" not in v:
                fail("invalid-shadow", path, line, selector, v,
                     f"invalid `border-color: {v}`; use var(--dx-ring-invalid-color)")

if errors:
    for e in errors:
        print(e)
    print(f"\ncheck-css-state-recipes: {len(errors)} problem(s). See the header of scripts/check-css-state-recipes.sh "
          f"and the \"State recipes\" section of preview/assets/dx-components-theme.css.", file=sys.stderr)
    sys.exit(1)

problems = ALLOW.problems(stale=not explicit)
if problems:
    for p in problems:
        print(f"check-css-state-recipes: {p}")
    sys.exit(1)
n_used = sum(1 for e in ALLOW if ALLOW.used(e))
print(f"check-css-state-recipes: OK ({len(files)} stylesheet(s): focus, invalid and disabled use the recipe tokens; "
      f"{n_used} allowlist entr{'y' if n_used == 1 else 'ies'} in use)")
PYEOF
