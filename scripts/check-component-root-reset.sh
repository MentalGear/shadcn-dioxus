#!/usr/bin/env bash
#
# check-component-root-reset.sh  (backlog row 328)
#
# An in-flow component must not inherit its host's typography, once and by construction: every namesake component root
# (`dx-<dir>`, read from `component.rs`; the four `<dialog>`-panel roots are left to the popup-root rule) carries
# `"data-dx-root": ""`, or a reasoned `typography-inherit(dx-<dir>): <reason>` comment for a wrapper around the
# caller's content, and the theme's one zero-specificity `:where([data-dx-root]:not([data-dx-root] *, button, input,
# select, textarea))` rule resets the inherited text properties (line-height, weight, style, spacing, transform,
# alignment, white-space, ...) to the theme base on the outermost root. Font family, colour, direction, cursor,
# pointer-events, visibility, user-select, form controls and `font-size` keep inheriting. The gate checks the rule is
# single, zero-specificity and complete against mdn-data's inherited properties; its self-test mutations must turn
# red.
#
#   * every component's namesake root (`dx-<dir>` in preview/src/components/<dir>/component.rs) carries the
#     valueless attribute `data-dx-root`, and
#   * the theme (preview/assets/dx-components-theme.css, "Component root") has ONE zero-specificity rule over
#     `:where([data-dx-root]:not([data-dx-root] *, button, input, select, textarea))` (the outermost root: a
#     component nested in another keeps its host component's declared typography; a form control keeps the UA
#     sheet's own `font`/`text-align`/..., which the reset would replace) that gives it the theme's base value of
#     every inherited property that styles TEXT.
#
# WHAT THIS GATE CHECKS
#   1. the theme has exactly one such rule, its whole selector is one `:where( ... )` (specificity 0,0,0) and names
#      `[data-dx-root]`;
#   2. the rule declares every property in RESET (a shorthand or alias that covers a longhand counts, COVERED_BY),
#      declares none of EXCLUDED (font-family, colour, direction, cursor, pointer-events, visibility, user-select, ...:
#      identity, interaction and document state inherit on purpose) and no custom property (slots keep composing);
#   3. when preview/node_modules/mdn-data is installed, every inherited property it lists is declared or named in
#      EXCLUDED with a reason, so a property the platform adds later cannot slip past unseen;
#   4. every component directory whose component.rs renders the namesake root (`dx-<dir with - for _>`) either
#      carries `"data-dx-root"` on that element (within MARKER_WINDOW lines of the class literal, same function) or
#      has a reasoned opt-out comment in its style.css or component.rs
#          typography-inherit(dx-tabs): wraps the caller's panels; shadcn's Tabs declares no typography either
#      (a root cannot have both; the reason needs MIN_REASON characters); a `data-dx-root` that sits on no root
#      literal is a stray marker and fails;
#   5. the four roots that ARE the `<dialog>` panel (POPUP_ROOTS) carry no marker: check-popup-root-reset.sh resets
#      them already;
#   6. its own self-test: the same evaluation runs on in-memory mutations and each must come out RED.
#
# FONT-SIZE is deliberately NOT reset (FONT_SIZE below; evidence in the theme's "Component root" section and row 328).
#
# Usage: scripts/check-component-root-reset.sh [--inventory]
# Exit status: 0 = clean; 1 = a violation / self-test not red; 2 = bad usage or parse failure.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

mode=check
for arg in "$@"; do
    case "$arg" in
        --inventory) mode=inventory ;;
        -h|--help) sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "check-component-root-reset: unknown option: $arg" >&2; exit 2 ;;
    esac
done

MODE="$mode" python3 - <<'PYEOF'
import copy
import json
import os
import re
import sys

sys.path.insert(0, "scripts/lib")
from css import rules as css_rules, split_decl  # noqa: E402

