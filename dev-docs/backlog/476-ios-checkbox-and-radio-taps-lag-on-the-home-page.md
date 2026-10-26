---
id: 476
title: "iOS: checkbox and radio taps lag about a second on the home page (Filter results demo, Radio Group card)"
status: "BACKLOG. Owner report 2026-10-26 (iPhone); the next session reproduces with touch emulation and CPU…"
size: "M"
area: components
updated: 2026-10-26
---

# Row 476: iOS: checkbox and radio taps lag about a second on the home page (Filter results demo, Radio Group card)

## Item

Owner report 2026-10-26, on an iPhone, on the home page (the component gallery):

- In the "Filter results" demo, checking or unchecking a checkbox takes about one second to show the new state. The radio buttons below it in the same demo update immediately.
- The separate Radio Group card on the home page also lags on its radio buttons.

So the same control kind is fast in one place and slow in another, which points at what each tap re-renders or waits on (a page-wide re-render, a hydration or layout cost, or a gesture delay on one element), not at the control itself. Inferred, not reproduced.

## Status / trigger

BACKLOG. Owner report 2026-10-26 (iPhone); the next session reproduces with touch emulation and CPU throttling first.

## Size

M

## Detail

- **Reproduce first:** on a release build, Chromium with touch emulation (`hasTouch`, `isMobile`) and 4x-6x CPU throttling, tap each checkbox and radio on the home page and record the time from `pointerup` to the painted state change; compare the fast radios with the slow checkbox and the slow Radio Group card. Then a performance trace of one slow tap.
- **Files owned:** to be found by the reproduction.
- **Proof:** a timing spec (release builds only) that bounds tap-to-paint for the home page's checkbox and radio demos, red before the fix.
