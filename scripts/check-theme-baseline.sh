#!/usr/bin/env bash
#
# check-theme-baseline.sh
#
# What the theme owns once is not declared per component. No component `style.css` may declare `touch-action:
# manipulation` (the theme's one zero-specificity rule covers every `dx-` button, link, form control, `tabindex` and
# interactive ARIA role; a drag surface may still declare `none` or `pan-*`), paint a `::backdrop` (the modal scrim is
# the theme's one rule on `dialog[class^="dx-"]`, retuned by the `--dx-overlay-*` tokens), reset with `all:` (it
# outranks every zero-specificity theme rule), or re-declare `user-select: none` for an element the theme's one
# control rule already covers (every other use sits on the gate's reasoned `USER_SELECT_ALLOW`). The theme's
# `:where(dialog[class^="dx-"]) { padding: 0 }` reset means every dialog sheet declares its own inset. All these theme
# rules must still exist, and the vendor-prefixed line carries the one scoped `property-no-vendor-prefix` disable.
#
#  1. `touch-action: manipulation` (no double-tap-to-zoom on a control). Declared per sheet it is copied
#     into dozens of stylesheets and kept in step by hand with a `main.css` `!important` backstop,
#     which hides every sheet that forgets it. The theme rule
#     covers every `dx-` element that is a button, link, form control, `summary`, focusable by
#     `tabindex`, or an interactive ARIA widget role. A component sheet may still declare a DIFFERENT
#     value where the element is a drag surface or a scroller (`none`, `pan-x`, `pan-y`, ...); it may
#     not re-declare `manipulation`. If you are here because an element is clickable but not in the
#     theme's list, give it a role or `tabindex` (which it needs for the keyboard anyway) or extend
#     the theme rule's list -- do not paste the declaration.
#
#  2. The modal scrim: a `::backdrop` rule that paints (`background`, `backdrop-filter`, `transition`).
#     Dialog, AlertDialog, Sheet, Drawer, CommandDialog and the modal Popover share ONE
#     rule on any `<dialog>` with a `dx-` class, driven by the
#     `--dx-overlay-*` tokens. A component sheet has no `::backdrop` rule to write; to retune the
#     scrim change the tokens, to remove it use `overlay: false` (`data-dx-overlay="off"`).
#
#  3. (see 5) The theme rules themselves: the touch rule, the scrim, and
#     the dialog-box rule `:where(dialog[class^="dx-"]) { padding: 0 }`, which takes away the UA's
#     `dialog { padding: 1em }` (font-size dependent: a panel's `font-size` change silently moves
#     the inset, 16 -> 14px) so a dialog sheet's inset is only ever one it declares.
#
#  4. `user-select: none` on a control (row 229: interactive elements like buttons
#     should not allow user text selection). Declared per sheet, only the parts that remember to say so
#     are unselectable (menu items, select options, command items, the calendar arrows);
#     Button, Tab, Accordion trigger, Switch, Toggle... show a selection on a double-click. The theme has
#     ONE zero-specificity rule (`-webkit-user-select` + `user-select: none`) for every `dx-` button,
#     `[role=button]`, `summary` and menu-item/option/tab/switch/checkbox/radio/slider role, and deliberately
#     NOT for text fields, `spinbutton` segments, the `[tabindex]` catch-all or `a[href]`. A component sheet
#     may not re-declare it for an element that rule covers (its last compound is `button`/`summary` or has a
#     covered `[role=...]`), and, because a class selector hides what element it lands on, EVERY other
#     `user-select: none` in a component sheet must be on USER_SELECT_ALLOW below with the reason the baseline
#     does not reach it (a non-interactive label, a drag surface). A stale entry fails.
#
#  4b. `all:` (unset/initial/revert) is banned in component stylesheets (row 239): at (0,1,0) it beats every
#     zero-specificity rule above on the element it lands on (Switch and the slider and colour-picker thumbs lose
#     user-select, touch-action and press feedback to it). Write the resets out instead.
#
#  5. The theme rules themselves are still there (a deleted rule would otherwise pass 1, 2 and 4 and silently
#     drop the behaviour from every component at once): the touch rule, the scrim, the dialog-box rule, and the
#     `user-select` rule with its exclusions intact.
#
# Comments are stripped before scanning, so prose that mentions either is fine.
#
# Usage: scripts/check-theme-baseline.sh [FILE ...]   (default: preview/src/components/*/style.css;
# explicit files are for self-tests, and skip the theme arms)
# Exit status: 0 when clean.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import glob
import re
import sys

