---
id: 109
title: "SSG interaction tests race hydration — a click can land before wasm attaches Dioxus's delegated listeners; a…"
status: "FIXED 2026-09-25/26 (2235815 the hydration signal, bb51a02 carousel specs on gotoHydrated, c0df941…"
size: "S — one client-side effect, one shared…"
area: build
updated: 2026-10-20
---

# Row 109: SSG interaction tests race hydration — a click can land before wasm attaches Dioxus's delegated listeners; a…

## Item

SSG interaction tests race hydration — a click can land before wasm attaches Dioxus's delegated listeners; a hydration-ready signal fixes it (landed)

## Status / trigger

**FIXED 2026-09-25/26** (2235815 the hydration signal, bb51a02 carousel specs on `gotoHydrated`, c0df941 every other SSG interaction spec migrated). Closed in the 2026-10-20 backlog sync.

## Size

S — one client-side effect, one shared Playwright helper, two call sites wired up; the sweep to the rest of the suite (below) is the open-ended part -- **sweep ROLLED OUT 2026-09-26 (`c0df941`)**

## Detail

**2026-09-26 (audit batch): ROLLED OUT** in `c0df941` -- `gotoHydrated` in the 24 interacting SSG specs the row lists (vendored W3C reference pages and render-only tests stay on plain `goto`). **Evidence:** `carousel.spec.ts:633` — a Next click, sampled repeatedly, read `scrollLeft` 0 every time; the click hit prerendered markup before hydration had attached the delegated listeners, so the event was dropped outright, not queued for later. `waitUntil: "networkidle"` alone only got the flaky test to 4/5 — network-quiet is a different signal from hydrated, and no amount of retrying closes that gap. **Construction:** `preview/src/main.rs`'s `App` gained a signal-free one-shot `use_effect` running `document::eval("document.documentElement.dataset.hydrated = 'true'")`, client-only, after hydration commits — `<html>` is not vdom-owned, so this cannot affect SSR/hydration parity, confirmed by `hydration-parity.spec.ts` staying at 40/41 throughout. `playwright/hydration.ts` gained `gotoHydrated(page, url, opts)` (networkidle, then `waitForSelector('html[data-hydrated="true"]')`), wired into both carousel specs. `main.rs`'s own comment already pointed at "dev-docs/backlog.md: SSG interaction tests race hydration" before this row existed — this row is that pointer's target. **Fixed in `2235815`** (the signal itself) **and `bb51a02`** (both carousel specs switched onto `gotoHydrated`); confirmed by execution: the two previously-flaky tests 5/5 each at `--repeat-each=5`, the full `carousel.spec.ts` + tier-1 oracle 108/108 on both the SSG static serve and `dx serve`, `hydration-parity.spec.ts` unchanged at 40/41, all 12 `CLAUDE.md` gates green at `62183b3`. **Follow-up (DONE 2026-09-26, `c0df941`; was: not done — every other SSG-lane interaction spec still `goto`d directly and should migrate to `gotoHydrated` too):** `accordion-animation.spec.ts`, `date-picker.spec.ts`, `input_otp.spec.ts`, `oracle-focus-restore.spec.ts`, `oracle/tier1-apg/{keyboard-matrix,menu-roles,menu-submenu,select-only-combobox}.spec.ts`, `oracle/tier2-html/{form-label-association,form-participation,main-thread,native-dialog,top-layer,top-layer-ink,touch-double-tap,touch-focus-zoom}.spec.ts`, `oracle/tier3-radix/scroll-lock.spec.ts`, `pagination.spec.ts`, `pie_chart.spec.ts`, `radar_chart.spec.ts`, `radial_chart.spec.ts`, `select.spec.ts`, `tag_group.spec.ts`, plus `oracle/hydration-parity.spec.ts`'s own Rule 3 click. Row 90's `f468f78` `networkidle` fix for `pie_chart.spec.ts`/`radial_chart.spec.ts` is the same class under a weaker remedy and should move onto this helper too
