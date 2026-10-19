---
id: 115
title: "tooltip.spec.ts:6 \"tabbing out of the trigger element should hide the tooltip\" failed once in a full-suite…"
status: "Seen 2026-09-28 in the mode-D integration run (--workers=4, SSG release build): the tooltip was…"
size: "S (diagnosis, when it recurs)"
area: tests
updated: 2026-09-28
---

# Row 115: tooltip.spec.ts:6 "tabbing out of the trigger element should hide the tooltip" failed once in a full-suite…

## Item

`tooltip.spec.ts:6` "tabbing out of the trigger element should hide the tooltip" failed once in a full-suite run -- not reproduced

## Status / trigger

Seen 2026-09-28 in the mode-D integration run (`--workers=4`, SSG release build): the tooltip was still present 5s after `Tab` (`toHaveCount(0)` received 1). Re-run immediately: `tooltip.spec.ts` `--repeat-each=6 --workers=4`, 180/180 passed

## Size

S (diagnosis, when it recurs)

## Detail

**2026-10-01: DIAGNOSED** (`e698054`, `58a8e36`) -- not one bug: one product race plus three test-methodology classes. (1) tooltip:6 / hover-card:6 were a real race: `use_popover_shown_while_mounted` mirrored `toggle(open)` -- the async echo of its own `showPopover()` -- back into the signal, so a blur/mouseleave landing before that task left the popover stuck open; reproduced deterministically by blurring inside `showPopover()`; fixed by forwarding only native closes; regression tests in both specs. (2) `dx-class-migration` (5 failures in ~1400 runs): measured before hydration, the body fallback picked up the navbar's `dx-select-expand-icon` -- row 109's class, fixed with `gotoHydrated` (also applied to the tooltip/hover-card/drawer specs). (3) close-fade tests (hover-card:57, tooltip, popover): `toBeVisible()` passes at opacity 0 and a frame-count threshold is load-dependent -- row 112's class; one entry point `sampleCloseFade` with an event-based "fades, not snaps" check plus rate-independent invariants. (4) drawer:110 (inferred, fixed 500ms wait) -> shared `awaitAnimationsSettled`; drawer:140 read state after a backdrop-dismissing release -> read before `mouse.up()`. Verified 870/870 and 210/210 at x30 under mixed load, 250/250 at 25x CPU throttle. **Still open:** `combobox.spec.ts:134` (~10% under load on main too; same transient-window class, needs in-page sampling); `menu-submenu.spec.ts:307` hover-intent delay (load-sensitive on main too); carousel timing tests under heavy load -- autoplay racing setup, rapid paging, and `virtual_many` "Previous/Next are genuinely disabled at the real, non-looping ends" (4/6 failures on main in isolation, off-by-one counts); product: a drag released over the drawer's `::backdrop` dismisses it even when dragged away from the dismiss direction; `use_popover_sync` likely has the mirror-image stale-`toggle(closed)` hazard (unproven). **New instance, same day:** the new `sampleCloseFade` helper itself flaked once in the integration full suite (`popover.spec.ts:238`, `--workers=4`): 4 samples over 58ms, opacity 1, no animation started -- either the close trigger had not taken effect yet or the sampling window ended too early; 80/80 on an immediate `--repeat-each=10 --workers=4` rerun. Next step: make the helper wait for the close to begin (e.g. `data-state` flip or `toggle` event) before judging the fade. **2026-09-30 addendum -- two more one-offs of the same shape:** in the carousel true-end integration run (`--workers=4`, SSG release build, 1373 passed / 8 failed), `dx-class-migration.spec.ts:162` "tabs: stylesheet delivered, classes unhashed" and `hover-card.spec.ts:6` each failed once; on a rerun of both files `--repeat-each=8 --workers=4` each failed 1 of 8, the other rows passing (`drawer.spec.ts:110`, also red once in the full run, passed 8/8). None load carousel code, and that batch changed only carousel code, demos, docs and specs. Three overlay/stylesheet specs flaking only under parallel load is a pattern, not three coincidences: next step is one diagnosis lane across all three. Not root-caused, and unrelated to the carousel change in that run. If it recurs, check whether a hover left over from Playwright's own pre-scroll or a neighbouring frame re-opens the tooltip after the focus-out close, and sample the tooltip's open state in-page across the `Tab` rather than via `toHaveCount` polling.