THEME = "preview/assets/dx-components-theme.css"
COMPONENTS = "preview/src/components"
MDN = "preview/node_modules/mdn-data/css/properties.json"
MODE = os.environ["MODE"]
MARKER_ATTR = '"data-dx-root"'
MARKER_WINDOW = 15
MIN_REASON = 12
SELECTOR = ":where([data-dx-root]:not([data-dx-root] *, button, input, select, textarea))"
OPT_OUT = re.compile(r"typography-inherit\(\s*(dx-[a-z0-9-]+)\s*\)\s*:\s*(.*)")

# Namesake roots that ARE the popup panel (the `<dialog>` the primitive renders carries the class): the popup-root
# rule resets them, so they carry no marker. Each needs its reason; a stale entry fails.
POPUP_ROOTS = {
    "dialog": "the class is on the `<dialog>` panel (data-slot dialog-content)",
    "alert_dialog": "the class is on the `<dialog>` panel the AlertDialogContent primitive renders",
    "sheet": "the class is on the `<dialog>` panel (data-slot sheet-content)",
    "drawer": "the class is on the `<dialog>` panel the DrawerContent primitive renders",
}

# Every inherited property that styles TEXT, which the rule must declare.
RESET = """
font-feature-settings font-kerning font-language-override font-optical-sizing font-palette font-size-adjust
font-stretch font-style font-synthesis font-synthesis-position font-variant font-variation-settings font-weight
hanging-punctuation hyphenate-character hyphenate-limit-chars hyphens letter-spacing line-break line-height
line-height-step orphans overflow-wrap quotes ruby-align ruby-merge ruby-overhang ruby-position tab-size text-align
text-align-last text-autospace text-decoration-skip text-decoration-skip-ink text-emphasis text-emphasis-position
text-indent text-justify text-shadow text-spacing-trim -webkit-text-stroke-color -webkit-text-stroke-width
text-transform text-underline-offset text-underline-position text-wrap-style white-space widows word-break
word-spacing word-wrap
""".split()

# FONT-SIZE: set by the measurement (row 328). "inherit" = deliberately not declared (listed in EXCLUDED), "reset" =
# declared as `var(--dx-text-base)` (then it moves from EXCLUDED to RESET).
FONT_SIZE = "inherit"

# A longhand mdn-data lists that the rule covers through a shorthand or alias it declares instead.
COVERED_BY = {
    "font-width": "font-stretch",
    "font-synthesis-weight": "font-synthesis", "font-synthesis-style": "font-synthesis",
    "font-synthesis-small-caps": "font-synthesis",
    "font-variant-alternates": "font-variant", "font-variant-caps": "font-variant",
    "font-variant-east-asian": "font-variant", "font-variant-emoji": "font-variant",
    "font-variant-ligatures": "font-variant", "font-variant-numeric": "font-variant",
    "font-variant-position": "font-variant",
    "text-emphasis-style": "text-emphasis", "text-emphasis-color": "text-emphasis",
    "white-space-collapse": "white-space", "text-wrap-mode": "white-space", "text-wrap": "white-space",
    "-webkit-text-stroke": "-webkit-text-stroke-width",
}

