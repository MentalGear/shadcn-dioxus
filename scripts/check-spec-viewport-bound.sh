#!/usr/bin/env bash
#
# check-spec-viewport-bound.sh  (backlog row 427)
#
# A Playwright spec reads the viewport and the page's own numbers through two helpers in `playwright/viewport-box.ts`,
# and never through the raw gutter-inclusive sources: an overlay's bound is `fixedViewportBox(page)` (a temporary
# `position: fixed; inset: 0` probe), and every other number (a scroll extent, sideways overflow, the gutter, the size a
# test restores, an `elementFromPoint` coordinate domain) is `pageMetrics(page)`, which names it. The preview sets
# `html { scrollbar-gutter: stable }` (preview/assets/main.css), so `documentElement.clientWidth` / `clientHeight`,
# `window.innerWidth` / `innerHeight` and `page.viewportSize()` include the reserved gutter (390 at a 390px phone) while
# a fixed overlay's containing block excludes it (375): a panel landing 11-15px past the right edge still passes, which
# masks an overflow of the kind `primitives/src/top_layer/scroll_clamp.rs`'s `viewportMetrics()` had.
#
# This gate fails when a spec (every `playwright/**/*.ts` except `node_modules`, `fixtures/`, `oracle/reference/` and
# `viewport-box.ts`, where the helpers live), comments stripped, names one of the gutter-inclusive sources:
#
#   `innerWidth` / `innerHeight` (bare or `window.`-prefixed),
#   `<documentElement|scrollingElement|de|html|docEl>.clientWidth` / `.clientHeight`,
#   `.viewportSize()`,
#
# whatever a comment says (the `viewport-ok:` marker is retired). Code that runs inside `page.evaluate` takes the number it
# needs as an argument read with `pageMetrics`. The one exception is code that lives IN the page and runs later on its own
# (an init script's stub, a requestAnimationFrame sampler, a probe function installed once): a node-side helper cannot
# reach it, so it sits on ALLOW below with its reason, through the shared allowlist engine (`scripts/lib/allow.py`: a
# blank reason, a duplicate and a stale entry fail). `--self-test` (also run by every default run) turns each source into
# a bare read and requires red, and requires the two helpers to be exported by `viewport-box.ts`.
#
# Usage: scripts/check-spec-viewport-bound.sh [FILE ...]   (default: every spec under playwright/, then the self-test;
# --self-test runs only the self-test; explicit files skip the allowlist stale-entry report)
# Exit status: 0 when clean.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
self="$repo_root/scripts/check-spec-viewport-bound.sh"

self_test() {
  local dir fails=0 n=0 name
  dir="$(mktemp -d)"
  trap 'rm -rf "$dir"' RETURN
  expect() {  # expect <0|1> <name> <body>
    local want="$1" name="$2" body="$3" rc=0
    printf '%s\n' "$body" >"$dir/$name.spec.ts"
    bash "$self" "$dir/$name.spec.ts" >/dev/null 2>&1 || rc=$?
    n=$((n + 1))
    if [ "$rc" -ne "$want" ]; then
      echo "check-spec-viewport-bound --self-test: FAIL $name: exit $rc, wanted $want" >&2
      fails=$((fails + 1))
    fi
  }
  expect 0 helper-fixed-viewport-box "const box = await fixedViewportBox(page);"
  expect 0 helper-page-metrics "const m = await pageMetrics(page); const x = m.scrollExtent + m.sidewaysOverflow + m.gutter;"
  expect 0 helper-page-metrics-viewport "const v = (await pageMetrics(page)).viewportSize;"
  expect 1 bare-inner-width "const w = await page.evaluate(() => window.innerWidth);"
  expect 1 bare-inner-height "const h = await page.evaluate(() => innerHeight);"
  expect 1 bare-client-width "const w = await page.evaluate(() => document.documentElement.clientWidth);"
  expect 1 bare-client-height "const h = await page.evaluate(() => document.scrollingElement.clientHeight);"
  expect 1 bare-viewport-size "const v = page.viewportSize();"
  expect 1 marker-retired $'// viewport-ok: page scroll extent\nconst h = await page.evaluate(() => window.innerHeight);'
  expect 1 marker-retired-same-line "const v = page.viewportSize(); // viewport-ok: restoring the size"
  [ "$fails" -eq 0 ] || exit 1
  for name in fixedViewportBox pageMetrics; do
    if ! grep -Eq "export[[:space:]]+(async[[:space:]]+)?function[[:space:]]+$name\b" "$repo_root/playwright/viewport-box.ts"; then
      echo "check-spec-viewport-bound --self-test: playwright/viewport-box.ts does not export $name" >&2
      exit 1
    fi
  done
  echo "check-spec-viewport-bound --self-test: OK ($n fixtures; the helpers are green, every raw source is red, with or without a viewport-ok marker)"
}

