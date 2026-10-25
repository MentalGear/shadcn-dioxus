#!/usr/bin/env bash
#
# check-spec-font-state.sh  (backlog row 439)
#
# A spec that compares one page load with another must put both loads in the SAME font state. Geist is self-hosted
# with `font-display: optional`, so a load uses it only if it lands inside the ~100 ms block window, and otherwise not
# at all for that document (a woff2 delayed 600 ms ends `loaded` in `document.fonts`, the text still in the fallback
# face; `document.fonts.ready` cannot select the state). Two independent loads race the font, and under load one gets
# Geist's metrics and the other the fallback's: a description wraps to a second line on one side and every box below it
# moves (+20 px of card header on `/charts/`, +14 px of footer on `/charts/pie/`; measurements in row 439). The
# construction is one helper, `noWebFonts(context|page)` in `playwright/web-fonts.ts`, which aborts the web-font requests
# so the fallback face is the only state, with `expectNoWebFont(page, label)` to assert it.
#
# Rule 1. In a spec file (`playwright/**/*.ts` except `node_modules`, `fixtures/`, `oracle/reference/` and `web-fonts.ts`
# itself), comments stripped, that reads rendered geometry or styles (`getBoundingClientRect`, `.boundingBox(`,
# `getComputedStyle`, `offsetWidth/Height`, `scrollWidth/Height`, `clientWidth/Height`), every `javaScriptEnabled:`
# context option (the JavaScript-off load of a pre-hydration comparison, whether the value is `false` or a variable)
# must be followed, within the next 8 lines, by a `noWebFonts(` call; or the line, or the line directly above it,
# carries `font-ok: <reason>` (a read that compares nothing across loads, a context whose only subject is markup).
#
# Rule 2. `noWebFonts` is defined once: outside `web-fonts.ts`, a `.route(` whose pattern names `woff2` fails
# (a local copy drifts from the helper's pattern and its assertion; a static fixture's own Google Fonts link is not the app's font).
#
# EXCEPTIONS are in ALLOW below, each with its reason; an entry that matches nothing fails the run. The gate's own
# self-test (run on every invocation, mutations inline) must turn red on an unpinned context and on a local font route.
#
# Usage: scripts/check-spec-font-state.sh [FILE ...]   (default: every spec under playwright/; explicit files are for
# self-tests and skip the stale-entry report)
# Exit status: 0 when clean.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import os
import re
import sys

args = sys.argv[1:]
SKIP_DIRS = {"node_modules", "fixtures", "reference"}
HELPER = "web-fonts.ts"

# (file suffix, substring of the `javaScriptEnabled:` line, reason)
ALLOW = [
    ("accordion.spec.ts", "javaScriptEnabled: false", "one JavaScript-off load that asserts a default-open item's markup (`data-open`, text); no box or style is read from it and nothing is compared with another load"),
    ("questionnaire.spec.ts", "javaScriptEnabled: false", "one JavaScript-off load that asserts the server markup (item count, `inert`, `aria-*`, visibility); the `getComputedStyle` read is on a hydrated page, no box is compared across loads"),
]
used = set()

COMMENT_BLOCK = re.compile(r"/\*.*?\*/", re.S)
COMMENT_LINE = re.compile(r"(?<![:\"'\w])//[^\n]*")
READS = re.compile(r"getBoundingClientRect|\.boundingBox\s*\(|getComputedStyle|\.(?:offset|scroll|client)(?:Width|Height)\b")
CONTEXT = re.compile(r"\bjavaScriptEnabled\s*:")
PIN = re.compile(r"\bnoWebFonts\s*\(")
FONT_ROUTE = re.compile(r"\.route\s*\([^\n]*woff2")
MARKER = re.compile(r"font-ok:\s*\S")
WINDOW = 8


def blank(m):
    return re.sub(r"[^\n]", " ", m.group(0))


def collect():
    out = []
    for root, dirs, names in os.walk("playwright"):
        dirs[:] = sorted(d for d in dirs if d not in SKIP_DIRS)
        out += [os.path.join(root, n) for n in sorted(names) if n.endswith(".ts") and n != HELPER]
    return out


