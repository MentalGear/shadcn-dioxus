---
id: 416
title: "Load flakes, second set (masonry mouse drag, grid find reveal by text fragment, hover overlays after the simulated on-screen keyboard)"
status: "Done on lane/load-flakes-416. Trigger for reopening: one of these specs red under load on an…"
size: "M"
area: tests
updated: 2026-10-25
---

# Row 416: Load flakes, second set (masonry mouse drag, grid find reveal by text fragment, hover overlays after the simulated on-screen keyboard)

## Item

Lanes of rows 395, 396 and 397 saw the same reds in `test:changed` under load, each green alone:
`drag_and_drop_list.spec.ts` "a mouse drag on the masonry card dims the ghost and opens the gap",
`virtual-grid-modes.spec.ts` "a browser reveal lands on the real item although the row estimate is off" (row 406),
the NavigationMenu entries of `oracle/tier2-html/top-layer.spec.ts` Rule 11 (b) and (c) (the simulated iOS on-screen
keyboard), and `layout-stability` "/" light at 1 of 64 widths when paired with `top-layer` at 2 workers.

## Status / trigger

Done on `lane/load-flakes-416`. Trigger for reopening: one of these specs red under load on an unchanged component.

## Size

M

## Detail

Each was reproduced on a debug `lane-ssg` build of `queue/batch-17` (18bbb8d5) with six busy loops and two workers
before it was fixed, and verified under the same load. The shared shape, the same as row 407's: the test drives the
component with an emulated input that a real one does not match, and then reads a moment the component's contract
does not promise. In all three the component is right and the test is changed; no assertion lost its meaning.

**Masonry mouse drag (`drag_and_drop_list.spec.ts`).** Before 7 of 12 and 6 of 24 red, every time with the slot
still between items 1 and 2 (5.8 px against 31). Playwright's emulated drag fires `dragover` only for the moves it
is given, and the last one did not arrive (move to y = 460, last `dragover` logged at 448). Passing item 1 opens the
slot before item 2, which slides 25 px down under the pointer over the 180 ms transition, so the last answered event
read "before item 2" or "after" depending on how far the slide had got. A real browser keeps firing `dragover` while
the pointer rests, so a real drag converges. Construction: the pointer stops in the half of item 2 that reads
"after" whether or not it has shifted (97 %), and the poll that reads the slot moves the pointer before each read
(one pixel apart), as the repeating `dragover` does. After: 36 of 36. A ticket counter in the primitive (newest
`dragover` wins) was tried and reverted: the logs showed no reordering. No other test in the spec drags with the
mouse and then reads a settled slot.

**Grid find reveal (`virtual-grid-modes.spec.ts`).** Before 12 of 12 red, every time with no `beforematch` at all.
The test loaded `#:~:text=Photo%202108` with the page; the find placeholders mount after hydration
(`use_find_in_page`), 2.5 to 3.3 s after `load` on a loaded box with a debug build, and Chromium did not search the
fragment again then. That Chromium searches a first-load fragment only around `load` is inferred from this, not read
in its source. A same-document navigation to the fragment on the hydrated page fired `beforematch` on 2107 every
time, with the browser's own scroll. Construction: hydrate, wait for `hidden="until-found"` on the 2107 placeholder,
then navigate to the fragment; "it stays in view" counts 60 animation frames instead of waiting 500 ms, since the
settle (`SETTLE_MATCH_JS`) counts frames. After: 36 of 36. Not subsumed: `virtual-list-find.spec.ts`'s text-fragment
test has the same first-load shape, but it only records what happened and asserts "in view if it fired", so it
cannot go red; under load it stops covering the reveal without saying so.

**Hover overlays after the on-screen keyboard (`top-layer.spec.ts` Rule 11 (b), (c)).** Before 3 of 54 red, each a
90 s wait for `#clip-navigation-menu-content`. After the viewport shrink and the wheel, the trigger is under the
sticky navbar; the test's `trigger.hover()` scrolled it back (y 869 -> 853) before the pointer entered, and
NavigationMenu closes on any scroll (`use_close_on_scroll`, `AnyScroll`). That close reaches Rust as a message some
time after the scroll; under load it landed after `pointerenter`, so the panel shut under a pointer that did not
move again, and `readAtRest` waited for a removed element. Construction (`assertNoSelfOverlapOnceOpen`): the test
scrolls the trigger into view itself and lets two frames pass, so no scroll is left to close the panel, then
measures the first instant at which the panel is open (open state and rect in one in-page read, each read bounded at
2 s), re-entering the trigger while it is not. Click- and keyboard-driven overlays are measured as before. After: 0
of 320. It covers (b) and (c) for every hover-driven overlay in the loop (Tooltip, HoverCard, Navbar,
NavigationMenu); only NavigationMenu was load-tested.

**Home `layout-stability` with `top-layer`.** Covered by row 407's first-paint gate, which `queue/batch-17` contains:
the "/" light sweep paired with both `top-layer` specs at two workers and four busy loops was green 6 of 6. The row
395 report did not name the element that moved. If it shows again on a build with row 407, `LS_DUMP=1
LS_WIDTHS=<width>` names it.

**Tooling found on the way.** A lane branched from a queue branch that `origin/main` does not contain gets nearly the
whole suite from `test:changed`'s default range (`origin/main...HEAD`), which is how these reds surfaced in lanes;
`CHANGED_RANGE=<queue branch>..HEAD` selects the lane's own diff, while `tools/select.mjs` takes `--range`, not the
variable.
