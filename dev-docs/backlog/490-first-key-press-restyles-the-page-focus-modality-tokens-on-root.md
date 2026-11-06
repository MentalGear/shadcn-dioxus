---
id: 490
title: "The first key press restyles the whole page: focus-modality tokens are redefined on :root"
status: "Found 2026-11-01 while root-causing a release-suite timing red (main-thread.spec combobox open, 52…"
size: "S-M"
area: "theme / performance"
updated: 2026-11-06
---

# Row 490: The first key press restyles the whole page: focus-modality tokens are redefined on :root

## Item

The first key press on any page restyles about 933 rendered elements. `MODALITY_JS` (`primitives/src/modality.rs`,
the const at line 70) writes `html[data-dx-modality=keyboard]` on the first key press from a mouse-driven page. The
Focus modality section of `preview/assets/dx-components-theme.css` (row 226, around line 2475) then redefines
`--dx-ring`, `--dx-ring-focus` and `--dx-ring-focus-inset` on `:root` for every non-keyboard value. Because those
custom properties are inherited, the flip forces a style recalculation over the whole page. The cost is a one-time
hit on the first keyboard input of any page, and it is about half of the Combobox open task.

## Status / trigger

Found 2026-11-01 while root-causing a release-suite timing red (main-thread.spec combobox open, 52 ms vs 50 ms).
Fixed 2026-11-06 on `lane/modality-490` (see "Fix" below); awaiting merge.

## Size

S-M

## Detail

**Measured (release builds).**

- The `:root` redefinition makes the modality flip restyle about 933 rendered elements: 18-27 ms of forced style recalc,
  about half of the Combobox open task.
- In-page, six fresh loads each: with the rule deleted the first key press costs 0 ms of style recalc; with the
  override moved onto `:root[...] :focus-visible` it costs 10-18 ms.

**The release-suite red.** The combobox case in `playwright/main-thread.ts` sits on its 50 ms line in every tree
measured. Keypress task medians were 49.7 ms (7d50e068), 47.8 ms (2e171ed2) and 50.1 ms (c16027cc). It therefore fails a
deploy now and then (about 2-3 %), and a rerun passes. Fixing this row gives that case margin.

**Open question for the owner.** The tokens are read by consumers that may style an element which is not focused
(`:focus-within`, `:has()`). Scoping the override changes that behaviour for them. Two options are on the table:

- scope the modality tokens to the focused element (`:focus-visible`), or
- set the tokens per component.

Neither is chosen here. Whichever is chosen, the proof is the same.

**Proof (the spec written first).**

- The first key press costs under 5 ms of style recalc, measured in the main-thread spec's way on a release build.
- Every focus ring still shows correctly under mouse modality (no ring on a script-focused container or a tapped
  control, as row 226 specifies) and under keyboard modality (every ring shows).
- The focus-ring specs are the candidates to extend: `playwright/focus-ring.spec.ts`, `mouse-open-focus.spec.ts`,
  `touch-open-focus.spec.ts`, `open-focus-lift.spec.ts` and `ring-contrast.spec.ts`. The lane confirms which of them
  cover the modality rule.

**Evidence.** The probe specs and traces are container-local, in `/home/user/tgt/cbx-probes/` (`zz-cbx-*.spec.ts`).
The numbers above are the record.

## Fix (2026-11-06)

**Root cause, measured with counts.** Chromium `UpdateLayoutTree` trace events report the elements each style
recalculation touched (`playwright/modality-restyle.spec.ts`, Combobox page of about 5,170 elements, mouse click in the
input, then one key press, same build for both numbers):

| scenario | before | after |
| --- | --- | --- |
| first `a` typed in the Combobox input | 961 elements (one recalc of 942) | 26 |
| first ArrowDown from mouse modality | 944 elements (one recalc of 942) | 10 |

The earlier option "scope the tokens to `:root[...] :focus-visible`" does not help: measured 987 and 970 elements. A rule
whose ancestor is an attribute of `<html>` and whose subject is only `:focus-visible` has nothing the engine can index
the subject by, so the attribute flip invalidates the whole subtree. The cost is any rule or token keyed off an
attribute of `<html>` that every ring reads, not the particular token.

**Construction.** `MODALITY_JS` (still the one writer, three identical copies) keeps `data-dx-pointer-focus` on the
FOCUSED element while the modality is not `keyboard`: set on `focusin`, removed on `focusout`, moved to or taken off
`document.activeElement` when the modality changes. The theme sets `--dx-ring`, `--dx-ring-focus` and
`--dx-ring-focus-inset` to `0 0 #0000` on `[data-dx-pointer-focus]`; nothing is redefined on `:root`, so a modality flip
restyles one element. The writer was compacted (one `L()` listener helper with the single `listener-ok` marker, a regex
for the modifier keys), so the JS volume went down (inline script 7467 to 7453 bytes, js_lit unchanged) and the baseline
was lowered.

**Open question answered.** Of the owner's two options (scope to the focused element, or set per component) the first is
the base and the second is needed only where a container draws a ring for a focused descendant
(`:has(:focus-visible)`): Field, Attachment, Code Block, Virtual Grid has-control cells and the Calendar month/year
selects each reset the token on the container with `:has([data-dx-pointer-focus])`. `check-open-focus.sh` fails a
sheet that draws such a ring without that reset, and a `:root[data-dx-modality]` ring rule.

**Behaviour change to know about.** The Slider thumb reads `--dx-ring-focus` in `:hover` and `:active` rules too
(shadcn `hover:ring-3 active:ring-3`). Under mouse modality a hovered, unfocused thumb used to get the zeroed token (no
halo); it now shows the halo, as in shadcn. A pressed thumb is focused by the press, so it stays zeroed as before.

**Not done.** The "do not flip on typing in an editable control" half of the earlier proposal is subsumed: with the flip
costing one element, skipping it saves nothing, and a Tab out of a field must still flip.
