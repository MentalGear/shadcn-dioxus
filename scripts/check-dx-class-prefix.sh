#!/usr/bin/env bash
#
# check-dx-class-prefix.sh
#
# Every class selector in a component's own `preview/src/components/<name>/style.css` is spelled `dx-<name>` or
# `dx-<name>-...` (the folder name with `_` written as `-`), so classes stay collision-safe without a hash suffix. Selectors are read in selector position inside every at-rule; declaration bodies and comments are never
# scanned. Demo-only stylesheets under `variants/` are out of scope.
#
#   Every class selector defined in a themed component's own stylesheet
#   (`preview/src/components/<name>/style.css`) must be spelled
#   `dx-<name>` or `dx-<name>-...`, where `<name>` is the component's own
#   folder name with `_` written as `-` (e.g. `dropdown_menu` ->
#   `dx-dropdown-menu`).
#
# Why this exists: classes are plain, human-readable `dx-`-prefixed names
# (not `#[css_module]`'s hashed `.dx-checkbox-a1b2c3`, row 32), so two
# components could each write `.dx-foo` and collide in the built CSS. They
# cannot, as long as every component's classes are namespaced under its own
# name. This
# script is the mechanical check for that, the same way
# check-preview-composition.sh is the mechanical check for the
# themed-wrapper composition rule and check-cfg-axis.sh is the mechanical
# check for the cfg axis rule -- author a convention, then enforce it so it
# can't silently regress one component at a time.
#
# What counts as a "class selector": only text in *selector position* --
# i.e. text immediately preceding a `{` that itself doesn't start with `@`
# (so `@media (...) { .dx-checkbox { ... } }`'s inner `.dx-checkbox` IS
# checked -- unlike `#[css_module]`'s own `manganis-core::css_module_parser`,
# which does NOT recurse into `@supports` bodies
# (dev-docs/issues/css-module-supports-scoping.md), this lint recurses into
# *every* at-rule uniformly, since it has no hashing pass to share that
# blind spot). Declaration bodies (the `color: dx-red;`-shaped
# text between a `{` and its matching `}`) are never scanned, so a
# `content: ".foo"` string or a `url(icon.dx-thing.svg)` value can't produce
# a false positive. Comments (`/* ... */`) are stripped first -- several
# style.css files quote real CSS syntax inside prose comments (e.g.
# "the UA popover stylesheet's `[popover] { ... }`"), and those braces must
# not be mistaken for real ones.
#
# Parsing this precisely (comment-aware, brace-depth-aware, but NOT
# fooled by braces quoted inside comments) is more than a `grep`/bash-regex
# pass can do reliably -- unlike check-preview-composition.sh's `use`-line
# matching, a CSS selector list has no fixed line shape to anchor on. This
# script therefore shells out to `python3` (already a assumed-present tool
# in this repo's tooling -- `scripts/generate-dx-utilities.js` already
# requires `node` for the equivalent reason) for the actual tokenizing,
# while keeping the same bash driver/reporting shape as the other two
# check-*.sh scripts.
#
# Scope: every `preview/src/components/<name>/style.css` -- the file that
# is actually copied out by `dx components add <name>` (component.json's
# `exclude` list drops `variants/`, `docs.md`, `component.json`, but never
# `style.css`), which is what makes its classes matter for a consumer's
# page, not just this repo's own demo site. Demo-only stylesheets under a
# component's own `variants/` folder (never shipped) are intentionally out
# of scope.
#
# Usage: scripts/check-dx-class-prefix.sh
# Exit status: 0 if every component stylesheet only defines
# dx-<component>[-...] classes, 1 and a listing of every offending
# selector otherwise.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

components_dir="preview/src/components"
fail=0

# ---------------------------------------------------------------------------
# The extractor: reads one CSS file plus its required prefix on argv, prints
# one "path:line: .class-name" per violation, and exits 1 if it found any
# (0 if clean) -- mirrors a grep-like contract so the bash loop below can
# just check $? the way it already does for other tools in this repo.
# ---------------------------------------------------------------------------
extractor="$(mktemp -t dx-class-prefix-extract.XXXXXX.py)"
trap 'rm -f "$extractor"' EXIT

cat >"$extractor" <<'PYEOF'
import re
import sys


