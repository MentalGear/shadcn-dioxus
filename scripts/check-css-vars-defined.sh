#!/usr/bin/env bash
#
# check-css-vars-defined.sh
#
# Enforces dev-docs/backlog.md row 111: every CSS custom property a
# component reads with `var(--name)` must be declared somewhere in the scanned
# source, or be on the explicit allowlist below. A fallback in the `var()`
# does NOT excuse the name.
#
#   color: var(--primary-color-9);            -> FAIL: nothing declares it
#   color: var(--dx-muted-foreground);        -> FAIL if the theme lacks it
#   color: var(--dx-mutd-foreground, #666);   -> FAIL: a typo the fallback hides
#   height: var(--dx-select-scroll-button-height, 1.5rem)
#                                             -> ok: allowlisted caller slot
#
# Why: an undefined custom property is not an error in CSS. `color:
# var(--nope)` is "invalid at computed-value time" and silently computes to
# `unset` -- the text renders in the inherited colour, a `background` vanishes,
# a border disappears -- with no console message and no failing test. It bites
# hardest on the role-token migration: `dx components add` copies a component
# stylesheet into a user's tree next to THEIR copy of dx-components-theme.css,
# and an older theme copy lacks any `--dx-<role>` token added since. The
# component then renders wrongly in exactly the consumer's tree and nowhere in
# ours. With a fallback the failure is quieter still: a retired or misspelt
# token renders the fallback forever, so a theme change never reaches it.
#
# WHAT COUNTS AS A USE
#   * `var(--name` in preview/src/**/*.css, preview/assets/*.css,
#     primitives/src/**/*.css (comments stripped), including every name inside
#     a fallback (`var(--a, var(--b))` reads both);
#   * `var(--name` in preview/src/**/*.rs and primitives/src/**/*.rs --
#     inline `style:` strings, `format!` colours, embedded CSS. Comment lines
#     and everything after a file's `#[cfg(test)]` marker are ignored, as are
#     files that a parent declares `#[cfg(test)] mod name;` (test modules
#     assert on rendered strings and shadcn fixtures, not on shipped styles).
#
# WHAT COUNTS AS A DEFINITION (any file in the same scope)
#   * a `--name:` declaration (CSS, or inside a Rust string/inline style);
#   * `@property --name`;
#   * `setProperty('--name', ...)` in embedded JS;
#   * an interpolated Rust declaration `--prefix-{..}:` defines the PREFIX
#     `--prefix-`; a `var(--prefix-{..})` use is satisfied by any definition
#     or defined prefix that starts with `--prefix-`.
#
# THE ALLOWLIST (below) holds the names read on purpose with nothing in the
# tree declaring them: caller-provided slots (a component reads
# `var(--dx-<component>-<knob>, <default>)` so a parent or the app can tune it)
# and properties embedded JS writes under a computed name. Each entry names
# where it is documented or written. The gate also fails on a stale entry --
# one that is declared in the tree, or read nowhere -- so the list cannot outlive
# its reasons.
#
# Usage: scripts/check-css-vars-defined.sh
# Exit status: 0 when every use is declared or allowlisted and no allowlist
# entry is stale.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - <<'PYEOF'
import glob
import os
import re
import sys

# --------------------------------------------------------------------------
# Allowlist: name (or `prefix-*`) -> why it may be used without a definition
# or fallback. Add an entry only with a reason a reviewer can check.
# --------------------------------------------------------------------------
ALLOWLIST = {
    # Caller-provided slots: read with a default, set by a parent or the app.
    "--dx-button-line-height": "slot, button/docs.md (default 1.25rem, 1rem in xs)",
    "--dx-drawer-nest-scale": "slot, drawer/docs.md (parent shrink while a nested drawer is open)",
    "--dx-drawer-nest-shift": "slot, drawer/docs.md (parent push while a nested drawer is open)",
    "--dx-dropdown-menu-trigger-focus-ring": "slot, dropdown_menu/style.css header (parent restyles the ring)",
    "--dx-label-font-size": "slot, label/docs.md",
    "--dx-label-font-weight": "slot, label/docs.md",
    "--dx-label-gap": "slot, label/docs.md",
    "--dx-message-scroller-row-estimate": "slot, message_scroller/docs.md (first-render height guess)",
    "--dx-scroll-fade-size": "slot, dx-effects.css scroll-fade header",
    "--dx-scroll-fade-t-size": "slot, dx-effects.css scroll-fade header (one edge)",
    "--dx-scroll-fade-b-size": "slot, dx-effects.css scroll-fade header (one edge)",
    "--dx-scroll-fade-s-size": "slot, dx-effects.css scroll-fade header (one edge)",
    "--dx-scroll-fade-e-size": "slot, dx-effects.css scroll-fade header (one edge)",
    "--dx-scroll-fade-reveal": "slot, dx-effects.css scroll-fade header",
    "--dx-shimmer-color": "slot, dx-effects.css shimmer header",
    "--dx-shimmer-duration": "slot, dx-effects.css shimmer header",
    "--dx-shimmer-spread": "slot, dx-effects.css shimmer header",
    "--dx-select-scroll-button-height": "slot, select/style.css (scroll arrow height, default 1.5rem)",
    "--dx-skeleton-radius": "slot, skeleton/style.css (default --dx-radius-md)",
    # Written by embedded JS under a name the SETPROP pattern cannot read.
    "--front-toast-height": "primitives/src/toast.rs TOAST_SIZE_JS (`const FRONT`), unset until measured",
    "--toast-swipe-x": "primitives/src/toast.rs TOAST_SWIPE_JS (`prop(axis)`), set only while dragged",
    "--toast-swipe-y": "primitives/src/toast.rs TOAST_SWIPE_JS (`prop(axis)`), set only while dragged",
}

