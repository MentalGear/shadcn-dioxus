#!/usr/bin/env bash
#
# check-theme-preset-axes.sh  (backlog row 390)
#
# The theme picker has two colour axes that own DISJOINT variables, so picking one never moves what the other owns: the base preset
# (`[data-theme-base]`) writes only the neutral palette and the base-owned slots of the no-accent primary, the accent preset
# (`[data-theme-accent]`) writes only primary, charts and sidebar-primary. The ownership table is written once, in `OWNERSHIP` of
# scripts/gen-theme-presets.mjs (this gate reads it with `--print-ownership`). A base block that sets a chart colour, an accent block that sets
# `--secondary`, a base block that sets a bare `--primary`, a variable in a block of the wrong axis, a slot the theme never reads, or a
# committed preview/assets/theme-presets.css that differs by one byte from the generator's output (a hand edit, or data that changed
# without a regeneration) all fail. The gate runs its own mutation self-test on every run; each mutation must turn it red.
#
# Usage: scripts/check-theme-preset-axes.sh
# Exit status: 0 when clean, 1 on a violation (file and block named on stderr), 2 on a setup problem.

set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

command -v node >/dev/null || { echo "check-theme-preset-axes: node is required" >&2; exit 2; }
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
node scripts/gen-theme-presets.mjs --print-ownership > "$tmp/ownership.json"
node scripts/gen-theme-presets.mjs --out "$tmp/generated.css" 2>/dev/null

python3 -I - "$tmp/ownership.json" "$tmp/generated.css" <<'PYEOF'
import json
import re
import sys

ownership = json.load(open(sys.argv[1]))
generated = open(sys.argv[2], encoding="utf-8").read()
PRESETS = "preview/assets/theme-presets.css"
THEME = "preview/assets/dx-components-theme.css"

base_ok = set(ownership["base"]["vars"]) | set(ownership["base"]["slots"].values()) | set(ownership["base"]["derived"])
accent_ok = set(ownership["accent"]["vars"])
slots = list(ownership["base"]["slots"].items())  # (bare name, slot name)

BLOCK = re.compile(r'\[data-theme-(base|accent)(?:="([a-z0-9.]+)")?\]\s*\{(.*?)\n\}', re.S)


def strip_comments(css):
    return re.sub(r"/\*.*?\*/", lambda m: re.sub(r"[^\n]", " ", m.group(0)), css, flags=re.S)


def check(presets, theme, expected):
    """Violations of the axes rules in `presets` (the stylesheet text), `theme` (dx-components-theme.css) and `expected` (generator output)."""
    errors = []
    css = strip_comments(presets)
    seen = {"base": set(), "accent": set()}
    for m in BLOCK.finditer(css):
        axis, name, body = m.group(1), m.group(2), m.group(3)
        label = f'[data-theme-{axis}{"=" + chr(34) + name + chr(34) if name else ""}]'
        allowed = base_ok if axis == "base" else accent_ok
        written = re.findall(r"^\s*--([a-z0-9-]+)\s*:", body, re.M)
        if name is None and axis == "accent":
            errors.append(f"{label}: an accent block with no name")
        for var in written:
            if var not in allowed:
                owner = "accent" if var in accent_ok else "base" if var in base_ok else None
                why = f"owned by the {owner} axis" if owner else "owned by neither axis"
                errors.append(f"{PRESETS}: {label} sets --{var} ({why}; the {axis} axis may write only its own variables)")
        if name is not None:
            seen[axis].add(name)
    if not seen["base"] or not seen["accent"]:
        errors.append(f"{PRESETS}: no [data-theme-base=...] or no [data-theme-accent=...] block found")
    both = base_ok & accent_ok
    if both:
        errors.append(f"the ownership table gives {sorted(both)} to both axes")
    th = strip_comments(theme)
    for bare, slot in slots:
        if not re.search(r"var\(--" + re.escape(bare) + r",\s*var\(--" + re.escape(slot) + r"\b", th):
            errors.append(f"{THEME}: no role token reads var(--{bare}, var(--{slot}, ...)); the slot --{slot} would never show")
    if presets != expected:
        errors.append(f"{PRESETS} differs from `node scripts/gen-theme-presets.mjs` (hand edit, or data/table changed without regenerating; run it and commit the result)")
    return errors


presets = open(PRESETS, encoding="utf-8").read()
theme = open(THEME, encoding="utf-8").read()
errors = check(presets, theme, generated)
if errors:
    print("check-theme-preset-axes: FAIL", file=sys.stderr)
    for e in errors:
        print("  " + e, file=sys.stderr)
    sys.exit(1)


# Self-test: every mutation of a clean copy must be reported, and the clean copy must pass.
def mutate(text, anchor, insert):
    i = text.index(anchor) + len(anchor)
    return text[:i] + insert + text[i:]


def first_block(text, axis):
    return re.search(r'\[data-theme-' + axis + r'="[a-z]+"\] \{\n', text).group(0)


assert not check(presets, theme, presets), "self-test: the clean tree must pass with its own text as the expectation"
mutations = {
    "a base block with a chart colour": lambda p, t: (mutate(p, first_block(p, "base"), "  --chart-1: oklch(0.5 0.2 20);\n"), t),
    "an accent block with --secondary": lambda p, t: (mutate(p, first_block(p, "accent"), "  --secondary: oklch(0.9 0 0);\n"), t),
    "a base block with a bare --primary": lambda p, t: (mutate(p, first_block(p, "base"), "  --primary: oklch(0.2 0 0);\n"), t),
    "an accent block with --background": lambda p, t: (mutate(p, first_block(p, "accent"), "  --background: oklch(1 0 0);\n"), t),
    "an accent block with a base slot": lambda p, t: (mutate(p, first_block(p, "accent"), "  --dx-base-primary: oklch(0.2 0 0);\n"), t),
    "a slot the theme never reads": lambda p, t: (p, t.replace("var(--dx-base-sidebar-primary,", "var(--dx-base-sidebar-primar,")),
}
for what, fn in mutations.items():
    p, t = fn(presets, theme)
    if p == presets and t == theme:
        print(f"check-theme-preset-axes: self-test mutation did not apply: {what}", file=sys.stderr)
        sys.exit(1)
    # The mutated text is its own expectation, so only the axes rules can catch it; the byte-identity rule is proven below.
    if not check(p, t, p):
        print(f"check-theme-preset-axes: self-test FAILED: {what} was not reported", file=sys.stderr)
        sys.exit(1)
if not check(presets + "\n/* edit */\n", theme, presets):
    print("check-theme-preset-axes: self-test FAILED: a hand edit was not reported", file=sys.stderr)
    sys.exit(1)
n_base = len(re.findall(r'\[data-theme-base="', strip_comments(presets)))
n_accent = len(re.findall(r'\[data-theme-accent="', strip_comments(presets)))
print(f"check-theme-preset-axes: OK ({n_base} base and {n_accent} accent blocks on disjoint variables; {len(mutations) + 1} mutations red)")
PYEOF
