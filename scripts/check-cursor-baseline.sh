#!/usr/bin/env bash
#
# check-cursor-baseline.sh
#
# The cursor of an interactive element is decided ONCE, by the theme, so every pressable element shows the
# same cursor and an app chooses it in one place (backlog row 381 records the decision). Per-sheet
# `cursor: pointer` / `cursor: default` declarations made one press look different in each component.
#
# `preview/assets/dx-components-theme.css` ("Interactive-element cursor") has ONE zero-specificity rule,
# the same shape as the `touch-action` and `user-select` baselines, that gives every pressable `dx-` element
# `cursor: var(--dx-interactive-element-cursor, default)`; `<html data-dx-interactive-element-cursor="pointer">`
# (or any container) sets the variable to `pointer`. shadcn v4 leaves a button at the arrow, so that is the
# default. This gate makes the per-sheet copies inexpressible again, so the class cannot grow back:
#
#  1. No component sheet (`preview/src/components/*/style.css`) and no `preview/assets/main.css` rule may
#     declare `cursor: pointer`, `cursor: default` or `cursor: auto`: at (0,1,0) it out-ranks the theme rule
#     and the option silently stops reaching that element. `grab`/`grabbing` (drag), `*-resize`, `text`,
#     `not-allowed`, `help` and friends stay the component's own. The ONE spelling for a clickable thing the
#     theme list cannot name (a `label` that wraps a radio, a drag thumb, an SVG hit band, a selectable chip)
#     is `cursor: var(--dx-interactive-element-cursor, default)`; any other `var()` fails.
#     A genuine exception goes on ALLOW below with its reason; an entry that matches nothing is a FAILURE.
#  2. The theme has no other `cursor: pointer`/`default` declaration than the baseline rule (a second one, such as the shared
#     menu-item row's, has equal specificity and wins by coming later in the file).
#  3. The theme rule itself is still there, and complete: a zero-specificity selector list (every entry a
#     `:where()`) covering `button`, `summary`, `select` (both class-prefix spellings), the roles button,
#     menuitem, menuitemcheckbox, menuitemradio, option, tab, switch, checkbox, radio and slider, and the
#     `[data-dx-menu-item]` row, declaring `cursor: var(--dx-interactive-element-cursor, default)`; and the two
#     attribute rules (`="pointer"` and `="default"`) that set the variable.
#  4. The wiring: `PREPAINT_JS` (preview/src/theme.rs) and its byte-identical copy in preview/index.html list
#     the `dx-interactive-element-cursor` key and accept only `pointer` and `default` for it.
#
# Comments are stripped before scanning, so prose that mentions a cursor is fine.
#
# Usage: scripts/check-cursor-baseline.sh [--self-test]
# Exit status: 0 when clean.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import glob
import re
import sys

THEME = "preview/assets/dx-components-theme.css"
MAIN = "preview/assets/main.css"
THEME_RS = "preview/src/theme.rs"
INDEX = "preview/index.html"
VAR = "--dx-interactive-element-cursor"
TOKEN = f"var({VAR}, default)"
KEY = "dx-interactive-element-cursor"

# (stylesheet, whitespace-normalised selector, reason the baseline cannot cover it)
ALLOW = []

REQUIRED_TAGS = ["button", "summary", "select"]
REQUIRED_ROLES = ["button", "menuitem", "menuitemcheckbox", "menuitemradio", "option", "tab", "switch",
                  "checkbox", "radio", "slider"]

COMMENT = re.compile(r"/\*.*?\*/", re.S)


def blank_comments(text):
    # Same-length whitespace, newlines kept, so reported line numbers are the file's.
    return COMMENT.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)


DECL = re.compile(r"(?<![\w-])cursor\s*:\s*([^;}]*)", re.I)


def declarations(text):
    """Yield (line, selector, value) for every `cursor` declaration of comment-stripped CSS text."""
    for m in DECL.finditer(text):
        pos = m.start()
        depth, i = 0, pos
        while i >= 0:
            c = text[i]
            if c == "}":
                depth += 1
            elif c == "{":
                if depth == 0:
                    break
                depth -= 1
            i -= 1
        j = max(text.rfind("}", 0, i), text.rfind(";", 0, i), text.rfind("{", 0, i))
        selector = " ".join(text[j + 1:i].split())
        value = " ".join(m.group(1).replace("!important", "").split())
        yield text.count("\n", 0, pos) + 1, selector, value


def split_top(selector):
    out, depth, cur = [], 0, ""
    for ch in selector:
        if ch in "([":
            depth += 1
        elif ch in ")]":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur)
            cur = ""
        else:
            cur += ch
    out.append(cur)
    return [s.strip() for s in out if s.strip()]


