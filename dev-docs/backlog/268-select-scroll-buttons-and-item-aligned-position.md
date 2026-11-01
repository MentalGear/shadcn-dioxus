---
id: 268
title: "Select scroll buttons and position=item-aligned"
status: "Only the native (Blitz) arm is left: position=item-aligned emits data-position there and nothing…"
size: "S (native arm only)"
area: select
updated: 2026-10-31
---

# Row 268: Select scroll buttons and position=item-aligned

## Item

Two Select parts shadcn has and this Select does not. Split out of row 188 by the features batch parts lane.

## Status / trigger

**Only the native (Blitz) arm is left:** `position=item-aligned` emits `data-position` there and nothing positions the list. The web arm is complete: scroll buttons and item-aligned landed (3bbf20c5, 3165d011, specs 11/11 green on a debug SSG build, 2026-10-23), and the no-selection flip above/below landed in batch 14 (7a3a571d, 4c2bcd7d; with a selected option the list clamps to the viewport and scrolls, which is Radix's behaviour).

## Size

S (native arm only)

## Detail

**2026-10-28.** The web arm's item-aligned placement moves to row 477's rule (document coordinates, selected option over the trigger, clamped to the visible region; measured 0 px alignment on the iPhone). Whether item-aligned becomes the default (both shadcn variants; Base UI falls back to popper for touch opens) is an owner question, recorded in [row 478](./478-overlay-defaults-to-base-ui.md).

**Scroll buttons (S-M).** Styled `SelectScrollUpButton` / `SelectScrollDownButton`, sticky inside the list, shown only while the list can scroll in that direction. The scroll listener is passive and goes through `primitives/src/js_listener.rs` `listen` (gate: `check-js-listeners.sh`); hover auto-scroll is motion and is gated by `use_motion` (`check-motion-gating.sh`). Needs a select spec: buttons appear at the right end, hovering scrolls, leaving stops, reduced motion and hidden-tab pause.

**`position="item-aligned"` (M-L).** A primitive change that aligns the selected option over the trigger (Radix's default mode). `primitives/src/select/context.rs` documents it as deliberately unimplemented (popper positioning only). Needs an anchor change plus the offset math, including RTL, collision with the viewport edges and the scroll-into-view of the selected option before measuring.

Related: [row 188](done/188-demo-parity-second-audit-the-owner-s-per.md).


## Progress

The web arm is done: the scroll buttons (`select/components/scroll_button.rs`, `SelectScrollUpButton`/`SelectScrollDownButton`, hover auto-scroll gated by `use_motion`, paused in a hidden tab; the buttons set their own `font-size` token so a small host does not shrink them) and `position="item-aligned"` (`SelectPosition`, `use_item_aligned` in `select/components/list.rs`, now on row 477's placement helpers; the default since row 478 item 5) have landed with their specs in `playwright/select-scroll-buttons.spec.ts` (11/11 green on a debug SSG build, 2026-10-23, and green in every release suite since) and the no-selection flip above/below. What remains is the native (Blitz) arm (S): it emits `data-position` and nothing positions the list.
