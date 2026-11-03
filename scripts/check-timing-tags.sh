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
# `TIMING_TAG` (playwright/timing.ts) or the literal "@timing". It also fails when a spec file measures time outside
# any `test(` (a helper the cases share) and has no tagged case at all, because the skip or the measurement then
# reaches an untagged case through the helper. A body measures time when it names:
#   - `assertResponsive`, `withGesture`, `perFrameCost`, `frameCostMeter` or `buildProfile()` (the main-thread and
#     per-frame-cost harnesses);
#   - a frame sampler of its own: an `addInitScript(...)` whose callback calls `requestAnimationFrame(...)`. A raw loop from
#     the first byte reads a half-parsed document and only the states the frames land on, so its verdict depends on
#     the machine; `installFrameRecorder` (playwright/frame-recorder.ts) is the deterministic form and is not matched;
#   - a fling or flick (`fling`, `flick`, ... in a title or helper name) in a spec that drives the pointer with
#     `page.mouse` and has no clock of its own (`dragOnTimeline`, `flick: true`, `spin(`): the release velocity is then
#     the spacing of the CDP round trips (playwright/pointer-timeline.ts);
#   - an upper bound on a measured duration: `expect(<expr>).toBeLessThan(N)` / `.toBeLessThanOrEqual(N)` where `<expr>`
#     reads a clock (`Date.now`, `performance.now`) or names `elapsed(`, `waited`, `took`, `latency` or a `...Ms`
#     identifier (a computed style, `parseFloat(...)` or a `...Transition...` read, is not a measurement), unless that line or the line above carries
#     `timing-bound-ok: <reason>` (a non-empty reason: the bound is a relation to something the page configures, such
#     as a 600 ms open delay, not a budget for the machine).
#   - (a different remedy, no tag) a wait on an app timer in real time: `waitForTimeout(<N>)` with N a literal of 1000 ms or more, or
#     an UPPER_CASE constant (a delay or settle window the page configures). The app's timers are `setTimeout`s, so the
#     verdict of "after the toast's 3 s it is gone" or "the 600 ms open delay has passed" belongs on `page.clock`
#     (playwright/app-clock.ts: `pauseAppClock`, then `page.clock.runFor`), where the test decides when the timer fires. The
#     wait passes when its file uses `page.clock` or `pauseAppClock`, when its case is tagged, or when its line or one of the
#     two above carries `timing-bound-ok: <reason>` (a window in which nothing may happen, so a loaded machine only
#     lengthens it, or a wait for a real-time transition or network the clock does not control).
# A deterministic test does not need the tag: it drives the clock itself, and the gate then has nothing to match.
#
# Limits, stated: it reads those names and shapes, so a spec that measures time with its own `performance.now()` pair
# and asserts a bound through a neutral variable name is the business of `check-timing-relative` (a constant bound on
# a duration needs its reason) and is tagged by hand; a `test.describe` body is not read as a case.
#
# It also checks the deactivation switch (playwright/tools/timing-switch.sh, row 494): while the default is
# `off`, the row it names (`TIMING_REENABLE_ROW`) must be an open backlog row (a file in dev-docs/backlog/, not done/),
# and timing-pass.sh, run-suite.sh and deploy-preview.sh must read the switch, so the deactivation cannot be forgotten
# silently or bypassed by one script.
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
  cases=0
  expect() {
    local want="$1" name="$2" body="$3"
    cases=$((cases + 1))
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
  expect red rafinit 'test("a", async ({ page }) => { await page.addInitScript(() => { const tick = () => { requestAnimationFrame(tick); }; requestAnimationFrame(tick); }); });'
  expect green rafinittagged 'test("a", { tag: TIMING_TAG }, async ({ page }) => { await page.addInitScript(() => { requestAnimationFrame(() => {}); }); });'
  expect green rafevaluate 'test("a", async ({ page }) => { await page.evaluate(() => new Promise((r) => requestAnimationFrame(r))); });'
  expect green recorder 'test("a", async ({ page }) => { await installFrameRecorder(page, "__f", ({ parsed }) => null); });'
  expect red rafinithelper 'async function sample(page) { await page.addInitScript(() => { requestAnimationFrame(() => {}); }); }
test("a", async ({ page }) => { await sample(page); });'
  expect red flingmouse 'test("a fling coasts", async ({ page }) => { await page.mouse.move(1, 1); await page.mouse.up(); });'
  expect red flickhelper 'async function flick(page) { await page.mouse.down(); await page.mouse.up(); }
test("dismisses", async ({ page }) => { await flick(page); });'
  expect green flingtimeline 'test("a fling coasts", async ({ page }) => { await dragOnTimeline(page, { x: 1, y: 1 }, []); await page.mouse.move(1, 1); });'
  expect green flicksync 'test("a flick closes", async ({ page }) => { await pointerDrag(page, { flick: true }); await page.mouse.move(1, 1); });'
  expect green flingtagged 'test("a fling coasts", { tag: TIMING_TAG }, async ({ page }) => { await page.mouse.move(1, 1); });'
  expect green flingnomouse 'test("a flick closes", async ({ page }) => { await page.locator("a").dispatchEvent("pointerdown"); });'
  expect red boundwaited 'test("a", async () => { expect(waited).toBeLessThan(4_000); });'
  expect red boundelapsed 'test("a", async () => { expect(elapsed(await readProbe(page), "x", "y")!).toBeLessThanOrEqual(600); });'
  expect red boundnow 'test("a", async () => { expect(performance.now() - t0).toBeLessThan(900); });'
  expect red boundms 'test("a", async () => { expect(openMs).toBeLessThan(900); });'
  expect red boundemptyreason 'test("a", async () => {
  // timing-bound-ok:
  expect(waited).toBeLessThan(4000);
});'
  expect green boundmarked 'test("a", async () => {
  // timing-bound-ok: the open delay is 600 ms, the bound is the delay
  expect(waited).toBeLessThan(600);
});'
  expect green boundinline 'test("a", async () => {
  expect(waited).toBeLessThan(600); // timing-bound-ok: relative to the configured delay
});'
  expect green rafpatch 'test("a", async ({ page }) => { await page.addInitScript(() => { const raf = window.requestAnimationFrame.bind(window); window.requestAnimationFrame = (cb) => raf(cb); }); });'
  expect green boundtransition 'test("a", async () => { expect(await transformTransitionMs(list, 3)).toBeLessThan(1); });'
  expect green boundlower 'test("a", async () => { expect(waited).toBeGreaterThanOrEqual(595); });'
  expect green boundstyle 'test("a", async () => { expect(parseFloat(await style(probe, "transition-duration"))).toBeLessThan(0.001); });'
  expect green boundcount 'test("a", async () => { expect(rows).toBeLessThan(40); });'
  expect red appwait 'test("a", async ({ page }) => { await page.waitForTimeout(3500); });'
  expect red appwaitconst 'test("a", async ({ page }) => { await page.waitForTimeout(HOVER_SETTLE_MS); });'
  expect red appwaitunderscore 'test("a", async ({ page }) => { await page.waitForTimeout(1_200 * 2); });'
  expect green appwaitshort 'test("a", async ({ page }) => { await page.waitForTimeout(350); });'
  expect green appwaitclock 'test("a", async ({ page }) => { await pauseAppClock(page); await page.clock.runFor(3500); });'
  expect green appwaitclockfile 'test("a", async ({ page }) => { await page.clock.install(); await page.waitForTimeout(3500); });'
  expect green appwaitmarked 'test("a", async ({ page }) => {
  // timing-bound-ok: nothing may happen in this window, a slow machine only lengthens it
  await page.waitForTimeout(3500);
});'
  expect red appwaitemptyreason 'test("a", async ({ page }) => {
  await page.waitForTimeout(3500); // timing-bound-ok:
});'
  expect green appwaittagged 'test("a", { tag: TIMING_TAG }, async ({ page }) => { await page.waitForTimeout(3500); });'
  echo "check-timing-tags --self-test: OK ($cases cases)"
  exit 0
