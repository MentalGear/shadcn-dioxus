---
id: 81
title: "Fixtures-72 follow-ups, base-URL migration completion, and three full-suite-only test-timing fixes"
status: "Landed 2026-09-18/19 (batch 2)"
size: "M"
area: tests
updated: 2026-09-18
---

# Row 81: Fixtures-72 follow-ups, base-URL migration completion, and three full-suite-only test-timing fixes

## Item

Fixtures-72 follow-ups, base-URL migration completion, and three full-suite-only test-timing fixes

## Status / trigger

Landed 2026-09-18/19 (batch 2)

## Size

M

## Detail

**Row 72's two cheapest bullets, closed** (`8f3b10e`): `NavigationMenu` gets the identical 7-location oracle footprint `Navbar` already has in `top-layer.spec.ts` (Rule 1, axe, Rule 5, Rule 11 a/b + `CONFORMING_CASES` marked `supported: false`, Rule 13, Rule 14) via the same themed-wrapper fixture instance, and a raw-primitive `Select` fixture case with `scroll_lock: false` was added to `scroll-lock.spec.ts`, exercising row 9's opt-out prop for the first time; building the two cases found and fixed two methodology bugs along the way (the scroll-lock case's own `assertScrollIsUnlocked` helper, and an `evaluate(el => el.click())` call that bypassed real hit-testing). **Base-URL migration, complete** (`e0bb23f` + `2cb1b06` + `3cc0881`): every spec in the suite now honours `PLAYWRIGHT_BASE_URL` (`playwright/base-url.ts`, row 76's own proof-of-concept). `e0bb23f` finished the 10 spec files the original migration (row 76) had deliberately excluded because their owning lanes hadn't landed yet, minus the 2 navigation_menu-owned specs, which had to wait for the navmenu lane; `2cb1b06` migrated 8 of those, and `3cc0881` finished the last 2 (`navigation_menu.spec.ts`, `oracle/tier1-apg/disclosure-navigation.spec.ts`) once navmenu landed. A repo-wide grep for the old hardcoded literal now turns up only `base-url.ts`'s own default/doc comment, one pre-existing prose mention in a header comment, and `ssg.local.config.ts`'s own unrelated doc comment — every executable occurrence is migrated. Recipe, one line (full account in `dev-loop.md`): set `PLAYWRIGHT_BASE_URL` to point Playwright at any already-running server instead of the default `:8080`. **Three test-timing fixes found only by the contended full suite, not by any lane's own lighter-loaded run.** `3416551`: the shared `escapesClip` helper (`top-layer.spec.ts`, 11 call sites — native, Tooltip, HoverCard, Popover, DropdownMenu, ContextMenu, Menubar, Select, Combobox, Navbar, NavigationMenu) probed `document.elementFromPoint` immediately after `toBeVisible()`, with no wait for the probed element's own entrance animation — latent since the helper was written, because every one of those 11 subjects' entrance animations were themselves dead/instant until now; `NavigationMenu`'s own fix (row 77, `08a14d8`) was the first to make one genuinely run long enough to lose the race. Fixed in the helper itself, protecting all 11 callers: await `content.getAnimations({subtree: true})` before measuring. `f40c2b3`: Sheet's own new close-button test (row 77) measured the button's opacity and position immediately after `data-state` flipped, before the 500ms slide-in settled — the same fix, applied to the one caller that needed it directly. `4f05f75`: the drawer short-drag test's single `{ steps: 2 }` mouse move produced two internal samples with no timing guarantee, and when they landed under 1ms apart, `drawer.rs`'s own `elapsed_ms.max(1.0)` division guard (not a tuning knob) turned a near-instant step into an artificially explosive velocity, above the dismiss threshold — the file's own long-drag test already avoided this with an explicit per-step pause; the short-drag test now does too. `f7987aa`: `oracle/tier1-apg/sortable-table.spec.ts`'s DataTable R1 case read `sortableHeaders.all()` (a one-shot, non-retrying snapshot) before the CSR app's wasm bundle had rendered the table at all, unprotected by the auto-retrying `toBeVisible()`-style call every sibling test in the file happens to make first — fixed with one `expect(...).toBeAttached()` before the existing snapshot. **The lesson, worth stating plainly for the next round:** a lane's own green run against a lighter-loaded, single-purpose server is necessary but not sufficient evidence for a new Playwright test with any gesture or animation timing in it — full-suite contention is where these three surfaced, and none were flakes in the sense of being unfixable; each had a real, missing wait. The `getAnimations({subtree: true})` "wait for the entrance animation before measuring" idiom now exists in three independent places (`toast.spec.ts`, pre-existing; `escapesClip`; Sheet's close-button test) — worth extracting to a shared helper once a fourth caller needs it; not done here, since it would mean touching `toast.spec.ts`, an unrelated file with no defect of its own, for a pure refactor
