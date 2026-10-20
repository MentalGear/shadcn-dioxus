---
id: 268
title: "Select scroll buttons and position=item-aligned"
status: "OPEN. Scroll buttons written and compiling, browser spec not yet run (2026-10-20); item-aligned not…"
size: "S-M (scroll buttons), M-L (item-aligned)"
area: select
updated: 2026-10-19
---

# Row 268: Select scroll buttons and position=item-aligned

## Item

Two Select parts shadcn has and this Select does not. Split out of row 188 by the features batch parts lane.

## Status / trigger

OPEN. Scroll buttons written and compiling, browser spec not yet run (2026-10-20); item-aligned not started.

## Size

S-M (scroll buttons), M-L (item-aligned)

## Detail

**Scroll buttons (S-M).** Styled `SelectScrollUpButton` / `SelectScrollDownButton`, sticky inside the list, shown only while the list can scroll in that direction. The scroll listener is passive and goes through `primitives/src/js_listener.rs` `listen` (gate: `check-js-listeners.sh`); hover auto-scroll is motion and is gated by `use_motion` (`check-motion-gating.sh`). Needs a select spec: buttons appear at the right end, hovering scrolls, leaving stops, reduced motion and hidden-tab pause.

**`position="item-aligned"` (M-L).** A primitive change that aligns the selected option over the trigger (Radix's default mode). `primitives/src/select/context.rs` documents it as deliberately unimplemented (popper positioning only). Needs an anchor change plus the offset math, including RTL, collision with the viewport edges and the scroll-into-view of the selected option before measuring.

Related: [row 188](188-demo-parity-second-audit-the-owner-s-per.md).


## Progress (2026-10-20, claude/batch-b)

Scroll buttons implemented (`primitives/src/select/components/scroll_button.rs`, `SelectScrollUpButton`/`SelectScrollDownButton`, wired into themed `Select`/`SelectMulti`, `scroll` demo variant, `playwright/select-scroll-buttons.spec.ts`). `cargo check --workspace --all-targets` is green; the spec has NOT yet been run (the SSG rebuild was refused for disk space), so the row stays OPEN. `position="item-aligned"` is not started.