THEME = "preview/assets/dx-components-theme.css"
explicit = sys.argv[1:]
files = explicit or sorted(glob.glob("preview/src/components/*/style.css"))
if not files:
    print("check-theme-baseline: no stylesheets found (run from a checkout)", file=sys.stderr)
    sys.exit(2)

sys.path.insert(0, "scripts/lib")
from allow import Allowlist  # noqa: E402
# blank_comments: same-length whitespace, newlines kept, so reported line numbers are the file's.
from css import blank_comments, rules as css_rules, split_top as split_top_raw, subject_compound  # noqa: E402

# Elements the theme's user-select rule covers: tag button/summary, or one of these roles (button included).
BASELINE_TAGS = {"button", "summary"}
BASELINE_ROLES = {"button", "menuitem", "menuitemcheckbox", "menuitemradio", "option", "tab", "switch",
                  "checkbox", "radio", "slider"}
# (stylesheet, whitespace-normalised selector entry, reason the baseline does not reach it)
USER_SELECT_ALLOW = Allowlist("check-theme-baseline", [
    ("preview/src/components/avatar/style.css", ".dx-avatar",
     "a non-interactive circle: the initials fallback is a label, not a control"),
    ("preview/src/components/avatar/style.css", ".dx-avatar-badge",
     "a non-interactive status dot on the avatar"),
    ("preview/src/components/calendar/style.css", ".dx-calendar-grid-day-header",
     "a non-interactive weekday heading (`div`), not a control"),
    ("preview/src/components/calendar/style.css", ".dx-calendar-grid-weeknum",
     "a non-interactive week-number cell, not a control"),
    ("preview/src/components/calendar/style.css", ".dx-calendar-caption-text",
     "the non-interactive month/year caption text (`span`)"),
    ("preview/src/components/calendar/style.css", ".dx-calendar-month-select-value",
     "the month dropdown's value readout (`div` inside the select, not a button of its own)"),
    ("preview/src/components/calendar/style.css", ".dx-calendar-year-select-value",
     "the year dropdown's value readout (`div` inside the select, not a button of its own)"),
    ("preview/src/components/carousel/style.css", ".dx-carousel-content[data-draggable=\"true\"][data-dragging=\"true\"]",
     "a drag surface (carousel content), unselectable only while the pointer drags it"),
    ("preview/src/components/drag_and_drop_list/style.css", ".dx-drag-and-drop-list-item",
     "a draggable `li` row: the drag gesture must not sweep a text selection (not a button/role the baseline lists)"),
    ("preview/src/components/chart/style.css", ".dx-chart",
     "a data graphic (`div` container): axis labels, legend and tooltip are part of the picture, so a drag or "
     "double-tap across it must not select them; the container is not a button/role, and the focusable "
     "`[data-slot=chart]` root is a `[tabindex]` the baseline leaves selectable on purpose"),
    ("preview/src/components/kbd/style.css", ".dx-kbd",
     "a key cap (`kbd` element, not a button or widget role): shadcn's Kbd carries `select-none`"),
    ("preview/src/components/label/style.css", ".dx-label",
     "a form label (`label` element, not a control role): shadcn's Label carries `select-none`; it labels a field and "
     "the field itself (input, textarea) stays selectable"),
    ("preview/src/components/input_group/style.css", ".dx-input-group-addon",
     "the icon, text or button row beside a field (`div`, not a control role): shadcn's InputGroupAddon carries "
     "`select-none`, so a drag across the group selects the field's value and not the addon"),
    ("preview/src/components/color_picker/style.css", ".dx-color-picker-area-thumb",
     "the area's draggable thumb is a `div` (not a button or widget role): a drag must not sweep a text selection"),
    ("preview/src/components/drawer/style.css", ".dx-drawer[data-dragging=\"true\"]",
     "the drawer panel while the user drags it (descendants included): a drag surface"),
    ("preview/src/components/drawer/style.css", ".dx-drawer-handle",
     "the drag handle bar (`div`): a drag can start on it before `data-dragging` flips"),
    ("preview/src/components/sidebar/style.css", ".dx-sidebar-menu-badge",
     "a decorative count badge beside a menu button (`pointer-events: none`), not a control"),
], fields=("file", "entry"), source="scripts/check-theme-baseline.sh")

