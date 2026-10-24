---
id: 406
title: "Find-in-page lands a match on the item's estimated position, not on the item"
status: "Fixed on lane/find-photo-wall: use_find_in_page (primitives/src/virtual/find.rs) now settles every…"
size: "S"
area: virtual
updated: 2026-10-24
---

# Row 406: Find-in-page lands a match on the item's estimated position, not on the item

## Item

Owner report: in the `virtual_grid` page's `find_in_page` demo the text grid works, but the photo grid (`VirtualWrapGrid`) shows two matches for "Photo 2108" and one of them scrolls to some other photos with nothing highlighted.

## Status / trigger

Fixed on `lane/find-photo-wall`: `use_find_in_page` (`primitives/src/virtual/find.rs`) now settles every match on the real item's own box. Trigger for reopening: a windowed view whose match ends outside its scrollport, or a consumer that passes `None` for the item selector without scrolling the real item itself.

## Size

S

## Detail

**Root cause (measured with the browser's own find bar, headed Chromium under Xvfb driven by `xdotool`, and with a `#:~:text=` fragment, which Playwright can drive):** a placeholder sits at its item's ESTIMATED position, and the browser scrolls the container to the placeholder after `beforematch`; the engine's reveal (`Bridge::reveal`) also scrolls to the estimate. The demo's `estimate_row_height` is 64 but a real row is 72 (the item's 56 + 8 margin, plus the grid's own `padding-block-end: 8px` on `[role=row]`). Rows then mount around the target, are measured at 72, and every measured row above the item pushes it 8 px down while the container stays where the browser scrolled it: item 2108 (its 422nd row) ended 129 px below the estimate, at 247..311 px of a 256 px viewport, so the grid showed Photo 2081-2100 and the item was half out of view. `VirtualGrid` (the text grid) had its own second step (`reveal_cell` scrolls the mounted cell into view by its real box), and its row heights are exact, which is why it "works"; `VirtualList` and `VirtualWrapGrid` ended at `Bridge::reveal` only, and the list only stayed in view because its drift per buffered row is smaller against a taller viewport.

The two matches are the documented double count (a mounted item's text is in the DOM twice: its placeholder and the item, `find.rs`); the text grid has it too (its second match for "Order 01777" was the demo's own description paragraph). It is not what broke the photo grid: in the real find bar run match 2/2 highlighted the real item correctly, and 1/2 (the placeholder) was the one that landed on the wrong rows. Pressing Enter alternated the grid between the two positions (scrollTop 26834 and 27171).

**Fix, by construction:** `use_find_in_page` takes the selector of the real item (`item: Option<fn(&str) -> String>`) and, after the consumer's `on_match` has moved the window, runs `SETTLE_MATCH_JS`: wait for the item, then until it has stayed inside the scrollport for four frames (at most sixty), scroll the container to centre it, reading the item's real rect each frame; a newer match or the reader's wheel/touch/pointer/key input on the container ends it. `VirtualList` and `VirtualWrapGrid` pass their item selectors; `VirtualGrid` passes `None` because `reveal_cell` already does this beside its pinned edges. A new consumer cannot forget the step without writing `None`.

**Spec:** `playwright/virtual-grid-modes.spec.ts` "a browser reveal lands on the real item although the row estimate is off" (red on `fc5a3bd8`: the item at 247..311 of 256 px; the demo's estimate stays deliberately below the real row so the test keeps meaning).

**Not changed:** the double count. Keeping the placeholders of mounted items is what makes find-next not skip items as the window moves (row 153); hiding them would change the DOM the find bar walks while it walks it.
