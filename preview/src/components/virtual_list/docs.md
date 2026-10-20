The VirtualList component keeps a long list cheap. It has two modes, picked by the `virtualize` prop, and by default it picks for you from the row count.

## Component Structure

```rust
VirtualList {
    // Total number of items.
    count: 2000usize,
    // Optional: the height guess for a row that has not been measured or rendered yet.
    estimate_size: |idx: usize| 80,
    // Optional: how off-screen rows are kept cheap. `Auto` (the default) picks by `count`.
    virtualize: VirtualListMode::Auto,
    // Row renderer callback receives the absolute index.
    render_item: move |idx: usize| rsx! {
        article { "{rows[idx].title}" }
    },
}
```

Give the list a height (or `max-height`) and `overflow-y: auto`: it is the scroll container. The list owns its `id`, `role="list"` and `tabindex="0"`; every row is a `role="listitem"` with `aria-setsize` and `aria-posinset`, in both modes.

## Modes

| Behaviour | `ContentVisibility` | `Windowed` |
|---|---|---|
| What is in the DOM | every row | the rows in view plus `buffer` rows each side |
| Off-screen rows | skipped by the browser, 20 rows at a time (`content-visibility: auto`) | not mounted |
| Scrolling costs Rust | nothing: no listener, no re-render | one re-render every few rows (`buffer / 2`) |
| Server-rendered HTML | every row | none |
| Find-in-page (Ctrl+F), selection, accessibility tree | reach every row (find-in-page: not Safari 18-25) | only the mounted window |
| Focus in a row that scrolls away | kept (the row stays rendered) | lost (the row is unmounted) |
| Row heights | the browser measures; `estimate_size` is its first guess | measured by the list; `estimate_size` is its first guess |
| Cost grows with the row count | yes: DOM and memory are linear (about 4.5 KB per 9-node row: roughly 72 MB at 10,000 rows, 445 MB at 100,000) | no: flat from 10,000 to millions of rows |
| Best for | up to a few thousand rows | long lists, 10,000+ |

### Auto (the default)

`Auto` uses `ContentVisibility` while `count <= auto_threshold` and `Windowed` above it. `auto_threshold` defaults to **1,000** rows, because the cost of keeping every row grows with the count: mounting them is one main-thread task proportional to the row count (in the demo, release build, headless Chromium: about 90 ms for 1,000 rows, 205 ms for 5,000, as one 147 ms task), while a window mounts about 40 rows whatever the count. The mode follows the live `count`: a list that grows past the threshold re-mounts its rows in the other mode (its scroll position resets), so choose an explicit mode if the count hovers around the threshold.

```rust
// Every row in the DOM, however many (a long list of one-line rows can afford it):
VirtualList { count: 20_000usize, virtualize: VirtualListMode::ContentVisibility, .. }
// Windowed from the first row (heavy rows, or find-in-page is not needed):
VirtualList { count: 800usize, virtualize: VirtualListMode::Windowed, .. }
// Move the automatic switch point:
VirtualList { count: 12_000usize, auto_threshold: 10_000usize, .. }
```

### Content-visibility mode

