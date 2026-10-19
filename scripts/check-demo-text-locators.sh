#!/usr/bin/env bash
#
# check-demo-text-locators.sh   (backlog row 220)
#
# A SHARED spec -- one that drives the demo pages of many components (the oracle suites, the theme-contrast
# sweep, focus restore, overlay switching) -- must not locate a demo's controls by the demo's visible COPY.
# Copy is wording nobody meant as a contract: three rounds in one session a demo that mirrored shadcn's example
# more closely (row 212's labelled demos, row 155 group 3's toggles) renamed a button and broke
# `keyboard-matrix`, `main-thread`, `rtl` and `theme-preset-contrast` at once. The construction is
# `playwright/demo-frame.ts`: a control is named by its ARIA ROLE and its POSITION inside the demo frame
# (`demoRole(page, "button")`, `demoRole(page, "tab", 2)`, `demoRole(page, "button", 0, "checkboxes")` for the
# `#component-preview-frame-checkboxes` variant), and an item of an open popup by its position in that popup
# (`popupItem(page, "menu", "menuitem", 2)`).
#
# This gate fails when a spec in scope (comments stripped) uses a text locator:
#
#   `.getByText(` / `.getByLabel(` / `.getByPlaceholder(` / `.getByTitle(` / `.getByAltText(`,
#   `getByRole(<role>, { ... name: ... })`,
#   `hasText:` / `hasNotText:` (a `.filter({ hasText })` or a `locator(sel, { hasText })`),
#   a selector string using `text=`, `:has-text(`, `:text(`, `:text-is(`, `[aria-label=...]`, `[title=...]`,
#   `[placeholder=...]`,
#
# unless the line, or the line directly above it, carries `copy-ok: <reason>` (a non-empty reason), or a
# reasoned ALLOW entry covers it. A legitimate use is one where the name is NOT demo copy: a name that belongs
# to the COMPONENT or the SITE (a Close button's `aria-label`, a toast's "close", the header's "Theme" button),
# a vendored reference page (W3C's own example, not a demo), a conformance fixture whose labels were authored
# for these very specs, or site content that has no demo frame (the home page, the dashboard). Say which.
#
# SCOPE: every `playwright/oracle/**/*.spec.ts` (except `oracle/reference/`), plus EXTRA_SPECS below. A
# per-component spec that asserts its OWN component's labels (`dialog.spec.ts` checking a dialog's title) is
# not a shared spec and not in scope.
#
# EXCEPTIONS are in ALLOW below, each with its reason (a needle of '' covers every hit in the file); an entry
# that matches nothing fails the run, so it is deleted when its use goes away.
#
# Usage: scripts/check-demo-text-locators.sh [FILE ...]   (default: every spec in scope; explicit files are
# for self-tests, and skip the allowlist stale-entry report)
# Exit status: 0 when clean.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import os
import re
import sys

explicit = sys.argv[1:]

# Shared specs outside playwright/oracle/ (path under playwright/).
# NOT yet in scope: `modal-exit.spec.ts` and `modal-initial-focus.spec.ts` drive the `CONSUMERS` table of
# `modal-consumers.ts`, which still carries each demo's trigger and closer COPY (`trigger: "Show Dialog"`); moving it to
# role/position changes the table's type for both specs, so it goes in with the lane that owns modal-initial-focus.
EXTRA_SPECS = [
    "theme-preset-contrast.spec.ts",
    "oracle-focus-restore.spec.ts",
    "overlay-switch.spec.ts",
]

