#!/usr/bin/env bash
#
# check-timing-relative.sh
#
# A release-only timing spec compares two measurements taken in the same run, never one measurement with a
# constant number of milliseconds: the shared 4-core box runs other lanes' builds, and load multiplies the
# measurement but not the constant, so an absolute bound goes red while the component is fine. What a component
# promises is a relation (chunked rows cost
# a fraction of one-skippable-element-per-row), so the spec measures the component's path and a reference path on
# the same page (`playwright/frame-cost.ts`) and asserts the ratio (backlog row 402).
#
# This gate fails when a timing spec (a `playwright/**/*.ts` file, except `node_modules`, `fixtures/` and
# `oracle/reference/`, that gates itself on `buildProfile() !== "release"` or drives the long-task observer of
# `main-thread.ts`), comments stripped, asserts `expect(<expr>).toBeLessThan(N)` or `.toBeLessThanOrEqual(N)` with a
# numeric literal N where `<expr>` names a duration (an identifier such as `ms`, `msPerFrame`, `maxTaskMs`,
# `elapsed`, `duration`, `latency`, `cost`, `PerFrame`), unless that line, the line of the `expect(`, or the line
# above either carries `timing-absolute-ok: <reason>` (a non-empty reason: the promise IS an absolute budget, such as
# the 50 ms Long Tasks definition, and it says so). The expression may span lines.
#
# Limits, stated: it reads identifier names, so a duration held in a variable with a neutral name, or compared with
# a constant defined elsewhere, is not seen; the `LONG_TASK_THRESHOLD_MS` constants of `main-thread.ts` carry the
# marker in their own comment instead. Counts (rows, layouts, fires per second) are not durations and are not matched.
#
# Usage: scripts/check-timing-relative.sh [--self-test] [FILE ...]   (default: every spec under playwright/, after the
# mutation self-test that must turn red on the forbidden shapes)
# Exit status: 0 when clean.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if [[ "${1:-}" == "--self-test" ]]; then
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  gate="$repo_root/scripts/check-timing-relative.sh"
  rel='import { buildProfile } from "./base-url";
test.skip(buildProfile() !== "release", "timing");'
  expect_red() {
    local name="$1" body="$2"
    printf '%s\n%s\n' "$rel" "$body" >"$tmp/$name.spec.ts"
    if "$gate" "$tmp/$name.spec.ts" >/dev/null 2>&1; then echo "check-timing-relative --self-test: $name should be red" >&2; exit 1; fi
  }
  expect_green() {
    local name="$1" body="$2" head="${3:-$rel}"
    printf '%s\n%s\n' "$head" "$body" >"$tmp/$name.spec.ts"
    if ! "$gate" "$tmp/$name.spec.ts" >/dev/null 2>&1; then echo "check-timing-relative --self-test: $name should be green" >&2; exit 1; fi
  }
  expect_red absolute 'expect(msPerFrame).toBeLessThan(12);'
  expect_red orequal 'expect(r.maxPinnedMs).toBeLessThanOrEqual(200);'
  expect_red multiline 'expect(
  frameCost,
  "message",
).toBeLessThan(4);'
  expect_red emptyreason '// timing-absolute-ok:
expect(msPerFrame).toBeLessThan(12);'
  expect_red underscore 'expect(elapsed).toBeLessThan(1_500);'
  expect_green marked '// timing-absolute-ok: the Long Tasks definition is 50 ms
expect(maxTaskMs).toBeLessThanOrEqual(50);'
  expect_green markedinline 'expect(maxTaskMs).toBeLessThanOrEqual(50); // timing-absolute-ok: platform budget'
  expect_green ratio 'expect(ratio).toBeLessThan(0.8);'
  expect_green count 'expect(rows).toBeLessThan(80);'
  expect_green constant 'expect(msPerFrame).toBeLessThan(BUDGET);'
  expect_green commented '// expect(msPerFrame).toBeLessThan(12);'
  expect_green stringonly 'expect(rows, "ms per frame").toBeLessThan(80);'
  expect_green notrelease 'expect(msPerFrame).toBeLessThan(12);' 'import { test } from "./fixtures";'
  echo "check-timing-relative --self-test: OK (13 cases)"
  exit 0
fi

if [[ $# -eq 0 ]]; then "${BASH_SOURCE[0]}" --self-test; fi

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
    print("check-timing-relative: no specs found (run from a checkout)", file=sys.stderr)
    sys.exit(2)

TIMING_SPEC = re.compile(r"buildProfile\(\)\s*[!=]==?\s*[\"']release[\"']|from\s+[\"'][./]*main-thread[\"']")
TIMING_NAME = re.compile(
    r"^(?:ms|Ms)$|Ms$|^ms[A-Z_]|_ms$|[Ee]lapsed|[Dd]uration|[Ll]atency|^took|PerFrame|[Cc]ost|Millis"
)
MARKER = re.compile(r"timing-absolute-ok:\s*\S")
COMMENT_BLOCK = re.compile(r"/\*.*?\*/", re.S)
COMMENT_LINE = re.compile(r"(?<![:\"'\w])//[^\n]*")
STRING = re.compile(r"\"(?:\\.|[^\"\\\n])*\"|'(?:\\.|[^'\\\n])*'|`(?:\\.|[^`\\])*`", re.S)
LESS = re.compile(r"\s*\.\s*(?:not\s*\.\s*)?toBeLessThan(?:OrEqual)?\(\s*\d[\d_.]*\s*\)")

def blank(m):
    return re.sub(r"[^\n]", " ", m.group(0))

errors = []
considered = 0
total = 0

for path in files:
    with open(path, encoding="utf-8") as fh:
        raw = fh.read()
    if not TIMING_SPEC.search(COMMENT_LINE.sub(blank, COMMENT_BLOCK.sub(blank, raw))):
        continue
    considered += 1
    raw_lines = raw.split("\n")
    code = STRING.sub(blank, COMMENT_LINE.sub(blank, COMMENT_BLOCK.sub(blank, raw)))
    line_of = lambda off: code.count("\n", 0, off)
    for m in re.finditer(r"\bexpect\s*\(", code):
        depth, i = 1, m.end()
        while i < len(code) and depth:
            depth += {"(": 1, ")": -1}.get(code[i], 0)
            i += 1
        expr = code[m.end() : i - 1]
        tail = LESS.match(code, i)
        if not tail:
            continue
        names = re.findall(r"[A-Za-z_$][\w$]*", expr)
        if not any(TIMING_NAME.search(n) for n in names):
            continue
        total += 1
        first, last = line_of(m.start()), line_of(tail.end() - 1)
        near = {first, first - 1, last, last - 1}
        if any(0 <= n < len(raw_lines) and MARKER.search(raw_lines[n]) for n in near):
            continue
        errors.append(f"{path}:{last + 1}: `{' '.join(code[m.start():tail.end()].split())[:100]}` bounds a duration with a "
                      f"constant number of milliseconds")

if errors:
    for e in errors:
        print(e)
    print(f"\ncheck-timing-relative: {len(errors)} problem(s). Measure a same-run reference path next to the component's "
          f"and assert the ratio (`playwright/frame-cost.ts`, row 402); where the promise is an absolute budget, say so with "
          f"`// timing-absolute-ok: <reason>` on that line or the line above.", file=sys.stderr)
    sys.exit(1)
print(f"check-timing-relative: OK ({considered} timing spec file(s) read, {total} constant bound(s) on a duration, each marked)")
PYEOF
