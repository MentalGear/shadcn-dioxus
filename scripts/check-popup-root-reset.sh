#!/usr/bin/env bash
#
# check-popup-root-reset.sh  (backlog row 259, the typography-reset half of row 218)
#
# A popover or dialog is painted in the top layer but stays a DOM descendant of its trigger, so every INHERITED
# CSS property still flows from the host (a 14px card, a muted or monospace block, a button that is not
# selectable text) into the panel. Descendant selectors are closed by check-popup-boundary.sh; this gate is the
# other half, by construction: the theme (preview/assets/dx-components-theme.css, "Popup root") carries ONE
# zero-specificity rule over every `dx-` popup root,
#
#   :where([popover][class^="dx-"], [popover][class*=" dx-"], dialog[class^="dx-"], dialog[class*=" dx-"]) { ... }
#
# that gives the root the theme's ROOT value of every inherited property, so a popup starts from the same
# typography wherever it is rendered. This gate keeps that rule honest:
#
#   1. exactly one such rule exists in the theme, and its whole selector is one `:where()` (specificity 0,0,0),
#      covering `[popover]` and `dialog`, both the `class^="dx-"` and the `class*=" dx-"` form;
#   2. it declares every property of INHERITED below (a shorthand that covers a longhand counts, see COVERED_BY);
#   3. it never declares `direction` (an RTL page's popups are RTL), `color-scheme` (the active mode) or a
#      custom property (`--dx-*` slots must keep composing);
#   4. when preview/node_modules/mdn-data is installed, every inherited property it lists is either declared or
#      named in EXCLUDED with a reason, so a property added to the platform later cannot slip past unseen;
#   5. its own self-test: the same checks run on in-memory mutations of the rule (one property removed,
#      `direction` added, a custom property added, the specificity raised) and each must come out RED.
#
# Source of the list: the inherited longhands of mdn-data 2.27.1 (css/properties.json, `inherited: true`, which
# tracks the CSS specifications; checked against Chromium 141, where every standardised entry is either supported
# or dropped harmlessly). Not in the list on purpose, with reasons: EXCLUDED.
#
# Usage: scripts/check-popup-root-reset.sh [THEME.css]
# Exit status: 0 = the rule is complete; 1 = it is not (or the self-test is not red where it must be); 2 = bad usage.

set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import json
import os
import re
import sys

THEME = sys.argv[1] if len(sys.argv) > 1 else "preview/assets/dx-components-theme.css"
MDN = "preview/node_modules/mdn-data/css/properties.json"

# Every inherited property the popup root must set (longhands, plus the shorthands the rule writes).
INHERITED = """
border-collapse border-spacing caption-side caret-animation caret-color caret-shape clip-rule color
color-interpolation-filters cursor dominant-baseline dynamic-range-limit empty-cells fill fill-opacity fill-rule
font font-feature-settings font-kerning font-language-override font-optical-sizing font-palette font-size-adjust
font-synthesis font-synthesis-position font-variation-settings font-width forced-color-adjust hanging-punctuation
hyphenate-character hyphenate-limit-chars hyphens image-orientation image-rendering image-resolution interactivity
interpolate-size letter-spacing line-break line-height-step list-style marker-end marker-mid marker-start
math-depth math-shift math-style orphans overflow-wrap paint-order pointer-events print-color-adjust quotes
ruby-align ruby-merge ruby-overhang ruby-position scrollbar-color speak-as stroke stroke-dasharray
stroke-dashoffset stroke-linecap stroke-linejoin stroke-miterlimit stroke-opacity stroke-width tab-size text-align
text-align-last text-anchor text-autospace text-combine-upright text-decoration-skip text-decoration-skip-ink
text-emphasis text-emphasis-position text-indent text-justify text-orientation text-shadow text-spacing-trim
text-transform text-underline-offset text-underline-position text-wrap-style user-select visibility white-space
widows word-break word-spacing word-wrap writing-mode -webkit-text-fill-color -webkit-text-stroke-color
-webkit-text-stroke-width -webkit-user-select
""".split()

