#!/usr/bin/env bash
#
# check-popup-boundary.sh
#
# A component stylesheet's DESCENDANT selector on a generic element must not reach into a popup that happens to
# be rendered inside it.
#
# The failure. Our popovers and dialogs are top-layer elements (`popover` attribute / native `<dialog>`), but
# top-layer promotion changes PAINTING, not the DOM: the panel stays a descendant of its trigger's wrapper. So
#
#   .dx-input-group-addon button { height: var(--dx-control-height-xs); padding: 0 var(--dx-space-2) }
#   .dx-input-group-addon svg    { width: 1rem; height: 1rem }
#
# also matched every day cell and chevron of the Date Picker calendar whose trigger sits in an addon (28x24
# cells, 16px chevrons) -- invisible on the Date Picker's own page, wrong only in the one demo that nests it.
# Any `.dx-x <bare element | [role] | *>` rule has the same reach: Popover, DropdownMenu, Select, Tooltip,
# HoverCard, Combobox, Dialog/Sheet/Drawer, DatePicker/ColorPicker and Command all render inside their trigger.
#
# THE CONSTRUCTION: end the selector at the popup boundary, which is one fixed, zero-specificity phrase,
#
#   .dx-input-group-addon svg:not(:where([popover], dialog) *) { ... }
#
# (`:where` keeps the rule's weight what it was, so no cascade contest moves), or select the component's OWN
# part by class (`.dx-input-group-icon`), which no popup carries. `playwright/popup-style-isolation.spec.ts`
# proves the other half in a browser: it opens every popup on every component page and compares it with the
# same popup rendered alone.
#
# WHAT IT FLAGS: a rule in a component's `style.css` whose selector has an ancestor part (descendant or child
# combinator) and whose SUBJECT (rightmost compound) is generic -- a bare element (`svg`, `a`, `button`,
# `img`, `p`, `td`, ...), `*`, a `[role]`/`[data-*]` attribute selector, or a non-`dx-` class -- unless the
# selector carries a `:not(... *)` boundary, or the rule is a boundary reset itself (its subject names
# `[popover]`, `dialog` or a popup `role`: `.dx-button-group :is([popover], dialog, ...)`).
# A subject that carries a `dx-` class or a `[data-slot=...]` is the component's own part and is not flagged
# (check-dx-class-prefix.sh keeps another component's class out of the sheet).
#
# THE ALLOWLIST below is for rules whose ancestor cannot contain a popup, with the reason. An entry is
# `(component dir, regex searched in the selector, reason)`; it must match at least one flagged selector (a
# stale entry fails), and a reason is mandatory. Prefer adding the boundary to the rule over an entry: the boundary costs nothing.
#
# Usage: scripts/check-popup-boundary.sh [--list]      (--list prints every flagged site, allowlisted or not)
# Exit status: 0 = every generic descendant selector is bounded or allowlisted; 1 = a new one; 2 = bad usage.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

list=0
for arg in "$@"; do
    case "$arg" in
        --list) list=1 ;;
        -h|--help) sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "check-popup-boundary: unknown option: $arg" >&2; exit 2 ;;
    esac
done

LIST="$list" python3 - <<'PYEOF'
import os
import re
import sys

COMPONENTS = "preview/src/components"
LIST = os.environ.get("LIST") == "1"

