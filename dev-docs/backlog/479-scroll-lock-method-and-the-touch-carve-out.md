---
id: 479
title: "Scroll-lock method: Base UI's overflow:hidden with scrollbar-gutter, and no lock for touch-opened anchored popups"
status: "PARTIAL. Method, DropdownMenu and Select done (landed in PR #156, deployed). Remaining: remove…"
size: "M"
area: overlays
updated: 2026-10-31
---

# Row 479: Scroll-lock method and the touch carve-out

## Item

Move the scroll lock to Base UI's method and its touch rule. Source: [`radix-vs-base-ui-2026-10-28.md`](../research/radix-vs-base-ui-2026-10-28.md), appendix A (cross-cutting 3).

## Status / trigger

PARTIAL. Method, DropdownMenu and Select done (landed in PR #156, deployed). Remaining: remove `CONTEXT_MENU_SCROLL_BLOCK_JS` (a redundant second blocker beside the lock) and the owner's iPhone momentum check behind a modal. Owner decided 2026-10-28: Menubar is not modal (no scroll lock), and anchored overlays stay open and scrollable while the page scrolls; only ContextMenu (the owner's earlier request) and the modal surfaces (Dialog, AlertDialog, Sheet, Drawer) lock scroll. From the scan of 2026-10-28. Row 477 removes close-on-scroll and the lock from anchored overlays; this row changes how the remaining lock works.

**Done on `lane/scroll-lock-479` (2026-10-28):**
- Method: `scroll_lock.rs` locks with Base UI's three branches. Overlay scrollbars and iOS: `overflow: hidden` on the viewport scroller (`<html>` when it is a scroll container, else `<body>`). A classic scrollbar on an engine whose `scrollbar-gutter: stable` survives hidden overflow (the probe of Base UI's `supportsStableScrollbarGutter`; Firefox): the gutter plus `overflow: hidden`. A classic scrollbar on Chromium, which drops the gutter of a root with hidden overflow (measured on Chromium 153 under Xvfb, standards mode: `clientWidth` 1012 -> 1027): the root stays `overflow: scroll`, `<body>` is clipped to the visible viewport in px (not `100vw`, which already excludes the scrollbar once the root is `overflow: scroll`), and `window.scrollY` reads 0 while it is held. The exact previous inline values come back on release and the pre-lock scroll position is restored after the teardown. The wheel, touch and key listeners are gone (the lock script adds only a `resize` listener in the two classic-scrollbar branches), and `scroll_lock.rs`'s JS fell from 82 to 55 lines. The lock counter, the one-frame hand-over, the early scroll capture and the `__dxScrollLocked` marker are unchanged. Not ported from Base UI: the wait for an app-held lock to clear (`MutationObserver`) and its WebKit pinch-zoom skip.
- DropdownMenu takes no lock (the `modal` prop and its `ScrollLockGuard` are removed) and its page-scroll close is gone with it. Menubar had no lock code. ContextMenu keeps its lock; its `close_on_select` doc comments now describe the keep-open default of row 478.
- Spec: `oracle/tier3-radix/scroll-lock.spec.ts` pins the dropdown (wheel and touch pan move the page, the menu stays open, no lock marker), ContextMenu and Dialog on touch, the scroll position through a ContextMenu lock, and the inline styles before and after a lock cycle.
- Select's lock is gone (row 478 item 5: the `scroll_lock` prop, the context field and the `ScrollLockGuard` mounts were removed). Remaining: `CONTEXT_MENU_SCROLL_BLOCK_JS` (`context_menu.rs:38`, installed at `:773` and `:931`) blocks wheel and touch outside the menu in addition to the lock `ScrollLockGuard` already holds, so it is redundant and goes (the `scroll-lock.spec.ts` ContextMenu cases are its proof); and the iPhone check of a momentum fling behind a modal.


## Size

M

## Detail

- **Method.** Base UI locks with `overflow: hidden` plus `scrollbar-gutter: stable` on the viewport scroller; an overlay-scrollbar or iOS branch only hides overflow (`@base-ui/utils` `useScrollLock.mjs`). Ours is react-remove-scroll style event interception (`primitives/src/scroll_lock.rs`), with no iOS momentum guard. The method applies now to the modal surfaces and ContextMenu only (owner decision 2026-10-28, below).
- **Touch carve-out.** Base UI does not lock for a touch-opened anchored popup unless the popup is nearly full width (`useAnchoredPopupScrollLock.mjs`), and not for a hover-opened menu. Ours locks Select and modal menus for touch too. Because anchored overlays no longer lock (owner decision 2026-10-28), the touch carve-out is moot for them; it bears only on ContextMenu, the one anchored overlay that still locks.
- **Defaults upstream:** Menu `modal` true, Select true, Menubar true, Popover false, Combobox false, Dialog true. Ours: Menubar unlocked (the Radix model).

**Owner decision: Menubar is not modal.** Base UI's Menubar is modal by default (scroll lock plus backdrop while a menu is open); Radix passes `modal: false`, and ours stays unlocked. Anchored overlays stay open and scrollable while the page scrolls: an open panel may not be fully in view, so the user must be able to scroll to it, and a Popover with inputs stays open rather than closing on scroll. Only ContextMenu and the modal surfaces (Dialog, AlertDialog, Sheet, Drawer) lock scroll.

- **Files owned:** `primitives/src/scroll_lock.rs` and the `ScrollLockGuard` call sites: keep the context menu and the modal surfaces; the dropdown, menubar and select call sites lose their lock (owner decision 2026-10-28).
- **Proof:** lock specs on a touch-emulated context (ContextMenu and the modal surfaces lock on touch and mouse; Select, Menubar, DropdownMenu and the other anchored overlays do not lock and stay scrollable while the page scrolls), the dialog's lock unchanged, and a device check on the iPhone for momentum scroll behind a modal.
