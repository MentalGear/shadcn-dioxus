---
id: 223
title: "A modal returns focus to its trigger only after its exit animation, so a quick keyboard reopen is lost"
status: "LANDED 2026-10-18 (lane focus (row 223 driver change)), merged on the session branch; deploy…"
size: "S"
area: theme
updated: 2026-10-18
---

# Row 223: A modal returns focus to its trigger only after its exit animation, so a quick keyboard reopen is lost

## Item

A modal returns focus to its trigger only after its exit animation, so a quick keyboard reopen is lost

## Status / trigger

**LANDED 2026-10-18 (lane focus (row 223 driver change)), merged on the session branch; deploy pending** -- was: **Scheduled 2026-10-15 in `dev-docs/research/next-round-plan-2026-10-15.md`** (set up, waiting for the owner's go). **OPEN 2026-10-15**

## Size

S

## Detail

Found by the 83758d5 validation: on `/component/dialog/`, Enter, Escape, then Enter again on the trigger within ~200 ms (the `--dx-overlay-duration` exit) does not reopen the dialog (0/4 at 0 ms, 1/2 at 200 ms, 4/4 at 500 ms): focus is still inside the closing panel, so the second Enter lands there. Follows from closing modals only after their exit animation (a369194). Radix Dialog behaves the same (its FocusScope unmounts after the exit), so this is not a parity gap. **Construction (proposed):** in the shared `use_dialog_open_driver`, move focus back to the opener the moment the close starts (make the closing panel `inert`), so every modal (Dialog, AlertDialog, Sheet, Drawer, Command, modal Popover) gets it at once; spec: reopen by keyboard at 0 ms.
