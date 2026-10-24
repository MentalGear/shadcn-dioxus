#!/usr/bin/env bash
#
# check-coarse-sizing.sh  (backlog rows 349, 350)
#
# Touch sizing is ONE switch, `data-dx-coarse="auto|on|off"` on <html> or any container, and every size that
# differs by pointer is a theme variable (`--dx-coarse-*`, `--dx-menu-item-*`, `--dx-menu-trigger-*`) that
# the switch flips (dx-components-theme.css, "Coarse sizing"). A component sheet that says
# `@media (pointer: coarse)` (or `any-pointer: coarse`) itself would follow the device and ignore the
# attribute: a forced-on container would keep the desktop size in it (row 349).
#
# This gate fails on a `pointer: coarse` / `any-pointer: coarse` media feature in any stylesheet under
# preview/src (every component sheet, the dashboards) or preview/assets, comments stripped, except the
# reasoned ALLOW entries below; a stale ALLOW entry fails too. It also requires the theme's switch to
# exist (the two media rules, the `on` / `off` attribute rules and the `:root, [data-dx-coarse]` token
# block) so the variables cannot be left without a writer, and runs a self-test on a synthetic bad sheet.
#
# Usage: scripts/check-coarse-sizing.sh [FILE ...]   (explicit files are for self-tests: no ALLOW, no theme check)
# Exit status: 0 when clean.

set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import glob, re, sys

explicit = sys.argv[1:]
THEME = "preview/assets/dx-components-theme.css"

# (file, reason). One entry per file; the file may keep its raw media rules.
ALLOW = [
    (THEME, "the switch itself: the media rules that flip --dx-pick-coarse / --dx-pick-fine"),
    ("preview/assets/main.css",
     "the preview app's own backstop floor for RAW <input>/<select>/<textarea> (not library components, not shipped); "
     "it follows the device, the theme variables cover every `dx-` control"),
]

PATTERN = re.compile(r"@media[^{]*\b(?:any-)?pointer\s*:\s*coarse", re.I)


def strip(src):
    return re.sub(r"/\*.*?\*/", lambda m: re.sub(r"[^\n]", " ", m.group(0)), src, flags=re.S)


def scan(src):
    """Line numbers of every raw coarse media feature (comments stripped)."""
    src = strip(src)
    return [src.count("\n", 0, m.start()) + 1 for m in PATTERN.finditer(src)]


# Self-test: the scanner must see the bad shapes and ignore comments.
assert scan("@media (pointer: coarse) { .x { font-size: 1rem } }") == [1]
assert scan("a{}\n@media (any-pointer: coarse) and (hover: none) { .x {} }") == [2]
assert scan("@media (hover: hover) and (pointer: coarse) { .x {} }") == [1]
assert scan("/* @media (pointer: coarse) { } */ .x {}") == []
assert scan("@media (pointer: fine) { .x {} }") == []

if explicit:
    files = explicit
else:
    files = sorted(set(glob.glob("preview/src/**/*.css", recursive=True) + glob.glob("preview/assets/*.css")))
    files = [f for f in files if "node_modules" not in f]
violations, used = [], set()
for path in files:
    lines = scan(open(path, encoding="utf-8").read())
    if not lines:
        continue
    if not explicit and any(a[0] == path for a in ALLOW):
        used.add(path)
        continue
    violations += [(path, l) for l in lines]

problems = []
if violations:
    problems.append("a raw coarse-pointer media rule outside the theme's switch; read a theme variable instead "
                    "(`font-size: var(--dx-coarse-text-entry-font-size)`, ...) or add one in the theme's "
                    "\"Coarse sizing\" block:")
    problems += [f"  {p}:{l}" for p, l in violations]
if not explicit:
    for path, _ in ALLOW:
        if path not in used:
            problems.append(f"stale ALLOW entry {path}: it has no raw coarse media rule any more; delete the entry")
    theme = strip(open(THEME, encoding="utf-8").read())
    for label, needle in [
        ("the `auto` media rule", r"@media \(pointer: coarse\)\s*\{\s*:root,\s*\[data-dx-coarse=\"auto\"\]"),
        ("the fine default on `:root, [data-dx-coarse=\"auto\"]`", r"(?<![\w}])\n:root,\s*\[data-dx-coarse=\"auto\"\]\s*\{"),
        ("the `on` attribute rule", r"\[data-dx-coarse=\"on\"\]\s*\{"),
        ("the `off` attribute rule", r"\[data-dx-coarse=\"off\"\]\s*\{"),
        ("the token block on `:root, [data-dx-coarse]`", r":root,\s*\[data-dx-coarse\]\s*\{"),
    ]:
        if not re.search(needle, theme):
            problems.append(f"{THEME} lost {label}")
    block = re.search(r":root,\s*\[data-dx-coarse\]\s*\{(.*?)\n\}", theme, re.S)
    if block and "--dx-pick-coarse" not in block.group(1):
        problems.append("the `:root, [data-dx-coarse]` block no longer reads --dx-pick-coarse")
if problems:
    print("check-coarse-sizing: FAIL")
    print("\n".join(problems))
    sys.exit(1)
print(f"check-coarse-sizing: PASS ({len(files)} stylesheets)")
PYEOF
