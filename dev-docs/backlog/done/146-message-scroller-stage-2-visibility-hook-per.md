---
id: 146
title: "message_scroller stage 2: visibility hook, per-call scroll_margin, RTL pass, release-build timing spec"
status: "CLOSED 2026-10-19 (duplicate of row 213)"
size: "M"
area: tests
updated: 2026-10-19
---

# Row 146: message_scroller stage 2: visibility hook, per-call scroll_margin, RTL pass, release-build timing spec

## Item

`message_scroller` stage 2: visibility hook, per-call `scroll_margin`, RTL pass, release-build timing spec

## Status / trigger

**CLOSED 2026-10-19 (duplicate of row 213)**

## Size

M

## Detail

Stage 1 covers follow-the-stream, the jump button, new-turn anchoring with a tail spacer, the opening position, prepend preservation, `scroll_to_*`, and `Virtualization::{None, ContentVisibility}` (default `ContentVisibility`: `content-visibility: auto` on rows, with the last 8 rows, the focused row and the selection ends kept rendered via `data-keep-rendered`; the 2000-row demo renders fewer than 60). **Stage 2** (`shadcn-catalog-2026-10-04.md` section 4.4 and the module doc): `use_message_scroller_visibility` (IntersectionObserver, ref-counted; `currentAnchorId`/`visibleMessageIds`) with a jump-menu / table-of-contents demo, `scroll_margin`/`align`, an RTL pass over the button placement, and a release-build timing spec (timing specs only on release, CLAUDE.md). **Windowing (handing the viewport to `VirtualList`) is not planned for now, by the owner's decision;** the ceiling to document instead: DOM and RSS grow linearly (a chat row is ~20-50 nodes, so 10k rows is 200-500k nodes). The Safari 18-25 find-in-page gap (row 147) belongs on its docs page. **2026-10-14:** the stage-2 items (visibility hook, per-call `scroll_margin`, RTL pass, release timing spec) are items 1, 4, 7 and 8 of row 213, which compares the controller against the original line by line; track them there.
