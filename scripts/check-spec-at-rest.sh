#!/usr/bin/env bash
#
# check-spec-at-rest.sh   (backlog rows 222, 427)
#
# A spec that reads the GEOMETRY of an element must say whether it animates, in code. `boundingBox()` and
# `getBoundingClientRect()` include transforms, and every floating panel, dialog, sheet, drawer, select listbox ... plays an
# enter animation (`dx-pop-in` scales 0.95 -> 1, a dialog and a sheet move in), so a read before the animation settles is a
# race against the animation clock.
#
# Measured (row 222): a select listbox reads 142.16 px wide mid-animation against 145.89 at rest, and a dialog footer band
# compared with the dialog during the 200 ms enter is off by 1.25-1.86 px (1 run in 5).
#
# Two kinds of read, one helper family in `playwright/animations.ts`: an element that animates on enter is read with
# `measureAtRest(locator)` / `readAtRest(locator, fn)` (they await `awaitAnimationsSettled`: the element's, its
# descendants' AND its ancestors' animations; the footer sits inside the animating dialog); an element that does NOT
# animate (a page button that triggers an overlay, the navbar, a static row or label, a probe) is read with
# `staticBox(locator)` / `readStatic(locator, fn)`, which CHECK that no keyframe animation runs on the element or an
# ancestor and throw if one does, so "does not animate" is a verified fact rather than a comment.
#
# This gate fails when a spec in scope reads a box without one of them:
#
#   `.boundingBox(`  or  `getBoundingClientRect(`  (comments stripped),
#
# unless the read sits inside a `measureAtRest(...)` / `readAtRest(...)` / `readStatic(...)` call (`staticBox(...)` has no
# read to see), or the line, or the line directly above it, carries `rest-ok: <reason>` (a non-empty reason, for the
# reads no helper can wrap: a sampler that measures the motion itself frame by frame, a drag driver that places the
# pointer inside its own evaluate on a drawer the caller settled, and an in-page read settled by a named earlier step in
# the same evaluate), or a reasoned ALLOW entry covers it. The four helpers must be exported by
# `animations.ts`, and `--self-test` (also run by every default run) requires each to be green and the bare read it
# replaces to be red.
#
# SCOPE is derived, not listed, so a new overlay cannot dodge it: a component is "animated" when its
# `preview/src/components/<c>/style.css` animates a `[data-state=open|closed]` rule, or its sources emit
# `data-dx-surface` (the theme's floating-surface enter/exit animation), and every
# `playwright/<c>.spec.ts` / `<c>-*.spec.ts` (with `_` and `-` interchangeable) is in scope. EXTRA_SPECS adds the
# cross-component specs that open overlays and measure them.
#
# EXCEPTIONS are in ALLOW below (the shared allowlist engine, `scripts/lib/allow.py`), each with its reason (a needle of ''
# covers every hit in the file); an entry that matches nothing fails the run, so it is deleted when its use goes away.
#
# Usage: scripts/check-spec-at-rest.sh [FILE ...]   (default: every spec in scope, then the self-test; --self-test runs
# only the self-test; explicit files skip the allowlist stale-entry report)
# Exit status: 0 when clean.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
self="$repo_root/scripts/check-spec-at-rest.sh"

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
      echo "check-spec-at-rest --self-test: FAIL $name: exit $rc, wanted $want" >&2
      fails=$((fails + 1))
    fi
  }
  expect 0 helper-measure-at-rest "const b = await measureAtRest(panel);"
  expect 0 helper-read-at-rest "const w = await readAtRest(panel, (el) => el.getBoundingClientRect().width);"
  expect 0 helper-read-static "const h = await readStatic(trigger, (el) => el.getBoundingClientRect().height);"
  expect 0 helper-static-box "const b = await staticBox(label);"
  expect 1 bare-bounding-box "const b = await panel.boundingBox();"
  expect 1 bare-in-page-rect "const w = await panel.evaluate((el) => el.getBoundingClientRect().width);"
  expect 1 bare-rect-for-read-static "const h = await trigger.evaluate((el) => el.getBoundingClientRect().height);"
  expect 1 read-after-helper-call "await measureAtRest(a); const b = await panel.boundingBox();"
  expect 0 marker-sampler $'// rest-ok: the sampler measures motion itself\nconst r = el.getBoundingClientRect();'
  expect 1 marker-without-reason $'// rest-ok:\nconst r = el.getBoundingClientRect();'
  [ "$fails" -eq 0 ] || exit 1
  for name in measureAtRest readAtRest staticBox readStatic; do
    if ! grep -Eq "export[[:space:]]+(async[[:space:]]+)?function[[:space:]]+$name\b" "$repo_root/playwright/animations.ts"; then
      echo "check-spec-at-rest --self-test: playwright/animations.ts does not export $name" >&2
      exit 1
    fi
  done
  echo "check-spec-at-rest --self-test: OK ($n fixtures; each helper is green, the bare read it replaces is red, a reasonless marker is red)"
}

