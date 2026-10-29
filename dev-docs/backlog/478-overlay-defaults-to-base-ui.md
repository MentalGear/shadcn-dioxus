---
id: 478
title: "Overlay and control defaults follow Base UI (menu check items, Combobox keys, Tooltip, NavigationMenu timing, Select, carousel RTL)"
status: "PARTIAL. Owner decided 2026-10-28: Select defaults to item-aligned, as both shadcn variants do.…"
size: "S each"
area: overlays
updated: 2026-10-28
---

# Row 478: Overlay and control defaults follow Base UI

## Item

Where shadcn's `base-nova` style (Base UI) and this library differ in a default, and ours matches neither upstream or has a wrong comment, move to the Base UI behaviour. Source: [`radix-vs-base-ui-2026-10-28.md`](../research/radix-vs-base-ui-2026-10-28.md), appendix A (overlays) and C (carousel).

## Status / trigger

PARTIAL. Owner decided 2026-10-28: Select defaults to item-aligned, as both shadcn variants do. From the scan of 2026-10-28. Each item below is independent and S; a lane may take any subset.

**Done on `lane/overlay-defaults-478a` (2026-10-28; item 4 deferred), one commit per item, each with a spec assertion:**
- Item 1: `close_on_select` defaults to `false` for checkbox and radio items of all three hosts (`menu_item.rs`); the wrong "default of Base UI" sentence is fixed. The `checkboxes` and `radio_group` demos drop their explicit `false`; the demos that showed the closing default (the three `rtl` variants, context-menu `main`, the "Minimap" items) set `close_on_select: true`. The `close_on_select` doc comments in `context_menu.rs` still describe the Radix default (that file belonged to another lane); fix them with the next edit to it.
- Item 2: Home/End are left to the text field (caret) and Escape on a closed list clears the selection (every chip of a multiple) when one is set, stopping the event only then. Base UI also sets the input value to `""` on that Escape; ours shows no label once the selection is gone.
- Item 3: Tooltip and HoverCard open on `pointerenter` and ignore `touch`; a press on a tooltip trigger closes it and the focus the press moves onto the trigger does not reopen it (Base UI `useFocus` `visibleOnly`); the side offset is 4px (`--dx-tooltip-side-offset` slot in `tooltip/style.css`, out-ranking the engine's literal 8px in `top_layer/anchor.css`, plus the JS fallback's gap argument). HoverCard still opens on any focus, including the focus a tap moves onto its trigger (Base UI opens on focus-visible only): not in this row.
- Item 4 (deferred): Base UI's 50/50 NavigationMenu and 100/0 submenu delays are designed together with its `safePolygon` safe triangle, which this crate does not have, so taking the delays alone would make the menus worse; they stay at 150/300 and 200/200 until a safe-triangle exists.
- Item 6: `carousel_key_intent` takes no direction. Row 13 lists no carousel key-flip and `oracle/tier3-radix/rtl.spec.ts` has no carousel rule, so nothing recorded conflicts.

