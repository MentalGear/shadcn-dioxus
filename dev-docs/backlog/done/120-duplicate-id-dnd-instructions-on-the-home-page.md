---
id: 120
title: "Duplicate id=\"dnd-instructions\" on the home page: DragAndDropList hard-codes the instructions id"
status: "LANDED 2026-10-05 (round 3) -- found 2026-10-03 (chart round's full-page DOM scan); actionable when…"
size: "S"
area: a11y
updated: 2026-10-05
---

# Row 120: Duplicate id="dnd-instructions" on the home page: DragAndDropList hard-codes the instructions id

## Item

Duplicate `id="dnd-instructions"` on the home page: `DragAndDropList` hard-codes the instructions id

## Status / trigger

**LANDED 2026-10-05 (round 3)** -- found 2026-10-03 (chart round's full-page DOM scan); actionable when someone next touches `primitives/src/drag_and_drop_list.rs` or a page renders two lists

## Size

S

## Detail

`primitives/src/drag_and_drop_list.rs` sets `aria_describedby: "dnd-instructions"` (~line 450) on the list and `id: "dnd-instructions"` (~line 482) on `DragAndDropInstructions`; the home page gallery renders more than one list, so the document has duplicate ids and every list's `aria-describedby` resolves to the first instance (an a11y defect, invalid HTML). This is exactly the instance row 93 named ("a hardcoded `id: \"dnd-instructions\"` literal, beside `..props.attributes`") as currently unexercised; it is now exercised. **Fix by construction:** a per-instance id from `use_unique_id` (the `dxc-` / `dxc-c` namespaces from row 118 already make it SSR/hydration-safe) provided through the list's context so the `aria-describedby` and the instructions element always agree; add an axe/`document.querySelectorAll('[id]')` duplicate-id assertion over the home page to `preview.spec.ts` so the class (any hard-coded literal id in a repeated component) cannot recur silently. Distinct from row 114 (that row is a spec flake in the same component, not the id). **Landed 2026-10-05 (round 3):** the construction above, as proposed. `DragAndDropList` creates one `instructions_id` with `use_unique_id` and provides it through the list's context; the list's `aria-describedby` and `DragAndDropInstructions`' `id` both read it, so they always agree and two lists on one page no longer share an id. **Proofs:** a native test (`every_list_describes_itself_with_its_own_instructions_id`) and, in `playwright/drag_and_drop_list.spec.ts` ("Instructions id (backlog row 120)"), "every list's aria-describedby resolves to its own instructions element" (exactly one match, inside the list's own root, distinct per list, with the instructions text) and "the page has no duplicate element ids". **Not built:** the wider assertion this row proposed (a duplicate-id scan over the home page in `preview.spec.ts`), so the class (any hard-coded literal id in a repeated component) is covered only on the DnD page. **DnD ghost opacity (owner decision 2026-10-10):** the dragged-item ghost stays at **0.9** (already the default); the owner accepted that the muted text line is 4.0:1 at 0.9 and that 4.5:1 would need 0.96.