def scan(path, text, allow, used_entries):
    raw_lines = text.split("\n")
    code = COMMENT_LINE.sub(blank, COMMENT_BLOCK.sub(blank, text)).split("\n")
    problems = []
    reads = any(READS.search(l) for l in code)
    for i, line in enumerate(code):
        if FONT_ROUTE.search(line):
            problems.append(f"{path}:{i + 1}: a local web-font route; use `noWebFonts` from playwright/web-fonts.ts (one pattern, one assertion)")
        if not reads or not CONTEXT.search(line):
            continue
        if any(PIN.search(l) for l in code[i : i + 1 + WINDOW]):
            continue
        if MARKER.search(raw_lines[i]) or (i > 0 and MARKER.search(raw_lines[i - 1])):
            continue
        hit = False
        for entry in allow:
            suffix, needle, _reason = entry
            if path.endswith(suffix) and needle in line:
                used_entries.add(id(entry))
                hit = True
        if hit:
            continue
        problems.append(f"{path}:{i + 1}: `{line.strip()[:90]}` opens a context with no `noWebFonts(` in the next {WINDOW} lines, "
                        f"in a spec that reads geometry or styles: a second load races `font-display: optional` and the two loads can be in different font states")
    return problems


def self_test():
    cases = [
        ("unpinned JS-off context", 'const c = await browser.newContext({ javaScriptEnabled: false });\nconst r = el.getBoundingClientRect();\n', True),
        ("unpinned context, variable value", 'const c = await browser.newContext({ javaScriptEnabled: js });\nawait page.evaluate(() => getComputedStyle(document.body));\n', True),
        ("pinned context", 'const c = await browser.newContext({ javaScriptEnabled: false });\nawait noWebFonts(c);\nconst r = el.getBoundingClientRect();\n', False),
        ("marker on the line above", '// font-ok: reads attributes only\nconst c = await browser.newContext({ javaScriptEnabled: false });\nel.getBoundingClientRect();\n', False),
        ("no geometry read", 'const c = await browser.newContext({ javaScriptEnabled: false });\n', False),
        ("pin too far away", 'const c = await browser.newContext({ javaScriptEnabled: false });\n' + "x;\n" * 9 + 'await noWebFonts(c);\nel.getBoundingClientRect();\n', True),
        ("local font route", 'await page.route(/\\.woff2(\\?|$)/, (r) => r.abort());\n', True),
        ("commented-out pin", 'const c = await browser.newContext({ javaScriptEnabled: false });\n// await noWebFonts(c);\nel.getBoundingClientRect();\n', True),
    ]
    bad = []
    for name, text, expect_red in cases:
        red = bool(scan("self-test.ts", text, [], set()))
        if red != expect_red:
            bad.append(f"self-test '{name}': expected {'red' if expect_red else 'green'}, got {'red' if red else 'green'}")
    return bad


bad = self_test()
if bad:
    for b in bad:
        print(b)
    print("check-spec-font-state: the gate's own self-test failed", file=sys.stderr)
    sys.exit(1)

files = args or collect()
if not files:
    print("check-spec-font-state: no specs found (run from a checkout)", file=sys.stderr)
    sys.exit(2)

errors = []
contexts = 0
for path in files:
    with open(path, encoding="utf-8") as fh:
        text = fh.read()
    code = COMMENT_LINE.sub(blank, COMMENT_BLOCK.sub(blank, text))
    if any(READS.search(l) for l in code.split("\n")):
        contexts += sum(1 for l in code.split("\n") if CONTEXT.search(l))
    errors += scan(path, text, ALLOW, used)

if errors:
    for e in errors:
        print(e)
    print(f"\ncheck-spec-font-state: {len(errors)} problem(s). Call `await noWebFonts(context)` (playwright/web-fonts.ts) right after "
          f"the context is created, and assert it with `expectNoWebFont(page, label)`; if the spec compares nothing across loads, "
          f"say so with `// font-ok: <reason>` on the line or the line above.", file=sys.stderr)
    sys.exit(1)

stale = [e for e in ALLOW if id(e) not in used]
if stale and not args:
    for suffix, needle, reason in stale:
        print(f"FAIL: stale allowlist entry {suffix} {needle!r} -- {reason}")
    sys.exit(1)
print(f"check-spec-font-state: OK ({len(files)} spec file(s); {contexts} `javaScriptEnabled:` context(s) in specs that read geometry or styles, "
      f"each pinned with noWebFonts or marked font-ok; self-test green)")
PYEOF
