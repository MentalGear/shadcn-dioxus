#!/usr/bin/env bash
#
# check-focus-outline-reset.sh
#
# The focus ring of this library is a `box-shadow` token (`--dx-ring-field`, `--dx-ring-focus`,
# `--dx-ring-focus-inset`, see "State recipes" in `preview/assets/dx-components-theme.css`), and
# "`outline` is never the ring (`outline: none` + a token)". A focusable element that paints that
# shadow but never neutralises the UA focus outline shows BOTH: the browser's own outline, drawn
# outside the ring, in the UA colour. (Checkbox and Tag Group's tag are the shape: the ring is correct and the
# rule-set looks finished, because the UA outline only appears on a real keyboard focus, which no
# snapshot takes.)
#
# This gate fails when, in a component `style.css` (comments stripped):
#
#   a rule whose selector ends in `:focus-visible` or `:focus` (the focusable element itself; a
#   `:focus-within` container is not focusable, and `x:focus-visible + y` rings a sibling, which
#   this gate does not follow) declares a ring `box-shadow` (anything but `none`), and
#   NO rule of the same sheet whose last compound selects the same element (same class / tag
#   tokens or a subset of them, pseudo-classes limited to the focus ones) declares an outline reset:
#   `outline: none`, `outline: 0`, `outline: <w> <style> transparent`, `outline-style: none`,
#   `outline-width: 0`, `outline-color: transparent`, or `all: unset` (outline back to its initial `none`).
#
# The reset may sit on the element's base rule (the usual place: `.dx-x { outline: none }`) or on the
# focus rule itself; either satisfies it. A rule inside `@media (forced-colors: active)` does not
# count as a reset (it restores the outline). The theme's forced-colors rule is the one outline.
#
# EXCEPTIONS are in ALLOW below, each with its reason; an entry that matches nothing fails as stale
# (scripts/lib/allow.py). The sheets are read by scripts/lib/css.py, the one CSS reader of the gates.
#
# Usage: scripts/check-focus-outline-reset.sh [FILE ...]   (default: preview/src/components/*/style.css;
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
from css import rules as css_rules, split_top as split_top_raw, subject_compound as last_compound  # noqa: E402

explicit = sys.argv[1:]
files = explicit or sorted(glob.glob("preview/src/components/*/style.css"))
if not files:
    print("check-focus-outline-reset: no stylesheets found (run from a checkout)", file=sys.stderr)
    sys.exit(2)

# (file suffix, substring of the focus selector, reason)
ALLOW = Allowlist("check-focus-outline-reset", [], fields=("suffix", "needle"))

def selector_list(s):
    """The stripped, non-empty entries of a selector list."""
    return [p for p in (x.strip() for x in split_top_raw(s, ",")) if p]

def strip_groups(c):
    """Drop `:not(...)`/`:is(...)`/`:where(...)`/`:has(...)` groups and [attr] blocks."""
    out, i = [], 0
    while i < len(c):
        m = re.match(r":(not|is|where|has)\(", c[i:])
        if m:
            depth, j = 0, i + len(m.group(0)) - 1
            while j < len(c):
                if c[j] == "(":
                    depth += 1
                elif c[j] == ")":
                    depth -= 1
                    if depth == 0:
                        break
                j += 1
            i = j + 1
        elif c[i] == "[":
            j = c.find("]", i)
            i = (j if j >= 0 else len(c)) + 1
        else:
            out.append(c[i])
            i += 1
    return "".join(out)

def unwrap_where(c):
    """`:where(X)` / `:is(X)` -> `X` (balanced; a zero-specificity wrapper selects the same element)."""
    while True:
        m = re.search(r":(?:where|is)\(", c)
        if not m:
            return c
        depth, j = 0, m.end() - 1
        while j < len(c):
            if c[j] == "(":
                depth += 1
            elif c[j] == ")":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        c = c[:m.start()] + c[m.end():j] + c[j + 1:]

