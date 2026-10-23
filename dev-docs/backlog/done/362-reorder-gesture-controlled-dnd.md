---
id: 362
title: "Reorder gesture extracted, KeyOrder, DragAndDropList controlled mode with on_reorder"
status: "LANDED 2026-10-22 (batch 8, PR #135). BACKLOG (owner asked 2026-10-22 to deconstruct the collection…"
size: "M."
area: primitives
updated: 2026-10-23
---

# Row 362: Reorder gesture extracted, KeyOrder, DragAndDropList controlled mode with on_reorder

## Item

Migration step 4: one keyboard-accessible reorder gesture out of `drag_and_drop_list.rs`, a `KeyOrder` type (ordered keys with move operations) shared by row and column order, a controlled DragAndDropList (`order` + `on_reorder`; today it reads `items` once and never reports the new order), drop the deprecated `aria-grabbed`, one key derivation. Parallel with row 361.

## Status / trigger

**LANDED 2026-10-22 (batch 8, PR #135).** BACKLOG (owner asked 2026-10-22 to deconstruct the collection model to its roots; design note [`research/collection-model-2026-10-22.md`](../../research/collection-model-2026-10-22.md)). Order: 360, then 361 and 362 in parallel, then 363, then 341, then 342.

## Size

M.

## Detail

See the design note's "The model", "Migration path" and "Backlog impact" sections.

## Landed (claude/reorder)

- `KeyOrder` (primitives/src/data_view/state.rs) gained `len`, `is_empty`, `position`, `move_key`, `move_by`, `move_before`, `remove`, `arrange` (reconcile over the items themselves) and `into_keys`; law test `key_order_move_then_inverse_move_is_identity` (200 seeds) plus unit tests; the 360 laws stay green.
- The gesture is `primitives/src/drag_and_drop_list/reorder.rs`: `ReorderState` (pure), `command_for_key` (the one key map, `ReorderAxis` vertical or horizontal, horizontal arrows through `Direction::resolve_horizontal`), `pointer_position` (RTL swaps the halves on the inline axis) and the announcements, all unit-tested.
- `DragAndDropList`: controlled mode (`order: ReadSignal<Vec<String>>`, `on_reorder: Callback<Vec<String>>`, `on_remove`, `axis`); `items` alone is the uncontrolled mode on a `KeyOrder`; the displayed list is the items arranged by the one order. `aria-grabbed` dropped (`data-is-grabbing` stays). One key derivation, `sortable_item_key`, used by the themed wrapper too.
- Specs: `Reported order (backlog row 362)` in `playwright/drag_and_drop_list.spec.ts` (keyboard, Escape, pointer and app-written order against the DOM order on the new `controlled` demo; no `aria-grabbed`).
