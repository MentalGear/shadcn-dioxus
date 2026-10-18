---
id: 240
title: "Base UI focus model: what to lift beyond row 226"
status: "BACKLOG (research 2026-10-18)"
size: "S-M each"
area: a11y
updated: 2026-10-18
---

# Row 240: Base UI focus model: what to lift beyond row 226

## Item

Base UI focus model: what to lift beyond row 226

## Status / trigger

**BACKLOG (research 2026-10-18)**

## Size

S-M each

## Detail

Research `research/baseui-focus-2026-10-18.md`. Decided now (owner 2026-10-18): open focus = Base UI default (touch/pen -> panel; mouse and keyboard -> first focusable incl. fields, Drawer included -- a deliberate deviation from Base UI's always-popup Drawer, whose reasons (mobile keyboard, swipe) our touch rule and preventScroll already cover), lane mousefocus; text-field rings stay as shadcn (any focus rings a text field; option B, a quieter pointer state, rejected). Remaining: LIFT function-form `initial_focus`/`final_focus` props that receive the open modality (dialog, popover); LIFT a programmatic open returns focus to the previously focused element, not only the trigger (extend `__dxOpener` beyond dialogs); LEARN Select highlights the selected item on a pointer open too, Menu highlights the first item only on a keyboard open (check ours); LEARN `click.detail === 0` as the keyboard fallback in `MODALITY_JS`; LEARN the `focus({ focusVisible })` option (Chrome 144+, feature-detect) for script focus; SKIP `alignItemWithTrigger`, tabindex juggling, Safari blur-on-unmount, `openOnHover` parity.
