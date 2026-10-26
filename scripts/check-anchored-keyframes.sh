#!/usr/bin/env bash
#
# check-anchored-keyframes.sh
#
# No `transform` inside ANY `@keyframes` of a preview stylesheet, unless that keyframe is allowlisted below
# with a reason: animate `translate` / `scale` / `rotate` instead. Invisible to cargo and clippy, and with CI
# frozen (dev-docs/backlog.md row 33) nothing else would notice.
#
# THE CLASS (a keyframe that sets `transform` puts the date picker's fade 144px off-centre, the colour picker's 133px;
# dev-docs/research/round3-2026-10-05.md): an anchored overlay positions itself with `transform` -- `translateX(-50%)` for a centred
# anchor, `-100%` for an end-aligned one, `translateY(-50%)` beside it -- in the engine stylesheet
# `primitives/src/top_layer/styles.rs` injects, keyed on `[data-side]`/`[data-align]`, and that stylesheet covers
# EVERY `dx-anchor-*` overlay (tooltip, hover card, popover, dropdown menu, menubar, navbar, navigation menu,
# select, combobox). A `@keyframes` that sets `transform` REPLACES that value for as long as the animation
# runs (and, with `forwards`, afterwards), so the panel plays its whole fade at its un-centred spot and
# snaps to the centre on the last frame. The individual `translate`/`scale`/`rotate` properties are separate
# and compose with `transform`, so the centring survives. Every `4px drop` / `zoom-in-95` fade written the
# obvious way (`transform: translateY()`) is the bug. Context menus, sub-menus and the menus above set no
# `data-side` today, so they lose nothing today; the same shape applies the moment one does.
#
# WHY INVERTED (a ban with an allowlist, not "which stylesheets are anchored"): deriving the anchored stylesheets
# from three primitives (popover, tooltip, hover_card) would cover exactly the consumers already known about --
# the menus, select and combobox would sit outside it while sharing the engine rule that makes `transform`
# dangerous. Any derivation of "which content is anchored" is a second
# list that goes stale the day a primitive is added (the engine selector list in top_layer.rs, the primitives'
# `dx-anchor-*` markers and the preview wrappers are three places that would have to agree). The ban needs no
# such list: every stylesheet is in scope the moment it exists, a new overlay is covered by default, and the
# only thing a human maintains is the short ALLOW list of motion that is deliberately `transform` -- each entry
# carries its reason, and an entry that no longer matches a `transform` keyframe fails the run (so the list
# cannot rot into blanket permission).
#
# SCOPE: every `*.css` under `preview/src` and `preview/assets` (node_modules and build output skipped).
# CSS `@keyframes` live nowhere else (no `.rs`/`.js` carries one); the parse also fails if it sees more
# `@keyframes` tokens than it parsed, so an exotic spelling cannot slip past silently.
#
# Usage: scripts/check-anchored-keyframes.sh
# Exit 0: clean. Exit 1: a violation, with file:line detail on stderr.

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

python3 - <<'PYEOF'
import fnmatch
import re
import sys
from pathlib import Path

sys.path.insert(0, "scripts/lib")
from allow import Allowlist  # noqa: E402
from css import line_of, parse  # noqa: E402

# (stylesheet path, fnmatch pattern on the @keyframes name, reason). Every entry must match at least one
# keyframe that really sets `transform`, or the run fails as stale. Keep the reason about WHY this element can
# never sit on an anchor's centring transform -- "it is old" is not a reason.
ALLOW = Allowlist("check-anchored-keyframes", [
    ("preview/src/components/chart/style.css", "dx-chart-*",
     "SVG mark grow-in (scaleX/scaleY/scale from 0 with a transform-origin on <rect>/<path>), inside the plot; never an overlay"),
    ("preview/src/components/progress/style.css", "dx-indeterminate*",
     "indeterminate bar sweeps inside its own clipped track (translateX with an explicit RTL twin); not an overlay"),
    ("preview/src/components/spinner/style.css", "dx-spinner-spin",
     "rotates an inline <svg> glyph in place; not an overlay"),
], fields=("path", "pattern"))

errors = []

stylesheets = []
for root in ("preview/src", "preview/assets"):
    for p in sorted(Path(root).rglob("*.css")):
        if "node_modules" in p.parts or "target" in p.parts:
            continue
        stylesheets.append(p)

TRANSFORM_RE = re.compile(r"(?<![\w-])(?:-(?:webkit|moz|ms|o)-)?transform\s*:")

KF_NAME = re.compile(r"@keyframes\s+([\w-]+)$")
n_keyframes = 0
n_allowed = 0
for css in stylesheets:
    sheet = parse(css.read_text())
    src = sheet.text
    parsed = 0
    for node in sheet.walk():
        m = KF_NAME.match(node.prelude)
        if not m:
            continue
        parsed += 1
        body = node.body
        name = m.group(1)
        hits = list(TRANSFORM_RE.finditer(body))
        if not hits:
            continue
        if ALLOW.find(lambda e: e.path == css.as_posix() and fnmatch.fnmatchcase(name, e.pattern)):
            n_allowed += 1
            continue
        for t in hits:
            line = line_of(src, node.body_start + t.start())
            errors.append(
                f"{css}:{line}: @keyframes {name} sets `transform` -- an anchored overlay's own "
                f"`transform: translateX(-50%)` is replaced for the whole animation (the panel plays "
                f"off-centre); animate `translate`/`scale`/`rotate` instead, or allowlist it in "
                f"scripts/check-anchored-keyframes.sh with the reason it can never be anchored"
            )
    n_keyframes += parsed
    stray = len(re.findall(r"@keyframes\b", src)) - parsed
    if stray:
        errors.append(f"{css}: {stray} `@keyframes` token(s) this script could not parse -- fix the parse, do not skip them")

errors += ALLOW.problems()

if not stylesheets or not n_keyframes:
    errors.append("no stylesheet or no @keyframes was found -- the scan in this script is broken")

if errors:
    for e in errors:
        print(f"check-anchored-keyframes: {e}", file=sys.stderr)
    print(f"check-anchored-keyframes: FAILED -- {len(errors)} problem(s)", file=sys.stderr)
    sys.exit(1)
print(
    f"check-anchored-keyframes: OK -- {n_keyframes} @keyframes in {len(stylesheets)} stylesheets, none sets "
    f"`transform` except {n_allowed} allowlisted ({len(ALLOW)} entries)"
)
PYEOF
