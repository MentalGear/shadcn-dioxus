#!/usr/bin/env bash
#
# check-open-focus.sh
#
# Where focus goes when an overlay opens, and whether it rings, is decided in ONE place
# (docs/backlog.md rows 206 and 226): `primitives/src/modality.rs` writes the interaction modality
# (keyboard | mouse | touch | pen) to `<html data-dx-modality>` once, `dxOpenFocus` there is the one
# open-focus policy (`use_dialog_open_driver`, the non-modal Popover's show script, and the Command
# palette through `data-dx-autofocus`), and `dx-components-theme.css` ("Focus modality") draws a ring
# only for the keyboard. Every overlay that picked its own focus target used to be its own instance of
# the same bug on iOS: a ring around the panel, a text field focused while the panel was still being
# placed, so the software keyboard rose and moved the layout.
#
# This gate keeps a second, private open-focus decision from coming back. In every Rust source of
# `primitives/src` and `preview/src` that is on an open path -- it mentions `showModal`, `showPopover`,
# `use_dialog_open_driver`, `use_popover_sync`, `use_popover_shown_while_mounted`, `PopoverKind` or
# `CommandDialog`, or is one of the listed forced-focus files (roving collection focus, Command, Date
# Picker, Checkbox, Switch) -- it fails on
#
#   1. an `autofocus` ATTRIBUTE (`autofocus: ..`, `"autofocus"`, `autofocus=`),
#   2. a `.set_focus(true)` (imperative focus of a mounted element), and
#   3. a `.focus(` / `.focus()` call in a script string,
#   4. a `:focus-visible` READ in a script (`matches(':focus-visible')`, `closest(..)`, `querySelector(..)`):
#      the decision is the modality, never the browser's focus-ring heuristic, which is wrong on iOS
#      (a tapped button takes no focus, a previously tapped field always matches),
#
# unless the line, or one of the two lines above it, carries `focus-ok: <reason>` (a restore to the
# trigger, roving focus inside an open menu, Safari's click-focus remedy for a mouse press, ...). A
# `focus-ok` marker that no longer sits on or just above such a call is STALE and fails too.
# `primitives/src/modality.rs` is the helper itself and is not scanned.
#
# Also checked: the three copies of the modality writer (`MODALITY_JS` in modality.rs, inside
# `PREPAINT_JS` in preview/src/theme.rs, inside the `<script>` of preview/index.html) are identical, the
# attribute is written nowhere else, and the theme still holds the two halves of the ring rule (the
# non-field ring tokens resolve to nothing outside the keyboard modality, a programmatically focused
# container draws no outline) -- a deleted rule would otherwise silently bring the iOS ring back on every
# component at once.
#
# Comment-only lines are ignored for 1-4. Usage: scripts/check-open-focus.sh [FILE=repo/name ...]
# (explicit files are scanned as given and skip the cross-file checks; used to prove the gate red).
# Exit 0: clean. Exit 1: offending file:line on stdout.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import glob
import os
import re
import sys

OPEN_TOKENS = re.compile(
    r"showModal|showPopover|use_dialog_open_driver|use_popover_sync|use_popover_shown_while_mounted|PopoverKind|CommandDialog"
)
# Forced-focus files that are on an open path without mentioning the machinery above.
EXTRA = {
    "primitives/src/collection.rs",
    "primitives/src/command.rs",
    "primitives/src/date_picker.rs",
    "primitives/src/checkbox.rs",
    "primitives/src/switch.rs",
    "preview/src/components/command/component.rs",
}
HELPER = "primitives/src/modality.rs"

AUTOFOCUS = re.compile(r"""(^\s*autofocus\s*[:,])|("autofocus")|(\bautofocus=)""")
SET_FOCUS = re.compile(r"\.set_focus\(\s*true\s*\)")
FOCUS_CALL = re.compile(r"(?<!timers)\.focus\(\s*[){]")
FV_READ = re.compile(r"""(matches|closest|querySelector(All)?)\(\s*['"][^'"]*:focus-visible""")
MARKER = re.compile(r"focus-ok:\s*(\S.*)")

explicit = [a for a in sys.argv[1:] if a]
files = {}
if explicit:
    for spec in explicit:
        path, _, name = spec.partition("=")
        files[name or path] = path
else:
    for path in sorted(glob.glob("primitives/src/**/*.rs", recursive=True) + glob.glob("preview/src/**/*.rs", recursive=True)):
        if path == HELPER:
            continue
        files[path] = path

