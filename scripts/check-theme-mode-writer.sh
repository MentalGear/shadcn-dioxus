#!/usr/bin/env bash
#
# check-theme-mode-writer.sh
#
# The light/dark MODE has exactly one writer: `apply_mode` inside `PREPAINT_JS` (preview/src/theme.rs),
# embedded byte-identically in preview/index.html's <head>. Any other code that writes `data-theme` or
# the `dark` class on <html> fails here. Invisible to cargo and clippy, and with CI frozen
# (dev-docs/backlog.md row 33) nothing else would notice.
#
# THE CLASS (dev-docs/backlog.md row 158 (a)): the mode is spelled twice -- shadcn's `.dark` class
# (next-themes, Tailwind `dark:`, a pasted theme's `.dark { ... }` block) and our `data-theme="dark|light"`
# -- and `dx-components-theme.css` treats them as one selector list. That is only sound if the two can
# never disagree, and the only construction that guarantees it is a single writer that sets both together
# (and resolves "system" from `prefers-color-scheme`, following it live). A second writer -- a component
# that flips the attribute for a demo, a script that adds the class, an rsx `html { "data-theme": .. }` --
# re-opens the disagreement: a pasted shadcn `.dark` block would activate with the light tokens, or the
# reverse. This script keeps a second writer from coming back.
#
# WHAT COUNTS AS A WRITER (shipped code only: `*.rs`, `*.js`, `*.ts`, `*.html` under preview/src,
# preview/assets, primitives/src, templates, plus preview/index.html; `//` comment lines skipped):
#   - `setAttribute|removeAttribute|toggleAttribute('data-theme'`   - `dataset.theme =` / `dataset['theme'] =`
#   - an rsx/HTML attribute `"data-theme":` / `data-theme=` on any element
#   - `classList.add|remove|toggle|replace('dark'`, and `className =` / `setAttribute('class'` on `documentElement`
#   - a literal `class="dark"` / `class: "dark"`
# The ONE sanctioned place is the `PREPAINT_JS` line of preview/src/theme.rs and its copy in
# preview/index.html. The script also requires that both exist, that they are byte-identical, and that
# each contains `apply_mode`'s writes -- so the allowance cannot go stale or be emptied unnoticed.
# Playwright specs are out of scope: they drive the page through `window.__dxApplyMode`.
#
# NOT covered, said plainly: a write whose attribute name is built at run time (`'data-'+k` -- which is how
# PREPAINT_JS itself writes the other preferences), and a vendored script.
#
# ALLOW: (file, regex on the offending line, reason) for a write that is deliberately NOT the app's mode
# (e.g. an isolated demo document that is its own <html>). Every entry needs a reason and must match at
# least one offending line, or the run fails as stale.
#
# Usage: scripts/check-theme-mode-writer.sh [root]   (root defaults to the repo; a copy can be scanned to
#        prove the gate red)
# Exit 0: clean. Exit 1: a violation, with file:line detail on stderr.

set -euo pipefail

root="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
cd "$root"

python3 - <<'PYEOF'
import re
import sys
from pathlib import Path

# (file, regex on the offending line, reason). Empty today: every write that is not apply_mode is a bug.
ALLOW = []

SCAN_DIRS = ("preview/src", "preview/assets", "primitives/src", "templates")
EXTS = {".rs", ".js", ".ts", ".html"}
SKIP_PARTS = {"node_modules", "target", ".git"}

THEME_RS = "preview/src/theme.rs"
INDEX_HTML = "preview/index.html"

