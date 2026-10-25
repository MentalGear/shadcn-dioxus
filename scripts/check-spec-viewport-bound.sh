#!/usr/bin/env bash
#
# check-spec-viewport-bound.sh
#
# A Playwright spec that asserts "this overlay / panel / menu / sheet stays inside the viewport" (or
# sits flush against its edge) must bound it with `fixedViewportBox(page)` (`playwright/viewport-box.ts`:
# a temporary `position: fixed; inset: 0` probe), never with `documentElement.clientWidth` / `clientHeight`,
# `window.innerWidth` / `innerHeight` or `page.viewportSize()`. The preview sets
# `html { scrollbar-gutter: stable }` (preview/assets/main.css), so those numbers include the reserved
# gutter (390 at a 390px phone) while a fixed overlay's containing block excludes it (375): a panel
# landing 11-15px past the right edge still passes, which masks an overflow of the kind
# `primitives/src/top_layer/scroll_clamp.rs`'s `viewportMetrics()` had.
#
# This gate fails when a spec (every `playwright/**/*.ts` except `node_modules`, `fixtures/` and
# `oracle/reference/`), comments stripped, names one of the gutter-inclusive sources:
#
#   `innerWidth` / `innerHeight` (bare or `window.`-prefixed),
#   `<documentElement|scrollingElement|de|html|docEl>.clientWidth` / `.clientHeight`,
#   `.viewportSize()`,
#
# unless the line, or the line directly above it, carries `viewport-ok: <reason>` (a non-empty reason),
# or a reasoned ALLOW entry below covers it. The source is flagged, not the comparison: the bound is
# usually read on one line (`viewport: document.documentElement.clientWidth`) and compared on another
# (`expect(r.right).toBeLessThanOrEqual(m.viewport)`), so no same-line pattern would catch it.
# `el.clientWidth` of an ordinary element (a scroll container) is a different measurement and is not
# matched.
#
# Legitimate uses are the ones that are NOT an overlay-containment bound: page-level horizontal overflow
# (`scrollWidth` vs the document's own `clientWidth`), scroll extents (`scrollHeight - innerHeight`),
# the gutter itself (`innerWidth - clientWidth`), an `elementFromPoint` coordinate domain, a size
# fraction. Say which with the marker.
#
# EXCEPTIONS are in ALLOW below, each with its reason (a needle of '' covers every hit in the file); an
# entry that matches nothing fails the run, so it is deleted when its use goes away.
#
# Usage: scripts/check-spec-viewport-bound.sh [FILE ...]   (default: every spec under playwright/;
# explicit files are for self-tests, and skip the allowlist stale-entry report)
# Exit status: 0 when clean.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import os
import re
import sys

explicit = sys.argv[1:]
SKIP_DIRS = {"node_modules", "fixtures", "reference"}

def collect():
    out = []
    for root, dirs, names in os.walk("playwright"):
        dirs[:] = sorted(d for d in dirs if d not in SKIP_DIRS)
        out += [os.path.join(root, n) for n in sorted(names) if n.endswith(".ts")]
    return out

files = explicit or collect()
if not files:
    print("check-spec-viewport-bound: no specs found (run from a checkout)", file=sys.stderr)
    sys.exit(2)