# Inherited per mdn-data (or a property that must never be in the rule), deliberately NOT reset, grouped by reason.
_EXCLUDE_GROUPS = [
    ("the app picks its font; every component inherits it on purpose", ["font-family", "font"]),
    ("the app's colours and the active mode", ["color", "-webkit-text-fill-color", "color-scheme", "accent-color",
        "caret", "caret-animation", "caret-color", "caret-shape", "scrollbar-color", "forced-color-adjust",
        "print-color-adjust", "color-interpolation-filters", "dynamic-range-limit"]),
    ("script and writing direction: an RTL or vertical page stays so (preview's RTL toggle)",
        ["direction", "writing-mode", "text-orientation", "text-combine-upright"]),
    ("interaction state: a disabled, hidden or read-only host stays so", ["cursor", "pointer-events", "visibility",
        "interactivity", "user-select", "-webkit-user-select", "-webkit-tap-highlight-color", "-webkit-touch-callout",
        "-webkit-user-modify", "-webkit-overflow-scrolling"]),
    ("document-level settings an app makes once on the root", ["text-size-adjust", "text-rendering",
        "interpolate-size"]),
    ("SVG paint and images, not text styling: icons set their own", ["fill", "fill-opacity", "fill-rule", "clip-rule",
        "stroke", "stroke-color", "stroke-dasharray", "stroke-dashoffset", "stroke-linecap", "stroke-linejoin",
        "stroke-miterlimit", "stroke-opacity", "stroke-width", "paint-order", "marker", "marker-start", "marker-mid",
        "marker-end", "text-anchor", "dominant-baseline", "image-orientation", "image-rendering", "image-resolution",
        "object-position"]),
    ("table and list structure: the components that use them declare them", ["border-collapse", "border-spacing",
        "caption-side", "empty-cells", "list-style", "list-style-image", "list-style-position", "list-style-type"]),
    ("MathML and aural styling: no component renders either", ["math-depth", "math-shift", "math-style", "speak-as"]),
    ("non-standard, experimental or legacy, unknown to stylelint and Chromium", ["font-smooth", "interest-delay",
        "interest-delay-start", "interest-delay-end", "-webkit-border-before", "-webkit-border-before-color",
        "-webkit-border-before-style", "-webkit-border-before-width"]),
]
EXCLUDED = {p: reason for reason, props in _EXCLUDE_GROUPS for p in props}
EXCLUDED_PREFIXES = ("-ms-", "-moz-")
if FONT_SIZE == "inherit":
    EXCLUDED["font-size"] = "see FONT_SIZE: measured, not reset (the root's own rule or the host decides)"
else:
    RESET.append("font-size")


# ----------------------------------------------------------------------------- CSS
def find_rules(css):
    """[(selector, declared property names)] for every rule whose selector names `[data-dx-root]`, wherever it sits
    (the shared reader scripts/lib/css.py: a rule inside an at-rule is found like a top-level one)."""
    out = []
    for r in css_rules(css):
        if "[data-dx-root]" in r.prelude:
            out.append((" ".join(r.prelude.split()), [split_decl(d)[0] for d in r.decls if ":" in d]))
    return out


def zero_specificity(selector):
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


def check_theme(css, mdn):
    errs = []
    rules = find_rules(css)
    if len(rules) != 1:
        return [f"expected exactly 1 component-root rule (a selector naming `[data-dx-root]`) in the theme, found {len(rules)}"]
    selector, names = rules[0]
    if not zero_specificity(selector):
        errs.append(f"the component-root selector `{selector}` is not a single zero-specificity `:where( ... )`")
    if selector != SELECTOR:
        errs.append(f"the component-root selector is `{selector}`, expected `{SELECTOR}` (outermost root only)")
    have = set(names)
    for p in RESET:
        if p not in have:
            errs.append(f"the component-root rule does not declare `{p}`")
    for p in sorted(have):
        if p in EXCLUDED:
            errs.append(f"the component-root rule declares `{p}` (it must not: {EXCLUDED[p]})")
        elif p.startswith("--"):
            errs.append(f"the component-root rule declares the custom property `{p}` (slots must keep composing)")
        elif p not in RESET:
            errs.append(f"the component-root rule declares `{p}`, which is in neither RESET nor EXCLUDED in this gate")
    if mdn is not None:
        for k, v in mdn.items():
            if v.get("inherited") is not True or k == "--*":
                continue
            if k in have or k in EXCLUDED or k.startswith(EXCLUDED_PREFIXES):
                continue
            if COVERED_BY.get(k) in have:
                continue
            errs.append(f"mdn-data lists `{k}` as inherited: declare it in the component-root rule or add it to EXCLUDED with a reason")
    return errs


