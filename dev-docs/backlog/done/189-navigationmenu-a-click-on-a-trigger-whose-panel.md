---
id: 189
title: "NavigationMenu: a click on a trigger whose panel hover just opened CLOSED it"
status: "LANDED 2026-10-11, following Base UI, not Radix (owner may reverse) -- found by the row 121 lane…"
size: "S (done)"
area: theme
updated: 2026-10-11
---

# Row 189: NavigationMenu: a click on a trigger whose panel hover just opened CLOSED it

## Item

NavigationMenu: a click on a trigger whose panel hover just opened CLOSED it

## Status / trigger

**LANDED 2026-10-11, following Base UI, not Radix (owner may reverse)** -- found by the row 121 lane (the flake in `openPanel`). **Correction to the premise:** Radix does NOT guard this: its trigger `onClick` toggles (`wasClickCloseRef` only stops a re-open under the resting pointer, `hasPointerMoveOpenedRef` only makes the hover-open fire once per entry), so hover-then-click closes in Radix too. Base UI's `stickIfOpen` keeps it open

## Size

S (done)

## Detail

`primitives/src/navigation_menu.rs`: a `hover_opened` marker set only by the hover-intent timer and cleared inside `set_open`, the single writer of `open` (so keyboard, Escape, link clicks, blur and the close timer all clear it and it cannot go stale); the trigger's click on a hover-opened panel clears the marker and keeps it open, the next click toggles shut. Keyboard and `open_on_hover: false` unchanged; a click inside the 150ms delay still toggles. Two specs in `navigation_menu.spec.ts` (hover, click, still open, click, closed and stays closed; no stale marker after a keyboard open). `navbar.rs` is a separate design (immediate hover-open, a `pointerdown` snapshot) and still closes on hover-then-click: unchanged, a separate decision. To revert to strict Radix: drop the marker and the two specs.
