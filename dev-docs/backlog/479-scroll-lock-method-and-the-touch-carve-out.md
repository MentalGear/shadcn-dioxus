---
id: 479
title: "Scroll-lock method: Base UI's overflow:hidden with scrollbar-gutter, and no lock for touch-opened anchored popups"
status: "BACKLOG. Owner decided 2026-10-28: Menubar is not modal (no scroll lock), and anchored overlays…"
size: "M"
area: overlays
updated: 2026-10-28
---

# Row 479: Scroll-lock method and the touch carve-out

## Item

Move the scroll lock to Base UI's method and its touch rule. Source: [`radix-vs-base-ui-2026-10-28.md`](../research/radix-vs-base-ui-2026-10-28.md), appendix A (cross-cutting 3).

## Status / trigger

BACKLOG. Owner decided 2026-10-28: Menubar is not modal (no scroll lock), and anchored overlays stay open and scrollable while the page scrolls; only ContextMenu (the owner's earlier request) and the modal surfaces (Dialog, AlertDialog, Sheet, Drawer) lock scroll. From the scan of 2026-10-28. Row 477 removes close-on-scroll and the lock from anchored overlays; this row changes how the remaining lock works.

## Size

M

## Detail

- **Method.** Base UI locks with `overflow: hidden` plus `scrollbar-gutter: stable` on the viewport scroller; an overlay-scrollbar or iOS branch only hides overflow (`@base-ui/utils` `useScrollLock.mjs`). Ours is react-remove-scroll style event interception (`primitives/src/scroll_lock.rs`), with no iOS momentum guard. The method applies now to the modal surfaces and ContextMenu only (owner decision 2026-10-28, below).
- **Touch carve-out.** Base UI does not lock for a touch-opened anchored popup unless the popup is nearly full width (`useAnchoredPopupScrollLock.mjs`), and not for a hover-opened menu. Ours locks Select and modal menus for touch too. Because anchored overlays no longer lock (owner decision 2026-10-28), the touch carve-out is moot for them; it bears only on ContextMenu, the one anchored overlay that still locks.
- **Defaults upstream:** Menu `modal` true, Select true, Menubar true, Popover false, Combobox false, Dialog true. Ours: Menubar unlocked (the Radix model).

**Owner decision: Menubar is not modal.** Base UI's Menubar is modal by default (scroll lock plus backdrop while a menu is open); Radix passes `modal: false`, and ours stays unlocked. Anchored overlays stay open and scrollable while the page scrolls: an open panel may not be fully in view, so the user must be able to scroll to it, and a Popover with inputs stays open rather than closing on scroll. Only ContextMenu and the modal surfaces (Dialog, AlertDialog, Sheet, Drawer) lock scroll.

- **Files owned:** `primitives/src/scroll_lock.rs` and the `ScrollLockGuard` call sites: keep the context menu and the modal surfaces; the dropdown, menubar and select call sites lose their lock (owner decision 2026-10-28).
- **Proof:** lock specs on a touch-emulated context (ContextMenu and the modal surfaces lock on touch and mouse; Select, Menubar, DropdownMenu and the other anchored overlays do not lock and stay scrollable while the page scrolls), the dialog's lock unchanged, and a device check on the iPhone for momentum scroll behind a modal.