# ----------------------------------------------------------------------------- components
def load():
    comps = {}
    for d in sorted(os.listdir(COMPONENTS)):
        css_p, rs_p = f"{COMPONENTS}/{d}/style.css", f"{COMPONENTS}/{d}/component.rs"
        if not os.path.isfile(rs_p):
            continue
        css = open(css_p, encoding="utf-8").read() if os.path.isfile(css_p) else ""
        comps[d] = {"css": css, "rs": open(rs_p, encoding="utf-8").read()}
    return comps


FN_START = re.compile(r"^(pub(\([a-z]+\))?\s+)?(async\s+)?fn\s")


def root_literals(d, rs):
    """Line indexes of non-comment, non-test string literals that carry the root class as a whole token."""
    root = "dx-" + d.replace("_", "-")
    hits = []
    for i, ln in enumerate(rs.split("\n")):
        if ln.lstrip().startswith("//") or "assert" in ln or "contains(" in ln:
            continue
        for m in re.finditer(r'"([^"]*)"', ln):
            if root in re.split(r"[\s{}]+", m.group(1)):
                hits.append(i)
                break
    return root, hits


def family_literals(d, rs):
    """Line indexes of string literals naming the namesake root or one of its parts (`dx-<dir>` or `dx-<dir>-...`)."""
    prefix = "dx-" + d.replace("_", "-")
    hits = []
    for i, ln in enumerate(rs.split("\n")):
        if ln.lstrip().startswith("//") or "assert" in ln or "contains(" in ln:
            continue
        for m in re.finditer(r'"([^"]*)"', ln):
            if any(tok == prefix or tok.startswith(prefix + "-") for tok in re.split(r"[\s{}]+", m.group(1))):
                hits.append(i)
                break
    return hits


def fn_spans(lines):
    starts = [i for i, ln in enumerate(lines) if FN_START.match(ln)]
    spans = []
    for k, s in enumerate(starts):
        spans.append((s, starts[k + 1] if k + 1 < len(starts) else len(lines)))
    return spans


def span_of(spans, i):
    for s, e in spans:
        if s <= i < e:
            return s, e
    return 0, 10 ** 9


def marked_roots(rs, hits, fam_hits):
    """(root literal lines with a marker in the same fn within the window, marker lines near no literal of the
    component's own classes). A marker on another top-level part of the directory (Field's FieldSet, Trace's
    collapsible: a part a caller can place without the namesake root) is a deliberate SECONDARY root, not stray."""
    lines = rs.split("\n")
    spans = fn_spans(lines)
    mark_lines = [i for i, ln in enumerate(lines) if MARKER_ATTR in ln and not ln.lstrip().startswith("//")]
    ok_lit, near = set(), set()
    for h in hits:
        s, e = span_of(spans, h)
        for m in mark_lines:
            if s <= m < e and abs(m - h) <= MARKER_WINDOW:
                ok_lit.add(h)
                near.add(m)
    stray = [m for m in mark_lines if not any(abs(m - h) <= MARKER_WINDOW and span_of(spans, h) == span_of(spans, m) for h in fam_hits)]
    return ok_lit, stray


def opt_outs(c):
    out = {}
    for text in (c["css"], c["rs"]):
        for ln in text.split("\n"):
            m = OPT_OUT.search(ln)
            if m:
                out[m.group(1)] = re.sub(r"\*/\s*$", "", m.group(2)).strip()
    return out


