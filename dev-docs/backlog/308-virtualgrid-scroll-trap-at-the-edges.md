---
id: 308
title: "VirtualGrid traps page scroll at its edges; should VirtualList and VirtualGrid be unified?"
status: "OPEN (owner report 2026-10-20; with the Round 4 thread)"
size: "S"
area: virtual
updated: 2026-10-20
---

# Row 308: VirtualGrid traps page scroll at its edges; should VirtualList and VirtualGrid be unified?

## Item

When a VirtualGrid's own scroll reaches its top or bottom edge, further wheel or touch scrolling does not pass to the page (document), so the page will not scroll while the pointer is over the grid. The owner also asks how VirtualList avoids this, and whether VirtualList and VirtualGrid should be unified, with VirtualList as a list configuration of VirtualGrid that takes the best of each.

## Status / trigger

OPEN (owner report 2026-10-20; with the Round 4 thread)

## Size

S

## Detail

**Cause (confirmed by the Round 4 thread):** `overscroll-behavior: contain` on `.dx-virtual-grid` in `preview/src/components/virtual_grid/style.css` (line 17) stops scroll chaining at the grid's edges. `preview/src/components/virtual_list/style.css` declares no `overscroll-behavior`, so VirtualList's scroller chains to the page at its edges, which answers the owner's question. A code reading by this sync also found no wheel or touch handler calling `preventDefault` (`primitives/src/virtual_grid.rs` calls it only in its two keydown handlers). **Fix:** a small Round 4 PR is in flight that keeps the containment on the x axis only (so a horizontal swipe cannot trigger browser back-navigation, while vertical scroll chains to the page). Keep this row OPEN until it merges.

**Files:** `primitives/src/virtual_grid.rs`, `primitives/src/virtual_list.rs`, shared engine in `primitives/src/virtual/` (`grid.rs`, `virtualizer.rs`, ...); styled wrappers `preview/src/components/virtual_grid/{component.rs,style.css}` and `preview/src/components/virtual_list/{component.rs,style.css}`.

**Open design question (owner has not decided):** unify VirtualList as a list configuration of VirtualGrid? Round 4's recommendation: keep two components, because a list and a grid need different ARIA semantics (`role=list`/`listbox` rows versus `role=grid` rows, cells and column headers), and move the shared bridge and strategy layer into the engine, as row 154 (shared virtualization engine) already proposes. If the owner approves a unification, a follow-up row is held for it (not created yet). Related: row 244 (VirtualGrid, landed), row 153 (find-in-page for windowed lists).

Links: [row 154](154-shared-virtualization-engine-one-engine-in.md), [row 153](153-native-find-in-page-for-windowed-virtual-lists.md).
