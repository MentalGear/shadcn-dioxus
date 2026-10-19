---
id: 42
title: "iOS 18 Safari device review, item 1 — double-tap-to-zoom on rapid button taps — landed 2026-09-03. Every…"
status: "CLOSED 2026-10-19 (bookkeeping: status already said done). — (landed)"
size: "S–M"
area: tests
updated: 2026-10-19
---

# Row 42: iOS 18 Safari device review, item 1 — double-tap-to-zoom on rapid button taps — landed 2026-09-03. Every…

## Item

**iOS 18 Safari device review, item 1 — double-tap-to-zoom on rapid button taps — landed 2026-09-03.** Every interactive element (`button`, `[role="button"]`, `a[href]`, `input`, `select`, `textarea`, `summary`, `[tabindex]:not([tabindex="-1"])`, `[role="menuitem"/"option"/"tab"/"switch"/"checkbox"/"radio"/"slider"]`) now declares `touch-action: manipulation`, two layers: an `!important` app-wide rule (`preview/assets/main.css`, same construction as the existing font-size floor) plus the same property on every themed component's own interactive root class (`preview/src/components/*/style.css`, ~30 files, so a `dx components add`-copied component still carries it without `main.css`). `touch-action: none` was never introduced by this fix; `slider`/`color_picker`'s pre-existing `touch-action: none` on their *track* containers (not the thumbs, which now carry `manipulation`) was left untouched as out of this round's scope. Oracle: `playwright/oracle/tier2-html/touch-double-tap.spec.ts`, red before (every route with an interactive element failed — `computed touch-action: auto` everywhere, since nothing in the app declared the property at all) → green after

## Status / trigger

CLOSED 2026-10-19 (bookkeeping: status already said done). — (landed)

## Size

S–M

## Detail

Session report/spec header carries the literal red-table counts per route. Rejected alternative: `touch-action: none` (breaks one-finger scrolling — explicitly ruled out by the user's own report source, WebKit's own `touch-action: manipulation` guidance)