if [ "${1:-}" = "--self-test" ]; then self_test; exit $?; fi

python3 - "$@" <<'PYEOF'
import os
import re
import sys

sys.path.insert(0, "scripts/lib")
from allow import Allowlist  # noqa: E402
from css import rules as css_rules  # noqa: E402

explicit = sys.argv[1:]

# Cross-component specs that open overlays and measure them (path under playwright/).
EXTRA_SPECS = [
    "oracle/tier2-html/top-layer.spec.ts",
    "oracle/tier2-html/top-layer-ink.spec.ts",
    "oracle/tier2-html/native-dialog.spec.ts",
    "oracle/tier2-html/main-thread.spec.ts",
    "oracle/tier2-html/auto-popover-opening-gesture.spec.ts",
    "oracle/tier3-radix/scroll-lock.spec.ts",
    "oracle/tier3-radix/rtl.spec.ts",
    "oracle/tier1-apg/menu-submenu.spec.ts",
    "popup-style-isolation.spec.ts",
    "overlay-switch.spec.ts",
    "modal-exit.spec.ts",
    "modal-initial-focus.spec.ts",
    "toast.spec.ts",        # toasts slide in (an animation keyed on the container, not a data-state rule)
    "date-picker.spec.ts",  # the calendar popup animates through the date picker's own sheet
]

# (file suffix, substring of the code line ('' = every hit in the file), reason)
ALLOW = Allowlist("check-spec-at-rest", [
], fields=("suffix", "needle"))

def animated_components():
    """Components whose surface plays an enter/exit animation: the sheet animates a `[data-state=open|closed]`
    rule, or the sources emit `data-dx-surface` (the theme's floating-surface animation). Animations on other
    states (an avatar's loading shimmer, a chart's draw-in, a progress bar's indeterminate sweep) are not
    overlay surfaces and not this gate's subject."""
    out = set()
    base = "preview/src/components"
    for c in sorted(os.listdir(base)):
        d = os.path.join(base, c)
        if not os.path.isdir(d):
            continue
        css = os.path.join(d, "style.css")
        hit = False
        if os.path.exists(css):
            with open(css, encoding="utf-8") as fh:
                sheet_rules = css_rules(fh.read())
            for r in sheet_rules:
                if re.search(r"(?<![\w-])animation(?:-name)?\s*:", r.body) and re.search(r"data-state=[\"']?(?:open|closed)", r.prelude):
                    hit = True
                    break
        if not hit:
            for root, _dirs, names in os.walk(d):
                for n in names:
                    if n.endswith(".rs"):
                        with open(os.path.join(root, n), encoding="utf-8") as fh:
                            if "data-dx-surface" in fh.read():
                                hit = True
                                break
                if hit:
                    break
        if hit:
            out.add(c)
    return out

