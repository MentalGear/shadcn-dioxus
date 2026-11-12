#!/usr/bin/env bash
#
# check-demo-text-locators.sh   (backlog row 220)
#
# A SHARED spec -- one that drives the demo pages of many components (the oracle suites, the theme-contrast
# sweep, focus restore, overlay switching) -- must not locate a demo's controls by the demo's visible COPY.
# Copy is wording nobody meant as a contract: a demo that mirrors shadcn's example more closely (row 212's
# labelled demos, row 155 group 3's toggles) renames a button and breaks `keyboard-matrix`, `main-thread`,
# `rtl` and `theme-preset-contrast` at once (backlog row 220). The construction is
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
# a conformance fixture whose labels were authored for these very specs, or site content that has no demo frame (the
# home page, the dashboard). Say which. A vendored reference page (W3C's own example, not a demo) needs no marker and no
# entry (row 427): a locator inside a `test(...)` body that navigates with `gotoReference(page, url)`
# (`playwright/page-kinds.ts`, which refuses a URL under the app's origin) is that page's own markup, so the exemption is
# derived from the navigation helper.
#
# SECOND RULE (backlog row 512), every spec and helper under `playwright/` (not only the shared ones): a spec must not
# locate a demo VARIANT through the heading over it. That heading is the variant's `title.txt` (or a title derived from
# its directory), copy the owner edits freely (a title audit renamed 78 of them and a `getByRole("heading", { name })`
# locator then matched nothing and waited out a 20-minute test timeout, row 506). The variant is addressed by its slug,
# the directory name, which is the id of its demo frame: `demoFrame(page, "states")` / `demoRole(page, "row", 0,
# "states")` (`playwright/demo-frame.ts`). The gate fails on
#
#   `dx-component-variant-title` (the heading's class),
#   a `.dx-component-variant` locator whose statement filters by a heading, `hasText` or `getByText`,
#   an h3 or level-less heading locator (`getByRole("heading", { name: "..." })`, `.locator("h3", { hasText: "..." })`;
#   a variant title is an h3, so `level: 1`/`2` and `h1`/`h2` are skipped) whose literal is
#   the title of a variant under `preview/src/components/*/variants/` (title.txt, or the directory name with
#   underscores as spaces),
#
# unless the line, or the line above, carries `title-ok: <reason>` (a spec ABOUT the title itself, for example the test
# that `title.txt` replaces the derived title). Not covered: a heading locator whose literal is not a variant title
# (the gate cannot know it addresses a variant), and a variant found through a control's label inside it.
# Its scope is every `.ts` under `playwright/` except `node_modules/` and `tools/`.
#
# SCOPE (first rule): every `playwright/oracle/**/*.spec.ts` (except `oracle/reference/`), plus EXTRA_SPECS below. A
# per-component spec that asserts its OWN component's labels (`dialog.spec.ts` checking a dialog's title) is
# not a shared spec and not in scope.
#
# EXCEPTIONS are in ALLOW below (the shared allowlist engine, `scripts/lib/allow.py`), each with its reason (a needle of ''
# covers every hit in the file); an entry that matches nothing fails the run, so it is deleted when its use goes away.
#
# Usage: scripts/check-demo-text-locators.sh [FILE ...]   (default: every spec in scope, then the self-test; --self-test
# runs only the self-test; explicit files skip the allowlist stale-entry report)
# Exit status: 0 when clean.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
self="$repo_root/scripts/check-demo-text-locators.sh"

self_test() {
  local dir fails=0 n=0
  dir="$(mktemp -d)"
  trap 'rm -rf "$dir"' RETURN
  expect() {  # expect <0|1> <name> <body>
    local want="$1" name="$2" body="$3" rc=0
    printf '%s\n' "$body" >"$dir/$name.spec.ts"
    bash "$self" "$dir/$name.spec.ts" >/dev/null 2>&1 || rc=$?
    n=$((n + 1))
    if [ "$rc" -ne "$want" ]; then
      echo "check-demo-text-locators --self-test: FAIL $name: exit $rc, wanted $want" >&2
      fails=$((fails + 1))
    fi
  }
  expect 1 demo-text "test('x', async ({ page }) => { await gotoHydrated(page, u); await page.getByRole('button', { name: 'Show Dialog' }).click(); });"
  expect 0 reference-text "test('x', async ({ page }) => { await gotoReference(page, u); await page.getByRole('button', { name: 'About' }).click(); });"
  expect 1 text-beside-reference-test "test('r', async ({ page }) => { await gotoReference(page, u); });
test('x', async ({ page }) => { await gotoHydrated(page, u); await page.getByText('Show Dialog').click(); });"
  expect 1 reference-helper-spelled-bare "test('x', async ({ page }) => { await page.goto(u); await page.getByText('About').click(); });"
  expect 1 reference-in-a-comment "test('x', async ({ page }) => { // gotoReference(page, u)
  await page.getByText('About').click(); });"
  expect 0 role-and-position "test('x', async ({ page }) => { await demoRole(page, 'button', 1).click(); });"
  expect 1 variant-by-heading "test('x', async ({ page }) => { await page.locator('.dx-component-variant').filter({ has: page.getByRole('heading', { name: 'multi' }) }); });"
  expect 1 variant-title-class "test('x', async ({ page }) => { await page.locator('h3.dx-component-variant-title').first().click(); });"
  expect 1 variant-title-literal "test('x', async ({ page }) => { await page.getByRole('heading', { name: 'Disabled and non-selectable' }).click(); });"
  expect 0 variant-title-marked "test('x', async ({ page }) => { // title-ok: the title is what is asserted
  await page.locator('h3.dx-component-variant-title').first().click(); });"
  expect 0 variant-by-slug "test('x', async ({ page }) => { await demoFrame(page, 'states').getByRole('row').first().click(); });"
  expect 0 heading-not-a-title "test('x', async ({ page }) => { await page.getByRole('heading', { level: 1 }).click(); });"
  [ "$fails" -eq 0 ] || exit 1
  echo "check-demo-text-locators --self-test: OK ($n fixtures; a text locator is exempt only inside a test body that calls gotoReference; a variant is found by slug, never by its title)"
}

