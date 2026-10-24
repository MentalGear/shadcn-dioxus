---
id: 45
title: "Main-thread responsiveness oracle (INP / long tasks). Install a PerformanceObserver for longtask (and event…"
status: "LANDED 2026-09-19 (batch 3; first note below) -- original trigger: A device report of a late tap or…"
size: "M (landed)"
area: tests
updated: 2026-09-19
---

# Row 45: Main-thread responsiveness oracle (INP / long tasks). Install a PerformanceObserver for longtask (and event…

## Item

**Main-thread responsiveness oracle (INP / long tasks).** Install a `PerformanceObserver` for `longtask` (and `event` timing with
`durationThreshold`) in the page before each overlay's open/close and each form control's primary interaction, then assert no task
over 50 ms and no interaction over 200 ms during the gesture (the thresholds of `dev-docs/reference/platform-rationale.md` section 11,
kciter, *The Browser's Main Thread Is Expensive*). Rules 1-5 of that section were review-enforced only until this existed.

## Status / trigger

**LANDED 2026-09-19 (batch 3; first note below)** -- original trigger: A device report of a late tap or stalled animation that the existing frame samplers (`accordion-animation.spec.ts`) and position oracles do not explain; or the first component that does real work per interaction (Command palette filtering, [`component-backlog.md`](../../archive/rounds/component-backlog.md))

## Size

M (landed)

## Detail

Chromium-only in this sandbox, so the numbers are a regression guard, not an iOS measurement (the caveat of every tier-2 oracle
here). **Landed 2026-09-19 (batch 3):** `oracle/tier2-html/main-thread.spec.ts` (58 tests: 19 overlays open/close plus 20 form-control
gestures, with real pointer drags for Slider, Resizable and Drawer) and its helper `playwright/main-thread.ts`
(`installMainThreadObserver` through `page.addInitScript`, `withGesture`, `assertResponsive`: best of 3, never lowering the
threshold to compensate for noise, every measurement recorded in `test.info().annotations`). It cites the W3C Long Tasks API (over
50 ms) and Event Timing API (`durationThreshold`, 16 ms floor). Red proof without a build: a synthetic 120 ms busy `pointerdown`
handler turned the drawer and slider drag tests red (137/147 ms); a 250 ms busy `click` turned dialog-open red (312 ms).

**Three subjects were red by design, load-independent:** `combobox: open` 59-75 ms, `command: open` 59-83 ms, `date_picker: open`
60-88 ms (over the 50 ms task bound, under the 200 ms interaction bound). A debug lane root-caused one mechanism behind two trigger
sites: `top_layer.rs`'s `use_anchor_position_fallback` `reposition()` (combobox, select, date picker) and native `showModal()` via
`use_dialog_open_driver` (command) each do a synchronous DOM mount plus a forced-layout read inside the same event-handler task
(section 11 rules 1 and 5; DOM nodes added on open: dropdown_menu +10, combobox +10, select +14, command +39, date_picker +247). A
one-frame deferral of combobox's `initial_focus` write was measured (66-139 ms to 57-63 ms, insufficient) and reverted. Three
proposals for a dedicated batch, none applied: (1) defer the mount itself by a frame (most complete, HIGH risk: `top_layer.rs`
records a rAF-wrapped `reposition()` that "silently never ran at all on a real Firefox build"); (2) defer `showModal()` promotion
by a frame in `use_dialog_open_driver`; (3) give Combobox the menu family's deferred-collection-focus shape. A symbolicated wasm
profile should name the Rust function still costing ~55-65 ms on a 10-node mount first.

**Thresholds on a loaded box (round 3 verification, 2026-10-05).** In a 4-worker release run the sheet, drawer and command cases
measured 51-53 ms and passed serially: a red at 51-53 ms under load is a flake, not a regression. Do not widen the 50 ms bound;
judge a main-thread red by its serial rerun, and run timing specs on release builds only (debug gives false reds, row 116).

**Recurring locator class.** Building the oracle found `virtual_list.spec.ts`'s unscoped `page.getByRole("list").first()` resolving
the docs sidebar's `<ul>` once the sidebar was added (fixed `1462e9d`), and the class recurred when RTL fixtures made every
RTL-aware component render two variants on one page (`menubar`, `navbar`, `keyboard-matrix` unscoped `getByRole('menubar')`, fixed
`5953405`): a four-occurrence class (rows 74, 75 and two here). Standing review line: scope role and name queries on any component
page through `#component-preview-frame`. The candidate construction (a lint restricted to `playwright/*.spec.ts` flagging a bare
`page.getByRole(...)` with no container scope, the shape `check-self-subscribing-effects.sh` took) was not built here.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/045-main-thread-responsiveness-oracle-inp-long.md`.*
