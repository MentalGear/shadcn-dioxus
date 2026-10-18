---
id: 163
title: "Wrap hover styles in @media (hover: hover), by codemod plus a gate: 80 :hover blocks in 39 stylesheets leave…"
status: "LANDED 2026-10-14 (as row 205; theme lane badfa43, fda3b18, hoverwrap lane 31b0171) -- recorded…"
size: "M (one mechanical pass over 39 sheets…"
area: theme
updated: 2026-10-14
---

# Row 163: Wrap hover styles in @media (hover: hover), by codemod plus a gate: 80 :hover blocks in 39 stylesheets leave…

## Item

Wrap hover styles in `@media (hover: hover)`, by codemod plus a gate: 80 `:hover` blocks in 39 stylesheets leave a sticky hover on touch

## Status / trigger

**LANDED 2026-10-14 (as row 205; theme lane badfa43, fda3b18, hoverwrap lane 31b0171)** -- recorded 2026-10-05 (audit)

## Size

M (one mechanical pass over 39 sheets, plus the gate and a touch spec)

## Detail

**The defect.** On a touch screen a tap fires a compatibility `:hover` that stays on the tapped element until something else is tapped, so a pressed Button, Tab, menu item or Card keeps its hover look after the finger is gone (a sticky hover). The standard fix applies hover styles only where the primary input can hover: `@media (hover: hover) { .x:hover { ... } }`. The audit counted 80 `:hover` blocks in 39 sheets without it; re-count at step 0 (a plain `grep -c ':hover'` over `preview/` also hits comments, `:not(:hover)` and `main.css`, so use the codemod's own list). **Construction (a class, so no per-sheet patching):** (1) a codemod that wraps each `:hover` rule in `@media (hover: hover)` and keeps the rule where it is in source order (its place in the cascade is what lets it beat the base rule and lose to `:active` and `[data-state]`), run once; (2) a gate `scripts/check-hover-media.sh`, added to `scripts/run-gates.sh` and to CLAUDE.md's gate sentence, failing on any `:hover` in `preview/src/**/*.css` and `preview/assets/*.css` that is not inside an `@media (hover: hover)` block and carries no reasoned allowlist marker (the shape of `check-css-logical-properties.sh`); (3) a touch spec: tap a Button on an emulated phone and assert its computed `background-color` is the rest colour. **Proof it changes nothing on desktop:** row 135's hover pass (the computed-style snapshot with hover applied, A/A byte-identical) before and after; without 135 there is no cheap proof, so 163 follows 135. **Watch:** Chromium's mobile emulation may report `(hover: none)`, so a spec that asserts a hover style on a touch project changes meaning; find those first (`grep -rn "hover()" playwright/`). **Sequencing:** it edits the 39 sheets that phase C and the parity wave repaint, so it runs after them, as one lane that owns every stylesheet while it runs and is never dispatched beside a lane that edits CSS. **Landed 2026-10-14 under row 205:** the codemod and the gate shipped together. 95 rules wrapped and 31 mixed rules split in the sweep, then 11 wrapped and 5 split in the seven lane-deferred sheets (sheet, drawer, calendar, combobox, popover, select, dropdown_menu); the query is the stricter `(hover: hover) and (pointer: fine)`. Gate `scripts/check-hover-media.sh` is registered in `run-gates.sh` and CLAUDE.md. Details in row 205.
