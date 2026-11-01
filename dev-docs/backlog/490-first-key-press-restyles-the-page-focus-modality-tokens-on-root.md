---
id: 490
title: "The first key press restyles the whole page: focus-modality tokens are redefined on :root"
status: "BACKLOG. Found 2026-11-01 while root-causing a release-suite timing red (main-thread.spec combobox…"
size: "S-M"
area: "theme / performance"
updated: 2026-11-01
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

BACKLOG. Found 2026-11-01 while root-causing a release-suite timing red (main-thread.spec combobox open, 52 ms vs 50 ms).

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