def check_components(comps):
    """-> (errors, status {dir: marked|opt-out|popup|noroot}, stats)."""
    errs, status = [], {}
    for d in POPUP_ROOTS:
        if d not in comps or not root_literals(d, comps[d]["rs"])[1]:
            errs.append(f"POPUP_ROOTS lists `{d}`, which has no namesake root in its component.rs (stale entry)")
    for d, c in comps.items():
        root, hits = root_literals(d, c["rs"])
        ok_lit, stray = marked_roots(c["rs"], hits, family_literals(d, c["rs"]))
        oo = opt_outs(c)
        for r, reason in oo.items():
            if r != root:
                errs.append(f"{d}: typography-inherit names `{r}`, which is not this component's root `{root}`")
            elif len(reason) < MIN_REASON:
                errs.append(f"{d}: typography-inherit({r}) needs a reason (>= {MIN_REASON} characters)")
        for m in stray:
            errs.append(f"{d}/component.rs:{m + 1}: `data-dx-root` sits within {MARKER_WINDOW} lines of no `{root}` / `{root}-...` class literal (a stray marker; it belongs on the namesake root or another top-level part of this component)")
        if not hits:
            status[d] = "noroot"
            continue
        if d in POPUP_ROOTS:
            status[d] = "popup"
            if ok_lit:
                errs.append(f"{d}: `{root}` is the popup panel ({POPUP_ROOTS[d]}); it carries no `data-dx-root`, check-popup-root-reset.sh resets it")
            continue
        has_opt = root in oo and len(oo[root]) >= MIN_REASON
        # every rendered root site needs the marker (a component with two roots, e.g. Calendar and RangeCalendar)
        unmarked = [h for h in hits if h not in ok_lit]
        if has_opt:
            status[d] = "opt-out"
            if ok_lit:
                errs.append(f"{d}: `{root}` carries `data-dx-root` AND a typography-inherit opt-out; delete one")
        elif unmarked:
            status[d] = "missing"
            for h in unmarked:
                errs.append(f"{d}/component.rs:{h + 1}: the `{root}` root has no `data-dx-root` (add `\"data-dx-root\": \"\"` to the element, or a `typography-inherit({root}): <reason>` comment in style.css for a wrapper around the caller's content)")
        else:
            status[d] = "marked"
    stats = {k: sum(1 for v in status.values() if v == k) for k in ("marked", "opt-out", "popup", "noroot")}
    return errs, status, stats


# ----------------------------------------------------------------------------- self-test
def drop_decl(css, prop):
    m = re.search(r"(:where\(\[data-dx-root\][^{}]*\{)([^{}]*)(\})", css)
    body = m.group(2)
    new = re.sub(r"(^|;|\n)\s*" + re.escape(prop) + r"\s*:[^;]*;", r"\1", body, count=1)
    assert new != body, f"self-test could not drop {prop}"
    return css[: m.start(2)] + new + css[m.end(2):]


def add_decl(css, decl):
    return re.sub(r"(:where\(\[data-dx-root\][^{}]*\{)", r"\1 " + decl, css, count=1)