def check(sheets, theme, main, theme_rs, index_html):
    """Return (violations, used_allow_entries) for the given file texts. `sheets` maps path -> text."""
    violations, used = [], set()
    scanned = dict(sheets)
    scanned[MAIN] = main
    for path, raw in scanned.items():
        for line, selector, value in declarations(blank_comments(raw)):
            low = value.lower()
            if low in ("pointer", "default", "auto"):
                hit = next((a for a in ALLOW if a[0] == path and a[1] == selector), None)
                if hit:
                    used.add(id(hit))
                    continue
                violations.append(
                    f"{path}:{line}: `{selector}` declares `cursor: {value}`; the theme's baseline decides it "
                    f"(use `cursor: {TOKEN}` only for an element its list cannot name)")
            elif low.startswith("var("):
                if low != TOKEN.lower():
                    violations.append(f"{path}:{line}: `{selector}` declares `cursor: {value}`; the only var() "
                                      f"spelling is `cursor: {TOKEN}`")
    for a in ALLOW:
        if id(a) not in used:
            violations.append(f"stale ALLOW entry {a[0]} ({a[1]}): it matches no `cursor` declaration; delete it")

    # 2 + 3: the theme.
    t = blank_comments(theme)
    baseline = []
    for line, selector, value in declarations(t):
        if value == TOKEN:
            baseline.append((line, selector))
        elif value.lower() in ("pointer", "default"):
            violations.append(f"{THEME}:{line}: `{selector}` declares `cursor: {value}` outside the baseline rule; "
                              "it would out-rank or be out-ranked by it depending on source order")
    if len(baseline) != 1:
        violations.append(f"{THEME}: expected exactly one rule declaring `cursor: {TOKEN}`, found {len(baseline)}")
    else:
        line, selector = baseline[0]
        shape = re.fullmatch(r":where\(\s*:is\((.*)\)\s*:not\((.*)\)\s*\)", selector)
        if not shape:
            violations.append(f"{THEME}:{line}: the baseline rule must be `:where(:is(<list>):not(<disabled>))`: zero "
                              "specificity, and a disabled control left out of the list")
        else:
            entries = split_top(shape.group(1))
            for tag in REQUIRED_TAGS:
                for spelling in (f'{tag}[class^="dx-"]', f'{tag}[class*=" dx-"]'):
                    if spelling not in entries:
                        violations.append(f"{THEME}:{line}: the baseline rule lost `{spelling}`")
            for role in REQUIRED_ROLES:
                for spelling in (f'[role="{role}"][class^="dx-"]', f'[role="{role}"][class*=" dx-"]'):
                    if spelling not in entries:
                        violations.append(f"{THEME}:{line}: the baseline rule lost `{spelling}`")
            if "[data-dx-menu-item]" not in entries:
                violations.append(f"{THEME}:{line}: the baseline rule lost `[data-dx-menu-item]`")
            disabled = split_top(shape.group(2))
            for spelling in (":disabled", '[aria-disabled="true"]', '[data-disabled="true"]'):
                if spelling not in disabled:
                    violations.append(f"{THEME}:{line}: the baseline rule no longer leaves `{spelling}` out "
                                      "(a disabled control keeps its own cursor)")
    for value in ("pointer", "default"):
        rule = re.search(r'\[data-dx-interactive-element-cursor="%s"\]\s*\{([^{}]*)\}' % value, t)
        if not rule or not re.search(r"%s\s*:\s*%s\s*;" % (re.escape(VAR), value), rule.group(1)):
            violations.append(f'{THEME}: the `[data-dx-interactive-element-cursor="{value}"]` rule that sets '
                              f"`{VAR}: {value}` is missing")

    # 4: the pre-paint wiring, in both copies.
    for path, text in ((THEME_RS, theme_rs), (INDEX, index_html)):
        if f"'{KEY}'" not in text or f"(k!=='{KEY}'||v==='pointer'||v==='default')" not in text:
            violations.append(f"{path}: PREPAINT_JS must list `'{KEY}'` and accept only `pointer` and `default` for it")
    return violations, used


def load():
    sheets = {p: open(p, encoding="utf-8").read() for p in sorted(glob.glob("preview/src/components/*/style.css"))}
    rd = lambda p: open(p, encoding="utf-8").read()
    return sheets, rd(THEME), rd(MAIN), rd(THEME_RS), rd(INDEX)


def report(violations):
    print("check-cursor-baseline: FAIL -- the cursor of an interactive element is the theme's one rule "
          "(dx-components-theme.css, \"Interactive-element cursor\"):")
    for v in violations:
        print(f"  {v}")