# No component rule may reset with the `all` shorthand: `all: unset` (0,1,0) beats every zero-specificity theme
# baseline on that element (user-select, touch-action, press feedback, box-sizing), so the class is banned outright
# (row 239). Spell the resets out (appearance, background, border, padding, margin, font, color, outline).
ALL_RESET = re.compile(r"(?<![\w-])all\s*:\s*(unset|initial|revert|revert-layer)\b", re.I)

TOUCH = re.compile(r"(?<![\w-])touch-action\s*:\s*manipulation\b", re.I)
PAINTS = re.compile(r"(?<![\w-])(background(-color|-image)?|backdrop-filter|transition(-[a-z]+)?)\s*:", re.I)

USER_SELECT = re.compile(r"(?<![\w-])(-webkit-)?user-select\s*:\s*none\b", re.I)

def selector_list(selector):
    """The whitespace-normalised, non-empty entries of a selector list."""
    return [" ".join(e.split()) for e in split_top_raw(selector) if e.strip()]

def covered_by_baseline(entry):
    c = subject_compound(entry)
    tag = re.match(r"^[a-z][\w-]*", c)
    if tag and tag.group(0) in BASELINE_TAGS:
        return True
    return any(r in BASELINE_ROLES for r in re.findall(r"\[role=[\"']?([\w-]+)[\"']?\]", c))

errors = []
for path in files:
    with open(path, encoding="utf-8") as fh:
        raw = fh.read()
    text = blank_comments(raw)
    sheet_rules = css_rules(raw)
    for m in TOUCH.finditer(text):
        line = text.count("\n", 0, m.start()) + 1
        errors.append(f"{path}:{line}: `touch-action: manipulation` is declared once, in {THEME} "
                      f"(the shared `dx-` control rule); delete it here")
    for r in sheet_rules:
        if not re.search("::backdrop", r.prelude, re.I):
            continue
        selector = " ".join(r.prelude.split())
        # The overlay-off switch is a theme rule, never a component one; a component has none to write.
        if PAINTS.search(r.body):
            line = r.brace_line
            errors.append(f"{path}:{line}: `{selector}` paints a `::backdrop`; the modal scrim is the one "
                          f"shared rule in {THEME} (\"The modal scrim, once\"), retuned by the "
                          f"--dx-overlay-* tokens")

    # Arm 4: no component sheet re-declares the baseline's user-select, and every other one says why.
    if path.replace("\\", "/").startswith("preview/src/components/"):
        for r in sheet_rules:
            line = r.brace_line
            if ALL_RESET.search(r.body):
                for entry in selector_list(r.prelude):
                    errors.append(f"{path}:{line}: `{entry}` resets with `all:`, which beats every zero-specificity "
                                  f"theme baseline (user-select, touch-action, press feedback, box-sizing) on that "
                                  f"element; spell the resets out (appearance, background, border, padding, margin, "
                                  f"font, color, outline) -- `all:` is banned in component stylesheets (row 239)")
                continue
            if not USER_SELECT.search(r.body):
                continue
            for entry in selector_list(r.prelude):
                if covered_by_baseline(entry):
                    errors.append(f"{path}:{line}: `{entry}` re-declares `user-select: none` for an element the "
                                  f"theme's shared control rule already covers ({THEME}, \"Controls are not "
                                  f"text\"); delete it")
                    continue
                if not USER_SELECT_ALLOW.find_all(lambda e: e.file == path and e.entry == entry):
                    errors.append(f"{path}:{line}: `{entry}` declares `user-select: none`: if the element is a "
                                  f"button, summary or widget role the theme's shared control rule already covers it "
                                  f"(delete this); if it is a drag surface or a non-interactive label, add it to "
                                  f"USER_SELECT_ALLOW in scripts/check-theme-baseline.sh with that reason")

