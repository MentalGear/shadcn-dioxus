---
id: 29
title: "color-picker.spec.ts's \"clicking outside closes the color picker popover\" got measurably flakier after the…"
status: "RESOLVED 2026-10-04 (unreproduced, 50/50 green; see the 2026-10-04 addendum in the notes)…"
size: "S to re-measure (done 2026-10-04); no…"
area: tests
updated: 2026-10-04
---

# Row 29: color-picker.spec.ts's "clicking outside closes the color picker popover" got measurably flakier after the…

## Item

`color-picker.spec.ts`'s "clicking outside closes the color picker popover" got measurably flakier after the row 28 fix landed (pre-fix ~80% pass over 15 runs; post-fix ~50% pass over 22 runs, same sandbox)

## Status / trigger

**RESOLVED 2026-10-04 (unreproduced, 50/50 green; see the 2026-10-04 addendum in the notes)** -- original trigger, now met: A quieter environment to re-measure in (this sandbox had a second, unrelated concurrent session sharing the same dev server and a headless Chromium on the same container while these numbers were taken — a genuine confound, not ruled out) and, if it reproduces cleanly, root-causing why always-on tracking listeners / a permanently-pending `document::eval` channel on a CSS-anchor-conforming overlay would perturb an unrelated outside-pointerdown dismiss path

## Size

S to re-measure (done 2026-10-04); no fix needed

## Detail

Found 2026-09-02 verifying row 28's fix: this exact test was already flaky before the fix (~20% fail, unrelated pre-existing race — reproduced dark: 3/15 failures pre-fix), so this is "got worse," not "newly broken." Every other repeated-open/close-cycle regression check in the fix's own verification pass (`popover.spec.ts`'s rapid open/close/open, the full `top-layer.spec.ts`, `combobox.spec.ts`, `select.spec.ts`, `dropdown-menu.spec.ts`, `menubar.spec.ts`) stayed green; only this one test moved. Not fixed here: rule-cited oracle work should not chase a possibly-confounded flake-rate delta on an already-known-flaky test without a clean re-measurement first **2026-10-03 re-audit (main @ 2241da8) -- still OPEN, but the hypothesis in this row is refuted and the remaining question is purely empirical.** (1) The color picker's popover is the non-modal web `Popover` arm, `PopoverNonModalContent` (`primitives/src/popover.rs:~660-720`, `popover="auto"`), which installs NO JS outside-dismiss at all (the comment block at `popover.rs:~405-440` says so; `use_outside_dismiss` is called only from the Blitz (non-`web`) arms, `popover.rs:443` and `:718`). The dismiss under test is therefore the browser's own light-dismiss algorithm, which is driven only by trusted `pointerdown`/`pointerup` (WHATWG HTML, "To light dismiss open popovers, given a PointerEvent event") -- page JS listeners and a pending `document::eval` channel (the row's hypothesis: always-on tracking in `use_anchor_position_fallback`, `top_layer.rs:~1407`) have no way to perturb it. For this test's shape (a `mouse.click(2,2)` outside the popover with the popover open before `pointerdown`) the algorithm is deterministic. (2) Several full-suite diffs since (rows 82, 87, 88, 154-160: 775 -> 910 tests) list color-picker only for Rule 8 (row 21's anchor-freeze class), never for this test. (3) Not closed anyway: this lane could not build or run Playwright, so no clean re-measurement exists. **Closing condition:** on a quiet machine, `cd playwright && npx playwright test color-picker.spec.ts -g "clicking outside" --repeat-each=50 --workers=1` against a release build; 50/50 green -> mark this row resolved/unreproduced; any red -> capture the trace and look at the `toggle` forward path (`use_popover_shown_while_mounted`, `top_layer.rs:~905-940`: the toggle listener is attached by an async eval round-trip, so a click landing before it attaches closes the popover natively without telling Rust -- the element is hidden so the test still passes, but `aria-expanded`/`open` strand `true`; that is the one real defect found while auditing, filed here rather than fixed blind) **2026-10-04 addendum -- RESOLVED (unreproduced):** 50/50 green on release 9f8399d (`color-picker.spec.ts -g "clicking outside" --repeat-each=50 --workers=1`, 2.3 min, quiet 4-core machine, static SSG server, `baseline/RELEASE.md` section 3c) -- the closing condition set above is met, so the row is resolved. The filed-not-fixed `toggle`-forward observation (a click landing before the async toggle listener attaches leaves `aria-expanded`/`open` stranded `true`) stays recorded above.