# (component dir, regex searched in the selector, reason). See the header: prefer the boundary to an entry.
LEAF = "the ancestor is a leaf part of the component (a label, icon holder or control face) that holds its own icon/text only; a popup trigger is never rendered inside it, its panel is a sibling or an outer wrapper's child"
OWN = "the ancestor is the component's own control and the subject its own icon/internal; the component renders no popup inside it"
DIRECT = "a direct-child (`>`) rule: the matched child is one of the component's own items, and a popup panel is never a direct child of this host (it sits one wrapper deeper)"
SVGINNER = "the subject is an SVG-internal element (`text`, `line`, `path`, `tspan`, ...) of the chart's own <svg>; HTML cannot be nested in it, so no popup is reachable"
PENDING = "a free-content slot a user can put a link (and so a HoverCard/Popover trigger) in; sheet owned by a batch-2 lane, so the boundary is not added here -- add `:not(:where([popover], dialog) *)` to the subject when that lane merges and delete this entry"
ALLOW = [
    ("alert", r"^\.dx-alert > svg$", DIRECT),
    ("attachment", r"\.dx-attachment-media", "the ancestor is the attachment's media tile, which holds one icon or one image"),
    ("avatar", r"^\.dx-avatar > \*$", DIRECT),
    ("avatar", r"\.dx-avatar-(badge|group-count) > svg$", "the ancestor is a leaf part (the status badge dot or the +N tile) that holds one icon; no popup trigger is ever rendered inside it"),
    ("badge", r"^\.dx-badge svg$", "the ancestor is an inline label; a Badge holds text and an icon, never an interactive popup trigger"),
    ("breadcrumb", r"\.dx-breadcrumb-(separator|ellipsis) svg", LEAF),
    ("button", r"svg", "the ancestor is a <button>, which cannot contain interactive content, so no popup trigger (and no panel) is rendered inside it"),
    ("toggle", r"^\.dx-toggle(\[[^]]*\])? svg", "the ancestor is the Toggle, a <button>, which cannot contain interactive content, so no popup trigger (and no panel) is rendered inside it"),
    ("toggle_group", r"^\.dx-toggle-group-item svg", "the ancestor is a ToggleGroup item, a <button>, which cannot contain interactive content, so no popup trigger (and no panel) is rendered inside it"),
    ("button_group", r" > ", DIRECT),
    ("button_group", r"\.dx-button-group-text svg", LEAF),
    ("calendar", r"\.dx-calendar-(nav-(prev|next)|(month|year)-select-value) svg", LEAF),
    ("calendar", r"\.dx-calendar-grid-week td", "the ancestor is the calendar's own week row; the calendar renders no popup inside its grid"),
    ("card", r"^\.dx-card > img", DIRECT),
    ("carousel", r"\.dx-carousel-(previous|next|rotation-control) svg", LEAF),
    ("chart", r"\] (line|text|path|tspan)\b|\[data-axis=\"(radius|angle)\"\] text|tspan|\[data-series\] > \*", SVGINNER),
    ("chart", r"chart-icon\"\] svg", LEAF),
    ("chart_tooltip", r"chart-icon\"\] svg", LEAF),
    ("checkbox", r"^\.dx-checkbox-indicator > svg$", LEAF),
    ("command", r"\.dx-command-item", "the ancestor is a command row, which holds its icon, label and shortcut; the Command's own dialog is the popup and these rules sit inside it"),
    ("empty", r"\.dx-empty-media", LEAF),
    ("field", r"> \[role=\"(checkbox|radio)\"\]", DIRECT),
    ("item", r"\.dx-item-media", LEAF),
    ("kbd", r"^\.dx-kbd svg$", LEAF),
    ("marker", r"^\.dx-marker-icon svg$", LEAF),
    ("message_scroller", r"\[data-message-scroller-spacer\]", DIRECT),
    ("message_scroller", r"\.dx-message-scroller-button\[data-direction=\"start\"\] svg", LEAF),
    ("native_select", r"\b(option|optgroup)\b", "<option>/<optgroup> live inside a native <select>, which cannot contain a popup"),
    ("pagination", r"\.dx-pagination-(link|ellipsis) svg", LEAF),
    ("select", r"\.dx-select-trigger", "the ancestor is the Select's trigger face (value text and chevron); the list is the trigger's sibling inside `.dx-select` (popup-style-isolation.spec.ts opens it on every page)"),
    ("sidebar", r"menu-button|menu-action|menu-sub-button|group-label|group-action", "the ancestor is a sidebar menu button, action or group label (icon + label); a dropdown from a sidebar row is that row's sibling, not its child"),
    ("spinner", r"^\.dx-spinner svg$", LEAF),
    ("tabs", r"^\.dx-tabs-trigger svg", LEAF),
    ("toast", r"^\.dx-toast-container (ol|li)$", "the toast stack's own list and list items; a toast is not a place where a trigger and panel are composed by the user"),
]


def strip_comments(css):
    return re.sub(r"/\*.*?\*/", lambda m: "\n" * m.group(0).count("\n"), css, flags=re.S)


def parse_rules(css):
    """[(prelude, line, [enclosing at-rule preludes])] for every style rule (quote/paren aware)."""
    css = strip_comments(css)
    rules, stack, buf = [], [], []
    quote, paren, i, n = None, 0, 0, len(css)
    while i < n:
        ch = css[i]
        if quote:
            buf.append(ch)
            if ch == "\\" and i + 1 < n:
                buf.append(css[i + 1])
                i += 1
            elif ch == quote:
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
        elif paren > 0:
            buf.append(ch)
        elif ch == "{":
            prelude = "".join(buf)
            lead = len(prelude) - len(prelude.lstrip())
            start = i - len(prelude) + lead
            stack.append((prelude.strip(), css.count("\n", 0, start) + 1))
            buf = []
        elif ch == ";":
            buf = []
        elif ch == "}":
            if stack:
                prelude, line = stack.pop()
                if not prelude.startswith("@"):
                    rules.append((prelude, line, [p for p, _ in stack if p.startswith("@")]))
            buf = []
        else:
            buf.append(ch)
        i += 1
    return rules


def split_top(s, sep):
    out, depth, cur, quote = [], 0, [], None
    for ch in s:
        if quote:
            cur.append(ch)
            if ch == quote:
                quote = None
        elif ch in "\"'":
            quote = ch
            cur.append(ch)
        else:
            if ch in "([":
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