**Done on `lane/select-defaults-478b` (2026-10-29; item 5):**
- Item 5, Select: `SelectPosition::ItemAligned` is the default (primitives and the themed wrapper); a Select opened by touch (a tap on the trigger) is placed as popper whatever `position` says (Base UI `SelectPositioner`: `alignItemWithTrigger` is off for `openMethod === 'touch'`), tracked in `SelectContext::touch_open` and reset when the list unmounts. `roving_loop` defaults to `false` (Select, SelectMulti, the wrapper). A mouse press on the trigger opens on `pointerdown` (its `click` is ignored); touch and pen open on `click`, so a scroll that starts on the trigger does not open it (Base UI's `useClick` `event: 'mousedown'`, whose compatibility `mousedown` a touch scroll never produces). The list is a `popover="manual"` now: an `auto` popover opened between a `pointerdown` and its `pointerup` is hidden by that `pointerup` (the opening-gesture contract of `PopoverKind::Auto`), so the list owns its dismissal: Escape on the list or the trigger, `use_outside_dismiss` (a mouse press at once, a touch tap on `pointerup`), and the existing blur closes. Releasing the button over an option does not pick it (Base UI picks after a drag threshold; ours needs a press that started on the option).
- The scroll lock is gone from Select: the `scroll_lock` prop of `Select`/`SelectMulti` and the wrapper, the context field, the `ScrollLockGuard` mounts and the root's `ensure_scrollbar_gutter_baseline`. The page scrolls while the list is open and row 477's placement keeps the panel attached. The `top_layer` fixture's "Select scroll_lock opt-out" section and the Select cases of `scroll-lock.spec.ts` are removed with it.
- Close-on-scroll: HoverCard and NavigationMenu no longer close on scroll; `use_close_on_scroll` stays for the Tooltip alone (Radix's tooltip closes on scroll), so it has one user.
- Specs: `select.spec.ts` "defaults (row 478)" (item-aligned default, touch popper, touch scroll from the trigger, no wrap, open on press, outside press and Escape, page scroll with the panel attached in both placements); `close-on-scroll.spec.ts` and `hover-card.spec.ts` (stay open); `select-scroll-buttons.spec.ts` (popper variant). Demos: the `popper` variant is new, `aligned` is the default's demo.

## Size

S each

## Detail

1. **Menu checkbox and radio items stay open on click.** Base UI `closeOnClick = false` for both (`MenuCheckboxItem.mjs`, `MenuRadioItem.mjs`); Radix closes. Ours defaults `close_on_select = true`. Change the default to keep-open for checkable items (API-visible: check the demos and specs), and fix the wrong comment at `primitives/src/menu_item.rs:64-65`, which says closing is "the default of Base UI".
2. **Combobox.** Home and End always move the caret (Base UI `ComboboxInput.mjs`); ours jumps the highlight when the list is open (`combobox/components/input.rs:62-71`). Escape on a closed list clears the input and the selection; ours only closes.
3. **Tooltip.** Closes on press (`closeOnClick`); ignores a hover opened by touch (a pointer-type gate: on iOS a tap fires compatibility `mouseenter`, inferred, not run); side offset 4 as in `base-nova` (ours 8). HoverCard gets the same pointer-type gate.
4. **NavigationMenu delays.** Base UI 50 ms open and 50 ms close, submenu 100/0 (ours 150/300 and 200/200, `hover_intent.rs`, `menu_sub.rs`).
5. **Select (done on `lane/select-defaults-478b`, see Status).** Does not wrap (neither upstream does; ours defaults `roving_loop` true) and opens on pointerdown (both upstreams open on press; ours on click). The default position is item-aligned (owner decision 2026-10-28, below).
6. **Carousel RTL.** The arrow keys do not swap under RTL: shadcn with embla never swaps (`primitives/src/carousel/mod.rs:72-76` follows the Radix convention).

**Owner decision (Select default).** Both shadcn variants default to item-aligned (`base-nova` `alignItemWithTrigger=true`, `new-york-v4` `position="item-aligned"`); ours defaults to popper. The default becomes `SelectPosition::ItemAligned` (primitives/src/select/context.rs, currently `Popper`), and the switch lands after row 477, which places item-aligned on document coordinates. Base UI falls back to the popper position for a touch-opened Select; this row follows it unless the owner says otherwise (the device lab placed item-aligned on iOS with 0 px error, so either works). Related: [row 268](./268-select-scroll-buttons-and-item-aligned-position.md).

- **Files owned:** per item (`menu_item.rs`, `dropdown_menu.rs`, `context_menu.rs`, `menubar.rs`, `combobox/components/input.rs`, `tooltip.rs`, `hover_card.rs`, `hover_intent.rs`, `menu_sub.rs`, `select/`, `carousel/mod.rs`) and their specs. Menu items overlap with [row 435](./435-one-menu-family-behind-three-hosts.md): item 1 is cheaper after it, but is not blocked by it.
- **Proof:** a spec per changed default (checkable item keeps the menu open; Combobox Home/End and Escape; tooltip closes on press and ignores a touch-opened hover; Select does not wrap; carousel RTL arrows).
