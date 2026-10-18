---
id: 140
title: "Menubar: a pointer selection leaves DOM focus on <body>; the keyboard path restores it to the trigger"
status: "LANDED 2026-10-05 (round 3) -- recorded 2026-10-04 from the checkable-item lane…"
size: "S"
area: menus
updated: 2026-10-05
---

# Row 140: Menubar: a pointer selection leaves DOM focus on <body>; the keyboard path restores it to the trigger

## Item

Menubar: a pointer selection leaves DOM focus on `<body>`; the keyboard path restores it to the trigger

## Status / trigger

**LANDED 2026-10-05 (round 3)** -- recorded 2026-10-04 from the checkable-item lane (`playwright/menubar.spec.ts`, "by default (Radix onSelect default) ... by keyboard focus returns to its trigger"); deliberately not asserted there

## Size

S

## Detail

Keyboard `Enter` on a Menubar item closes the menu and hands focus back to the menu's own trigger (APG menubar rule; asserted by that spec). A pointer selection does not: `MenubarItem` commits on `pointerdown` (`primitives/src/menubar.rs`, the `MenubarMenu` roving-focus collection's doc), the menu then unmounts, and the click that follows focuses an item that is being removed, so focus ends on `<body>` (as reported by the lane). The spec's comment records that this is the plain `MenubarItem`'s behaviour and that the new `MenubarCheckboxItem`/`MenubarRadioItem` inherit it, so the pointer half is not asserted. **Next:** reproduce by execution first (click an item, read `document.activeElement`; also on DropdownMenu and ContextMenu, whose pointer paths are unverified here), then restore focus to the trigger on a pointer close unless the click landed outside the menu bar (Radix `onCloseAutoFocus` semantics), and add the pointer case to `oracle-focus-restore.spec.ts` red-first. **Landed 2026-10-05 (round 3):** reproduced first, as asked. Only Menubar had the gap (its items commit on `pointerdown` and close the menu right there, so the mousedown that follows focused the item being unmounted); DropdownMenu and ContextMenu commit on click/pointerup and were already fine. **Construction:** `menu_root::keep_focus_on_pointerdown` (`primitives/src/menu_root.rs`) is the single pointerdown guard (it cancels the default so the refocus the commit made is the last focus change) for every menu content wrapper (Menubar, DropdownMenu and the other hosts), so the class cannot reopen in one host only; the `context_menu` copies were migrated onto it later in the round. **Proof:** pointer cases added to `playwright/oracle-focus-restore.spec.ts` for Menubar, DropdownMenu and ContextMenu ("returns focus to its trigger after a pointer selection"); they pass.
