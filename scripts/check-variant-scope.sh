#!/usr/bin/env bash
#
# check-variant-scope.sh
#
# A component's VARIANT rule must style the component's own parts through slot variables its root
# re-declares, never by a descendant selector from the variant attribute (backlog row 218, item 1).
#
# The failure. A variant attribute (`data-variant`, `data-size`, `data-orientation`, `data-spacing`, `data-align`) sits on
# a component's root, and the rule that restyles a part reads
#
#   .dx-card[data-size="sm"] .dx-card-title { font-size: var(--dx-text-sm); }
#
# That also matches the title of every default-size Card nested anywhere inside the small one: the inner
# root carries no `data-size="sm"`, but its ancestor does. The docs page's DEMO/CODE `line` Tabs would restyle
# every Tabs demo inside it this way (Tabs re-declares its slot variable per variant, row 218);
# check-popup-boundary.sh does not
# see it, because its subjects are `dx-` classes. The same reach comes from a `:has()` that finds the
# variant through a descendant combinator (`.dx-message:has([data-variant="ghost"]) ...` matches an outer
# message whose NESTED message holds a ghost bubble) and from a bare `[data-variant="x"]` compound, which
# is any element of any component.
#
# THE CONSTRUCTION (the one Tabs uses): the root declares a slot variable for EVERY variant, the default
# included, the variant rule re-declares it on the root, and the part reads it --
#
#   .dx-card                  { --dx-card-title-font-size: var(--dx-text-base); }
#   .dx-card[data-size="sm"]  { --dx-card-title-font-size: var(--dx-text-sm); }
#   .dx-card-title            { font-size: var(--dx-card-title-font-size); }
#
# -- so a nested root re-declares its own value and never inherits the outer one. A direct-child rule
# (`.dx-avatar[data-size="sm"] > .dx-avatar-badge`) is fine as it is: a nested instance's parts are never
# direct children of the outer root.
#
# WHAT IT FLAGS, in every `preview/src/components/*/style.css` (comma lists split, comments ignored):
#   descendant      a top-level compound carries a variant attribute and a descendant (whitespace)
#                   combinator follows it anywhere to its right.
#   has-descendant  a `:has(...)` whose argument reaches a variant attribute through a descendant
#                   combinator (`:has(.x[data-size])`, not `:has(> .x[data-size])`), followed by any
#                   combinator.
#   bare            a compound whose only subject-identifying part is the variant attribute
#                   (`[data-variant="inset"] .dx-sidebar-gap`): it names no component.
# A variant attribute inside `:not(...)` counts too (`.dx-x:not([data-size="sm"]) .dx-x-part` reaches a
# nested small instance's part).
#
# WHAT IT DOES NOT FLAG: variant attributes inside `:is()`/`:where()` lists, sibling combinators (`+`, `~`),
# and variant attributes the component sets on a part rather than its root (`.dx-item-media[data-variant]`
# is flagged the same way: the part is a root of its own variant).
#
# THE SLOT HALF (undeclared-slot). The construction only holds when the BASE root declares every slot a
# variant root sets. A slot only a variant rule declares, read as `var(--slot, <default>)`, leaks the same way:
#
#   .dx-toolbar[data-orientation="vertical"] { --dx-toolbar-group-direction: column; }
#   .dx-toolbar-group { flex-direction: var(--dx-toolbar-group-direction, row); }
#
# A horizontal Toolbar nested in a vertical one declares nothing, so its groups inherit `column`. So for
# every rule with a selector alternative that is ONE compound carrying a variant attribute and one `dx-`
# class (the root; `:where()`/`:is()` around the whole selector opened, at-rules descended into), each
# custom property it declares must also be declared by a rule whose alternative is exactly that class. A
# slot that every variant value re-declares is still flagged: the attribute can be absent, and the base
# declaration is what makes the default explicit. SLOT_ALLOW is `(component dir, slot, reason)`, the same
# rules as the allowlist below (reason mandatory, a stale entry fails). The self-test runs mutations of the
# real sheets in memory (drop a base declaration, add an undeclared variant slot, the same inside
# `:where()` and `@media`, drop an allowlisted site) and fails unless each one comes out red, and unless a
# control with the base declaration comes out green.
#
# THE ALLOWLIST below is for sites where the component cannot be nested inside itself, with the reason. An
# entry is `(component dir, regex searched in the selector, reason)`; it must match at least one flagged
# selector (a stale entry fails), and a reason is mandatory. Prefer the construction to an entry.
#
# The browser half is `playwright/variant-nesting.spec.ts`: on every page of a component whose sheet styles by a
# variant attribute, each other-variant instance is cloned into a neutral host inside and beside each variant
# instance, and every computed property must agree.
#
# Usage: scripts/check-variant-scope.sh [--list]      (--list prints every flagged site, allowlisted or not)
# Exit status: 0 = every variant rule is scoped and every variant slot declared on its base root, or
# allowlisted; 1 = a new one, a stale entry or a self-test mutation that stayed green; 2 = bad usage.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

