---
id: 268
title: "Select scroll buttons and position=item-aligned"
status: "Only the native (Blitz) arm is left: position=item-aligned emits data-position there and nothing…"
size: "S (native arm only)"
area: select
updated: 2026-10-24
---

# Row 268: Select scroll buttons and position=item-aligned

## Item

Two Select parts shadcn has and this Select does not. Split out of row 188 by the features batch parts lane.

## Status / trigger

**Only the native (Blitz) arm is left:** `position=item-aligned` emits `data-position` there and nothing positions the list. The web arm is complete: scroll buttons and item-aligned landed (3bbf20c5, 3165d011, specs 11/11 green on a debug SSG build, 2026-10-23), and the no-selection flip above/below landed in batch 14 (7a3a571d, 4c2bcd7d; with a selected option the list clamps to the viewport and scrolls, which is Radix's behaviour).

## Size

S (native arm only)

## Detail

**Scroll buttons (S-M).** Styled `SelectScrollUpButton` / `SelectScrollDownButton`, sticky inside the list, shown only while the list can scroll in that direction. The scroll listener is passive and goes through `primitives/src/js_listener.rs` `listen` (gate: `check-js-listeners.sh`); hover auto-scroll is motion and is gated by `use_motion` (`check-motion-gating.sh`). Needs a select spec: buttons appear at the right end, hovering scrolls, leaving stops, reduced motion and hidden-tab pause.

**`position="item-aligned"` (M-L).** A primitive change that aligns the selected option over the trigger (Radix's default mode). `primitives/src/select/context.rs` documents it as deliberately unimplemented (popper positioning only). Needs an anchor change plus the offset math, including RTL, collision with the viewport edges and the scroll-into-view of the selected option before measuring.

Related: [row 188](188-demo-parity-second-audit-the-owner-s-per.md).


## Progress (2026-10-20, claude/batch-b)

Scroll buttons implemented (`primitives/src/select/components/scroll_button.rs`, `SelectScrollUpButton`/`SelectScrollDownButton`, wired into themed `Select`/`SelectMulti`, `scroll` demo variant, `playwright/select-scroll-buttons.spec.ts`). `cargo check --workspace --all-targets` is green; the spec has NOT yet been run (the SSG rebuild was refused for disk space), so the row stays OPEN. `position="item-aligned"` is not started.

## Progress (2026-10-21, lane/select-scroll)

`playwright/popup-style-isolation.spec.ts:258` failed for the cases chart, data_table, message_scroller, area_chart, pie_chart and site-charts: `.dx-select-scroll-button` and its `svg`/`path` computed `font-size: 14px` when the Select trigger sat in a small host (`dx-card-action`, `dx-card-footer`, `dx-data-table-pagination-size`) against 16px for the same popup alone. The scroll buttons set no `font-size`, so they inherited the host's through the popup's DOM ancestry. Fix: `.dx-select-scroll-button { font-size: var(--dx-text-sm) }`, the token the options set, in `preview/src/components/select/style.css`; the svg and path inherit it from the button. The site-home toast-region case of the same spec is row 218's and stays red.

## Progress (2026-10-22, claude/select-parts)

`position="item-aligned"` implemented. `SelectPosition { Popper (default), ItemAligned }` is a prop of `Select`/`SelectMulti` (primitive and themed), carried in `SelectContext::position`, emitted as `data-position` on the listbox. For item-aligned the popper engine sits out (`use_anchor_position_fallback` gets `open && popper`) and `use_item_aligned` (`list.rs`, web arm) installs one script through `JsListeners`: it waits for `:popover-open`, reads untransformed layout (`offsetTop`/`offsetHeight`; the pop-in animates `scale`), puts the selected option's centre on the trigger's centre, clamps the panel to the `position: fixed; inset: 0` probe box (not `clientWidth`, which includes the scrollbar gutter) with an 8px margin, scrolls the list when the panel cannot move further, publishes `--dx-anchor-width`/`--dx-available-height`, mirrors for RTL, re-places on resize, and sets `data-placed`. CSS: `.dx-select-list[data-position="item-aligned"]` in `select/style.css`. Demo variant `aligned`; specs in `playwright/select-scroll-buttons.spec.ts` (selected option on trigger line and list in viewport; trigger at the viewport edge keeps the list inside and the scroll buttons correct; selecting works and popper stays unchanged). Left open: the browser run of those specs and the scroll-button specs, native (Blitz) arm only emits the attribute, no collision flip of the whole list above/below.

## Progress (2026-10-23, claude/select-scroll-run)

First browser run of `playwright/select-scroll-buttons.spec.ts` on a debug SSG build of main `bcb4a291`: 9/9 green with no code change (scroll buttons and item-aligned). Added the two cases the Detail asks for: a hidden tab (`visibilityState` overridden, as `motion-gating.spec.ts` does) pauses the hover auto-scroll (`data-dx-motion="paused"`, scroll holds) and a visible tab resumes it; under `prefers-reduced-motion: reduce` the buttons still show and hovering still scrolls, because the hover scroll is the user's pointer driving the list (like holding an arrow key), not decorative motion. 11/11, 33/33 with `--repeat-each=3`; `select.spec.ts` + `popup-style-isolation.spec.ts` 79/79 on the same build.

