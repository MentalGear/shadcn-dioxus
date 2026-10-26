#!/usr/bin/env bash
# Slot variables are derived from the sheet, never documented by hand (backlog row 433). A SLOT is a custom property a
# component's `style.css` reads under its own prefix (`--dx-<dir with dashes>-`) with a fallback --
# `var(--dx-label-gap, var(--dx-space-2))` -- and never declares itself: the composing component or the app
# sets it and the fallback is the default. That is the composition contract of row 170. The component's docs page prints a "Styling (slot
# variables)" table generated from the sheet (`preview/build.rs`, the same scanner `preview/slot_scan.rs` as this
# gate), so a slot can be neither read yet undocumented nor documented yet unread. The one hand-written part is the
# description, next to the variable in the sheet.
#
#   /* slot: --dx-label-gap  Space between the label text and what sits beside it. */
#   /* slot-internal: --dx-drawer-drag  Written by the drag script; not a knob, so not on the page. */
#
# Strict, with no baseline and no allowlist:
#   - every slot of a component sheet has a `slot:` or `slot-internal:` comment in that sheet, with a description;
#   - every such comment names a slot of its sheet (not a variable the sheet declares itself, reads with no
#     fallback, reads under another prefix, or does not read at all), and no slot has two;
#   - a component `docs.md` may not declare `## Slot variables` or `## Styling (slot variables)` (as it may not
#     declare `## Props reference`).
# Slots in the theme's own tokens are the theme's documentation, and variables a script writes under a computed
# name stay on `check-css-vars-defined.sh`'s allowlist.
#
# Every run first scans a fixture with one slot of each shape and requires exactly the expected report, so a
# scanner that stops seeing a shape is red instead of silently clean.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
# shellcheck source=scripts/lib/slot-scan.sh
source scripts/lib/slot-scan.sh

out="$(mktemp -d)"
trap 'rm -rf "$out"' EXIT
build_slot_scan "$out" || { echo "check-slot-docs: FAIL" >&2; exit 1; }

# The fixture's sheet is `fixture_x`, so its prefix is `--dx-fixture-x-`. Expected: `ok` (documented), `multi`
# (documented across lines, read in a fallback and across a comment), `bare` (no note), `hidden` (internal note),
# `empty` (a note with no text); `own` is declared by the sheet, `nofb` is read without a fallback and `--dx-other-x`
# is another component's, so each of the three notes naming them is an orphan; `ok` has a second note.
mkdir -p "$out/fixture_x"
cat >"$out/fixture_x/style.css" <<'CSS'
/* slot: --dx-fixture-x-ok  Documented. */
/* slot: --dx-fixture-x-multi
 * Documented on two lines. */
/* slot-internal: --dx-fixture-x-hidden  A script writes it. */
/* slot: --dx-fixture-x-empty */
/* slot: --dx-fixture-x-own  Declared by the sheet itself. */
/* slot: --dx-fixture-x-nofb  Read with no fallback. */
/* slot: --dx-other-x  Another component's variable. */
/* slot: --dx-fixture-x-ok  A second note. */
/* slot: --dx-fixture-x-gone  Not read at all. */
.dx-fixture-x {
  --dx-fixture-x-own: 1px;
  padding: var(--dx-fixture-x-own, 2px);
  color: var(--dx-fixture-x-ok, red);
  margin: var(--dx-fixture-x-nofb);
  gap: var(--dx-other-x, 3px);
  width: var(--a, var(--dx-fixture-x-multi, /* inline */ 4px));
  height: var(--dx-fixture-x-bare, 5px);
  top: var(--dx-fixture-x-hidden, 0);
  left: var(--dx-fixture-x-empty, 0);
  content: "/* slot: --dx-fixture-x-str  inside a string */";
}
CSS
"$out/slot-scan" "$out/fixture_x/style.css" 2>/dev/null | sed "s#$out/##" | LC_ALL=C sort >"$out/fixture.got"
cat >"$out/fixture.want" <<'WANT'
DUP	fixture_x/style.css	--dx-fixture-x-ok	9
EMPTY	fixture_x/style.css	--dx-fixture-x-empty	5
ORPHAN	fixture_x/style.css	--dx-fixture-x-gone	10	Not read at all.
ORPHAN	fixture_x/style.css	--dx-fixture-x-nofb	7	Read with no fallback.
ORPHAN	fixture_x/style.css	--dx-fixture-x-own	6	Declared by the sheet itself.
ORPHAN	fixture_x/style.css	--dx-other-x	8	Another component's variable.
SLOT	fixture_x/style.css	--dx-fixture-x-bare	18	none
SLOT	fixture_x/style.css	--dx-fixture-x-empty	20	doc
SLOT	fixture_x/style.css	--dx-fixture-x-hidden	19	internal
SLOT	fixture_x/style.css	--dx-fixture-x-multi	17	doc
SLOT	fixture_x/style.css	--dx-fixture-x-ok	14	doc
WANT
LC_ALL=C sort -o "$out/fixture.want" "$out/fixture.want"
cmp -s "$out/fixture.got" "$out/fixture.want" \
  || { echo "check-slot-docs: self-test failed; got (<) against wanted (>):" >&2; diff "$out/fixture.got" "$out/fixture.want" >&2 || true; exit 1; }

"$out/slot-scan" preview/src/components/*/style.css >"$out/slots.tsv" 2>"$out/slots.log"

python3 -I - "$out/slots.tsv" <<'PYEOF'
import glob
import re
import sys

errors = []
slots = 0
for raw in open(sys.argv[1], encoding="utf-8"):
    f = raw.rstrip("\n").split("\t")
    kind, path, name, line = f[0], f[1], f[2], f[3]
    if kind == "SLOT":
        slots += 1
        if f[4] == "none":
            errors.append(f"{path}:{line}: slot `{name}` has no `/* slot: {name}  <description> */` "
                          f"(or `/* slot-internal: {name}  <who writes it> */`) comment")
    elif kind == "EMPTY":
        errors.append(f"{path}:{line}: the note for `{name}` has no description")
    elif kind == "DUP":
        errors.append(f"{path}:{line}: a second note for `{name}`; one description per slot")
    elif kind == "ORPHAN":
        errors.append(f"{path}:{line}: the note names `{name or '(no --name)'}`, which is not a slot of this sheet "
                      f"(a slot is read here under the component's own prefix, with a fallback, and not declared "
                      f"by this sheet); fix the name or delete the note")

HEADING = re.compile(r"^##\s+(slot variables|styling \(slot variables\))\s*$", re.I | re.M)
for path in sorted(glob.glob("preview/src/components/*/docs.md")):
    text = open(path, encoding="utf-8").read()
    m = HEADING.search(text)
    if m:
        lineno = text.count("\n", 0, m.start()) + 1
        errors.append(f"{path}:{lineno}: a docs.md may not declare its own slot-variable section; "
                      f"`preview/build.rs` generates it from style.css (describe a slot with `/* slot: --name  text */` there)")

for e in errors:
    print(e)
if errors:
    print()
    print(f"check-slot-docs: FAIL -- {len(errors)} problem(s)")
    sys.exit(1)
print(f"check-slot-docs: OK ({slots} slots read by component sheets, each described once in its own sheet; "
      f"no docs.md declares a slot table)")
PYEOF