# (file suffix, substring of the code line ('' = every hit in the file), reason)
ALLOW = [
    ("oracle/hydration-parity.spec.ts", "name: displayName",
     "the heading name is the route's own component id (name.replace('_', ' ')), generated per route, not demo copy"),
    ("oracle/tier1-apg/carousel.spec.ts", 'getByRole("region", { name: "Highlighted television shows" })',
     "W3C's own vendored carousel example page (oracle/reference), not a demo"),
    ("oracle/tier1-apg/carousel.spec.ts", "page.getByRole(\"button\", { name: /previous/i })",
     "W3C's own vendored carousel example page (oracle/reference), not a demo"),
    ("oracle/tier1-apg/carousel.spec.ts", "page.getByRole(\"button\", { name: /next/i })",
     "W3C's own vendored carousel example page (oracle/reference), not a demo"),
    ("oracle/tier1-apg/carousel.spec.ts", "page.getByRole(\"group\", { name:",
     "W3C's own vendored carousel example page (oracle/reference): its slides are named 'n of 6'"),
    ("oracle/tier1-apg/carousel.spec.ts", 'aria-label="${n} of',
     "the carousel's own generated 'n of m' tabpanel names (W3C's vendored tabbed example, and the indicators variant's component-generated names), not demo copy"),
    ("oracle/tier1-apg/disclosure-navigation.spec.ts", '{ name: "About" }',
     "W3C's own vendored disclosure-navigation example page (oracle/reference), not a demo"),
    ("oracle/tier1-apg/disclosure-navigation.spec.ts", '{ name: "Overview" }',
     "W3C's own vendored disclosure-navigation example page (oracle/reference), not a demo"),
    ("oracle/tier1-apg/disclosure-navigation.spec.ts", '{ name: "Mythical University" }',
     "W3C's own vendored disclosure-navigation example page (oracle/reference), not a demo"),
    ("oracle/tier1-apg/select-only-combobox.spec.ts", "(library)",
     "the form conformance fixture's trigger_aria_label, authored for these specs in preview/src/components/form/component.rs"),
    ("oracle/tier2-html/form-participation.spec.ts", "(library)",
     "the form conformance fixture's trigger_aria_label / list_aria_label, authored for these specs in preview/src/components/form/component.rs"),
    ("oracle/tier2-html/form-label-association.spec.ts", "getByText(text",
     "the form fixture's label text is the thing under test (a label's own text node, scoped under its `for`)"),
    ("oracle/tier2-html/demo-forms-no-navigation.spec.ts", "compose|new message",
     "the email client dashboard route: site content with no demo frame, the page under test IS this content"),
    ("oracle/tier2-html/main-thread.spec.ts", 'name: "close"',
     "the toast's own close button aria-label (the component's label, not demo copy)"),
    ("oracle/tier2-html/scroll-main-thread.spec.ts", "Play or pause",
     "the home page's player block aria-label: site content with no demo frame"),
    ("oracle/tier2-html/stylesheets-present.spec.ts", 'name: "Theme"',
     "the site header's Theme button: site chrome, not a demo"),
    ("oracle/tier2-html/top-layer.spec.ts", "nav[aria-label='Primary']",
     "the site's primary navbar, queried only to measure its height: site chrome, not a demo"),
    ("oracle/tier2-html/top-layer.spec.ts", "/Color picker/i",
     "the HOME page's widget masonry (a ColorPicker card among many widgets): site content with no demo frame"),
    ("oracle/tier2-html/touch-focus-zoom.spec.ts", "/Color picker/i",
     "the /demos page's BlockColorPalette: site content with no demo frame"),
    ("oracle/tier2-html/touch-focus-zoom.spec.ts", "/Compose/",
     "the email client dashboard route: site content with no demo frame, the page under test IS this content"),
]

def collect():
    out = []
    for root, dirs, names in os.walk("playwright/oracle"):
        dirs[:] = sorted(d for d in dirs if d not in {"node_modules", "reference"})
        out += [os.path.join(root, n) for n in sorted(names) if n.endswith(".spec.ts")]
    for e in EXTRA_SPECS:
        p = os.path.join("playwright", e)
        if os.path.exists(p):
            out.append(p)
        elif not explicit:
            print(f"FAIL: EXTRA_SPECS entry {e} does not exist", file=sys.stderr)
            sys.exit(1)
    return sorted(set(out))

files = explicit or collect()
if not files:
    print("check-demo-text-locators: no specs found (run from a checkout)", file=sys.stderr)
    sys.exit(2)

