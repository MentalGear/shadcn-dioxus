---
id: 154
title: "Shared virtualization engine: ONE engine in primitives/src/virtual/ (core, strategy, DOM bridge) that every…"
status: "FOLDED 2026-10-23 into row 309 (owner): this is the engine's design note, kept here for reference…"
size: "S (note); the build is not estimated"
area: virtual
updated: 2026-10-23
---

# Row 154: Shared virtualization engine: ONE engine in primitives/src/virtual/ (core, strategy, DOM bridge) that every…

## Item

Shared virtualization engine: ONE engine in `primitives/src/virtual/` (core, strategy, DOM bridge) that every virtualized or windowed component consumes, rather than merging the components

## Status / trigger

**FOLDED 2026-10-23 into row 309 (owner): this is the engine's design note, kept here for reference; open engine work lives in row 309.** **DESIGN NOTE** -- first note recorded 2026-10-05 from the owner question whether the virtualized components should share an implementation; the **owner's three-layer outline was added the same day**. Not scheduled: it waits for row 143 to land, and its first consumer is `message_scroller` (with row 146). No code change here. **Owner decision 2026-10-05: stays BACKLOG, with a REQUIRED first step: research the impact on Carousel before any migration (first paragraph of the notes)**

## Size

S (note); the build is not estimated

## Detail

Design note: one shared virtualization engine in `primitives/src/virtual/` rather than merging the virtualized components. First
note 2026-10-05 from the owner's question whether the components should share an implementation, with the owner's three-layer
outline added the same day. **Folded 2026-10-23 into row 309**, where open engine work lives.

**Owner decision 2026-10-05: required first step.** Before any migration, research Carousel's impact in writing. Carousel is not a
plain windowed list: it carries trackpad and wheel behaviour no other consumer has (the browser-idle gate
`CAROUSEL_SCROLL_TRACKING_JS`/`CAROUSEL_INPUT_QUIET_MS`, trusted-wheel handling, the mode-D edge band, a virtual window whose
re-align waits for quiet input, and a carousel-owned bridge registering `scroll`, `wheel`, `touchstart`, `touchend`,
`pointerdown`, `keydown` and `dx-carousel-rest`). The research must say whether the engine's DOM bridge (one passive scroll
listener plus one ResizeObserver, scope-owned, a render-signal write only when the range changes) can host that, listing each input
path as a bridge concern or a carousel-physics concern and prototyping against `carousel-virtual-wheel.spec.ts`,
`carousel-wheel-replay.spec.ts` and the tier-1 carousel oracle. Decision rule: if hosting needs carousel-specific branches in the
bridge, Carousel stays on the core `window()` alone and drops out of the consumer list.

**The outline: one engine, three layers.**

1. *Core*, pure Rust and DOM-free: `SizeIndex` (estimates plus measurements, O(log n) offset to index), `window(offset, viewport,
   overscan)`, anchoring (keep an item's on-screen position when sizes above change or items are prepended), `scroll_to(index,
   align)`. One axis; a grid is two indices.
2. *Strategy*, how out-of-window items exist: Windowed (a range plus spacers), ContentVisibility (all rendered, chunked), Auto (a
   threshold); later Windowed plus find-in-page (row 153, `hidden="until-found"` stubs).
3. *DOM bridge*, one `use_virtualizer` hook: a passive scroll listener, one ResizeObserver, scope-owned cleanup (row 159), a
   render-signal write only when the range changes.

Consumers: VirtualList (done), MessageScroller (a reverse list; stick-to-bottom is an anchor policy), Carousel only if the research
says so, long Select/Combobox/Command lists, DataTable rows, a Grid. Not covered: carousel physics, drag-and-drop autoscroll,
sticky headers. Order: Carousel research, then `message_scroller` (with row 146), then the Carousel loop if the verdict allows,
then the list components when a long-list need appears. **Do not merge the components:** their scroll models differ in kind (free
scroll with measured sizes, snap pages with loop rotation, stick-to-bottom with prepend preservation), so one component would be a
union of three state machines. Row 143 built the first pieces (`cv_chunks.rs`, shared with `message_scroller`, and the Fenwick
`size_index.rs`); row 159's `use_js_listeners` is the listener helper a bridge should use.

**Update 2026-10-20 (row 308; owner decision, row 309):** VirtualList stays its own component rather than a list configuration of
VirtualGrid, because a list and a grid need different ARIA semantics (`role=list`/`listbox` against `role=grid` with cells and
column headers); the two share as much of the engine as makes sense. The owner also approved a Justified layout mode for
VirtualGrid. Row 308's scroll trap landed in `7f172201`.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/154-shared-virtualization-engine-one-engine-in.md`.*