FOCUS_PSEUDO = re.compile(r":focus-visible|:focus(?![-\w])")
PSEUDO_EL = re.compile(r"::[\w-]+")

def tokens_of(compound):
    """(set of .class / tag tokens, set of non-focus pseudo-classes) of a compound, groups removed."""
    c = PSEUDO_EL.sub("", strip_groups(unwrap_where(compound)))
    c = FOCUS_PSEUDO.sub("", c)
    pseudos = set(re.findall(r":[\w-]+", c))
    c = re.sub(r":[\w-]+(\([^)]*\))?", "", c)
    toks = set(re.findall(r"\.[\w-]+|#[\w-]+|^[a-z][\w-]*|(?<=[\]\)])[a-z][\w-]*", c))
    return toks, pseudos

DECL = re.compile(r"(?<![\w-])([a-z-]+)\s*:\s*([^;]+)")

def is_reset(decls):
    for k, v in decls.items():
        v = v.strip()
        if k == "all" and v == "unset":
            return True  # resets outline to its initial `none`; an author declaration beats the UA focus rule
        if k == "outline" and (v in ("none", "0") or v.startswith("0 ") or "transparent" in v):
            return True
        if k == "outline-style" and v == "none":
            return True
        if k == "outline-width" and v in ("0", "0px"):
            return True
        if k == "outline-color" and "transparent" in v:
            return True
    return False

FORCED_COLORS = re.compile(r"@media[^{]*forced-colors")

errors = []

for path in files:
    with open(path, encoding="utf-8") as fh:
        raw = fh.read()
    resets = []   # (token set) of every non-forced-colors rule-selector with an outline reset
    rings = []    # (line, selector, subject token set, rule declares reset)
    for r in css_rules(raw):
        selector, body = r.prelude, r.body
        if not selector:
            continue
        if not re.search(r"[.\[:#]", selector):
            continue
        if any(FORCED_COLORS.search(at) for at in r.at):
            continue
        line = r.brace_line
        decls = {k: " ".join(v.split()) for k, v in DECL.findall(body)}
        reset = is_reset(decls)
        shadow = decls.get("box-shadow")
        for sel in selector_list(selector):
            comp = last_compound(sel)
            toks, pseudos = tokens_of(comp)
            has_focus = bool(FOCUS_PSEUDO.search(strip_groups(unwrap_where(comp))))
            if reset and not (pseudos - {":focus", ":focus-visible"}):
                resets.append(toks)
            if has_focus and shadow is not None and shadow != "none" and not pseudos - {":focus", ":focus-visible"}:
                rings.append((line, " ".join(sel.split()), toks, reset))
    for line, sel, toks, own_reset in rings:
        if own_reset:
            continue
        if toks and any(r and r <= toks for r in resets):
            continue
        if ALLOW.find_all(lambda e: path.endswith(e.suffix) and e.needle in sel):
            continue
        errors.append(f"{path}:{line}: `{sel[:80]}` paints a focus ring (box-shadow) but no rule of this sheet "
                      f"declares `outline: none` for that element: the UA outline shows next to the ring")

if errors:
    for e in errors:
        print(e)
    print(f"\ncheck-focus-outline-reset: {len(errors)} problem(s). Add `outline: none` to the element's base rule "
          f"(the ring is the box-shadow token; see \"State recipes\" in preview/assets/dx-components-theme.css).",
          file=sys.stderr)
    sys.exit(1)

problems = ALLOW.problems(stale=not explicit)
if problems:
    for p in problems:
        print(f"FAIL: {p}")
    sys.exit(1)
n_used = sum(1 for e in ALLOW if ALLOW.used(e))
print(f"check-focus-outline-reset: OK ({len(files)} stylesheet(s): every focus box-shadow ring has an outline reset; "
      f"{n_used} allowlist entr{'y' if n_used == 1 else 'ies'} in use)")
PYEOF