if [ "${1:-}" = "--self-test" ]; then self_test; exit $?; fi

python3 - "$@" <<'PYEOF'
import glob
import os
import re
import sys

sys.path.insert(0, "scripts/lib")
from allow import Allowlist  # noqa: E402

explicit = sys.argv[1:]

# Shared specs outside playwright/oracle/ (path under playwright/).
# Not in scope: `modal-exit.spec.ts` and `modal-initial-focus.spec.ts` drive the `CONSUMERS` table of
# `modal-consumers.ts`, which carries each demo's trigger and closer COPY (`trigger: "Show Dialog"`); moving it to
# role/position changes the table's type for both specs.
EXTRA_SPECS = [
    "theme-preset-contrast.spec.ts",
    "oracle-focus-restore.spec.ts",
    "overlay-switch.spec.ts",
]

# (file suffix, substring of the code line ('' = every hit in the file), reason)
ALLOW = Allowlist("check-demo-text-locators", [
    ("oracle/hydration-parity.spec.ts", "name: displayName",
     "the heading name is the route's own component id (name.replace('_', ' ')), generated per route, not demo copy"),
    ("oracle/tier1-apg/carousel.spec.ts", 'of 5"]',
     "the indicators variant's component-generated 'n of 5' tabpanel names, not demo copy"),
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
], fields=("suffix", "needle"))

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

TEST_CALL = re.compile(r"\btest(?:\.(?:only|fixme|slow|skip))?\s*\(")
GOTO_REFERENCE = re.compile(r"\bgotoReference\s*\(")

def reference_ranges(code):
    """Character ranges of the `test(...)` calls whose body navigates with `gotoReference(...)`: a vendored or
    route-fulfilled page that is not the app, so its locators are its own markup, not demo copy."""
    out = []
    for m in TEST_CALL.finditer(code):
        start = m.end() - 1
        end = matching_paren(code, start)
        if end < len(code) and GOTO_REFERENCE.search(code[start:end]):
            out.append((start, end))
    return out

errors = []
total = 0
n_reference = 0
for path in files:
    with open(path, encoding="utf-8") as fh:
        raw = fh.read()
    raw_lines = raw.split("\n")
    code = COMMENT_LINE.sub(blank, COMMENT_BLOCK.sub(blank, raw))
    code_lines = code.split("\n")
    hits = []  # (line index, label, offset)
    for rx, label in SIMPLE:
        for m in rx.finditer(code):
            hits.append((code.count("\n", 0, m.start()), label, m.start()))
    for m in GETBYROLE.finditer(code):
        start = m.end() - 1
        end = matching_paren(code, start)
        inner = code[start:end]
        if NAME_OPT.search(inner):
            hits.append((code.count("\n", 0, m.start()), "getByRole(..., { name })", m.start()))
    reference = reference_ranges(code)
    seen = set()
    for ln, label, pos in sorted(hits):
        if (ln, label) in seen:
            continue
        seen.add((ln, label))
        total += 1
        if any(lo <= pos < hi for lo, hi in reference):
            n_reference += 1
            continue
        if MARKER.search(raw_lines[ln]) or (ln > 0 and MARKER.search(raw_lines[ln - 1])):
            continue
        if ALLOW.find_all(lambda e: path.endswith(e.suffix) and e.needle in code_lines[ln]):
            continue
        errors.append(f"{path}:{ln + 1}: `{code_lines[ln].strip()[:100]}` locates by {label} (demo copy)")

