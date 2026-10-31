---
id: 476
title: "iOS: checkbox and radio taps lag about a second on the home page (Filter results demo, Radio Group card)"
status: "IN PROGRESS (lane tap-lag-476, 2026-10-31). The lag does not reproduce in Chromium, so the fix…"
size: "M"
area: components
updated: 2026-10-31
---

# Row 476: iOS: checkbox and radio taps lag about a second on the home page (Filter results demo, Radio Group card)

## Item

Owner report 2026-10-26, on an iPhone, on the home page (the component gallery):

- In the "Filter results" demo, checking or unchecking a checkbox takes about one second to show the new state. The radio buttons below it in the same demo update immediately.
- The separate Radio Group card on the home page also lags on its radio buttons.

So the same control kind is fast in one place and slow in another, which points at what each tap re-renders or waits on (a page-wide re-render, a hydration or layout cost, or a gesture delay on one element), not at the control itself. Inferred, not reproduced.

## Status / trigger

IN PROGRESS (lane `tap-lag-476`, 2026-10-31). The lag does not reproduce in Chromium, so the fix removes what a tap on a
label or a checkbox adds to the control's own click and the owner confirms on the iPhone.

Measured on a debug SSG build, iPhone 13 emulation (`hasTouch`, `isMobile`), 4x CPU throttling, `touchend` to the frame
after `aria-checked` changes, 15 taps each: Filter checkbox by its box median 128 ms, by its label 160 ms, Filter radio
187 ms, Radio Group card radio 217 ms, by its label 191 ms; 6x: 180 / 222 / 335 / 309 / 295 ms. No tap is near a second;
the checkbox is not slower than the radios. A release build is faster still, so the reported second is a wait, not CPU
work. The lagging controls are the ones with a `Label` (Filter checkbox, Radio Group card radios); the fast Filter
radios are a `<button>` that holds its own text.

What a tap on a label or a checkbox added to the click, each measured:
- `label` was `touch-action: auto` (computed on every home-page label) while the controls are `manipulation`; the
  tap target on a phone is the label text, and Mobile Safari reads `auto` as the first tap of a double-tap zoom.
- A touch synthesizes `mouseover`/`mouseenter`/`mousemove` before the click; `Label` armed its double-click guard from
  `mouseenter`/`mousedown`, so the first tap on each label wrote a signal, re-rendered and started a script ahead of
  the click.
- Every checkbox change (and mount) ran a `document::eval` (`new Function`) to set `.checked`, which the `checked`
  attribute already sets.

Fix: `touch-action: manipulation` on `label` (theme rule and `main.css` catch-all, `touch-double-tap` oracle list);
`Label` arms on a mouse or pen `pointerenter`/`pointerdown` and never on a touch; `Checkbox` runs its script only for an
indeterminate box. `playwright/tap-latency.spec.ts` pins each (red before: home-page labels at `auto`, a
`mousedown` listener live after one label tap, 3 `new Function` for 3 toggles) and bounds the median tap-to-paint
under 4x throttling at 600 ms (green before and after: Chromium does not show the wait).

Owner check on the iPhone (the wait is not reproducible here): tap the text of a Filter results checkbox and of a Radio
Group card radio, first tap after load and then repeated; if one still waits, record whether the first tap or every tap,
and whether the box itself is also slow. Open: the Filter radios and the card radios still dispatch a Rust click handler
each, and WebKit's own tap handling (content-change observation of the synthesized hover) is not testable here (row 4).

## Size

M

## Detail

- **Reproduce first:** on a release build, Chromium with touch emulation (`hasTouch`, `isMobile`) and 4x-6x CPU throttling, tap each checkbox and radio on the home page and record the time from `pointerup` to the painted state change; compare the fast radios with the slow checkbox and the slow Radio Group card. Then a performance trace of one slow tap.
- **Files owned:** to be found by the reproduction.
- **Proof:** a timing spec (release builds only) that bounds tap-to-paint for the home page's checkbox and radio demos, red before the fix.
