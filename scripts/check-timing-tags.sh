#!/usr/bin/env bash
#
# check-timing-tags.sh
#
# A timing case runs alone, on a quiet machine, after the rest of the suite (dev-docs/dev-loop.md, "Running tests"):
# `playwright/tools/run-suite.sh` and `test:changed` skip every test tagged `@timing` in their parallel pass and run
# the tagged ones in `playwright/tools/timing-pass.sh`. A timing case that is NOT tagged would run beside three other
# workers' browsers and go red on load, which is the class this gate closes.
#
# This gate fails when a `playwright/**/*.spec.ts` file (except `node_modules`, `fixtures/` and `oracle/reference/`),
# comments stripped, has a `test(` whose body measures time and whose arguments before the callback do not carry
# `TIMING_TAG` (playwright/timing.ts) or the literal "@timing". A body measures time when it names `assertResponsive`,
# `withGesture`, `perFrameCost`, `frameCostMeter` or `buildProfile()`. It also fails when a spec file names one of
# those outside any `test(` (a helper the cases share) and has no tagged case at all, because the skip or the
# measurement then reaches an untagged case through the helper.
#
# Limits, stated: it reads those five names, so a spec that measures time with its own `performance.now()` pair and
# asserts a bound is the business of `check-timing-relative` (a constant bound on a duration needs its reason) and is
# tagged by hand; a `test.describe` body is not read as a case.
#
# Usage: scripts/check-timing-tags.sh [--self-test] [FILE ...]   (default: every spec under playwright/, after the
# self-test that must turn red on the forbidden shapes)
# Exit status: 0 when clean.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if [[ "${1:-}" == "--self-test" ]]; then
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  gate="$repo_root/scripts/check-timing-tags.sh"
  expect() {
    local want="$1" name="$2" body="$3"
    printf '%s\n' "$body" >"$tmp/$name.spec.ts"
    if "$gate" "$tmp/$name.spec.ts" >/dev/null 2>&1; then got=green; else got=red; fi
    if [[ "$got" != "$want" ]]; then echo "check-timing-tags --self-test: $name should be $want" >&2; exit 1; fi
  }
  expect red untagged 'test("a", async ({ page }) => { await assertResponsive(page, "x", g); });'
  expect red untaggedskip 'test("a", async ({ page }) => { test.skip(buildProfile() !== "release", "t"); });'
  expect red optionsnotag 'test("a", { timeout: 5 }, async ({ page }) => { await meter.perFrameCost(f); });'
  expect green tagged 'test("a", { tag: TIMING_TAG }, async ({ page }) => { await assertResponsive(page, "x", g); });'
  expect green literal 'test("a", { tag: "@timing" }, async ({ page }) => { await withGesture(page, g); });'
  expect green templated 'test(`${n}: open`, { tag: TIMING_TAG }, async ({ page }) => { await assertResponsive(page, n, g); });'
  expect green plain 'test("a", async ({ page }) => { await page.click("a"); });'
  expect green commented '// test("a", async () => { assertResponsive(); });
test("b", async () => {});'
  expect green mixed 'test("a", async () => {});
test("b", { tag: TIMING_TAG }, async () => { await frameCostMeter(page); });'
  expect red helperonly 'async function f(page) { await assertResponsive(page, "x", g); }
test("a", async () => { await f(page); });'
  expect green helpertagged 'async function f(page) { await assertResponsive(page, "x", g); }
test("a", { tag: TIMING_TAG }, async () => { await f(page); });'
  echo "check-timing-tags --self-test: OK (11 cases)"
  exit 0
fi

if [[ $# -eq 0 ]]; then "${BASH_SOURCE[0]}" --self-test; fi

python3 -I - "$@" <<'PYEOF'
import os
import re
import sys

explicit = sys.argv[1:]
SKIP_DIRS = {"node_modules", "fixtures", "reference"}

def collect():
    out = []
    for root, dirs, names in os.walk("playwright"):
        dirs[:] = sorted(d for d in dirs if d not in SKIP_DIRS)
        out += [os.path.join(root, n) for n in sorted(names) if n.endswith(".spec.ts")]
    return out

files = explicit or collect()
if not files:
    print("check-timing-tags: no specs found (run from a checkout)", file=sys.stderr)
    sys.exit(2)

MEASURES = re.compile(r"\b(?:assertResponsive|withGesture|perFrameCost|frameCostMeter)\b|\bbuildProfile\s*\(\s*\)")
TAGGED = re.compile(r"\bTIMING_TAG\b|[\"'`]@timing[\"'`]")
COMMENT_BLOCK = re.compile(r"/\*.*?\*/", re.S)
COMMENT_LINE = re.compile(r"(?<![:\"'\w])//[^\n]*")
IMPORT = re.compile(r"^\s*import\b[^;]*;", re.M | re.S)
TEST_CALL = re.compile(r"(?<![\w.$])test\s*\(")

def blank(m):
    return re.sub(r"[^\n]", " ", m.group(0))

def call_end(code, i):
    """Offset just past the `)` closing the call whose `(` is at i - 1, reading strings and templates."""
    depth, quote = 1, None
    while i < len(code) and depth:
        c = code[i]
        if quote:
            if c == "\\":
                i += 1
            elif c == quote:
                quote = None
        elif c in "\"'`":
            quote = c
        elif c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
        i += 1
    return i

errors = []
considered = 0
for path in files:
    with open(path, encoding="utf-8") as fh:
        raw = fh.read()
    code = IMPORT.sub(blank, COMMENT_LINE.sub(blank, COMMENT_BLOCK.sub(blank, raw)))
    if not MEASURES.search(code):
        continue
    considered += 1
    line_of = lambda off: code.count("\n", 0, off) + 1
    spans, tagged_any = [], False
    for m in TEST_CALL.finditer(code):
        end = call_end(code, m.end())
        spans.append((m.start(), end))
        body = code[m.start():end]
        head = body.split("async", 1)[0]
        has_tag = bool(TAGGED.search(head))
        tagged_any = tagged_any or has_tag
        if MEASURES.search(body) and not has_tag:
            errors.append(f"{path}:{line_of(m.start())}: a test that measures time is not tagged `{{ tag: TIMING_TAG }}`")
    outside = list(code)
    for a, b in spans:
        for k in range(a, b):
            if outside[k] != "\n":
                outside[k] = " "
    if MEASURES.search("".join(outside)) and not tagged_any:
        errors.append(f"{path}: names a timing measurement outside any test(...) and has no `@timing` case")

if errors:
    for e in errors:
        print(e, file=sys.stderr)
    print(f"\ncheck-timing-tags: {len(errors)} problem(s). Declare the case `test(title, {{ tag: TIMING_TAG }}, ...)` "
          f"(import it from playwright/timing.ts) so it runs in the quiet timing pass.", file=sys.stderr)
    sys.exit(1)
print(f"check-timing-tags: OK ({considered} spec file(s) measure time, every such case is tagged)")
PYEOF
