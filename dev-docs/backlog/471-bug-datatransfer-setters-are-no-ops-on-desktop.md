---
id: 471
title: "Bug: DataTransfer setters in rsx drag handlers are silent no-ops on desktop"
status: "APPROVED 2026-10-26 by the owner (\"Ok for the rest\", backlog thread); for the next session (owner…"
size: "S"
area: primitives
updated: 2026-10-26
---

# Row 471: Bug: DataTransfer setters in rsx drag handlers are silent no-ops on desktop

## Item

Desktop drag events carry a `SerializedDataTransfer` whose `set_data`, `clear_data`, `set_effect_allowed` and `set_drop_effect` do nothing and return `Ok` (`dioxus-html-0.7.9/src/data_transfer.rs:116-146`), so the shipped drag handlers' effect and payload never reach the page on desktop. Sites: `primitives/src/drag_and_drop_list.rs:610,862-865,874`, `preview/src/components/data_table/reorder.rs:288,365-368` and `preview/src/components/data_table/component.rs:537`.

**Where:** desktop and mobile renderers (web is fine).

**Reproduce first:** run the reorder demos on a desktop (WebKitGTK) build and check that the drag image, effect and the Firefox-style payload take effect; on web they do.

**Source:** [`board-2026-10-26-r4/whole-core.md`](../research/board-2026-10-26-r4/whole-core.md) finding 4 (also in the round 4 synthesis); restated by round 5's admission report.

Related to [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) but not part of it: the plan's construct subsumes this defect later, and the fix does not wait for the plan. Source: the last section of [`board-2026-10-26-r5/SYNTHESIS.md`](../research/board-2026-10-26-r5/SYNTHESIS.md) (live defects the board found in today's code).

## Status / trigger

APPROVED 2026-10-26 by the owner ("Ok for the rest", backlog thread); for the next session (owner 2026-10-26: "We will do it all in a fresh session").

BACKLOG. Found by board round 5; reproduce first. Independent of plan approval.

## Size

S

## Detail

- **Files owned:** To be found by the reproduction.
- **Depends on:** None. The plan's `dropAnywhere` `dragSource` half (row 458) subsumes it; the clippy ban on the setters is in row 444.
- **Proof:** A Playwright spec that fails before the fix and passes after; run on the web build and, where the bug is renderer-dependent, the desktop-cadence build of [row 442](./442-desktop-cadence-harness-and-desktop-pass.md).
- **Hours (inferred):** not estimated (size guess S).

## Analysis (lane 2026-11-06; inferred from sources, nothing measured: no desktop build here)

- Inferred: on desktop the page's own `dragstart`/`dragover` run natively and Dioxus serializes the event to Rust
  afterwards, so the no-op setters (`dioxus-html-0.7.9/src/data_transfer.rs`) change nothing on the page. Only
  `prevent_default()` on `dragover`/`drop` takes effect, and the reorder relies on that, so it still works.
- What is lost is cosmetic: the effect cursor (`dropEffect`/`effectAllowed` stay at the engine default, which can show a
  copy cursor) and the drag-image/ghost control. The Firefox payload (`set_data("text/html", "")`) is not needed on the
  desktop engines (WebKitGTK, WKWebView, WebView2 are not Firefox). Not measured.
- No existing bridge op sets `dataTransfer` (`primitives/src/bridge/` has no drag verb; `dropAnywhere`'s `dragSource`
  half is planned, row 458). Setting these page-side needs a `dragstart` behaviour with a TS module and vectors, i.e. new
  JS, which this row may not add. Stopped; the construction is the row 458 `dragSource` behaviour on the list/row host
  (`effectAllowed=move`, `dropEffect=move`, payload `text/html`), after which the three rsx setter sites are deleted.
- Sites found beyond the row: `preview/src/components/virtual_list/variants/reorder/mod.rs:146-149,165`.