# ---- second rule: a variant is addressed by its slug, never by its title (row 512) ----
def variant_titles():
    titles = set()
    for d in sorted(glob.glob("preview/src/components/*/variants/*/")):
        name = os.path.basename(d.rstrip("/"))
        if name == "main":
            continue
        titles.add(name.replace("_", " ").lower())
        tp = os.path.join(d, "title.txt")
        if os.path.exists(tp):
            with open(tp, encoding="utf-8") as fh:
                titles.add(fh.read().strip().lower())
    titles.discard("")
    return titles

TITLE_MARKER = re.compile(r"title-ok:\s*\S")
TITLE_CLASS = re.compile(r"dx-component-variant-title")
CARD = re.compile(r"""["'`]\s*[a-z0-9]*\.dx-component-variant(?![-\w])""")
HEADING_CALL = re.compile(r"""\.\s*getByRole\s*\(\s*["']heading["']|\.\s*locator\s*\(\s*["']h[1-6]["']""")
STR_LIT = re.compile(r"""["'`]([^"'`\n]{2,})["'`]""")
CARD_TEXT = re.compile(r"""getByRole\s*\(\s*["']heading["']|\bhasText\s*:|\.getByText\s*\(|\bh[1-6]\b""")

def statement(code, pos):
    end = code.find(";", pos)
    return code[pos:end if end != -1 else len(code)]

def title_files():
    out = []
    for root, dirs, names in os.walk("playwright"):
        dirs[:] = sorted(d for d in dirs if d not in {"node_modules", "tools"})
        out += [os.path.join(root, n) for n in sorted(names) if n.endswith(".ts")]
    return sorted(out)

titles = variant_titles()
vfiles = explicit or title_files()
n_title = 0
for path in vfiles:
    with open(path, encoding="utf-8") as fh:
        raw = fh.read()
    raw_lines = raw.split("\n")
    code = COMMENT_LINE.sub(blank, COMMENT_BLOCK.sub(blank, raw))
    code_lines = code.split("\n")
    vhits = []  # (line index, label)
    for m in TITLE_CLASS.finditer(code):
        vhits.append((code.count("\n", 0, m.start()), "the variant title class"))
    for m in CARD.finditer(code):
        if CARD_TEXT.search(statement(code, m.start())):
            vhits.append((code.count("\n", 0, m.start()), "a variant card filtered by heading or text"))
    for m in HEADING_CALL.finditer(code):
        start = code.index("(", m.start())
        inner = code[start:matching_paren(code, start)]
        lv = re.search(r"\blevel\s*:\s*(\d)", inner) or re.search(r"""locator\s*\(\s*["']h(\d)""", code[m.start():start + 1])
        if lv and lv.group(1) != "3":
            continue  # a variant title is an h3; the page's h1 and a section's h2 are not
        for lit in STR_LIT.findall(inner):
            if lit.strip().lower() in titles:
                vhits.append((code.count("\n", 0, m.start()), f'a heading named "{lit}", a variant title'))
                break
    for ln, label in sorted(set(vhits)):
        if TITLE_MARKER.search(raw_lines[ln]) or (ln > 0 and TITLE_MARKER.search(raw_lines[ln - 1])):
            n_title += 1
            continue
        errors.append(f"{path}:{ln + 1}: `{code_lines[ln].strip()[:100]}` locates a variant through its displayed title ({label})")

if errors:
    for e in errors:
        print(e)
    print(f"\ncheck-demo-text-locators: {len(errors)} problem(s). Locate a demo's controls by role and position "
          f"(`demoRole(page, role, n)` / `popupItem(...)` in playwright/demo-frame.ts); if the name is NOT demo copy "
          f"(a component's own aria-label, site chrome, a spec-authored fixture), add "
          f"`// copy-ok: <reason>` on that line or the line above, or a reasoned ALLOW entry in this script; a vendored "
          f"reference page is navigated with `gotoReference(page, url)` in the same `test(...)` and needs neither. A variant is "
          f"addressed by its slug, never by its title: `demoFrame(page, \"<slug>\")` / `demoRole(page, role, n, \"<slug>\")` "
          f"(playwright/demo-frame.ts; the slug is the directory name under variants/); a spec about the title itself "
          f"carries `// title-ok: <reason>`.", file=sys.stderr)
    sys.exit(1)

problems = ALLOW.problems(stale=not explicit)
if problems:
    for p in problems:
        print(f"FAIL: {p}")
    sys.exit(1)
print(f"check-demo-text-locators: OK ({len(vfiles)} playwright file(s) with no variant located by its title, {n_title} marked title-ok; {len(files)} shared spec file(s): {total} text locator(s), {n_reference} of them on a "
      f"gotoReference page, the rest marked or allowlisted as not demo copy; {len(ALLOW)} allowlist entr{'y' if len(ALLOW) == 1 else 'ies'})")
PYEOF

if [ "$#" -eq 0 ]; then self_test; fi