def compounds(sel):
    """[(combinator-before, compound)] of a complex selector, bracket and quote aware."""
    parts, depth, cur, quote, comb, pending = [], 0, [], None, " ", " "
    for ch in sel.strip():
        if quote:
            cur.append(ch)
            if ch == quote:
                quote = None
        elif ch in "\"'":
            quote = ch
            cur.append(ch)
        else:
            if ch in "([":
                depth += 1
            elif ch in ")]":
                depth -= 1
            if depth == 0 and ch in " \t\n>+~":
                if cur:
                    parts.append((comb, "".join(cur)))
                    cur, comb = [], " "
                if ch in ">+~":
                    comb = ch
            else:
                cur.append(ch)
    if cur:
        parts.append((comb, "".join(cur)))
    return parts


def strip_functional(compound):
    """Drop `:not(..)`, `:has(..)`, `:where(..)`, `:is(..)` arguments: not what the rule styles."""
    out, i = [], 0
    while i < len(compound):
        m = re.match(r":(not|has|where)\(", compound[i:])
        if not m:
            out.append(compound[i])
            i += 1
            continue
        depth, j = 0, i + m.end() - 1
        while j < len(compound):
            if compound[j] == "(":
                depth += 1
            elif compound[j] == ")":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        i = j + 1
    return "".join(out)


# `:not( ... *)` -- a deliberate "not inside X" boundary (the canonical one is `:not(:where([popover], dialog) *)`).
BOUNDARY = re.compile(r":not\((?:[^()]|\([^()]*\))*\s\*\s*\)")
# A rule whose subject IS the popup boundary (a reset applied at the popup root).
RESET_SUBJECT = re.compile(r"\[popover\]|\bdialog\b|\[role=[\"']?(?:dialog|alertdialog|menu|listbox|tooltip)")


def generic_subject(subj):
    """True when the subject names no `dx-` class and no `[data-slot]`: any element of any component can match it."""
    bare = strip_functional(subj)
    if re.search(r"\.dx-", bare) or "[data-slot" in bare:
        return False
    return True


def scan():
    sites = []  # (file, line, selector)
    for x in sorted(os.listdir(COMPONENTS)):
        path = f"{COMPONENTS}/{x}/style.css"
        if not os.path.isfile(path):
            continue
        with open(path, encoding="utf-8") as f:
            rules = parse_rules(f.read())
        for prelude, line, ctx in rules:
            if any("keyframes" in c for c in ctx):
                continue
            for sel in split_top(prelude, ","):
                sel = " ".join(sel.split())
                parts = compounds(sel)
                if len(parts) < 2:
                    continue
                comb, subj = parts[-1]
                if comb in "+~":
                    continue  # a sibling is not a container
                if not generic_subject(subj) or RESET_SUBJECT.search(subj):
                    continue
                if BOUNDARY.search(sel):
                    continue
                sites.append((path, line, sel))
    return sites


try:
    sites = scan()
except Exception as e:  # a parse failure must not read as "clean"
    print(f"check-popup-boundary: internal error: {e!r}", file=sys.stderr)
    sys.exit(2)

used = [0] * len(ALLOW)
bad = []
for path, line, sel in sites:
    comp = path.split("/")[-2]
    hit = None
    for k, (f, sub, reason) in enumerate(ALLOW):
        if f == comp and re.search(sub, sel):
            hit = k
            used[k] += 1
            break
    if LIST:
        print(f"{path}:{line}\t{'allowed' if hit is not None else 'FLAGGED'}\t{sel}")
    if hit is None:
        bad.append((path, line, sel))

stale = [ALLOW[k] for k, n in enumerate(used) if n == 0]
for f, sub, reason in ALLOW:
    if not reason.strip():
        print(f"check-popup-boundary: allowlist entry '{f} | {sub}' has no reason", file=sys.stderr)
        sys.exit(1)

if LIST:
    sys.exit(0)

if bad:
    print("check-popup-boundary: FAILED -- a descendant selector on a generic element reaches into popups rendered inside it:", file=sys.stderr)
    for path, line, sel in bad:
        print(f"  {path}:{line}  {sel}", file=sys.stderr)
    print(file=sys.stderr)
    print("A popup (`[popover]` / `<dialog>`) stays a DOM descendant of its trigger, so this rule also styles the panel's", file=sys.stderr)
    print("buttons, icons, links and inputs when a trigger is nested in the ancestor. End the selector at the boundary:", file=sys.stderr)
    print("    .dx-x svg:not(:where([popover], dialog) *)   (zero specificity)", file=sys.stderr)
    print("or select the component's own part by class. Only if no popup can ever sit inside the ancestor, add a reasoned", file=sys.stderr)
    print("entry to ALLOW in scripts/check-popup-boundary.sh. See that file's header.", file=sys.stderr)
    sys.exit(1)

if stale:
    print("check-popup-boundary: FAILED -- stale allowlist entries (no flagged selector matches; delete them):", file=sys.stderr)
    for f, sub, reason in stale:
        print(f"  {f} | {sub}", file=sys.stderr)
    sys.exit(1)

print(f"check-popup-boundary: OK ({len(sites)} generic descendant selector(s), all bounded or allowlisted with a reason)")
PYEOF
