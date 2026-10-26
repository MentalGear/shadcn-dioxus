---
id: 435
title: "One menu-family implementation behind three hosts (DropdownMenu, ContextMenu, Menubar)"
status: "BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P10). Ordered before the JavaScript plan (owner…"
size: "L"
area: primitives
updated: 2026-10-26
---

# Row 435: One menu-family implementation behind three hosts

## Item

DropdownMenu, ContextMenu and Menubar should behave the same wherever APG says they do (item selection, focus, close-on-select, submenus), but their item code is copied per host and has drifted: a plain item's `value` has three types, selection is committed on `onclick` in one and on `pointerup` in another, and typeahead text differs. Construction: one `MenuPanel` / `MenuItem` / `MenuSub` family reads a small `MenuHostContext`, and each host keeps only its trigger, root and positioning. Selection becomes one explicit `SelectOn::Click | SelectOn::PressRelease`. A fix to an item then lands once.

## Status / trigger

BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P10). Ordered before the JavaScript plan (owner 2026-10-26: rows that can land before it go first), so the overlay port (row 456) ports one menu engine instead of three.

## Size

L

## Detail

Source: [final sweep note, P10](../research/final-sweep-2026-10-25.md). The riskiest item: it needs the cross-component spec run on a release build, not a code read.

- **Sizes:** `primitives/src/dropdown_menu.rs` 2,399 lines, `context_menu.rs` 2,378, `menubar.rs` 1,581. After normalising the host name, 477 non-comment lines longer than 12 characters are shared between dropdown and context (47% of the smaller file); 202 between dropdown and menubar.
- **Value types (verified):** `dropdown_menu.rs:962` `ReadSignal<T>` (generic `DropdownMenuItem<T>`); `context_menu.rs:1023` `ReadSignal<String>`; `menubar.rs:1151` a plain `String` cloned per closure.
- **Selection:** `DropdownMenuItem` selects on `onclick` (`dropdown_menu.rs:~1111-1135`) and does not clear the focus collection. `ContextMenuItem` records `pointerdown` and commits on `pointerup` (the long-press guard, `context_menu.rs:~1128-1160`) and calls `ctx.focus.clear_focus()` before `set_open(false)`. Whether the missing `clear_focus` is a bug is inferred, not tested.
- **Typeahead:** Dropdown uses `selection::option_text_value(&value, text_value, "DropdownMenuItem")`; Context and Menubar use `text_value.unwrap_or(value)`.
- **Prior step:** row 53 shared `menu_root.rs`, `menu_item.rs` (873 lines) and `menu_sub.rs` (442 lines); `menu_root.rs:1-60` records the keep-separate decision.
- **Subsumes:** the three `value` types, the two selection models, the typeahead rule, and the web/native arm duplication of the shared parts. **Does not subsume:** `Navbar` and `NavigationMenu` (row 53 scoped them out; measured overlap 24-29%), positioning (`top_layer` keeps one call per host), and the host triggers.
- **Reverses row 53.** Row 53 decided not to build one shared menu component. The owner approved this proposal on 2026-10-25 ("All ten"), so that keep-separate decision is reversed.
- **Sequence:** P10a lands first. It unifies only the item `value` typing and the selection model (S-M) and leaves the host structure alone, so it does not depend on the row-53 reversal. P10b, the full merge, follows it.
- **Gate:** one parity spec iterating the three hosts (click, keyboard and touch-tap selection; close-on-select; typeahead on a valued item). A line-overlap check is not worth a gate.