def collect():
    comps = animated_components()
    norm = {c: c.replace("_", "-") for c in comps}
    out = []
    for root, dirs, names in os.walk("playwright"):
        dirs[:] = sorted(d for d in dirs if d not in {"node_modules", "fixtures", "reference"})
        for n in sorted(names):
            if not n.endswith(".spec.ts"):
                continue
            stem = n[: -len(".spec.ts")].replace("_", "-")
            rel = os.path.relpath(os.path.join(root, n), "playwright")
            if os.path.dirname(rel) == "" and any(stem == v or stem.startswith(v + "-") for v in norm.values()):
                out.append(os.path.join(root, n))
    for e in EXTRA_SPECS:
        p = os.path.join("playwright", e)
        if os.path.exists(p) and p not in out:
            out.append(p)
        elif not os.path.exists(p) and not explicit:
            print(f"FAIL: EXTRA_SPECS entry {e} does not exist", file=sys.stderr)
            sys.exit(1)
    return sorted(out)

files = explicit or collect()
if not files:
    print("check-spec-at-rest: no specs found (run from a checkout)", file=sys.stderr)
    sys.exit(2)

COMMENT_BLOCK = re.compile(r"/\*.*?\*/", re.S)
COMMENT_LINE = re.compile(r"(?<![:\"'\w])//[^\n]*")

def blank(m):
    return re.sub(r"[^\n]", " ", m.group(0))

READ = re.compile(r"\.\s*boundingBox\s*\(|(?<![\w$])getBoundingClientRect\s*\(|\.\s*getBoundingClientRect\s*\(")
HELPER = re.compile(r"\b(?:measureAtRest|readAtRest|readStatic)\s*\($")
MARKER = re.compile(r"rest-ok:\s*\S")

def enclosed_by_helper(code, pos):
    """True when `pos` lies inside the argument list of a measureAtRest(...) / readAtRest(...) call."""
    depth = 0
    i = pos - 1
    while i >= 0:
        ch = code[i]
        if ch in ")]}":
            depth += 1
        elif ch in "([{":
            if depth == 0:
                if ch == "(" and HELPER.search(code[max(0, i - 20): i + 1]):
                    return True
            else:
                depth -= 1
        i -= 1
    return False

errors = []
total = 0
for path in files:
    with open(path, encoding="utf-8") as fh:
        raw = fh.read()
    raw_lines = raw.split("\n")
    code = COMMENT_LINE.sub(blank, COMMENT_BLOCK.sub(blank, raw))
    code_lines = code.split("\n")
    line_start = [0]
    for ln in code_lines:
        line_start.append(line_start[-1] + len(ln) + 1)
    for m in READ.finditer(code):
        ln = code.count("\n", 0, m.start())
        total += 1
        if enclosed_by_helper(code, m.start()):
            continue
        if MARKER.search(raw_lines[ln]) or (ln > 0 and MARKER.search(raw_lines[ln - 1])):
            continue
        if ALLOW.find_all(lambda e: path.endswith(e.suffix) and e.needle in code_lines[ln]):
            continue
        errors.append(f"{path}:{ln + 1}: `{code_lines[ln].strip()[:90]}` reads a box without measureAtRest/readAtRest/readStatic/staticBox "
                      f"(an enter animation may still be running)")

if errors:
    for e in errors:
        print(e)
    print(f"\ncheck-spec-at-rest: {len(errors)} problem(s). Read the geometry of an overlay with `await measureAtRest(locator)` "
          f"or `await readAtRest(locator, (el) => ...)`, of an element that does not animate with `await staticBox(locator)` "
          f"or `await readStatic(locator, (el) => ...)` (playwright/animations.ts); a sampler that measures the motion itself, "
          f"or an in-page read settled by a named earlier step, adds `// rest-ok: <reason>` on that line or the line above.", file=sys.stderr)
    sys.exit(1)

problems = ALLOW.problems(stale=not explicit)
if problems:
    for p in problems:
        print(f"FAIL: {p}")
    sys.exit(1)
print(f"check-spec-at-rest: OK ({len(files)} overlay spec file(s): {total} geometry read(s), each through a helper or marked as a "
      f"sampler / named settle; {len(ALLOW)} allowlist entr{'y' if len(ALLOW) == 1 else 'ies'})")
PYEOF

if [ "$#" -eq 0 ]; then self_test; fi
