---
id: 479
title: "Scroll-lock method: Base UI's overflow:hidden with scrollbar-gutter, and no lock for touch-opened anchored popups"
status: "BACKLOG. OWNER QUESTION open: Menubar modal (Detail). From the scan of 2026-10-28. Row 477 removes…"
size: "M"
area: overlays
updated: 2026-10-28
---

# Row 479: Scroll-lock method and the touch carve-out

## Item

Move the scroll lock to Base UI's method and its touch rule. Source: [`radix-vs-base-ui-2026-10-28.md`](../research/radix-vs-base-ui-2026-10-28.md), appendix A (cross-cutting 3).

## Status / trigger

BACKLOG. OWNER QUESTION open: Menubar modal (Detail). From the scan of 2026-10-28. Row 477 removes close-on-scroll and keeps the lock only where upstream locks; this row changes how the lock works.

## Size

M

## Detail

- **Method.** Base UI locks with `overflow: hidden` plus `scrollbar-gutter: stable` on the viewport scroller; an overlay-scrollbar or iOS branch only hides overflow (`@base-ui/utils` `useScrollLock.mjs`). Ours is react-remove-scroll style event interception (`primitives/src/scroll_lock.rs`), with no iOS momentum guard.
- **Touch carve-out.** Base UI does not lock for a touch-opened anchored popup unless the popup is nearly full width (`useAnchoredPopupScrollLock.mjs`), and not for a hover-opened menu. Ours locks Select and modal menus for touch too.
- **Defaults upstream:** Menu `modal` true, Select true, Menubar true, Popover false, Combobox false, Dialog true. Ours: Menubar unlocked (the Radix model).

**OWNER QUESTION: Menubar modal.** Base UI's Menubar is modal by default (scroll lock plus backdrop while a menu is open); Radix passes `modal: false`, and ours is unlocked. Adopt modal as the default, keep unlocked, or add the prop with the default left to the owner. Recorded in [`STATUS.md`](../STATUS.md), "Owner inputs pending".

- **Files owned:** `primitives/src/scroll_lock.rs` and the `ScrollLockGuard` call sites (dropdown, context menu, menubar, select, dialog).
- **Proof:** lock specs on a touch-emulated context (touch-opened Select and menu do not lock; mouse-opened do; a full-width popup locks), the dialog's lock unchanged, and a device check on the iPhone for momentum scroll behind a modal.