CSS_GLOBS = [
    "preview/src/**/*.css",
    "preview/assets/*.css",
    "primitives/src/**/*.css",
]
RS_GLOBS = [
    "preview/src/**/*.rs",
    "primitives/src/**/*.rs",
]

def files(globs):
    out = set()
    for g in globs:
        out.update(glob.glob(g, recursive=True))
    return sorted(out)

def strip_css_comments(text):
    return re.sub(r"/\*.*?\*/", lambda m: "\n" * m.group(0).count("\n"), text, flags=re.S)

def strip_rs(text):
    """Drop `#[cfg(test)]` tails, full-line `//` comments and trailing `//`
    comments (only when the code before them has balanced quotes, so a
    `https://` inside a string literal survives). Line count is preserved."""
    m = re.search(r"^[ \t]*#\[cfg\(test\)\]", text, flags=re.M)
    if m:
        text = text[: m.start()]
    out = []
    for line in text.split("\n"):
        s = line.lstrip()
        if s.startswith("//"):
            out.append("")
            continue
        i = line.find(" //")
        if i != -1 and line[:i].count('"') % 2 == 0:
            line = line[:i]
        out.append(line)
    return "\n".join(out)

sources = []  # (path, text)
for p in files(CSS_GLOBS):
    sources.append((p, strip_css_comments(open(p, encoding="utf-8").read())))

# A `#[cfg(test)] mod name;` declaration gates the whole file `name.rs` (or
# `name/mod.rs`) behind tests -- e.g. preview/src/chart_parity.rs, which
# rewrites shadcn's `var(--chart-N)` fixtures. Such files never ship.
test_only = set()
rs_files = files(RS_GLOBS)
for p in rs_files:
    d = os.path.dirname(p)
    for m in re.finditer(r"#\[cfg\(test\)\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;",
                         open(p, encoding="utf-8").read()):
        test_only.add(os.path.join(d, m.group(1) + ".rs"))
        test_only.add(os.path.join(d, m.group(1), "mod.rs"))
for p in rs_files:
    if p in test_only:
        continue
    sources.append((p, strip_rs(open(p, encoding="utf-8").read())))

NAME = r"--[A-Za-z_][A-Za-z0-9_-]*"

defined = set()      # exact names
prefixes = set()     # `--prefix-` from Rust-interpolated declarations
DECL = re.compile(r"(?<![\w-])(" + NAME + r")\s*:")
ATPROP = re.compile(r"@property\s+(" + NAME + r")")
SETPROP = re.compile(r"setProperty\(\s*['\"](" + NAME + r")['\"]")
DYN_DECL = re.compile(r"(?<![\w-])(--[A-Za-z0-9_-]*-)\{[^}]*\}\s*:")

for path, text in sources:
    for rx in (DECL, ATPROP, SETPROP):
        for m in rx.finditer(text):
            defined.add(m.group(1))
    for m in DYN_DECL.finditer(text):
        prefixes.add(m.group(1))

def allowed(name):
    return name in ALLOWLIST

def declared(name):
    if name in defined:
        return True
    if name.endswith("-"):  # interpolated use: `var(--prefix-{..})`
        return (any(d.startswith(name) for d in defined)
                or any(p.startswith(name) for p in prefixes))
    # a use like `--color-desktop` is also satisfied by a defined prefix
    return any(name.startswith(p) for p in prefixes)

def satisfied(name):
    return declared(name) or allowed(name)

USE = re.compile(r"var\(\s*(" + NAME + r")")

errors = []
used = set()
total = 0
for path, text in sources:
    for m in USE.finditer(text):
        total += 1
        name = m.group(1)
        used.add(name)
        # An interpolated name (`var(--color-{slot})`) stops the regex at the
        # `{`, leaving a name that ends in `-`; `satisfied` treats that as a
        # prefix.
        if satisfied(name):
            continue
        lineno = text.count("\n", 0, m.start()) + 1
        errors.append((path, lineno, name))

for path, lineno, name in errors:
    print(f"{path}:{lineno}: `var({name})` is never declared (a fallback does not count)")

stale = []
for name in sorted(ALLOWLIST):
    if declared(name):
        stale.append(f"ALLOWLIST entry {name} is declared now; remove the entry")
    elif name not in used:
        stale.append(f"ALLOWLIST entry {name} is read nowhere; remove the entry")
for line in stale:
    print(line)

if stale and not errors:
    print()
    print(f"check-css-vars-defined: FAILED -- {len(stale)} stale allowlist "
          f"entr{'y' if len(stale) == 1 else 'ies'}.")
    sys.exit(1)

if errors:
    names = sorted({n for _, _, n in errors})
    print()
    print(f"check-css-vars-defined: FAILED -- {len(errors)} use(s) of "
          f"{len(names)} undefined custom propert{'y' if len(names) == 1 else 'ies'}: "
          + ", ".join(names))
    print("Fix: point it at a declared token, or declare it (theme/stylesheet/"
          "inline style). A fallback alone does not count. If it is a "
          "caller-provided slot or JS writes it, add it to ALLOWLIST in this "
          "script WITH a reason. See dev-docs/backlog.md row 111.")
    sys.exit(1)

print(f"check-css-vars-defined: OK -- {total} var() uses checked, "
      f"{len(defined)} custom properties defined, {len(ALLOWLIST)} allowlisted.")
PYEOF