if not explicit:
    errors += [f"scripts/check-theme-baseline.sh: USER_SELECT_ALLOW: {p}" for p in USER_SELECT_ALLOW.problems()]
    with open(THEME, encoding="utf-8") as fh:
        theme = blank_comments(fh.read())
    # Arm 3: the theme still has both rules.
    touch_rule = re.search(r"([^{}]*)\{\s*touch-action\s*:\s*manipulation\s*;?\s*\}", theme)
    if not touch_rule:
        errors.append(f"{THEME}: the shared `touch-action: manipulation` rule is gone")
    else:
        sel = touch_rule.group(1)
        for needed in ['button[class^="dx-"]', '[role="menuitem"][class^="dx-"]', 'a[href][class^="dx-"]',
                       '[role="spinbutton"][class^="dx-"]', '[tabindex]:not([tabindex="-1"])[class^="dx-"]']:
            if needed not in sel:
                errors.append(f"{THEME}: the shared touch-action rule no longer lists `{needed}`")
        if re.search(r"(^|,)\s*(button|a\[href\]|input|\[role=[^\]]*\])\s*(,|$)", sel):
            errors.append(f"{THEME}: the shared touch-action rule has a bare (non-`dx-`) selector; it must stay "
                          f"scoped to the `dx-` class namespace like the box-model rule")
    scrim = re.search(r"dialog\[class\^=\"dx-\"\][^{}]*::backdrop[^{}]*\{[^{}]*backdrop-filter\s*:\s*none", theme)
    scrim_open = re.search(r"\[open\]\[data-state=\"open\"\][^{}]*::backdrop[^{}]*\{[^{}]*--dx-overlay-scrim", theme)
    if not scrim or not scrim_open:
        errors.append(f"{THEME}: the shared modal-scrim `::backdrop` rules (resting + `[open][data-state=\"open\"]`) are gone")
    # Arm 3b: the dialog box's UA padding (`1em`, font-size dependent) is reset once, at zero specificity.
    pad_ok = all(
        re.search(r":where\(\s*dialog\[class" + op + r"=\"" + lead + r"dx-\"\]\s*\)[^{}]*\{[^{}]*(?<![\w-])padding\s*:\s*0\s*;", theme)
        for op, lead in (("\\^", ""), ("\\*", " "))
    )
    if not pad_ok:
        errors.append(f"{THEME}: the shared dialog-box rule `:where(dialog[class^=\"dx-\"]) {{ padding: 0 }}` "
                      f"(the UA's font-size-dependent `padding: 1em` reset, \"The dialog box, once\") is gone")

    # Arm 5b: the theme's shared user-select rule is there, with its inclusions and its exclusions.
    us_rule = re.search(r"([^{}]*)\{[^{}]*-webkit-user-select\s*:\s*none\s*;[^{}]*(?<![\w-])user-select\s*:\s*none\s*;[^{}]*\}", theme)
    if not us_rule:
        errors.append(f"{THEME}: the shared control rule `-webkit-user-select: none; user-select: none` is gone "
                      f"(buttons, tabs, triggers would be text-selectable again)")
    else:
        sel = us_rule.group(1)
        for needed in ['button[class^="dx-"]', '[role="button"][class^="dx-"]', 'summary[class^="dx-"]',
                       '[role="menuitem"][class^="dx-"]', '[role="option"][class^="dx-"]', '[role="tab"][class^="dx-"]',
                       '[role="switch"][class^="dx-"]', '[role="checkbox"][class^="dx-"]',
                       '[role="radio"][class^="dx-"]', '[role="slider"][class^="dx-"]']:
            if needed not in sel:
                errors.append(f"{THEME}: the shared user-select rule no longer lists `{needed}`")
        for banned in ["input", "textarea", "select[", "spinbutton", "[tabindex", "a[href"]:
            if banned in sel:
                errors.append(f"{THEME}: the shared user-select rule lists `{banned}`: text fields, spinbutton "
                              f"segments, the [tabindex] catch-all (prose, code) and links must stay selectable")
        if re.search(r"(^|,)\s*(button|summary|\[role=[^\]]*\])\s*(,|$)", sel):
            errors.append(f"{THEME}: the shared user-select rule has a bare (non-`dx-`) selector; it must stay "
                          f"scoped to the `dx-` class namespace")

if errors:
    for e in errors:
        print(e)
    print(f"\ncheck-theme-baseline: {len(errors)} problem(s). See the header of scripts/check-theme-baseline.sh.",
          file=sys.stderr)
    sys.exit(1)

print(f"check-theme-baseline: OK ({len(files)} stylesheet(s): no per-component touch-action: manipulation, "
      f"::backdrop scrim or covered user-select: none; theme rules present)")
PYEOF