list=0
for arg in "$@"; do
    case "$arg" in
        --list) list=1 ;;
        -h|--help) sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "check-variant-scope: unknown option: $arg" >&2; exit 2 ;;
    esac
done

LIST="$list" python3 - <<'PYEOF'
import os
import re
import sys

sys.path.insert(0, "scripts/lib")
from allow import Allowlist  # noqa: E402
from css import rules as css_rules, split_top  # noqa: E402

COMPONENTS = "preview/src/components"
LIST = os.environ.get("LIST") == "1"
VARIANT = re.compile(r'\[data-(?:variant|size|orientation|spacing|align)\b')

# (component dir, regex searched in the selector, reason). See the header: prefer the construction.
ALLOW_ROWS = [
    ("item", r'^\.dx-item-media\[data-variant="(icon|image)"\] (svg|img)$',
     "the svg/img is the user's leaf icon or image inside ItemMedia; an ItemMedia does not hold another ItemMedia, "
     "and a slot default on the svg/img would override the element's own width/height attributes"),
]


def style_rules(css):
    """[(prelude, line, body)] for every style rule, at-rule bodies descended into (the shared reader, row 429)."""
    return sorted(((r.prelude, r.line, r.body) for r in css_rules(css)), key=lambda r: r[1])


def tokens(sel):
    """[compound, combinator, compound, ...] at depth 0; combinator is ' ', '>', '+' or '~'."""
    toks, cur, depth, quote, pending = [], [], 0, None, None
    for ch in sel.strip():
        if quote:
            cur.append(ch)
            if ch == quote:
                quote = None
            continue
        if ch in "\"'":
            quote = ch
            cur.append(ch)
            continue
        if ch in "([":
            depth += 1
        elif ch in ")]":
            depth -= 1
        if depth == 0 and ch in " \t\n>+~":
            if cur:
                toks.append("".join(cur))
                cur = []
                pending = " "
            if ch != " " and ch not in "\t\n":
                pending = ch
            continue
        if pending is not None and toks and len(toks) % 2 == 1:
            toks.append(pending)
        pending = None
        cur.append(ch)
    if cur:
        toks.append("".join(cur))
    return toks


def pseudo_args(compound, name):
    out, i = [], 0
    while True:
        j = compound.find(":" + name + "(", i)
        if j < 0:
            return out
        k = j + len(name) + 2
        depth, m = 1, k
        while m < len(compound) and depth:
            depth += {"(": 1, ")": -1}.get(compound[m], 0)
            m += 1
        out.append(compound[k:m - 1])
        i = m


def without_pseudo(compound, names):
    for name in names:
        for arg in pseudo_args(compound, name):
            compound = compound.replace(":" + name + "(" + arg + ")", "")
    return compound


def has_reaches_by_descendant(compound):
    for arg in pseudo_args(compound, "has"):
        for alt in split_top(arg, ","):
            t = tokens(alt.strip())
            for idx in range(0, len(t), 2):
                if VARIANT.search(t[idx]):
                    if idx == 0 or " " in t[1:idx:2]:
                        if not alt.strip().startswith((">", "+", "~")):
                            return True
    return False


def classify(sel):
    t = tokens(sel)
    comps, combs = t[0::2], t[1::2]
    for i, comp in enumerate(comps[:-1]):
        own = without_pseudo(comp, ("has", "is", "where"))
        if VARIANT.search(own):
            if " " in combs[i:]:
                if not re.search(r"[.#]|^[a-z*]", own):
                    return "bare"
                return "descendant"
        if has_reaches_by_descendant(comp):
            return "has-descendant"
    return None


