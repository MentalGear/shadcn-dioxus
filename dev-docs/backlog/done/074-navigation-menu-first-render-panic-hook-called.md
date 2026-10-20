---
id: 74
title: "Navigation Menu first-render panic (hook called inside another hook's closure) + demo/site-chrome…"
status: "CLOSED 2026-10-19 (bookkeeping: status already said done). Found in integration round 2…"
size: "S"
area: menus
updated: 2026-10-19
---

# Row 74: Navigation Menu first-render panic (hook called inside another hook's closure) + demo/site-chrome…

## Item

Navigation Menu first-render panic (hook called inside another hook's closure) + demo/site-chrome accessible-name collisions

## Status / trigger

CLOSED 2026-10-19 (bookkeeping: status already said done). Found in integration round 2 (2026-09-18), fixed round 3 same day

## Size

S

## Detail

Round 2's live integration run found 7/7 `navigation_menu.spec.ts` tests failing (2 fast, 5 hanging the 5-minute ceiling) on `getByRole('button', { name: 'Getting started' })` never resolving, plus `disclosure-navigation.spec.ts`'s "Library" block failing the same way (its Reference-page block, R1-R5, passed). Root cause #1: `NavigationMenu`'s root component called `use_delayed_action()` (a hook wrapping `use_signal`, `menu_sub.rs`) three times *inside* the closure passed to `use_context_provider` — calling a hook from within another hook's initializer closure re-enters the scope's hook-list `RefCell` while it is already borrowed, panicking at runtime ("BorrowMutError: The hook list is already borrowed ... hook inside a hook", dioxus-core's `scope_context.rs`) on first render, a wasm `unreachable` trap before anything — not even `main`/`h1` — mounts. **This is a runtime-only invariant**: not `cargo check`, not clippy, not the doc tests catch it, which is why the original lane's own thorough verification never surfaced it. Because `ComponentGallery` (the home page, `/`) renders every demo to build its gallery, this one bad demo's panic took down the entire home page and every home-page test with it (native-dialog Rule 8's "Show Popover" lookup, `preview.spec.ts`, hydration-parity Rule 4b) — **the class: one bad demo takes down `/`**, worth a standing awareness for anything `ComponentGallery` renders. Fixed (commit `c3dda45`, cherry-picked from `963d630`): hoisted all three `use_delayed_action()` calls out of the closure into the component body; audited every other `use_context_provider`/`use_hook`/`use_memo`/`use_effect`/`use_callback` closure in the file, no other instance (every other closure only calls plain non-hook constructors, the same safe pattern `AccordionContext::new` already uses). Root cause #2, found by the same live reproduction: the demo's featured-card link text ("dioxus-components") and its "Docs" link both collide with accessible names the site's own persistent chrome already uses on every route (header navbar brand, footer), so unscoped `page.getByRole('link', { name: ... })` queries resolved 2-3 elements page-wide instead of 1. Fixed by construction, not per-locator: featured-card title renamed "Component Library"; both spec files scope every query to the demo's own `aria-label="Component navigation menu"` landmark via a `nav(page)`/`libraryNav(page)` helper — demo content must not reuse accessible names the persistent chrome already renders, or specs must scope to the demo landmark. Verified live (own server, direct Playwright-API script, coordinator's server being read-only): 22/22 — R1-R5 plus every smoke-spec assertion. **Closed 2026-09-18/19, batch 2, row 79:** root cause #1's exact class (a hook called inside another hook's closure) is now guarded by `scripts/check-hooks-in-closures.sh` (`f11b98b`), proven against this row's own before/after commits (fails on `bb65cd6`, passes on `c3dda45` and current `HEAD`). **Family note (2026-09-19, batch 3):** row 83's `DateRangePicker` hang is a related-but-distinct class from this row's own hook-in-closure panic — grouped together only because both were runtime-only defects a live Playwright run caught that no static check (`cargo check`, clippy, doctests) could