fi

if [[ $# -eq 0 ]]; then
  "${BASH_SOURCE[0]}" --self-test
  switch=playwright/tools/timing-switch.sh
  [[ -f "$switch" ]] || { echo "check-timing-tags: $switch (the timing pass switch) is missing" >&2; exit 1; }
  for user in playwright/tools/timing-pass.sh playwright/tools/run-suite.sh scripts/deploy-preview.sh; do
    grep -q 'timing-switch\.sh' "$user" || { echo "check-timing-tags: $user does not read $switch" >&2; exit 1; }
  done
  default="$(sed -n 's/^TIMING_PASS_DEFAULT=//p' "$switch")"
  row="$(sed -n 's/^TIMING_REENABLE_ROW=//p' "$switch")"
  if [[ "$default" == off ]]; then
    compgen -G "dev-docs/backlog/${row}-*.md" >/dev/null \
      || { echo "check-timing-tags: the timing pass is off but backlog row '${row}' (TIMING_REENABLE_ROW in $switch) is not an open row in dev-docs/backlog/; re-enable the pass (TIMING_PASS_DEFAULT=on) or name the open row" >&2; exit 1; }
    echo "check-timing-tags: timing pass deactivated, re-enabled by open row $row"
  elif [[ "$default" != on ]]; then
    echo "check-timing-tags: TIMING_PASS_DEFAULT in $switch must be on or off (got '$default')" >&2; exit 1
  fi
fi

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
INIT_SCRIPT = re.compile(r"\baddInitScript\s*\(")
RAF = re.compile(r"\brequestAnimationFrame\s*\(")
FLING = re.compile(r"\b(?:fling|flick)\w*", re.I)
PRESS = re.compile(r"\bmouse\s*\.\s*(?:down|up)\b")
HELPER_DEF = re.compile(r"(?:\basync\s+)?\bfunction\s+(\w+)\s*(?:<[^>(]*>)?\s*\(|\b(?:const|let)\s+(\w+)\s*=\s*(?:async\s*)?(?:\([^()]*\)|\w+)\s*(?::[^=;]*)?=>")
OWN_CLOCK = re.compile(r"\bdragOnTimeline\b|\bflick\s*:\s*true\b|\bspin\s*\(")
BOUND = re.compile(r"\bexpect\s*\(")
BOUND_TAIL = re.compile(r"\s*\.\s*(?:toBeLessThan|toBeLessThanOrEqual)\s*\(\s*[\d_.]+\s*\)")
BOUND_EXPR = re.compile(r"\bDate\.now\b|\bperformance\.now\b|\belapsed\s*\(|\b(?:waited|elapsed|took|latency)\b|\b\w+Ms\b")
BOUND_OK = re.compile(r"timing-bound-ok:\s*\S")

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

def measure_hits(code, raw_lines):
    """Offsets in `code` where the file measures time (the names, the shapes in the header), with the reason."""
    hits = [(m.start(), "names " + m.group(0).strip()) for m in MEASURES.finditer(code)]
    for m in INIT_SCRIPT.finditer(code):
        body = code[m.start():call_end(code, m.end())]
        r = RAF.search(body)
        if r:
            hits.append((m.start() + r.start(), "starts its own requestAnimationFrame loop in an addInitScript (use installFrameRecorder)"))
    for m in BOUND.finditer(code):
        end = call_end(code, m.end())
        tail = BOUND_TAIL.match(code, end)
        if not tail or not BOUND_EXPR.search(code[m.end():end]) or re.search(r"parseFloat|ransition", code[m.end():end]):
            continue
        first, last = code.count("\n", 0, m.start()), code.count("\n", 0, tail.end())
        near = raw_lines[max(0, first - 1):last + 1]
        if not any(BOUND_OK.search(line) for line in near):
            hits.append((m.start(), "asserts an upper bound on a measured duration (give the bound a `timing-bound-ok: <reason>` or tag the case)"))
    return hits

def block_end(code, i):
    """Offset just past the `}` that closes the block whose `{` is at or after i, or the end of the statement."""
    j = code.find("{", i)
    k = code.find(";", i)
    if j == -1 or (k != -1 and k < j and "=>" in code[i:k]):
        return k + 1 if k != -1 else len(code)
    depth, quote = 0, None
    while j < len(code):
        c = code[j]
        if quote:
            if c == "\\":
                j += 1
            elif c == quote:
                quote = None
        elif c in "\"'`":
            quote = c
        elif c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return j + 1
        j += 1
    return len(code)

def helper_bodies(code):
    """Name -> source of every function declared in the file (declarations and arrow constants)."""
    out = {}
    for m in HELPER_DEF.finditer(code):
        name = m.group(1) or m.group(2)
        out[name] = code[m.start():block_end(code, m.end())]
    return out

def gesture_text(text, helpers, depth=3, seen=None):
    """`text` plus the bodies of the file-local helpers it calls, transitively."""
    seen = seen if seen is not None else set()
    out = text
    if depth:
        for name, body in helpers.items():
            if name not in seen and re.search(r"\b%s\s*\(" % re.escape(name), text):
                seen.add(name)
                out += "\n" + gesture_text(body, helpers, depth - 1, seen)
    return out

def paced_flings(code, spans, helpers):
    """(offset, reason) of every test whose title or body speaks of a fling/flick, presses the mouse (itself or through a
    helper of the file) and has no clock of its own."""
    hits = []
    for a, b in spans:
        text = code[a:b]
        f = FLING.search(text)
        if not f:
            continue
        full = gesture_text(text, helpers)
        if PRESS.search(full) and not OWN_CLOCK.search(full):
            hits.append((a + f.start(), f"drives a `{f.group(0)}` with page.mouse and no clock of its own (use dragOnTimeline)"))
    return hits

WAIT = re.compile(r"\bwaitForTimeout\s*\(")
WAIT_LONG = re.compile(r"\s*(?:[A-Z][A-Z0-9_]*[A-Z0-9]\b|\d[\d_]*(?:\s*[-+*]\s*[\d_]+)*)")
CLOCK = re.compile(r"\bpage\s*\.\s*clock\b|\bpauseAppClock\b")

def long_waits(code, raw_lines, spans):
    """(offset, text) of every real-time `waitForTimeout` on an app-sized duration with no clock, tag or marker."""
    out = []
    if CLOCK.search(code):
        return out
    for m in WAIT.finditer(code):
        end = call_end(code, m.end())
        arg = code[m.end():end - 1].strip()
        if not WAIT_LONG.fullmatch(arg):
            continue
        if not re.search(r"[A-Z]{2}", arg) and eval_ms(arg) < 1000:
            continue
        line = code.count("\n", 0, m.start())
        if any(BOUND_OK.search(l) for l in raw_lines[max(0, line - 2):line + 1]):
            continue
        if any(a <= m.start() < b and TAGGED.search(code[a:b].split("async", 1)[0]) for a, b in spans):
            continue
        out.append((m.start(), arg))
    return out

def eval_ms(arg):
    try:
        return eval(arg.replace("_", ""), {"__builtins__": {}})
    except Exception:
        return 0

errors = []
considered = 0
for path in files:
    with open(path, encoding="utf-8") as fh:
        raw = fh.read()
    code = IMPORT.sub(blank, COMMENT_LINE.sub(blank, COMMENT_BLOCK.sub(blank, raw)))
    spans = [(m.start(), call_end(code, m.end())) for m in TEST_CALL.finditer(code)]
    for off, arg in long_waits(code, raw.split("\n"), spans):
        errors.append(f"{path}:{code.count(chr(10), 0, off) + 1}: waitForTimeout({arg}) waits on an app timer in real time (drive it with page.clock, playwright/app-clock.ts, or say `timing-bound-ok: <reason>`)")
    hits = measure_hits(code, raw.split("\n")) + paced_flings(code, spans, helper_bodies(code))
    if not hits:
        continue
    considered += 1
    line_of = lambda off: code.count("\n", 0, off) + 1
    tagged_any = False
    for start, end in spans:
        head = code[start:end].split("async", 1)[0]
        has_tag = bool(TAGGED.search(head))
        tagged_any = tagged_any or has_tag
        inside = [h for h in hits if start <= h[0] < end]
        if inside and not has_tag:
            errors.append(f"{path}:{line_of(inside[0][0])}: a test that measures time ({inside[0][1]}) is not tagged `{{ tag: TIMING_TAG }}`")
    outside = [h for h in hits if not any(a <= h[0] < b for a, b in spans)]
    if outside and not tagged_any:
        errors.append(f"{path}:{line_of(outside[0][0])}: {outside[0][1]} outside any test(...) and the file has no `@timing` case")

if errors:
    for e in sorted(errors):
        print(e, file=sys.stderr)
    print(f"\ncheck-timing-tags: {len(errors)} problem(s). A case that measures time is declared `test(title, {{ tag: TIMING_TAG }}, ...)` "
          f"(import it from playwright/timing.ts) so it runs in the quiet timing pass; a wait on an app timer uses page.clock "
          f"(playwright/app-clock.ts) or says `timing-bound-ok: <reason>`.", file=sys.stderr)
    sys.exit(1)
print(f"check-timing-tags: OK ({considered} spec file(s) measure time, every such case is tagged)")
PYEOF
