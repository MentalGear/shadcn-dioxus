---
id: 143
title: "VirtualList: add a content-visibility mode (default up to ~5k rows), quantize the scroll signal, drop the…"
status: "LANDED 2026-10-05 (round 3; owner-approved) -- from the virtualization survey…"
size: "M"
area: virtual
updated: 2026-10-05
---

# Row 143: VirtualList: add a content-visibility mode (default up to ~5k rows), quantize the scroll signal, drop the…

## Item

`VirtualList`: add a content-visibility mode (default up to ~5k rows), quantize the scroll signal, drop the O(N) per-resize clone

## Status / trigger

**LANDED 2026-10-05 (round 3; owner-approved)** -- from the virtualization survey (`dev-docs/research/virtualization-survey-2026-10-04.md` sections 2, 7.1, 8)

## Size

M

## Detail

Evidence (code reading, not benchmarked): every scroll event re-renders the whole `VirtualList` (bridge `virtual_list.rs:126-170`, `virtualizer.rs:257-258`); `onresize` does `measurements.peek().clone()`, O(N), per row resize (`virtual_list.rs:208`); `compute_measurements` rebuilds an O(N) `Vec` per newly measured row (`:105-117`); `item_size_cache` never evicts (`virtualizer.rs:186`); SSR renders 0 rows (`virtualizer.rs:260`) and find-in-page sees ~30 of 2000. Probe (Chromium 141, loaded box, trust ratios): cv is 3-5x faster to first layout and 2.5-3.5x lighter than rendering every row (10k rows +72 MB vs +194 MB) but stays linear in N (+445 MB at 100k vs +12 MB windowed), so windowing stays for > ~10-50k rows. **Plan:** (1) a `ContentVisibility` mode mirroring `message_scroller`'s `Virtualization::ContentVisibility`, default for N <= ~5k, windowing above; (2) quantize the scroll signal to the visible range so a scroll event re-renders only when the range changes; (3) remove the per-resize clone, and ignore ResizeObserver entries from skipped rows (`checkVisibility({contentVisibilityAuto: true})`), which would otherwise cache a skipped row's `contain-intrinsic-size` as a real measurement. Specs: find-in-page and a 10k-row case; none exist. **Not a grid:** we have no 2-D virtualization; the survey recommends CSS grid + cv (two-value `contain-intrinsic-size`) before any 2-D windowing engine. **2026-10-05 addendum -- Owner-approved 2026-10-05; in progress in this session.** Engine-level helpers this row needs (content-visibility mode helper, quantized range signal) belong in `primitives/src/virtual/` (row 154); find-in-page for the windowed mode above the cv threshold is its own research row (153). **Landed 2026-10-05 (round 3):** `virtualize: Auto, ContentVisibility or Windowed` (`VirtualListMode`) on `VirtualList`, same prop name and `data-virtualize` spelling as `message_scroller`. **`Auto` (default) is content-visibility while `count <= auto_threshold`, windowed above; `auto_threshold` defaults to 1000, not the ~5k this row planned**, because mounting every row is one main-thread task proportional to the count (~90 ms at 1,000 rows, 205 ms at 5,000, one 147 ms task). **Chunked content-visibility:** `primitives/src/virtual/cv_chunks.rs` groups 20 consecutive rows per skippable element (the browser does per-frame work for every element it may skip), keyed by absolute position; per scroll frame (release): 1,000 rows 7.7 ms one-per-row against 1.1 ms chunked, 5,000 rows 31 against 1.5 (windowed 1.4). `message_scroller` uses the same module: its 2,000-row demo went 20.3 -> 3.7 ms per frame (debug numbers, heavily loaded machine). **Windowed engine:** a Fenwick tree, `primitives/src/virtual/size_index.rs`, replaces the per-measure O(N) rebuild and the per-resize clone, and the scroll signal is quantized (the mounted range keeps hysteresis): 100k rows, 1000 px scroll, 53 -> 3 renders and 275 -> 48 ms. Plan item 3's second half (ignoring ResizeObserver entries from skipped rows) is moot: content-visibility mode has no measurement at all. **Specs:** `playwright/virtual_list.spec.ts` (the cv mode with find-in-page, the windowed trade-off stated as a test, renders per scroll, the unmount listener count, and a release-only per-frame assertion under 4 ms at 1,000 rows) and `message_scroller.spec.ts`; the per-frame thresholds only mean something on a release build. Find-in-page above the threshold stays row 153; the shared engine outline is row 154. Numbers and the table: [round-3 note](../../research/round3-2026-10-05.md), section 5.