if [ "${1:-}" = "--self-test" ]; then self_test; exit $?; fi

python3 - "$@" <<'PYEOF'
import os
import re
import sys

sys.path.insert(0, "scripts/lib")
from allow import Allowlist  # noqa: E402

explicit = sys.argv[1:]
SKIP_DIRS = {"node_modules", "fixtures", "reference"}

def collect():
    out = []
    for root, dirs, names in os.walk("playwright"):
        dirs[:] = sorted(d for d in dirs if d not in SKIP_DIRS)
        out += [os.path.join(root, n) for n in sorted(names) if n.endswith(".ts") and n != "viewport-box.ts"]
    return out

files = explicit or collect()
if not files:
    print("check-spec-viewport-bound: no specs found (run from a checkout)", file=sys.stderr)
    sys.exit(2)

# (file suffix, substring of the code line ('' = every hit in the file), reason): code that lives IN the page and runs
# later on its own, which a node-side helper cannot reach.
ALLOW = Allowlist("check-spec-viewport-bound", [
    ("keyboard-inset.spec.ts", "",
     "the stub of the visual viewport an init script installs: its getters run later, inside the page, against the live layout viewport"),
    ("chart-entry-animation.spec.ts", "r.top < innerHeight",
     "the recorder installed with addInitScript samples on every animation frame inside the page"),
    ("computed-style-states.ts", "window.inner",
     "probe functions installed once in the page (the scroll identity key, the click-point clamp) that run there on every call"),
    ("oracle/tier2-html/listener-inventory.spec.ts", "document.documentElement.clientWidth",
     "a requestAnimationFrame sampler inside the page records clientWidth on every frame"),
], fields=("suffix", "needle"))

COMMENT_BLOCK = re.compile(r"/\*.*?\*/", re.S)
COMMENT_LINE = re.compile(r"(?<![:\"'\w])//[^\n]*")

def blank(m):
    return re.sub(r"[^\n]", " ", m.group(0))

SOURCES = [
    (re.compile(r"(?<![\w.$])(?:window\s*\.\s*)?inner(?:Width|Height)\b|\bwindow\s*\.\s*inner(?:Width|Height)\b"), "innerWidth/innerHeight"),
    (re.compile(r"\b(?:documentElement|scrollingElement|de|html|docEl)\s*\.\s*client(?:Width|Height)\b"), "documentElement.clientWidth/clientHeight"),
    (re.compile(r"\.\s*viewportSize\s*\(\s*\)"), "page.viewportSize()"),
]

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
        if ALLOW.find_all(lambda e: path.endswith(e.suffix) and e.needle in line):
            continue
        errors.append(f"{path}:{i + 1}: `{line.strip()[:90]}` reads {hit}, which includes the `scrollbar-gutter: stable` "
                      f"strip a fixed overlay is NOT placed in")

if errors:
    for e in errors:
        print(e)
    print(f"\ncheck-spec-viewport-bound: {len(errors)} problem(s). Bound an overlay with `await fixedViewportBox(page)`; read "
          f"any other page number (page overflow, scroll extent, the gutter, the viewport size) with `await pageMetrics(page)` "
          f"(playwright/viewport-box.ts) and pass it into `page.evaluate` as an argument.", file=sys.stderr)
    sys.exit(1)

problems = ALLOW.problems(stale=not explicit)
if problems:
    for p in problems:
        print(f"FAIL: {p}")
    sys.exit(1)
print(f"check-spec-viewport-bound: OK ({len(files)} spec file(s): {total} gutter-inclusive read(s), all inside the page and allowlisted; "
      f"{len(ALLOW)} allowlist entr{'y' if len(ALLOW) == 1 else 'ies'})")
PYEOF

if [ "$#" -eq 0 ]; then self_test; fi