def extract_class_selectors(css_text):
    """Yield (class_name, line_number) for every class selector in css_text.

    Strips comments first (non-greedy, DOTALL -- comments here routinely
    quote CSS syntax including literal braces, e.g. "the UA popover
    stylesheet's `[popover] { ... }`", which must not be mistaken for real
    rule boundaries). Then does a single linear scan, accumulating
    characters into `buf` and resetting it on every `{` and `}`: whatever
    sat in `buf` right before a `{` is that rule's prelude -- a real
    selector list unless it starts with `@` (an at-rule -- @media,
    @supports, @keyframes, ... -- whose own prelude is a condition/name,
    not a selector, but whose nested body is still scanned normally since
    the reset-per-brace loop just keeps going). Declaration bodies (text
    between a `{` and its matching `}`) are discarded on `}` and never
    inspected, so property values can't produce false positives.
    """
    css = re.sub(r"/\*.*?\*/", "", css_text, flags=re.S)
    selectors = []
    buf = []
    buf_start_line = 1
    line = 1
    for ch in css:
        if ch == "\n":
            line += 1
        if ch == "{":
            prelude = "".join(buf)
            stripped = prelude.strip()
            if stripped and not stripped.startswith("@"):
                for m in re.finditer(r"\.([A-Za-z][A-Za-z0-9_-]*)", prelude):
                    selectors.append((m.group(1), buf_start_line))
            buf = []
            buf_start_line = line
        elif ch == "}":
            buf = []
            buf_start_line = line
        else:
            buf.append(ch)
    return selectors


if __name__ == "__main__":
    path, prefix = sys.argv[1], sys.argv[2]
    with open(path, encoding="utf-8") as f:
        text = f.read()
    bad = 0
    for cls, ln in extract_class_selectors(text):
        if cls == prefix or cls.startswith(prefix + "-"):
            continue
        bad += 1
        print(f"{path}:{ln}: .{cls} does not start with `{prefix}-` "
              f"(expected dx-<component>-... naming, dev-docs/backlog.md row 32)")
    sys.exit(1 if bad else 0)
PYEOF

# ---------------------------------------------------------------------------
# Scan every component's shipped stylesheet.
# ---------------------------------------------------------------------------
# A component whose Rust source still carries a `#[css_module]` attribute is hashed, so its
# out-of-namespace classes cannot collide with another sheet's: for those a violation is a
# WARNING and does not fail the run (today only the `top_layer` fixture).
#
# Once the attribute is gone (row 32), that protection is gone and the same violation is a hard
# ERROR, so the exemption expires per component exactly when it stops being true. The order is
# enforced too: namespace a component's classes FIRST, drop its hashing SECOND, because two
# sheets that define one class name with different rules would merge in the cascade the moment
# both were unhashed without a rename.
violating_components=0
warned_components=0

for dir in "$components_dir"/*/; do
    name="$(basename "$dir")"
    css="${dir%/}/style.css"
    [[ -f "$css" ]] || continue

    dashed="${name//_/-}"
    prefix="dx-$dashed"

    if output="$(python3 "$extractor" "$css" "$prefix")"; then
        continue
    fi

    # Still hashed? Then this is a warning, not yet a failure.
    # Match the ATTRIBUTE in Rust source only -- several `style.css` files
    # mention `#[css_module]` in prose comments explaining this very
    # migration, and matching those would keep a component exempt forever,
    # even once migrated.
    if grep -rqsE --include='*.rs' '^[[:space:]]*#\[css_module' "$dir"; then
        warned_components=$((warned_components + 1))
        echo "$output" | sed 's/^/warning: /'
        echo "warning: ^-- $name still has #[css_module], so its hash still prevents collisions;"
        echo "warning:     namespace these classes BEFORE dropping its hashing (dev-docs/backlog.md row 32)."
    else
        fail=1
        violating_components=$((violating_components + 1))
        echo "$output"
    fi
done

if [[ "$fail" -ne 0 ]]; then
    echo
    echo "check-dx-class-prefix: FAILED -- $violating_components component(s) define a class outside their own dx-<component>-... namespace. See dev-docs/backlog.md row 32."
    exit 1
fi

if [[ "$warned_components" -ne 0 ]]; then
    echo
    echo "check-dx-class-prefix: OK -- $warned_components component(s) still rely on #[css_module] hashing to stay collision-free; each must be namespaced before its hashing is dropped (dev-docs/backlog.md row 32)."
    exit 0
fi

echo "check-dx-class-prefix: OK"