errors = []
scanned = 0
markers = 0
for name, path in files.items():
    if name == HELPER:
        continue
    with open(path, encoding="utf-8") as fh:
        lines = fh.read().split("\n")
    text = "\n".join(lines)
    if not explicit and name not in EXTRA and not OPEN_TOKENS.search(text):
        continue
    scanned += 1
    hits = []  # (line index, kind)
    marker_at = []
    for i, line in enumerate(lines):
        if MARKER.search(line):
            marker_at.append(i)
        if re.match(r"\s*//", line) or re.match(r'\s*code\s*\{\s*"', line):
            continue  # a comment, or documentation text such as `code { ".focus()" }`
        code = line
        for kind, rx in (("autofocus attribute", AUTOFOCUS), (".set_focus(true)", SET_FOCUS),
                         ("script .focus()", FOCUS_CALL), (":focus-visible read", FV_READ)):
            if rx.search(code):
                hits.append((i, kind))
    # In a `#[cfg(test)] mod tests` block the lines are fixtures, not paths.
    test_start = next((i for i, l in enumerate(lines) if re.match(r"\s*#\[cfg\(test\)\]", l)), len(lines))
    hits = [h for h in hits if h[0] < test_start]
    marker_at = [m for m in marker_at if m < test_start]
    for i, kind in hits:
        if not any(m in (i, i - 1, i - 2) for m in marker_at):
            errors.append(f"{name}:{i + 1}: {kind} on an open path without `focus-ok: <reason>`; where focus lands "
                          f"when an overlay opens is `dxOpenFocus` in primitives/src/modality.rs (see the header of "
                          f"scripts/check-open-focus.sh)")
    for m in marker_at:
        markers += 1
        if not any(h in (m, m + 1, m + 2) for h, _ in hits):
            errors.append(f"{name}:{m + 1}: stale `focus-ok` marker: no autofocus attribute, set_focus(true), "
                          f".focus() call or :focus-visible read on this line or the two below it")

if not explicit:
    def read(p):
        with open(p, encoding="utf-8") as fh:
            return fh.read()

    mod = read(HELPER)
    m = re.search(r'pub const MODALITY_JS: &str = "(.*)";\n', mod)
    if not m:
        errors.append(f"{HELPER}: `pub const MODALITY_JS` (one line) not found")
    else:
        writer = m.group(1)
        theme = read("preview/src/theme.rs")
        pre = re.search(r'pub const PREPAINT_JS: &str = "(.*)";\n', theme)
        if not pre or writer not in pre.group(1):
            errors.append("preview/src/theme.rs: PREPAINT_JS does not contain MODALITY_JS verbatim")
        if writer not in read("preview/index.html"):
            errors.append("preview/index.html: the pre-paint <script> does not contain MODALITY_JS verbatim")
    if "dxOpenFocus" not in mod:
        errors.append(f"{HELPER}: `dxOpenFocus` (the one open-focus policy) is gone")

    # The attribute has ONE writer.
    write_rx = re.compile(r"""setAttribute\(\s*['"]data-dx-modality|dataset\.dxModality\s*=|"data-dx-modality"\s*:""")
    roots = ["primitives/src", "preview/src", "preview/assets", "templates"]
    allowed = {HELPER, "preview/src/theme.rs", "preview/index.html"}
    for root in roots:
        for dirpath, _, names in os.walk(root):
            for n in names:
                if not n.endswith((".rs", ".js", ".ts", ".html", ".css")):
                    continue
                p = os.path.join(dirpath, n)
                if p in allowed or "node_modules" in p:
                    continue
                for i, line in enumerate(read(p).split("\n")):
                    if write_rx.search(line):
                        errors.append(f"{p}:{i + 1}: writes data-dx-modality; the modality has ONE writer "
                                      f"(MODALITY_JS in {HELPER})")

    css = read("preview/assets/dx-components-theme.css")
    if not re.search(
        r':root\[data-dx-modality\]:not\(\[data-dx-modality="keyboard"\]\)\s*\{[^{}]*--dx-ring:\s*0 0 #0000;[^{}]*'
        r'--dx-ring-focus:\s*0 0 #0000;[^{}]*--dx-ring-focus-inset:\s*0 0 #0000;', css):
        errors.append("preview/assets/dx-components-theme.css: the modality ring rule is gone (--dx-ring, "
                      "--dx-ring-focus and --dx-ring-focus-inset must be `0 0 #0000` outside the keyboard modality)")
    if re.search(r':root\[data-dx-modality\][^{}]*\{[^{}]*--dx-ring-field', css):
        errors.append("preview/assets/dx-components-theme.css: --dx-ring-field must not be modality-gated (a tapped "
                      "text field still rings)")
    for needed in (":where(dialog):focus", ":where([popover]):focus", ':where([role="listbox"]):focus',
                   ':where([role="menu"]):focus'):
        if needed not in css:
            errors.append(f"preview/assets/dx-components-theme.css: the programmatic-focus-container outline reset "
                          f"no longer lists `{needed}`")

if errors:
    for e in errors:
        print(e)
    print(f"\ncheck-open-focus: {len(errors)} problem(s). See the header of scripts/check-open-focus.sh.", file=sys.stderr)
    sys.exit(1)

print(f"check-open-focus: OK ({scanned} open-path source(s), {markers} focus-ok marker(s); one modality writer, "
      f"three identical copies, ring rule present)")
PYEOF