ROOT_CLASS = re.compile(r"\.dx-[A-Za-z0-9_-]+")
SLOT_DECL = re.compile(r"(?:^|[;{\s])(--[A-Za-z0-9_-]+)\s*:")

# (component dir, slot variable, reason): a variant-set slot the base root may leave undeclared. It
# must match a reported slot (a stale entry fails) and carry a reason. Prefer declaring the default on
# the base root.
SLOT_ALLOW = [
    ("sidebar", "--dx-button-radius",
     "the trigger is also a `.dx-button`, whose base rule declares the slot; `[data-size]` only lifts the "
     "rule over Button's per-size radius, and a button holds no nested button to leak into"),
]


def unwrap(sel):
    """The alternatives of a selector, a `:where(...)`/`:is(...)` around the whole selector opened."""
    sel = " ".join(sel.split())
    for name in ("where", "is"):
        if sel.startswith(":" + name + "(") and sel.endswith(")"):
            args = pseudo_args(sel, name)
            if args and len(args[0]) == len(sel) - len(name) - 3:
                return [a for alt in split_top(args[0], ",") for a in unwrap(alt)]
    return [sel]


def variant_root(compound):
    """The one root class a compound with a variant attribute sets slots on, or None."""
    own = without_pseudo(compound, ("has", "is", "where", "not"))
    if not VARIANT.search(own):
        return None
    roots = ROOT_CLASS.findall(own)
    return roots[0] if len(roots) == 1 else None


def undeclared_slots(comp, css):
    """[(line, root, var, selector)]: a slot variable a variant root sets that the base root never declares.

    A rule is a VARIANT ROOT rule when a selector alternative is one compound carrying a variant attribute
    and one `dx-` class (the root); it is a BASE rule when the alternative is exactly that class. Both are
    collected across the whole sheet, inside at-rules too."""
    base, variant_sets, out = {}, [], []
    for prelude, line, body in style_rules(css):
        decls = {m.group(1) for m in SLOT_DECL.finditer(body)}
        if not decls:
            continue
        for sel in split_top(prelude, ","):
            for alt in unwrap(sel):
                t = tokens(alt)
                if len(t) != 1:
                    continue
                if ROOT_CLASS.fullmatch(t[0]):
                    base.setdefault(t[0], set()).update(decls)
                    continue
                root = variant_root(t[0])
                if root:
                    variant_sets.append((line, root, decls, alt))
    for line, root, decls, alt in variant_sets:
        for var in sorted(decls - base.get(root, set())):
            out.append((line, root, var, alt))
    return out


def slot_report(sheets):
    """(sites, unallowed, stale entries) for {component dir: css}."""
    sites = [(comp, line, root, var, sel)
             for comp in sorted(sheets) for line, root, var, sel in undeclared_slots(comp, sheets[comp])]
    allow = Allowlist("check-variant-scope", SLOT_ALLOW, fields=("comp", "slot"))  # fresh `used` per report
    unallowed = []
    for site in sites:
        if not allow.find_all(lambda e: (e.comp, e.slot) == (site[0], site[3])):
            unallowed.append(site)
    return sites, unallowed, allow.unused()


def slot_self_test(sheets):
    """Mutations of the real sheets that must each come out red."""
    def red(label, mutated):
        if mutated is None:
            return label + " (its anchor text is missing from the sheet)", False
        _, unallowed, stale = slot_report(mutated)
        return label, bool(unallowed or stale)

    def edit(comp, old, new):
        if old not in sheets.get(comp, ""):
            return None
        return {**sheets, comp: sheets[comp].replace(old, new, 1)}

    fake = ".dx-fake { color: red; }\n.dx-fake[data-size=\"sm\"] { --dx-fake-gap: 1px; }\n"
    results = [
        red("toolbar's base root drops the group direction",
            edit("toolbar", "--dx-toolbar-group-direction: row;", "")),
        red("carousel's base root drops a Previous placement slot",
            edit("carousel", "--dx-carousel-previous-inset-inline-start: 0;", "")),
        red("a new sheet sets a size slot its base root lacks", {**sheets, "fake": fake}),
        red("the same inside :where() and @media",
            {**sheets, "fake": "@media (width > 1px) { :where(.dx-fake[data-variant=\"x\"]) { --dx-fake-gap: 1px; } }"}),
        red("the sidebar trigger rule is gone (its allowlist entry goes stale)",
            edit("sidebar", "--dx-button-radius: var(--dx-radius-lg);", "")),
    ]
    control = {"fake": fake.replace("color: red;", "--dx-fake-gap: 2px;")}
    control_green = not undeclared_slots("fake", control["fake"])
    return results, control_green