# A longhand an mdn-data entry lists that the rule covers through a shorthand it declares instead.
COVERED_BY = {
    "font-family": "font", "font-size": "font", "font-style": "font", "font-weight": "font",
    "font-stretch": "font", "line-height": "font", "font-variant": "font",
    "font-variant-alternates": "font", "font-variant-caps": "font", "font-variant-east-asian": "font",
    "font-variant-emoji": "font", "font-variant-ligatures": "font", "font-variant-numeric": "font",
    "font-variant-position": "font",
    "font-synthesis-weight": "font-synthesis", "font-synthesis-style": "font-synthesis",
    "font-synthesis-small-caps": "font-synthesis",
    "list-style-type": "list-style", "list-style-position": "list-style", "list-style-image": "list-style",
    "text-emphasis-style": "text-emphasis", "text-emphasis-color": "text-emphasis",
    "white-space-collapse": "white-space", "text-wrap-mode": "white-space", "text-wrap": "white-space",
    "marker": "marker-start", "caret": "caret-color", "-webkit-text-stroke": "-webkit-text-stroke-width",
}

# Inherited per mdn-data, deliberately NOT reset by the popup root.
EXCLUDED = {
    "direction": "RTL must reach the panel; direction is inherited on purpose (preview's RTL toggle)",
    "color-scheme": "the active light/dark mode (theme-mode-writer); resetting it would lighten a dark page's popups",
    "text-size-adjust": "a document-level setting an app makes once on the root; no component host changes it",
    "text-rendering": "a document-level setting an app makes once on the root; no component host changes it",
    "accent-color": "a document-level setting an app makes once on the root; no component host changes it",
    "-webkit-tap-highlight-color": "a document-level touch setting made once on the root",
    "object-position": "mdn-data lists it as inherited; the CSS Images spec says it is not",
    "font-smooth": "non-standard, unknown to stylelint and Chromium",
    "interest-delay": "experimental, unknown to stylelint",
    "interest-delay-start": "experimental, unknown to stylelint",
    "interest-delay-end": "experimental, unknown to stylelint",
    "stroke-color": "SVG2 draft, shipped nowhere",
    "-webkit-overflow-scrolling": "legacy iOS, unsupported",
    "-webkit-touch-callout": "legacy iOS, unsupported",
    "-webkit-user-modify": "legacy, unsupported",
    "-webkit-border-before": "legacy logical alias; mdn-data marks it inherited, the property is not",
    "-webkit-border-before-color": "legacy logical alias; not inherited in practice",
    "-webkit-border-before-style": "legacy logical alias; not inherited in practice",
    "-webkit-border-before-width": "legacy logical alias; not inherited in practice",
}
# Vendor families that are never reset (legacy engines).
EXCLUDED_PREFIXES = ("-ms-", "-moz-")

COMMENT = re.compile(r"/\*.*?\*/", re.S)


def strip_comments(text):
    return COMMENT.sub("", text)


def find_rule(css):
    """Return (selector, body) for every rule whose selector names the popup-root `[popover][class^=..` form."""
    css = strip_comments(css)
    out = []
    for m in re.finditer(r"([^{}]*\[popover\]\[class\^=\"dx-\"\][^{}]*)\{([^{}]*)\}", css):
        out.append((" ".join(m.group(1).split()), m.group(2)))
    return out


def declarations(body):
    props = []
    depth = 0
    cur = ""
    for ch in body:
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
        if ch == ";" and depth == 0:
            props.append(cur)
            cur = ""
        else:
            cur += ch
    props.append(cur)
    names = []
    for d in props:
        d = d.strip()
        if ":" in d:
            names.append(d.split(":", 1)[0].strip().lower())
    return names


def zero_specificity(selector):
    """The whole selector is one `:where( ... )`."""
    s = selector.strip()
    if not s.startswith(":where("):
        return False
    depth = 0
    for i, ch in enumerate(s):
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                return i == len(s) - 1
    return False


