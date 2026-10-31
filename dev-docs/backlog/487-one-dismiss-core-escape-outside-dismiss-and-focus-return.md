---
id: 487
title: "One dismiss core: Escape, outside dismiss and focus return for every overlay"
status: "BACKLOG, prioritised with row 435 (owner 2026-10-31): the single overlay engine is finished before…"
size: "M-L"
area: overlays
updated: 2026-10-31
---

# Row 487: One dismiss core: Escape, outside dismiss and focus return for every overlay

## Item

Every overlay closes the same three ways (Escape, a press outside, focus handed back to the trigger), yet each does it by its own code: placement and top-layer sync are shared already, dismissal is not. Construction: one dismiss core that takes a per-overlay policy (which keys, which outside presses, whether focus returns, whether a nested layer owns the Escape), so a rule such as "a touch press outside is a tap, not a touch" lands once and reaches every overlay.

## Status / trigger

BACKLOG, prioritised with [row 435](./435-one-menu-family-behind-three-hosts.md) (owner 2026-10-31): the single overlay engine is finished before the Rust ports. [Row 456](./456-port-3-ov-overlay-core-escape-dismiss-open-focus.md) then ports one core instead of the scattered handlers.

## Size

M-L

## Detail

State measured 2026-10-31 on `248f35cb`, after rows 477-479 (PR #156).

- **Outside dismiss, today.** `use_outside_dismiss` (`primitives/src/lib.rs:391`) is used by Popover, DropdownMenu, ContextMenu, Menubar, Select and Dialog, and carries row 477's touch tap rule (a mouse press dismisses on `pointerdown`; a touch or pen press is a tap, `pointerup` within 10 px, `pointercancel` drops it). `use_sub_outside_dismiss` (`menu_sub.rs:420`) is the submenu variant. Not on it: Tooltip, HoverCard, Combobox, NavigationMenu (its own document `pointerdown` listener, `navigation_menu.rs:~947-955`, so the tap rule does not reach it: a scroll that starts outside a NavigationMenu panel closes it on a phone) and Navbar (native `popover=auto` light dismiss).
- **Escape, today.** Handled by hand in 12+ places: `tooltip.rs`, `hover_card.rs` (3), `navigation_menu.rs` (3), `navbar.rs`, `combobox/components/input.rs`, Select's trigger and list, `dropdown_menu.rs` (3), `context_menu.rs` (5), `menubar.rs`, `menu_sub.rs`, `menu_item.rs`, plus the dialog and popover roots (`use_global_escape_listener`, `lib.rs:334`). Which layer takes the Escape of a nested stack is decided per host.
- **Focus return, today.** `RefocusTrigger` (`lib.rs:455`; `Signal<String>` for Select, `TriggerHandle` for the menus) is wired per host, with the touch/pen "never refocus by script" rule living in the trait impls only.
- **Construction.** One core in `primitives/src/` (module name chosen by the lane) owning: the Escape key (layer-aware, so the innermost open layer closes first), the outside press (the tap rule of row 477, mouse versus touch/pen) with an "inside" predicate (root, trigger, nested layers) and a focus-leaves arm where the surface wants it, and the focus return (`RefocusTrigger`, modality rule). Each overlay passes a policy value; nothing else is per-host.
- **Brings under the tap rule:** NavigationMenu first (its own listener is deleted), then Tooltip, HoverCard and Combobox where an outside press applies (Tooltip and HoverCard are hover surfaces, so the policy may say "Escape and focus only"; the lane records the choice per surface). **Navbar** keeps its native light dismiss only if the lane shows it equivalent (Escape, outside mouse press, outside tap versus scroll, focus return); otherwise it joins the core.
- **Subsumes:** the 12+ Escape handlers, the NavigationMenu listener, `use_sub_outside_dismiss` as a separate function, the per-host `RefocusTrigger` wiring. **Does not subsume:** the scroll lock (row 459/479), open focus (`OPEN_FOCUS_JS`, row 456), hover intent and its delays (row 478 item 4), the menu item and submenu code (row 435; the two rows touch the same host files, so land 435 step 1 first or coordinate the files per lane).
- **Row 456 follows.** 456 ports JavaScript; it does not unify. After this row it ports one dismiss core (L3/L6/L7/L8/L9) instead of the scattered sites.
- **Proof:** one dismiss parity spec iterating every overlay (Popover, Dialog, AlertDialog, Sheet/Drawer where they dismiss, DropdownMenu, ContextMenu, Menubar, Select, Combobox, NavigationMenu, Navbar, Tooltip, HoverCard, DatePicker): Escape (nested stack closes innermost first), outside mouse press, outside tap versus scroll on touch, focus return to the trigger (and none after touch/pen). The existing `playwright/anchored-placement.spec.ts` touch cases stay green. A release-build run, since it touches every overlay.