def self_test(css, comps, mdn):
    results = []

    def red(label, errs):
        results.append((label, bool(errs)))

    red("a rule missing `line-height`", check_theme(drop_decl(css, "line-height"), mdn))
    red("a rule missing `font-weight`", check_theme(drop_decl(css, "font-weight"), mdn))
    red("a rule that also declares `color`", check_theme(add_decl(css, "color: red;"), mdn))
    red("a rule that also declares `font-family`", check_theme(add_decl(css, "font-family: serif;"), mdn))
    red("a rule that also declares `direction`", check_theme(add_decl(css, "direction: ltr;"), mdn))
    red("a rule that also declares a custom property", check_theme(add_decl(css, "--dx-leak: 1;"), mdn))
    red("a rule that also declares `user-select`", check_theme(add_decl(css, "user-select: none;"), mdn))
    red("a rule with a class in the selector (specificity > 0)", check_theme(css.replace(SELECTOR, ".x" + SELECTOR, 1), mdn))
    red("a rule that resets every root, not the outermost", check_theme(css.replace(SELECTOR, ":where([data-dx-root]:not(button, input, select, textarea))", 1), mdn))
    red("a rule that also resets form controls", check_theme(css.replace(SELECTOR, ":where([data-dx-root]:not([data-dx-root] *))", 1), mdn))

    marked = [d for d, v in check_components(comps)[1].items() if v == "marked"]
    if not marked:
        print("check-component-root-reset: self-test found no marked root to mutate", file=sys.stderr)
        sys.exit(1)
    d = marked[0]
    root = "dx-" + d.replace("_", "-")

    def with_rs(dirname, fn):
        m = copy.deepcopy(comps)
        m[dirname]["rs"] = fn(m[dirname]["rs"])
        return m

    red("a root whose marker was removed", check_components(with_rs(d, lambda s: s.replace(MARKER_ATTR + ": \"\"", "\"data-x\": \"\"")))[0])
    m_short = copy.deepcopy(comps)
    m_short[d]["css"] += f"\n/* typography-inherit({root}): ok */\n"
    red("an opt-out comment with no reason", check_components(m_short)[0])
    m_both = copy.deepcopy(comps)
    m_both[d]["css"] += f"\n/* typography-inherit({root}): wraps the caller's content, deliberate */\n"
    red("a root with both a marker and an opt-out", check_components(m_both)[0])
    m_foreign = copy.deepcopy(comps)
    m_foreign[d]["css"] += "\n/* typography-inherit(dx-not-this-root): deliberate, wraps caller content */\n"
    red("an opt-out naming a foreign root", check_components(m_foreign)[0])
    popup = next(iter(POPUP_ROOTS))
    proot = "dx-" + popup.replace("_", "-")

    def add_marker_to_popup(s):
        out, done = [], False
        for ln in s.split("\n"):
            out.append(ln)
            if not done and f'"{proot}"' in ln and not ln.lstrip().startswith("//") and "assert" not in ln:
                out.append(f"        {MARKER_ATTR}: \"\",")
                done = True
        return "\n".join(out)

    red("a marker on a popup panel root", check_components(with_rs(popup, add_marker_to_popup))[0])
    red("a stray marker on a non-root part", check_components(with_rs(d, lambda s: s + f"\n\nfn stray_part() {{\n    let _ = {MARKER_ATTR};\n}}\n"))[0])
    return results


# ----------------------------------------------------------------------------- main
try:
    css = open(THEME, encoding="utf-8").read()
    comps = load()
    mdn = json.load(open(MDN)) if os.path.exists(MDN) else None
    theme_errs = check_theme(css, mdn)
    comp_errs, status, stats = check_components(comps)
except Exception as e:  # a parse failure must not read as "clean"
    print(f"check-component-root-reset: internal error: {e!r}", file=sys.stderr)
    sys.exit(2)

if MODE == "inventory":
    for d, v in sorted(status.items()):
        print(f"{d}\tdx-{d.replace('_', '-')}\t{v}")
    print(f"# {stats}", file=sys.stderr)
    sys.exit(0)

errors = theme_errs + comp_errs
if errors:
    print("check-component-root-reset: FAILED")
    for e in errors:
        print(f"  {e}")
    sys.exit(1)

results = self_test(css, comps, mdn)
bad = [label for label, was_red in results if not was_red]
if bad:
    print("check-component-root-reset: FAILED -- self-test: these mutations did NOT turn the gate red:", file=sys.stderr)
    for label in bad:
        print(f"  {label}", file=sys.stderr)
    sys.exit(1)

note = "mdn-data cross-check ran" if mdn is not None else "mdn-data not installed: cross-check skipped"
print(
    f"check-component-root-reset: OK -- one zero-specificity rule declaring {len(RESET)} text properties "
    f"(font-size: {FONT_SIZE}); {stats['marked']} namesake roots marked, {stats['opt-out']} reasoned opt-outs, "
    f"{stats['popup']} popup panels left to the popup-root rule, {stats['noroot']} components without a namesake root; "
    f"self-test red on {len(results)}/{len(results)} mutations ({note})"
)
PYEOF