WRITERS = [
    (re.compile(r"""(?:set|remove|toggle)Attribute\(\s*['"]data-theme['"]"""), "writes the data-theme attribute"),
    (re.compile(r"""dataset(?:\.theme|\[\s*['"]theme['"]\s*\])\s*=(?!=)"""), "assigns dataset.theme"),
    (re.compile(r"""['"]data-theme['"]\s*:"""), "sets a data-theme attribute in rsx/object form"),
    (re.compile(r"""(?<![\w-])data-theme\s*=\s*['"\\]"""), "sets data-theme in markup"),
    (re.compile(r"""classList\s*\.\s*(?:add|remove|toggle|replace)\(\s*['"]dark['"]"""), "writes the `dark` class"),
    (re.compile(r"""documentElement\s*\.\s*className\s*=(?!=)"""), "assigns <html>'s className"),
    (re.compile(r"""documentElement\s*\.\s*setAttribute\(\s*['"]class['"]"""), "assigns <html>'s class attribute"),
    (re.compile(r"""(?<![\w-])class\s*[=:]\s*\\?['"]dark\\?['"]"""), "hard-codes class=\"dark\""),
]

errors = []
files = []
for d in SCAN_DIRS:
    base = Path(d)
    if not base.is_dir():
        continue
    for p in sorted(base.rglob("*")):
        if p.is_file() and p.suffix in EXTS and not (SKIP_PARTS & set(p.parts)):
            files.append(p)
if Path(INDEX_HTML).is_file():
    files.append(Path(INDEX_HTML))

def allowed_line(path: str, line: str) -> bool:
    return path == THEME_RS and line.lstrip().startswith("pub const PREPAINT_JS") or \
        path == INDEX_HTML and line.lstrip().startswith("<script>(function(){var K=[")

used = [0] * len(ALLOW)
n_lines = 0
for p in files:
    rel = p.as_posix()
    for no, line in enumerate(p.read_text(errors="replace").splitlines(), 1):
        n_lines += 1
        if line.lstrip().startswith(("//", "<!--", "*", "/*")):
            continue
        hit = next((why for rx, why in WRITERS if rx.search(line)), None)
        if hit is None or allowed_line(rel, line):
            continue
        k = next((i for i, (f, rx, _) in enumerate(ALLOW) if f == rel and re.search(rx, line)), None)
        if k is not None:
            used[k] += 1
            continue
        errors.append(
            f"{rel}:{no}: {hit} -- the mode has ONE writer, `apply_mode` in PREPAINT_JS "
            f"(preview/src/theme.rs, copied into preview/index.html); call `window.__dxApplyMode(mode)` / "
            f"`theme::set_theme` instead, or allowlist it in scripts/check-theme-mode-writer.sh with a reason"
        )

for (f, rx, reason), k in zip(ALLOW, used):
    if not reason.strip():
        errors.append(f"allowlist entry ({f}, {rx}) has no reason")
    if not k:
        errors.append(f"stale allowlist entry ({f}, {rx}): no offending line matches it any more -- remove it")

# The sanctioned writer must exist, in both places, identical.
def prepaint(path, pat):
    try:
        text = Path(path).read_text()
    except OSError:
        return None
    m = re.search(pat, text)
    return m.group(1) if m else None

rs = prepaint(THEME_RS, r'pub const PREPAINT_JS: &str = "(.*)";')
html = prepaint(INDEX_HTML, r"<script>(\(function\(\)\{var K=.*?)</script>")
for name, body in ((THEME_RS, rs), (INDEX_HTML, html)):
    if body is None:
        errors.append(f"{name}: the PREPAINT_JS writer was not found -- the scan in this script is broken or the writer moved")
    elif not ("setAttribute('data-theme'" in body and "classList.toggle('dark'" in body):
        errors.append(f"{name}: PREPAINT_JS no longer sets both data-theme and the dark class together")
if rs is not None and html is not None and rs != html:
    errors.append(f"{THEME_RS} PREPAINT_JS and {INDEX_HTML}'s script differ (they must be byte-identical)")

if not files or not n_lines:
    errors.append("no source file was scanned -- the scan in this script is broken")

if errors:
    for e in errors:
        print(f"check-theme-mode-writer: {e}", file=sys.stderr)
    print(f"check-theme-mode-writer: FAILED -- {len(errors)} problem(s)", file=sys.stderr)
    sys.exit(1)
print(
    f"check-theme-mode-writer: OK -- {len(files)} files, {n_lines} lines: no writer of data-theme or the `dark` "
    f"class outside PREPAINT_JS ({len(ALLOW)} allowlist entries)"
)
PYEOF
