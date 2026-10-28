---
id: 478
title: "Overlay and control defaults follow Base UI (menu check items, Combobox keys, Tooltip, NavigationMenu timing, Select, carousel RTL)"
status: "BACKLOG. Owner decided 2026-10-28: Select defaults to item-aligned, as both shadcn variants do.…"
size: "S each"
area: overlays
updated: 2026-10-28
---

# Row 478: Overlay and control defaults follow Base UI

## Item

Where shadcn's `base-nova` style (Base UI) and this library differ in a default, and ours matches neither upstream or has a wrong comment, move to the Base UI behaviour. Source: [`radix-vs-base-ui-2026-10-28.md`](../research/radix-vs-base-ui-2026-10-28.md), appendix A (overlays) and C (carousel).

## Status / trigger

BACKLOG. Owner decided 2026-10-28: Select defaults to item-aligned, as both shadcn variants do. From the scan of 2026-10-28. Each item below is independent and S; a lane may take any subset.

## Size

S each

## Detail

1. **Menu checkbox and radio items stay open on click.** Base UI `closeOnClick = false` for both (`MenuCheckboxItem.mjs`, `MenuRadioItem.mjs`); Radix closes. Ours defaults `close_on_select = true`. Change the default to keep-open for checkable items (API-visible: check the demos and specs), and fix the wrong comment at `primitives/src/menu_item.rs:64-65`, which says closing is "the default of Base UI".
2. **Combobox.** Home and End always move the caret (Base UI `ComboboxInput.mjs`); ours jumps the highlight when the list is open (`combobox/components/input.rs:62-71`). Escape on a closed list clears the input and the selection; ours only closes.
3. **Tooltip.** Closes on press (`closeOnClick`); ignores a hover opened by touch (a pointer-type gate: on iOS a tap fires compatibility `mouseenter`, inferred, not run); side offset 4 as in `base-nova` (ours 8). HoverCard gets the same pointer-type gate.
4. **NavigationMenu delays.** Base UI 50 ms open and 50 ms close, submenu 100/0 (ours 150/300 and 200/200, `hover_intent.rs`, `menu_sub.rs`).
5. **Select.** Does not wrap (neither upstream does; ours defaults `roving_loop` true) and opens on pointerdown (both upstreams open on press; ours on click). The default position is item-aligned (owner decision 2026-10-28, below).
6. **Carousel RTL.** The arrow keys do not swap under RTL: shadcn with embla never swaps (`primitives/src/carousel/mod.rs:72-76` follows the Radix convention).

**Owner decision (Select default).** Both shadcn variants default to item-aligned (`base-nova` `alignItemWithTrigger=true`, `new-york-v4` `position="item-aligned"`); ours defaults to popper. The default becomes `SelectPosition::ItemAligned` (primitives/src/select/context.rs, currently `Popper`), and the switch lands after row 477, which places item-aligned on document coordinates. Base UI falls back to the popper position for a touch-opened Select; this row follows it unless the owner says otherwise (the device lab placed item-aligned on iOS with 0 px error, so either works). Related: [row 268](./268-select-scroll-buttons-and-item-aligned-position.md).

- **Files owned:** per item (`menu_item.rs`, `dropdown_menu.rs`, `context_menu.rs`, `menubar.rs`, `combobox/components/input.rs`, `tooltip.rs`, `hover_card.rs`, `hover_intent.rs`, `menu_sub.rs`, `select/`, `carousel/mod.rs`) and their specs. Menu items overlap with [row 435](./435-one-menu-family-behind-three-hosts.md): item 1 is cheaper after it, but is not blocked by it.
- **Proof:** a spec per changed default (checkable item keeps the menu open; Combobox Home/End and Escape; tooltip closes on press and ignores a touch-opened hover; Select does not wrap; carousel RTL arrows).
