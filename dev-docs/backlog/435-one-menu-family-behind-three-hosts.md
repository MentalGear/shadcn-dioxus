---
id: 435
title: "One menu-family implementation behind three hosts (DropdownMenu, ContextMenu, Menubar)"
status: "BACKLOG, prioritised. Owner approved 2026-10-25 (row 365 sweep, P10); owner 2026-10-31: finish the…"
size: "L"
area: primitives
updated: 2026-10-31
---

# Row 435: One menu-family implementation behind three hosts

## Item

DropdownMenu, ContextMenu and Menubar should behave the same wherever APG says they do (item selection, focus, close-on-select, submenus), but their item code is copied per host and has drifted: a plain item's `value` has three types, selection is committed on `onclick` in one and on `pointerup` in another, and typeahead text differs. Construction, in this order: (1) one shared submenu for Dropdown and Context; (2) one value type, one selection model (`SelectOn::Click | SelectOn::PressRelease`) and one typeahead rule behind a three-host parity spec; (3) one `MenuPanel` / `MenuItem` / `MenuSub` family that reads a small `MenuHostContext`, each host keeping only its trigger, root and positioning. A fix to an item then lands once.

## Status / trigger

BACKLOG, prioritised. Owner approved 2026-10-25 (row 365 sweep, P10); owner 2026-10-31: finish the single overlay engine (this row and [row 487](./487-one-dismiss-core-escape-outside-dismiss-and-focus-return.md)) before the Rust ports, so the overlay port ([row 456](./456-port-3-ov-overlay-core-escape-dismiss-open-focus.md)) ports one menu family and one dismiss core instead of three and twelve.

## Size

L

## Detail

Source: [final sweep note, P10](../research/final-sweep-2026-10-25.md); state re-measured 2026-10-31 on `248f35cb` after rows 477-479 (PR #156). The riskiest item: it needs the cross-component spec run on a release build, not a code read.

- **Sizes (2026-10-31):** `primitives/src/dropdown_menu.rs` 2,371 lines, `context_menu.rs` 2,358, `menubar.rs` 1,569; the shared files are `menu_root.rs` 173, `menu_item.rs` 914 and `menu_sub.rs` 441. Dropdown and context share about 504 normalised lines (host name normalised); dropdown and menubar about 202.
- **Already shared:** `menu_root.rs`, `menu_item.rs` (checkable, separator, group, shortcut) and `menu_sub.rs` for the three hosts (row 53; `menu_root.rs:1-60` records the keep-separate decision), placement (`top_layer::use_anchor_position_fallback`; `use_point_anchor_placement` for ContextMenu), top-layer sync (`use_popover_sync`) and `use_outside_dismiss` (`lib.rs:391`).
- **Value types (verified):** dropdown `ReadSignal<T>` (generic `DropdownMenuItem<T>`), context `ReadSignal<String>`, menubar a plain `String` cloned per closure.
- **Selection:** `DropdownMenuItem` selects on `onclick` and does not clear the focus collection. `ContextMenuItem` and the Menubar item record `pointerdown` and commit on `pointerup` (the long-press guard) and call `clear_focus()` before `set_open(false)`. Whether the missing `clear_focus` in Dropdown is a bug is inferred, not tested.
- **Typeahead:** Dropdown uses `selection::option_text_value(&value, text_value, "DropdownMenuItem")`; Context and Menubar use `text_value.unwrap_or(value)`.
- **Per-host copies:** each host has its own `*_checkable_item` (about 90-110 lines each). The Dropdown and Context submenu code is near-identical: `SubTrigger` 52/53 lines, `SubContentRendered` web arm 59/61 and `Sub` 25/26 normalised lines, about 140 identical logic lines (about 450 raw), doubled on the native arm.
- **Step 1, the submenu (S-M, lands first):** merge the identical Dropdown/Context submenu lines into one shared submenu in `menu_sub.rs` that both hosts call; Menubar joins when its differences are named. It does not depend on the value-type or selection work, and it is the cheapest cut of the duplication.
- **Step 2, one model (M):** one value type, `SelectOn::Click | SelectOn::PressRelease` as an explicit per-host choice (touch and the long-press guard decide it), one typeahead rule, one `clear_focus` rule; the three `*_checkable_item` copies collapse into the one in `menu_item.rs`. Lands behind the three-host parity spec of the Gate below, written first.
- **Step 3, the family (M-L):** one `MenuPanel` / `MenuItem` / `MenuSub` family reading a `MenuHostContext`; each host keeps its trigger, root and positioning.
- **Subsumes:** the three `value` types, the two selection models, the typeahead rule, the checkable-item copies, and the web/native arm duplication of the shared parts. **Does not subsume:** `Navbar` and `NavigationMenu` (row 53 scoped them out; measured overlap 24-29%), positioning (`top_layer` keeps one call per host), the host triggers, and Escape, outside dismiss and focus return, which row 487 unifies across every overlay (menu hosts included).
- **Reverses row 53.** Row 53 decided not to build one shared menu component. The owner approved this proposal on 2026-10-25 ("All ten"), so that keep-separate decision is reversed.
- **Independent of** [row 477](./477-anchored-overlays-placed-in-document-coordinates.md) (the placement call per host is unchanged). The Base UI defaults of [row 478](./478-overlay-defaults-to-base-ui.md) (checkable items keep the menu open) have landed in `menu_item.rs`, so the shared item already carries them.
- **Gate:** one parity spec iterating the three hosts (click, keyboard and touch-tap selection; close-on-select; typeahead on a valued item; checkable items). A line-overlap check is not worth a gate.