def check(css, with_mdn=True):
    errs = []
    rules = find_rule(css)
    if len(rules) != 1:
        return [f"expected exactly 1 popup-root rule in the theme, found {len(rules)}"]
    selector, body = rules[0]
    if not zero_specificity(selector):
        errs.append("the popup-root selector is not a single zero-specificity `:where( ... )`")
    for needle in ('[popover][class^="dx-"]', '[popover][class*=" dx-"]', 'dialog[class^="dx-"]', 'dialog[class*=" dx-"]'):
        if needle not in selector:
            errs.append(f"the popup-root selector lacks {needle}")
    names = declarations(body)
    have = set(names)
    for p in INHERITED:
        if p not in have:
            errs.append(f"the popup-root rule does not declare `{p}`")
    for p in ("direction", "color-scheme"):
        if p in have:
            errs.append(f"the popup-root rule declares `{p}` (it must not: {EXCLUDED[p]})")
    for n in names:
        if n.startswith("--"):
            errs.append(f"the popup-root rule declares the custom property `{n}` (slots must keep composing)")
    if with_mdn and os.path.exists(MDN):
        with open(MDN) as f:
            mdn = json.load(f)
        for k, v in mdn.items():
            if v.get("inherited") is not True or k == "--*":
                continue
            if k in have or k in EXCLUDED or k.startswith(EXCLUDED_PREFIXES):
                continue
            if COVERED_BY.get(k) in have:
                continue
            errs.append(f"mdn-data lists `{k}` as inherited: declare it in the popup-root rule or add it to EXCLUDED with a reason")
    return errs


def drop_property(css, prop):
    """Remove one declaration of `prop` from the popup-root rule body only."""
    m = re.search(r"(\[popover\]\[class\^=\"dx-\"\][^{}]*\{)([^{}]*)(\})", css)
    body = m.group(2)
    new_body = re.sub(r"(^|;|\n)\s*" + re.escape(prop) + r"\s*:[^;]*;", r"\1", body, count=1)
    assert new_body != body, f"self-test could not drop {prop}"
    return css[: m.start(2)] + new_body + css[m.end(2):]


def self_test(css):
    errs = []
    mutations = {
        "a rule missing `cursor`": drop_property(css, "cursor"),
        "a rule missing the `font` shorthand": drop_property(css, "font"),
        "a rule that also declares `direction`": re.sub(r"(\[popover\]\[class\^=\"dx-\"\][^{}]*\{)", r"\1 direction: ltr;", css, count=1),
        "a rule that also declares a custom property": re.sub(r"(\[popover\]\[class\^=\"dx-\"\][^{}]*\{)", r"\1 --dx-leak: 1;", css, count=1),
        "a rule with a class in the selector (specificity > 0)": css.replace(":where(\n  [popover][class^=\"dx-\"]", ".x:where(\n  [popover][class^=\"dx-\"]", 1),
    }
    for label, mutated in mutations.items():
        if mutated == css:
            errs.append(f"self-test: could not build the mutation `{label}`")
        elif not check(mutated):
            errs.append(f"self-test: `{label}` did NOT fail the gate")
    return errs


try:
    with open(THEME) as f:
        css = f.read()
except OSError as e:
    print(f"check-popup-root-reset: cannot read {THEME}: {e}", file=sys.stderr)
    sys.exit(2)

errors = check(css)
if not errors:
    errors = self_test(css)
if errors:
    print("check-popup-root-reset: FAIL")
    for e in errors:
        print("  " + e)
    sys.exit(1)

mdn_note = "mdn-data cross-check ran" if os.path.exists(MDN) else "mdn-data not installed: cross-check skipped"
print(f"check-popup-root-reset: OK -- {len(INHERITED)} inherited properties declared by the one zero-specificity popup-root rule, no direction, no custom property; self-test red on 5/5 mutations ({mdn_note})")
PYEOF