Every row is rendered into the page (and into the server's HTML), and the browser skips layout and paint for the rows that are off-screen. Rows are grouped in chunks of 20 consecutive rows and each chunk carries `content-visibility: auto; contain-intrinsic-block-size: auto <sum of its rows' estimate_size>px`: the estimate is the height the browser assumes for a chunk it has not rendered yet, and `auto` makes it remember the real height afterwards. Close estimates keep the scrollbar steady while you scroll up through rows the browser has never rendered.

Why chunks: the browser does per-frame work for every element it may skip, so skipping each row separately costs the main thread time proportional to the row count on every scroll frame. Measured in the demo (release build, headless Chromium, main-thread time per scroll frame): one skippable element per row cost 7.7 ms at 1,000 rows and 31 ms at 5,000; one per 20 rows costs 1.1 ms and 1.5 ms, the same as a windowed list (1.4 ms at 5,001 rows). The price is up to 19 extra rows rendered around the viewport.

- The chunk holding focus, or an end of a selection, is never skipped: that is the browser's own rule for `content-visibility: auto`, so there is nothing to configure.
- Find-in-page finds text in skipped rows and un-skips the match in Chromium, Firefox 125+ and Safari 26+. **Safari 18-25 does not search skipped content** (WebKit bug 283846); there, use `Windowed` and provide your own search, or accept it.
- `content-visibility: auto` applies paint containment to each chunk, so a shadow, focus ring or non-top-layer popover that overflows a chunk's box is clipped (rows inside a chunk do not clip each other).
- The first mount renders every row, and that is the cost that grows with the count (see the default threshold above: about 90 ms at 1,000 rows, 205 ms at 5,000). Any signal a row reads re-renders all of them. For heavy rows or much larger lists, use `Windowed`.

### Windowed mode

Only the rows in view are mounted, over a spacer that gives the scrollbar the full height. Rows may have any height: each is measured once mounted, and `estimate_size` is the first guess (without it, the list estimates from the average of the rows it has measured). While you scroll, the scrollbar's total is held fixed, and a row that resizes above the viewport adjusts the scroll position so content does not jump.

- Rows that are not mounted do not exist for native find-in-page, the accessibility tree (each mounted row announces its position, `3 of 100000`) or the server: the server renders no rows, and the first client render is empty until the container has been measured.
- `buffer` (default 8) is the overscan; the focused row stays mounted; see "The shared windowing engine" and "Focus stays mounted" below, which `VirtualGrid` documents in the same words.
- Browsers cap an element's height (about 17 million px in Firefox, 33 million px in Chromium), which limits the spacer: with 100px rows that is roughly 300,000 rows.

### Find-in-page in windowed mode (experimental)

`text_value: |idx| String` keeps every row's plain text in the DOM as a `hidden="until-found"` placeholder, so the browser's find-in-page and `#:~:text=` links can match rows that are not mounted. When the browser reveals a match it fires `beforematch`, and the list scrolls the real row into view.

- It costs about one element and one text node per row, mounted after hydration (100,000 rows: ~101,000 elements and ~10 MB more JS heap); scrolling costs the same as without it, because the placeholders are skipped in `content-visibility: auto` chunks.
- A mounted row's text is in the DOM twice, so its match counts twice.
- Placeholders are not in the accessibility tree; browsers without `until-found` (Firefox before 139, Safari before 26.2) treat it as `hidden`, so those rows stay unfindable, as without the prop.
- Example: the `find_in_page` variant. Findings and open checks: `dev-docs/backlog/153-native-find-in-page-for-windowed-virtual-lists.md`.

<!-- shared:virtual-engine:engine -->
## The shared windowing engine

`VirtualList` (windowed mode), `VirtualGrid`, `VirtualWrapGrid` and `VirtualJustifiedGrid` mount only what is in view, on one engine and one DOM bridge (`primitives/src/virtual/`).

- **Overscan.** Items beyond each edge of the viewport are mounted too, so a scroll never shows a blank edge: `buffer` rows for `VirtualList` (default 8) and `VirtualWrapGrid` (default 4), `row_buffer` (default 6) and `column_buffer` (default 4) for `VirtualGrid`, and `overscan` px (default 320) for `VirtualJustifiedGrid`. The window is re-centred only once fewer than half of that margin remains, so between half and all of it is always ready beyond each edge, and a steady scroll re-renders about once per half margin rather than once per scroll event.
- **Positions.** Sizes are kept in a Fenwick tree: measuring an item and finding the item at a pixel are `O(log N)`, and nothing is copied per measurement. Where items are measured (a list's rows, a wrapping grid's rows) an estimate is used until then; an item that resizes above the viewport adjusts the scroll position so content does not jump, and the scroll canvas keeps its size while a scroll is running.
- **Listeners.** One passive `scroll` listener and one `ResizeObserver` on the container, plus a `resize` listener on the window, all removed when the component unmounts. Scrolling costs Rust nothing unless the window moves.
<!-- /shared:virtual-engine:engine -->

<!-- shared:virtual-engine:focus -->
## Focus stays mounted

The item that last received focus (a link or button inside a row, or the active cell or tile of a grid) stays mounted when you scroll away from it: it is drawn at its own position outside the window (`VirtualList` marks it `data-virtual-pinned`), so focus never falls to the page and Tab or the arrow keys continue from where they were. Moving focus elsewhere releases it, and when a scroll brings the window back to it the same DOM node rejoins the flow without remounting. (`VirtualList` in content-visibility mode keeps every row in the DOM, so there is nothing to pin.)
<!-- /shared:virtual-engine:focus -->

<!-- shared:virtual-engine:scroll-chaining -->
## Scroll chaining

None of the virtual components traps the page's scroll: at the top or bottom edge of the container a wheel or touch scroll continues on the page (the block axis keeps `overscroll-behavior: auto`). A container that also scrolls sideways contains only the inline axis (`overscroll-behavior-x: contain`), so a swipe at its edge does not trigger the browser's back and forward navigation.
<!-- /shared:virtual-engine:scroll-chaining -->

<!-- shared:virtual-engine:find-in-page -->
## Find-in-page and the windowed DOM

Items outside the window do not exist: the browser's find-in-page and `#:~:text=` links cannot reach them, the accessibility tree holds only what is mounted (each mounted row or cell announces its position in the whole, with `aria-posinset` or `aria-rowindex`), and the server renders none of them or only the active one. The opt-in `text_value` of a windowed `VirtualList` keeps every row's plain text in the DOM as a `hidden="until-found"` placeholder, so find-in-page can match rows that are not mounted (the `find_in_page` variant); the grids have no such placeholders, because a cell is not a line of text. Where findability matters more than a flat cost, a plain list or CSS grid of a few thousand items with `content-visibility: auto` keeps everything findable while the browser skips the off-screen work (`VirtualList`'s content-visibility mode is exactly that).
<!-- /shared:virtual-engine:find-in-page -->

<!-- shared:virtual-engine:fade -->
## Edge fade

`fade: ScrollFade` (`None` by default; `Y`, `X`, `Both`, `Top`, `Bottom`, `Start`, `End`) puts the matching `dx-scroll-fade-*` class on the container, which is the element that scrolls, exactly as on `ScrollArea`. The fade is a CSS scroll-driven mask driven by the container's own scroll offset, so it follows the scrollbar whichever rows or cells are mounted (a windowed list of 20,000 rows or a grid of 500,000 cells fades exactly like a short one), and a container whose content fits has no fade. `Both` masks all four edges of a container that scrolls on both axes (a `VirtualGrid`, or a justified wall with a `canvas_width`), each edge only while there is content past it.

```rust
VirtualList { count: 20_000usize, fade: ScrollFade::Y, render_item: .. }
VirtualGrid { row_count: 1_000usize, column_count: 30usize, fade: ScrollFade::Both, render_cell: .. }
```

- It needs `dx-effects.css` linked once at the app root next to the theme (the `virtual_list` and `virtual_grid` `component.json` list it in `globalAssets`). Without it the prop is inert. See the `dx-scroll-fade` utility in the ScrollArea docs for the variables that tune depth and reveal distance.
- The mask covers everything inside the container: a border or background on it fades with the content (put those on a wrapper and set `border: none` on the container), and so does a `VirtualGrid`'s sticky header, which a top fade dims once you scroll (prefer `ScrollFade::Bottom` or `End` with `render_header`).
- Examples: the `fade` variants (a windowed list and a content-visibility list; a wrapping gallery and a two-axis data grid).
<!-- /shared:virtual-engine:fade -->

## Props

- `count` (required): the number of rows.
- `render_item` (required): renders row `idx`.
- `estimate_size`: row height guess in px, by index.
- `virtualize`: `VirtualListMode::{Auto, ContentVisibility, Windowed}`, default `Auto`.
- `auto_threshold`: the largest `count` `Auto` keeps in `ContentVisibility`; default `1000`.
- `buffer`: windowed mode only; default `8`.
- `fade`: `ScrollFade::{None, Y, X, Both, Top, Bottom, Start, End}`, default `None`; the scroll-edge fade above.
- `text_value`: windowed mode only, experimental; the find-in-page placeholders above.
