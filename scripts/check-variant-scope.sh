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
# root carries no `data-size="sm"`, but its ancestor does. The docs page's DEMO/CODE `line` Tabs restyled
# every Tabs demo inside it this way (fixed for Tabs by `lane/p5tabs`); check-popup-boundary.sh does not
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
# THE ALLOWLIST below is for sites where the component cannot be nested inside itself, with the reason. An
# entry is `(component dir, regex searched in the selector, reason)`; it must match at least one flagged
# selector (a stale entry fails), and a reason is mandatory. Prefer the construction to an entry.
#
# The browser half is `playwright/variant-nesting.spec.ts`: on every page of a component whose sheet styles by a
# variant attribute, each other-variant instance is cloned into a neutral host inside and beside each variant
# instance, and every computed property must agree.
#
# Usage: scripts/check-variant-scope.sh [--list]      (--list prints every flagged site, allowlisted or not)
# Exit status: 0 = every variant rule is scoped or allowlisted; 1 = a new one; 2 = bad usage.

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

COMPONENTS = "preview/src/components"
LIST = os.environ.get("LIST") == "1"
VARIANT = re.compile(r'\[data-(?:variant|size|orientation|spacing|align)\b')

# (component dir, regex searched in the selector, reason). See the header: prefer the construction.
ALLOW = [
    ("item", r'^\.dx-item-media\[data-variant="(icon|image)"\] (svg|img)$',
     "the svg/img is the user's leaf icon or image inside ItemMedia; an ItemMedia does not hold another ItemMedia, "
     "and a slot default on the svg/img would override the element's own width/height attributes"),
]


def strip_comments(css):
    return re.sub(r"/\*.*?\*/", lambda m: "\n" * m.group(0).count("\n"), css, flags=re.S)


def style_rules(css):
    """[(prelude, line)] for every style rule, at-rule bodies descended into."""
    css = strip_comments(css)
    out, stack, buf, quote, paren = [], [], [], None, 0
    for i, ch in enumerate(css):
        if quote:
            buf.append(ch)
            if ch == quote:
                quote = None
        elif ch in "\"'":
            quote = ch
            buf.append(ch)
        elif ch == "(":
            paren += 1
            buf.append(ch)
        elif ch == ")":
            paren = max(0, paren - 1)
            buf.append(ch)
        elif paren:
            buf.append(ch)
        elif ch == "{":
            prelude = "".join(buf)
            start = i - len(prelude) + (len(prelude) - len(prelude.lstrip()))
            prelude = prelude.strip()
            is_at = prelude.startswith("@")
            stack.append(is_at)
            if not is_at:
                out.append((prelude, css.count("\n", 0, start) + 1))
            buf = []
        elif ch == ";":
            buf = []
        elif ch == "}":
            if stack:
                stack.pop()
            buf = []
        else:
            buf.append(ch)
    return out


def split_top(s, sep):
    out, depth, cur, quote = [], 0, [], None
    for ch in s:
        if quote:
            cur.append(ch)
            if ch == quote:
                quote = None
            continue
        if ch in "\"'":
            quote = ch
        elif ch in "([":
            depth += 1
        elif ch in ")]":
            depth -= 1
        if ch == sep and depth == 0:
            out.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    out.append("".join(cur))
    return out


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


sites = []
for comp in sorted(os.listdir(COMPONENTS)):
    path = os.path.join(COMPONENTS, comp, "style.css")
    if not os.path.isfile(path):
        continue
    with open(path) as f:
        css = f.read()
    for prelude, line in style_rules(css):
        for sel in split_top(prelude, ","):
            sel = " ".join(sel.split())
            kind = classify(sel)
            if kind:
                sites.append((comp, path, line, kind, sel))

used = [0] * len(ALLOW)
bad = []
for comp, path, line, kind, sel in sites:
    hit = None
    for n, (acomp, rx, reason) in enumerate(ALLOW):
        if acomp == comp and re.search(rx, sel):
            if not reason.strip():
                print(f"check-variant-scope: allowlist entry {acomp!r} {rx!r} has no reason", file=sys.stderr)
                sys.exit(2)
            used[n] += 1
            hit = n
            break
    if LIST:
        print(f"{path}:{line}: {kind}: {sel}" + ("  [allowlisted]" if hit is not None else ""))
    if hit is None:
        bad.append((path, line, kind, sel))

stale = [ALLOW[n] for n, u in enumerate(used) if not u]
for acomp, rx, _ in stale:
    print(f"check-variant-scope: stale allowlist entry ({acomp}, {rx!r}) matches no flagged selector", file=sys.stderr)
for path, line, kind, sel in bad:
    print(f"{path}:{line}: {kind}: {sel}", file=sys.stderr)
if bad:
    print(f"check-variant-scope: {len(bad)} variant rule(s) reach descendants of the variant root; "
          "re-declare a slot variable on the root instead (see the header)", file=sys.stderr)
if bad or stale:
    sys.exit(1)
print(f"check-variant-scope: OK ({len(sites)} flagged site(s), all allowlisted)")
PYEOF