used = set()
COMMENT_BLOCK = re.compile(r"/\*.*?\*/", re.S)
COMMENT_LINE = re.compile(r"(?<![:\"'\w])//[^\n]*")

def blank(m):
    return re.sub(r"[^\n]", " ", m.group(0))

SIMPLE = [
    (re.compile(r"\.\s*getBy(?:Text|Label|Placeholder|Title|AltText)\s*\("), "getBy{Text,Label,Placeholder,Title,AltText}"),
    (re.compile(r"\b(?:hasText|hasNotText)\s*:"), "hasText"),
    (re.compile(r"""(?:locator|waitForSelector|querySelector(?:All)?|\$\$?|filter|has)\s*\(\s*["'`]\s*text\s*=|:has-text\(|:text\(|:text-is\(|:text-matches\(|\[\s*(?:aria-label|title|placeholder)\s*[~|^$*]?=|>>\s*text\s*="""), "text selector"),
]
GETBYROLE = re.compile(r"\.\s*getByRole\s*\(")
NAME_OPT = re.compile(r"\bname\s*:")
MARKER = re.compile(r"copy-ok:\s*\S")

def matching_paren(code, open_idx):
    depth = 0
    i = open_idx
    n = len(code)
    in_str = None
    while i < n:
        ch = code[i]
        if in_str:
            if ch == "\\":
                i += 2
                continue
            if ch == in_str:
                in_str = None
        elif ch in "\"'`":
            in_str = ch
        elif ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return n

errors = []
total = 0
for path in files:
    with open(path, encoding="utf-8") as fh:
        raw = fh.read()
    raw_lines = raw.split("\n")
    code = COMMENT_LINE.sub(blank, COMMENT_BLOCK.sub(blank, raw))
    code_lines = code.split("\n")
    hits = []  # (line index, label)
    for rx, label in SIMPLE:
        for m in rx.finditer(code):
            hits.append((code.count("\n", 0, m.start()), label))
    for m in GETBYROLE.finditer(code):
        start = m.end() - 1
        end = matching_paren(code, start)
        inner = code[start:end]
        if NAME_OPT.search(inner):
            hits.append((code.count("\n", 0, m.start()), "getByRole(..., { name })"))
    seen = set()
    for ln, label in sorted(hits):
        if (ln, label) in seen:
            continue
        seen.add((ln, label))
        total += 1
        if MARKER.search(raw_lines[ln]) or (ln > 0 and MARKER.search(raw_lines[ln - 1])):
            continue
        ok = False
        for entry in ALLOW:
            suffix, needle, _reason = entry
            if path.endswith(suffix) and needle in code_lines[ln]:
                used.add(id(entry))
                ok = True
        if ok:
            continue
        errors.append(f"{path}:{ln + 1}: `{code_lines[ln].strip()[:100]}` locates by {label} (demo copy)")

if errors:
    for e in errors:
        print(e)
    print(f"\ncheck-demo-text-locators: {len(errors)} problem(s). Locate a demo's controls by role and position "
          f"(`demoRole(page, role, n)` / `popupItem(...)` in playwright/demo-frame.ts); if the name is NOT demo copy "
          f"(a component's own aria-label, site chrome, a vendored reference page, a spec-authored fixture), add "
          f"`// copy-ok: <reason>` on that line or the line above, or a reasoned ALLOW entry in this script.", file=sys.stderr)
    sys.exit(1)

stale = [e for e in ALLOW if id(e) not in used]
if stale and not explicit:
    for suffix, needle, reason in stale:
        print(f"FAIL: stale allowlist entry {suffix} {needle!r} -- {reason}")
    sys.exit(1)
print(f"check-demo-text-locators: OK ({len(files)} shared spec file(s): {total} text locator(s), each marked or allowlisted as "
      f"not demo copy; {len(used)} allowlist entr{'y' if len(used) == 1 else 'ies'} in use)")
PYEOF
