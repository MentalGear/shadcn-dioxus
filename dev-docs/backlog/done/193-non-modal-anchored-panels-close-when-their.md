---
id: 193
title: "Non-modal anchored panels close when their anchor leaves the viewport (one shared helper)"
status: "LANDED 2026-10-14 (overlays lane 51a6aa8, 927fb72; plan item 7; with row 210)"
size: "M"
area: overlays
updated: 2026-10-14
---

# Row 193: Non-modal anchored panels close when their anchor leaves the viewport (one shared helper)

## Item

Non-modal anchored panels close when their anchor leaves the viewport (one shared helper)

## Status / trigger

**LANDED 2026-10-14 (overlays lane 51a6aa8, 927fb72; plan item 7; with row 210)**

## Size

M

## Detail

The owner found an open Combobox list staying on screen after its input scrolled out of view. Same class on every anchored panel that does not lock scroll: Combobox (`popover="manual"`, own dismissal), Popover (non-modal since row 169), HoverCard, Tooltip, NavigationMenu, DatePicker/ColorPicker popovers, and Select/DropdownMenu/ContextMenu/Menubar when `scroll_lock` is off. **Decision:** do not lock page scroll for these (a combobox is a text field; mobile keyboards move the page; Chrome's native `<select>`/`<datalist>` close on scroll rather than lock); modal surfaces (Dialog, AlertDialog, Sheet, Drawer) and menus/pickers (Select, DropdownMenu, ContextMenu, Menubar -- Radix `RemoveScroll`, macOS NSPopUpButton) keep `ScrollLockGuard`. **Construction:** one helper in the top-layer code (IntersectionObserver on the anchor through `js_listener`-style scoped cleanup) that closes the panel when the anchor is fully outside the viewport (Tooltip/HoverCard: close on any scroll), applied by every non-locking anchored primitive; a Playwright spec per surface (open, scroll anchor out, closed; scroll a little, still open). Radix only offers `hideWhenDetached` (hides, stays open). **Landed 2026-10-14:** `use_close_on_scroll`, next to `viewportMetrics()` in `top_layer.rs`: a passive capture scroll listener via `js_listener` plus an IntersectionObserver on the anchor; `visualViewport` events never close (they open a 300 ms quiet window, as do focus changes and key presses inside the panel or anchor; scrolls inside the panel or anchor are ignored). Policies: any scroll closes Tooltip, HoverCard, Menubar, NavigationMenu, and Select/DropdownMenu/ContextMenu when unlocked; anchor-leaves-viewport closes Popover (so DatePicker and ColorPicker) and Combobox (a typed query plus a phone keyboard scrolling the page would close it on tap under "any scroll"). `close-on-scroll.spec` 26/26; existing overlay specs 475 passed vs 474 baseline. Behaviour changes: Menubar now closes on scroll (Rule 9 updated); a panel whose anchor scrolled out stays beside it (off-screen) instead of being dragged on screen. **Open:** `navbar.rs` is not wired (outside the lane's file set); the unlocked DropdownMenu/ContextMenu/Select paths are wired but no demo opens them unlocked, so there is no spec case.
