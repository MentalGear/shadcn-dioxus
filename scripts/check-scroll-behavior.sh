#!/usr/bin/env bash
#
# check-scroll-behavior.sh
#
# Smooth scrolling is opt-in per interaction, never ambient: nothing may declare `scroll-behavior: smooth` on the
# document's own scroller (`html`, `:root`, `body`, `*`, or a bare attribute/pseudo compound that can match
# them) unless the same selector carries `[data-anchor-scroll]`. Invisible to cargo and clippy, and with CI frozen
# (dev-docs/backlog.md row 33) nothing else would notice.
#
# THE CLASS (row 228, the owner's finding "clicking a demo link from the overview navigates to the component page,
# but to its bottom"): `html { scroll-behavior: smooth }` made EVERY window scroll that names no `behavior`
# animate, not only the in-page `#`-anchor jump it was written for. The router's `scrollTo(0, 0)` after a push
# started a smooth scroll on the long page, the short destination page replaced it mid-animation, the browser
# clamped the offset to the new page's bottom, and the visitor saw the BOTTOM of the component page and then
# watched it glide up (Back is the mirror: the list renders at the top and glides to the saved offset). The same
# ambient rule had already forced a per-call `behavior: 'instant'` into the charts tab restore and would have hit
# the scroll-lock restore (`primitives/src/scroll_lock.rs`) and every future `window.scrollTo`. The construction:
# the document is instant by default; `preview/assets/main.css` has ONE smooth rule,
# `html[data-anchor-scroll] { scroll-behavior: smooth }`, and `preview/index.html`'s same-page `a[href^="#"]` click
# handler sets that attribute for exactly the jump it lets the browser animate and clears it when the jump settles.
#
# WHAT IT SCANS: every `*.css`, `*.rs` and `*.html` under `preview/src`, `preview/assets`, `preview/index.html`,
# `primitives/src` and `templates` (node_modules and build output skipped), comments stripped (newlines kept).
#   1. a rule whose body declares `scroll-behavior: smooth` and whose selector list has an entry that can match
#      the root scroller, with no `[data-anchor-scroll]` on that entry, fails. An entry "can match the root" when
#      its last compound has the tag `html`/`body`, is `:root` or `*`, or has no class, id or tag at all (a bare
#      `[data-theme]` or `:where(...)` compound matches `html` too). An entry whose last compound has a class or
#      id is a component's own scroll container (a scroll area, a carousel) and is fine: `scroll-behavior` is not
#      inherited, so it never reaches the document.
#   2. a script write to the document's scroller (`documentElement`/`body` `.style.scrollBehavior = ...`,
#      `setProperty('scroll-behavior', ...)`) fails.
# ALLOW format: (path, fnmatch pattern on the whitespace-normalised selector entry or the script line, reason).
# An entry that matches nothing fails the run as stale, so the list cannot rot into blanket permission.
#
# The browser half is `playwright/nav-scroll-instant.spec.ts`; it must not run through a fixture that forces
# `scroll-behavior: auto !important`, which would mask the very bug it guards.
#
# Usage: scripts/check-scroll-behavior.sh [FILE ...]   (explicit files are for self-tests; the positive checks that
# main.css still has the opt-in rule and index.html still sets it run only on the default scan)
# Exit status: 0 when clean.

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

python3 - "$@" <<'PYEOF'
import fnmatch
import re
import sys
from pathlib import Path

ALLOW = [
    # (path, fnmatch pattern, reason) -- none today: no stylesheet needs ambient smooth scrolling.
]

explicit = [Path(a) for a in sys.argv[1:]]
ROOTS = ["preview/src", "preview/assets", "primitives/src", "templates"]
SKIP = {"node_modules", "target", "public", "dist", ".git"}

def files():
    if explicit:
        return explicit
    out = [Path("preview/index.html")]
    for root in ROOTS:
        for p in sorted(Path(root).rglob("*")):
            if p.suffix in (".css", ".rs", ".html") and not (SKIP & set(p.parts)):
                out.append(p)
    return [p for p in out if p.exists()]

COMMENT = re.compile(r"/\*.*?\*/", re.S)
HTML_COMMENT = re.compile(r"<!--.*?-->", re.S)
RS_LINE_COMMENT = re.compile(r"^[ \t]*//[^\n]*", re.M)   # `//`, `///`, `//!` lines: prose that names the property
def blank(text):
    for rx in (COMMENT, HTML_COMMENT, RS_LINE_COMMENT):
        text = rx.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)
    return text