sites = []
sheets = {}
for comp in sorted(os.listdir(COMPONENTS)):
    path = os.path.join(COMPONENTS, comp, "style.css")
    if not os.path.isfile(path):
        continue
    with open(path) as f:
        css = f.read()
    sheets[comp] = css
    for prelude, line, _body in style_rules(css):
        for sel in split_top(prelude, ","):
            sel = " ".join(sel.split())
            kind = classify(sel)
            if kind:
                sites.append((comp, path, line, kind, sel))

ALLOW = Allowlist("check-variant-scope", ALLOW_ROWS, fields=("comp", "rx"))
bad = []
for comp, path, line, kind, sel in sites:
    hit = ALLOW.find(lambda e: e.comp == comp and re.search(e.rx, sel))
    if LIST:
        print(f"{path}:{line}: {kind}: {sel}" + ("  [allowlisted]" if hit is not None else ""))
    if hit is None:
        bad.append((path, line, kind, sel))

slot_sites, slot_bad, slot_stale = slot_report(sheets)
if LIST:
    allowed_sites = set(slot_sites) - set(slot_bad)
    for comp, line, root, var, sel in slot_sites:
        print(f"{COMPONENTS}/{comp}/style.css:{line}: undeclared-slot: {sel} sets {var}"
              + ("  [allowlisted]" if (comp, line, root, var, sel) in allowed_sites else ""))
bad_tables = ALLOW.problems(stale=False) + Allowlist("check-variant-scope", SLOT_ALLOW, fields=("comp", "slot")).problems(stale=False)
if bad_tables:
    for problem in bad_tables:
        print(f"check-variant-scope: {problem}", file=sys.stderr)
    sys.exit(2)
results, control_green = slot_self_test(sheets)
not_red = [label for label, is_red in results if not is_red]

stale = ALLOW.unused()
for acomp, rx, _ in stale:
    print(f"check-variant-scope: stale allowlist entry ({acomp}, {rx!r}) matches no flagged selector", file=sys.stderr)
for acomp, avar, _ in slot_stale:
    print(f"check-variant-scope: stale slot allowlist entry ({acomp}, {avar}) matches no undeclared slot",
          file=sys.stderr)
for path, line, kind, sel in bad:
    print(f"{path}:{line}: {kind}: {sel}", file=sys.stderr)
if bad:
    print(f"check-variant-scope: {len(bad)} variant rule(s) reach descendants of the variant root; "
          "re-declare a slot variable on the root instead (see the header)", file=sys.stderr)
for comp, line, root, var, sel in slot_bad:
    print(f"{COMPONENTS}/{comp}/style.css:{line}: undeclared-slot: `{sel}` sets {var}, but no base `{root}` "
          "rule declares it", file=sys.stderr)
for label in not_red:
    print(f"check-variant-scope: self-test mutation did NOT turn the gate red: {label}", file=sys.stderr)
if not control_green:
    print("check-variant-scope: self-test control (the base root declares the slot) is red", file=sys.stderr)
if slot_bad:
    print(f"check-variant-scope: {len(slot_bad)} variant-set slot variable(s) not declared on the base root; a "
          "nested default instance inherits the outer variant's value. Declare the default on the base root "
          "(see the header)", file=sys.stderr)
if bad or stale or slot_bad or slot_stale or not_red or not control_green:
    sys.exit(1)
print(f"check-variant-scope: OK ({len(sites)} flagged site(s), all allowlisted; "
      f"{len(slot_sites)} undeclared variant slot(s), all allowlisted; self-test red on "
      f"{len(results)}/{len(results)} mutations)")
PYEOF
