---
id: 205
title: "No sticky hover on touch; press feedback as a switchable system utility"
status: "LANDED 2026-10-14 (theme lane badfa43, fda3b18; hoverwrap lane 31b0171; also closes row 163)"
size: "M"
area: theme
updated: 2026-10-14
---

# Row 205: No sticky hover on touch; press feedback as a switchable system utility

## Item

No sticky hover on touch; press feedback as a switchable system utility

## Status / trigger

**LANDED 2026-10-14 (theme lane badfa43, fda3b18; hoverwrap lane 31b0171; also closes row 163)**

## Size

M

## Detail

Owner request. Every `:hover` under `@media (hover: hover) and (pointer: fine)` (new gate `check-hover-media.sh`); theme press-feedback tokens + one zero-specificity rule, `data-dx-press-feedback=on\|off`, toggle in the theme/palette popover, reduced-motion aware; hover-intent opens only for `pointerType=mouse`. Item 2. Plan: `dev-docs/research/next-round-plan-2026-10-14.md`. **Landed:** every positive `:hover` is inside `@media (hover: hover) and (pointer: fine)`: 95 rules wrapped and 31 mixed rules split in the sweep, then 11 wrapped and 5 split in the seven lane-deferred sheets (sheet, drawer, calendar, combobox, popover, select, dropdown_menu). Gate `scripts/check-hover-media.sh` (exact query; hover-only rules inside it; no `:is`/`:where`/`:has` mixing hover with other states; negated `:not(:hover)` allowlisted with reasons: `main.css` scrollbar rest, toast collapsed stack; Rust strings scanned, combobox `option.rs:34` JS `matches(':hover')` allowlisted; nested CSS fails; stale entries fail), registered in `run-gates.sh` and CLAUDE.md. Touch behaviour changes: sidebar show-on-hover actions and toast expand-on-hover open on focus only; `main.css` scrollbar-color is transparent on touch. **Press feedback:** theme tokens `--dx-press-scale` 0.98, `--dx-press-duration` (the fast motion token, 100 ms), `--dx-press-fill` 8% foreground; ONE zero-specificity `:where` rule on the touch-action interactive list minus text fields/sliders/spinbuttons/bare tabindex (pressing a field must not shrink it; `:active` matches ancestors), disabled excluded, gated by `html:not([data-dx-press-feedback="off"])`, transition only under `prefers-reduced-motion: no-preference`; it uses `scale` plus a background-image wash, so it stacks with components' own `:active` transform (button, alert_dialog actions, pagination translateY(1px); tag_group and dnd drop their `scale(0.92)`). Theme picker (`preview/src/theme.rs`): a "Press feedback" switch in the popover and the phone drawer, `Pref::PressFeedback`, cookie `dx_dx_press-feedback`, `PREPAINT_JS` and `preview/index.html` updated so a reload paints already switched; Reset clears it. `hover_intent.rs` opens only for `pointerType` mouse (capture pointerover/pointerdown via `js_listener`). Specs: `press-feedback.spec` 8/8, `hover-intent` 12/12. **Open:** the press-feedback list and the touch-action list are two hand-kept lists; a drift check belongs in `check-theme-baseline.sh`.