# (file suffix, substring of the code line ('' = every hit in the file), reason)
ALLOW = [
    ("popover-non-modal.spec.ts", "", "scroll-height arithmetic and the 'no scrollbar-gutter shift on open' clientWidth stability check; no overlay bound"),
    ("component-catalog.spec.ts", "documentElement.clientWidth", "page-level horizontal overflow: scrollWidth <= clientWidth of the same document"),
    ("charts_gallery.spec.ts", "documentElement.clientWidth", "page-level horizontal overflow: scrollWidth - clientWidth of the same document"),
    ("theme-picker-phone.spec.ts", "", "page-level horizontal overflow and the exact-viewport-width assertion (innerWidth is what setViewportSize set); no overlay bound"),
    ("computed-style-states.ts", "", "scroll-position identity key and click-point clamp (elementFromPoint coordinate domain), not a containment bound"),
    ("select.spec.ts", "viewportSize()", "size fraction (listbox narrower than 0.6 x viewport), not an edge containment check"),
    ("dropdown-menu.spec.ts", "viewportSize()", "size fraction (menu narrower than 0.6 x viewport), not an edge containment check"),
    ("context-menu.spec.ts", "viewportSize()", "tap point 30px inside the corner, an elementFromPoint coordinate far from any edge; no containment bound"),
    ("oracle/tier2-html/native-dialog.spec.ts", "", "'not viewport-centred' discriminator with a hundreds-of-pixels margin; the ~7px gutter is irrelevant to it"),
    ("oracle/tier2-html/top-layer.spec.ts", "", "elementFromPoint candidate filter: that API's coordinate domain is [0, innerWidth) x [0, innerHeight)"),
    ("oracle/tier2-html/anchor-landing.spec.ts", "", "scroll extent (scrollHeight - innerHeight), not a containment bound"),
    ("oracle/tier2-html/home-scroll-stability.spec.ts", "", "scroll extent / viewport height for scroll-position stability"),
    ("oracle/tier2-html/motion-gating.spec.ts", "", "page-relative offset and viewport height for the 'is it on screen' scroll, not a containment bound"),
    ("oracle/tier2-html/demo-placeholder-links.spec.ts", "", "scroll room (scrollHeight - innerHeight - scrollY), not a containment bound"),
    ("oracle/tier2-html/scroll-main-thread.spec.ts", "", "scroll extent (scrollHeight - innerHeight), not a containment bound"),
    ("oracle/tier2-html/listener-inventory.spec.ts", "", "measures the scrollbar gutter itself (innerWidth - clientWidth) and its stability across frames"),
    ("oracle/tier3-radix/scroll-lock.spec.ts", "", "measures clientWidth stability / the scrollbar gutter and scroll extents; the subject is the gutter, not an overlay bound"),
]
used = set()

COMMENT_BLOCK = re.compile(r"/\*.*?\*/", re.S)
COMMENT_LINE = re.compile(r"(?<![:\"'\w])//[^\n]*")

def blank(m):
    return re.sub(r"[^\n]", " ", m.group(0))

SOURCES = [
    (re.compile(r"(?<![\w.$])(?:window\s*\.\s*)?inner(?:Width|Height)\b|\bwindow\s*\.\s*inner(?:Width|Height)\b"), "innerWidth/innerHeight"),
    (re.compile(r"\b(?:documentElement|scrollingElement|de|html|docEl)\s*\.\s*client(?:Width|Height)\b"), "documentElement.clientWidth/clientHeight"),
    (re.compile(r"\.\s*viewportSize\s*\(\s*\)"), "page.viewportSize()"),
]
MARKER = re.compile(r"viewport-ok:\s*\S")

errors = []
total = 0

for path in files:
    with open(path, encoding="utf-8") as fh:
        raw = fh.read()
    raw_lines = raw.split("\n")
    code = COMMENT_LINE.sub(blank, COMMENT_BLOCK.sub(blank, raw)).split("\n")
    for i, line in enumerate(code):
        hit = next((label for rx, label in SOURCES if rx.search(line)), None)
        if hit is None:
            continue
        total += 1
        if MARKER.search(raw_lines[i]) or (i > 0 and MARKER.search(raw_lines[i - 1])):
            continue
        ok = False
        for entry in ALLOW:
            suffix, needle, _reason = entry
            if path.endswith(suffix) and needle in line:
                used.add(id(entry))
                ok = True
        if ok:
            continue
        errors.append(f"{path}:{i + 1}: `{line.strip()[:90]}` reads {hit}, which includes the `scrollbar-gutter: stable` "
                      f"strip a fixed overlay is NOT placed in")

if errors:
    for e in errors:
        print(e)
    print(f"\ncheck-spec-viewport-bound: {len(errors)} problem(s). Bound an overlay with `await fixedViewportBox(page)` "
          f"(playwright/viewport-box.ts); if the number measures something else (page overflow, scroll extent, the gutter), "
          f"add `// viewport-ok: <reason>` on that line or the line above.", file=sys.stderr)
    sys.exit(1)

stale = [e for e in ALLOW if id(e) not in used]
if stale and not explicit:
    for suffix, needle, reason in stale:
        print(f"FAIL: stale allowlist entry {suffix} {needle!r} -- {reason}")
    sys.exit(1)
print(f"check-spec-viewport-bound: OK ({len(files)} spec file(s): {total} gutter-inclusive viewport read(s), each marked or allowlisted "
      f"as not an overlay bound; {len(used)} allowlist entr{'y' if len(used) == 1 else 'ies'} in use)")
PYEOF
