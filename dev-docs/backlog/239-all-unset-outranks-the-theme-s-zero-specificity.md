---
id: 239
title: "all: unset outranks the theme's zero-specificity baselines"
status: "BACKLOG (found 2026-10-18 by lane baseline)"
size: "S-M"
area: theme
updated: 2026-10-18
---

# Row 239: all: unset outranks the theme's zero-specificity baselines

## Item

`all: unset` outranks the theme's zero-specificity baselines

## Status / trigger

**BACKLOG (found 2026-10-18 by lane baseline)**

## Size

S-M

## Detail

`all: unset` (specificity 0,1,0) beats every `:where()` theme rule, so Switch, the Slider thumb and both ColorPicker thumbs computed `user-select: auto` (row 229 restates it in those four rules) and Switch likely also loses the theme's `touch-action: manipulation` and press feedback (not verified). `check-theme-baseline.sh`'s new `ALL_RESET_ALLOW` lists every `all:` site. Construction to decide: replace `all: unset` on controls with explicit resets, or have the gate require each `all:` rule to restate every theme baseline declaration; spec: computed touch-action, user-select and press feedback on each listed control.