def self_test():
    sheets, theme, main, theme_rs, index_html = load()
    base, _ = check(sheets, theme, main, theme_rs, index_html)
    if base:
        report(base)
        print("check-cursor-baseline: self-test needs a clean tree first", file=sys.stderr)
        return 1
    any_sheet = next(iter(sheets))
    failures = []

    def expect_red(name, **kw):
        args = dict(sheets=sheets, theme=theme, main=main, theme_rs=theme_rs, index_html=index_html)
        args.update(kw)
        v, _ = check(**args)
        if not v:
            failures.append(name)

    def expect_green(name, **kw):
        args = dict(sheets=sheets, theme=theme, main=main, theme_rs=theme_rs, index_html=index_html)
        args.update(kw)
        v, _ = check(**args)
        if v:
            failures.append(f"{name} (should be clean: {v[0]})")

    def with_sheet(extra):
        s = dict(sheets)
        s[any_sheet] = sheets[any_sheet] + "\n" + extra
        return s

    expect_red("a component sheet declares cursor: pointer", sheets=with_sheet(".dx-x { cursor: pointer; }"))
    expect_red("a component sheet declares cursor: default", sheets=with_sheet(".dx-x { cursor: default; }"))
    expect_red("cursor: pointer !important", sheets=with_sheet(".dx-x { cursor: pointer !important; }"))
    expect_red("cursor: pointer inside @media",
               sheets=with_sheet("@media (hover: hover) { .dx-x:hover { cursor: pointer; } }"))
    expect_red("a foreign var()", sheets=with_sheet(".dx-x { cursor: var(--other, pointer); }"))
    expect_red("main.css declares cursor: pointer", main=main + "\nbutton.dx-x { cursor: pointer; }\n")
    expect_green("grab/not-allowed/text/resize stay the component's",
                 sheets=with_sheet(".dx-x { cursor: grab; } .dx-y { cursor: not-allowed; } .dx-z { cursor: text; } "
                                   ".dx-w { cursor: col-resize; }"))
    expect_green("the sanctioned var() spelling", sheets=with_sheet(f".dx-x {{ cursor: {TOKEN}; }}"))
    expect_green("a comment that says cursor: pointer", sheets=with_sheet("/* cursor: pointer was here */"))
    # The theme.
    expect_red("the baseline rule deleted", theme=theme.replace(f"cursor: {TOKEN};", "cursor: auto;"))
    def baseline_block(transform):
        end = theme.index("{\n  cursor: " + TOKEN)
        start = theme.rfind("}\n\n", 0, end) + 3
        return theme[:start] + transform(theme[start:end]) + theme[end:]

    def drop_lines(needle):
        return lambda block: "".join(l for l in block.splitlines(True) if needle not in l)

    expect_red("a role dropped from the baseline list", theme=baseline_block(drop_lines('[role="tab"]')))
    expect_red("`select` dropped from the baseline list", theme=baseline_block(drop_lines("select[")))
    expect_red("the menu-item row dropped from the list", theme=baseline_block(drop_lines("data-dx-menu-item")))
    expect_red("the rule loses :where() (non-zero specificity)",
               theme=baseline_block(lambda b: b.replace(":where(\n  :is(", ":is(\n  :is(", 1)))
    expect_red("a disabled control no longer left out",
               theme=baseline_block(lambda b: b.replace(', [data-disabled="true"])', ")", 1)))
    expect_red("the :disabled exclusion deleted",
               theme=baseline_block(lambda b: b.replace(":disabled, ", "", 1)))
    expect_red("the pointer attribute rule deleted", theme=theme.replace(f"  {VAR}: pointer;", "  color: inherit;"))
    expect_red("the default attribute rule deleted", theme=theme.replace(f"  {VAR}: default;", "  color: inherit;"))
    expect_red("a second cursor: pointer in the theme", theme=theme + "\n:where([data-dx-menu-item]) { cursor: pointer; }\n")
    # The wiring.
    expect_red("theme.rs loses the key", theme_rs=theme_rs.replace(f"'{KEY}'", "'gone'"))
    expect_red("index.html accepts any value", index_html=index_html.replace(f"(k!=='{KEY}'||v==='pointer'||v==='default')", "true"))
    # A stale ALLOW entry fails.
    ALLOW.append((any_sheet, ".dx-nothing-matches", "self-test"))
    v, _ = check(sheets, theme, main, theme_rs, index_html)
    ALLOW.pop()
    if not any("stale ALLOW" in x for x in v):
        failures.append("a stale ALLOW entry")
    if failures:
        print("check-cursor-baseline: self-test FAIL -- these mutations were not caught:")
        for f in failures:
            print(f"  {f}")
        return 1
    print("check-cursor-baseline: self-test PASS (every mutation caught)")
    return 0


if "--self-test" in sys.argv[1:]:
    sys.exit(self_test())

sheets, theme, main, theme_rs, index_html = load()
if not sheets:
    print("check-cursor-baseline: no stylesheets found (run from a checkout)", file=sys.stderr)
    sys.exit(2)
violations, _ = check(sheets, theme, main, theme_rs, index_html)
if violations:
    report(violations)
    sys.exit(1)
print(f"check-cursor-baseline: PASS ({len(sheets)} component stylesheets, the theme rule and the pre-paint wiring)")
PYEOF
