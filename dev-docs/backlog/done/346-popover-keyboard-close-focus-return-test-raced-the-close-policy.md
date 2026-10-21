---
id: 346
title: "Popover keyboard-close focus-return spec raced the close policy"
status: "Done 2026-10-21. Resolution: not a product race. The spec waited on dialog[popover]:popover-open to…"
size: "S"
area: tests
updated: 2026-10-21
---

# Row 346: Popover keyboard-close focus-return spec raced the close policy

## Item

`playwright/open-focus-lift.spec.ts` "Popover: a keyboard close goes to the named target" failed about 1 run in 10 to 2 in 3 on a debug SSG of main `8c4656f6` (2 of 20 on this branch's base build): `document.activeElement` was the trigger (auto id `dxc-NNNN`), not `#popover-focus-return`.

## Status / trigger

**Done 2026-10-21.** Resolution: not a product race. The spec waited on `dialog[popover]:popover-open` to disappear and then read `activeElement` once.

## Size

S

## Detail

Focus and event trace of one Escape close (non-modal `popover="auto"`, `performance.now()` ms): `keydown` 0, `beforetoggle:closed` +0.5, the browser hides the popover and restores focus to the trigger (`focusin` trigger) +2, the `toggle` event task reaches Rust +10..+25, `dxCloseFocus` calls `focus()` on `#popover-focus-return` +3 later. `:popover-open` stops matching at the first step, so the spec's `toHaveCount(0)` is satisfied roughly 10-25 ms before the author's `final_focus` is applied; a read in that window sees the trigger. Six traced runs showed the same order every time and the final target always won; the spec's read landed on either side of the `toggle` task depending on timing.

The fix is in the spec: `await expect(page.locator("#popover-focus-return")).toBeFocused()` (a polling wait on the right signal; it still fails if focus never reaches the target). `open-focus-lift.spec.ts` as a whole (Dialog, Popover, AlertDialog, programmatic opens, Select, Menu, modality writer: 15 tests) passes 300 of 300 at `--repeat-each=20 --workers=2`. The Dialog and AlertDialog tests do not have the window: their native `close()` runs inside `use_dialog_open_driver` and `dxCloseFocus` follows it in the same eval.

Known, accepted: between Escape and the `toggle` task a native popover close shows focus on the trigger for ~10-25 ms before moving to the named target (a focus flash a screen reader may announce). Closing it would mean applying `final_focus` from `beforetoggle` in JS; not done, no user report.
