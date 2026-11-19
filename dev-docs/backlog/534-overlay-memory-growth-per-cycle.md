---
id: 534
title: "Wasm memory growth per overlay open/close cycle: find the cause, gate the preview's logger feature"
status: "BACKLOG, NOT SCHEDULED, last in every ordering (it has the highest id of rows 529-534 and no…"
size: "S for the diagnosis (a soak harness…"
area: primitives
updated: 2026-11-19
---

# Row 534: Wasm memory growth per overlay open/close cycle

## Item

Wasm linear memory grows about 1.75 KB per dialog open/close cycle, 2.9 KB per combobox cycle and 3.0 KB per
dropdown-menu cycle, with the same slope in both halves of a 1,500-cycle run; popover is flat for about 300 cycles, then
slow. The cause is not identified. At this size (1,000 opens cost about 2 to 3 MB) the owner and the main loop judge it
negligible. This row records the work if it ever matters: identify the cause of the wasm slope; test whether the slope
scales with an overlay's content; drop or gate the preview's `logger` feature in release builds; consider a soak budget
once the cause is known.

## Status / trigger

BACKLOG, NOT SCHEDULED, last in every ordering (it has the highest id of rows 529-534 and no trigger). Start only when the
owner asks, or when a measured page shows memory that matters (a soak over thousands of opens, or a content-heavy overlay).

## Size

S for the diagnosis (a soak harness exists in the evaluation's scratch work only; the lane rebuilds it).

## Detail

- **Measured (2026-11-19, local `docs/` and the deployed site, identical):** dialog 20.25 to 22.94 MB over 1,500 cycles
  (1.82 KB per cycle in the first half, 1.77 in the second); combobox 26.69 to 30.75 MB over 1,500 (2.80 in both halves);
  dropdown menu 21.38 to 22.81 MB over 500; popover 19.31 to 19.44 MB over 500. Continuous growth, not a high-water mark.
- **`logger`:** `preview/Cargo.toml:8` lists the `logger` feature of `dioxus`; tracing-wasm then records about 48
  performance marks and 48 measures per dialog cycle (190 and 190 per combobox cycle). That raises the JS heap (3.91 to
  6.86 MB after forced GC over 1,500 dialog cycles; 5.98 MB with the calls stubbed) and leaves the wasm slope unchanged. It
  affects the preview site only: no other manifest lists `logger`. Drop it or gate it off in release builds.
- **Not ours:** a reported combobox DOM-node leak is Chromium's editing undo stack (a plain `<input>` page with no Dioxus
  grows 1.01 nodes per cycle).
- **Open question:** whether the slope scales with an overlay's content (a dialog with 10 versus 1,000 nodes).
- **Proof (when started):** an allocation diff (a counting allocator in a native test, or a wasm heap snapshot) that names
  the retained allocation; the slope after the fix is flat over 1,500 cycles on the three overlays above.
- Source: [`research/repo-evaluation-2026-11-19.md`](../research/repo-evaluation-2026-11-19.md), section 2.