SMOOTH = re.compile(r"(?<![\w-])scroll-behavior\s*:\s*smooth\b", re.I)
RULE = re.compile(r"([^{}]*)\{([^{}]*)\}")
JS_WRITE = re.compile(
    r"(documentElement|document\.body)\s*\.\s*style\s*\.\s*scrollBehavior\s*=|"
    r"(documentElement|document\.body)\s*\.\s*style\s*\.\s*setProperty\(\s*['\"]scroll-behavior['\"]", re.I)

def split_top(s, sep=","):
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch in "([":
            depth += 1
        elif ch in ")]":
            depth -= 1
        if ch == sep and depth == 0:
            out.append(cur); cur = ""
        else:
            cur += ch
    out.append(cur)
    return [e.strip() for e in out if e.strip()]

def last_compound(entry):
    depth, start = 0, 0
    for i, ch in enumerate(entry):
        if ch in "([":
            depth += 1
        elif ch in ")]":
            depth -= 1
        elif depth == 0 and ch in " >+~":
            start = i + 1
    return entry[start:].strip()

def can_match_root(entry):
    if "[data-anchor-scroll]" in entry:
        return False
    c = re.sub(r"::?[\w-]+(\((?:[^()]|\([^()]*\))*\))?", lambda m: m.group(0) if m.group(0).startswith(":where(") or m.group(0).startswith(":root") else "", last_compound(entry))
    c = c.strip()
    if re.match(r"^(html|body|\*|:root)\b", c) or c in ("", "html", "body", "*"):
        return True
    # a compound with a class, an id or any other tag cannot be the root element
    if re.search(r"[.#]", c) or re.match(r"^[a-z][\w-]*", c):
        return False
    return True   # attribute/pseudo only, e.g. `[data-theme="dark"]`

errors, used = [], set()

def allowed(path, text):
    for i, (p, pat, _reason) in enumerate(ALLOW):
        if p == path and fnmatch.fnmatch(text, pat):
            used.add(i)
            return True
    return False

scanned = 0
for path in files():
    scanned += 1
    raw = path.read_text(encoding="utf-8", errors="replace")
    text = blank(raw)
    for m in RULE.finditer(text):
        if not SMOOTH.search(m.group(2)):
            continue
        line = text.count("\n", 0, m.start(2)) + 1
        for entry in split_top(m.group(1)):
            norm = " ".join(entry.split())
            if can_match_root(norm) and not allowed(str(path), norm):
                errors.append(f"{path}:{line}: `{norm}` declares `scroll-behavior: smooth` and can match the "
                              f"document's scroller; smooth scrolling is opt-in, `html[data-anchor-scroll]` only "
                              f"(preview/index.html sets it for one same-page anchor jump)")
    if path.suffix != ".css":
        for m in JS_WRITE.finditer(text):
            line = text.count("\n", 0, m.start()) + 1
            src = " ".join(text.splitlines()[line - 1].split())
            if not allowed(str(path), src):
                errors.append(f"{path}:{line}: writes the document's `scroll-behavior` from a script; use the "
                              f"`data-anchor-scroll` opt-in instead")

if not explicit:
    css = Path("preview/assets/main.css").read_text(encoding="utf-8")
    if not re.search(r"html\[data-anchor-scroll\]\s*\{[^{}]*scroll-behavior\s*:\s*smooth", blank(css)):
        errors.append("preview/assets/main.css: the opt-in rule `html[data-anchor-scroll] { scroll-behavior: smooth }` is gone "
                      "(the `/docs` anchor jump would be an instant cut)")
    html = blank(Path("preview/index.html").read_text(encoding="utf-8"))
    if "data-anchor-scroll" not in html:
        errors.append("preview/index.html: the same-page anchor handler no longer sets `data-anchor-scroll`")
    for i, (p, pat, reason) in enumerate(ALLOW):
        if i not in used:
            errors.append(f"stale ALLOW entry: ({p}, {pat}) matched nothing -- delete it")

if errors:
    for e in errors:
        print(e)
    print(f"\ncheck-scroll-behavior: {len(errors)} problem(s). See the header of scripts/check-scroll-behavior.sh.",
          file=sys.stderr)
    sys.exit(1)
print(f"check-scroll-behavior: OK ({scanned} file(s): no ambient `scroll-behavior: smooth` on the document)")
PYEOF
