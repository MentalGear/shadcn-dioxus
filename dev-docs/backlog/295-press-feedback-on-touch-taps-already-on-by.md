---
id: 295
title: "Press feedback on touch taps: already on by default, so check why taps don't show it"
status: "OPEN. Row 205 (LANDED 2026-10-14) already ships press feedback on by default, for every pointer.…"
size: "S (or none)"
area: theme
updated: 2026-10-19
---

# Row 295: Press feedback on touch taps: already on by default, so check why taps don't show it

## Item

"Press feedback default on, on touch taps."

## Status / trigger

OPEN. Row 205 (LANDED 2026-10-14) **already ships press feedback on by default**, for every pointer. Turning it off takes `html[data-dx-press-feedback="off"]`, set by the theme picker switch (`theme.rs:~625`).

## Size

S (or none)

## Detail

- **Today:** the rule is `dx-components-theme.css:966-1030`: `:active { scale: 0.98 }` plus an 8% wash. It has zero specificity, so a component with its own `:active`, `background` shorthand or `transition` silently overrides it (the comment there says so).
- `index.html:63` already registers a passive `touchstart` listener, which iOS needs before `:active` fires on a tap.
- **Proposed row:** check the components where a tap shows no feedback on a phone, and fix those overrides.
- Touch-only alternative (not the default): a one-line gate on `data-dx-modality="touch"`, and it reverses row 205's design.
- Owner default (2026-10-19): keep it on for every pointer and fix the components that miss it.
- If a particular control showed nothing when you tapped it, tell me which one and I'll name it in the row.
