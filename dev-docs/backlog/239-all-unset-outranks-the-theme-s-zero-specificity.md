---
id: 239
title: "all: unset outranks the theme's zero-specificity baselines"
status: "LANDED 2026-10-19 (lane theme: every `all:` removed, gate bans it)"
size: "S-M"
area: theme
updated: 2026-10-18
---

# Row 239: all: unset outranks the theme's zero-specificity baselines

## Item

`all: unset` outranks the theme's zero-specificity baselines

## Status / trigger

**LANDED 2026-10-19 (lane theme: every `all:` removed, gate bans it)**

## Size

S-M

## Detail

`all: unset` (specificity 0,1,0) beats every `:where()` theme rule, so Switch, the Slider thumb and both ColorPicker thumbs computed `user-select: auto` (row 229 restates it in those four rules) and Switch likely also loses the theme's `touch-action: manipulation` and press feedback (not verified). `check-theme-baseline.sh`'s new `ALL_RESET_ALLOW` lists every `all:` site. Construction to decide: replace `all: unset` on controls with explicit resets, or have the gate require each `all:` rule to restate every theme baseline declaration; spec: computed touch-action, user-select and press feedback on each listed control.

## Update 2026-10-19

Construction chosen: no component rule may use `all:` at all. The four sites (Switch, Slider thumb, both ColorPicker
thumbs) now spell out the native-button reset (appearance, margin, padding, background-color, color, font, letter/word
spacing, text-align/indent/shadow/transform, outline; border only on the Switch, the thumbs declare their own), and
`scripts/check-theme-baseline.sh` fails on any `all:` in a component stylesheet (`ALL_RESET_ALLOW` deleted). The row-229
`user-select` restatements that only undid `all: unset` are deleted; the theme's rule reaches the four controls.
Care taken: the reset uses `background-color: transparent`, NOT `background: none` -- the shorthand also resets
`background-image` and would itself have beaten the theme's zero-specificity press wash (caught by the new spec).

Evidence: full computed-style dump (every property, light+dark) of the four controls captured on the pre-change build and
diffed after: the only deltas are `box-sizing` content-box -> border-box on Switch (theme baseline now applies; no border
or padding, so zero size change) and `background-position` `0% 0%` -> `0px 0px` (a serialisation of the same value).
The ColorPicker area thumb is a `div`, so it lost its restated `user-select: none` with no baseline to take over: it is
now an explicit `USER_SELECT_ALLOW` entry (a drag surface). The two ColorPicker thumbs keep `box-sizing: content-box`
explicitly (their 16px + 2px border = the 20px box they always had). Spec: `playwright/theme-all-reset.spec.ts`
(computed touch-action, user-select, press feedback on Switch, and a pinned before-snapshot equality per control).
Press feedback is by design absent on slider-role thumbs (theme excludes drag surfaces). Remaining: none.
