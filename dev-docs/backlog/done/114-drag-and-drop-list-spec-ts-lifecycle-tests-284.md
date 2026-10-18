---
id: 114
title: "drag_and_drop_list.spec.ts lifecycle tests (:284 mouse drop to the side, :300 cancelled mouse drag) compare…"
status: "LANDED 2026-10-05 (round 3) -- Found 2026-09-26 by the audit batch's integration run (--workers=4)…"
size: "S"
area: tests
updated: 2026-10-05
---

# Row 114: drag_and_drop_list.spec.ts lifecycle tests (:284 mouse drop to the side, :300 cancelled mouse drag) compare…

## Item

`drag_and_drop_list.spec.ts` lifecycle tests (`:284` mouse drop to the side, `:300` cancelled mouse drag) compare an item's text captured before its avatar fallback initials render with one read after -- fails under parallel load

## Status / trigger

**LANDED 2026-10-05 (round 3)** -- Found 2026-09-26 by the audit batch's integration run (`--workers=4`): 1 failure in the full suite; isolated, 7-8 failures in ~100 runs at `--repeat-each=6 --workers=4`, 0/8 at `--workers=1`. **Pre-existing:** `78cff69`'s own spec against `78cff69`'s own build fails the same way (8 failed / 94 passed)

## Size

S

## Detail

Received texts end in the avatar initials the expected text lacks (`...Apr29` vs `...Apr29KT`): the test snapshots `itemText()` while the avatar is still on its image path, and the fallback text arrives later. Fix the comparison, not the timing: identify items by a stable key (the task id such as `LNC-147`, or a `data-` attribute) rather than their whole `textContent`, or wait for the avatars to settle before the first read. Same family as row 112: an assertion comparing two reads of a value that is still changing. **Landed 2026-10-05 (round 3):** fixed in the comparison, not the timing, as this row proposed. `itemText` in `playwright/drag_and_drop_list.spec.ts` now clones the item and drops the `.dx-avatar` subtree before reading `textContent`: the avatar is decoration, not part of an item's identity, and what remains (title, task code, status, due date) is stable from the first render, so nothing waits for the avatars to settle. 45/45 repeats pass (the 7-8 failures in ~100 runs at `--repeat-each=6 --workers=4` are gone). Related: the demo avatars are now local geometric SVGs (`scripts/gen-demo-avatars.mjs`, gate `check-demo-remote-assets.sh`), so no third-party image is on the page to wait for.
