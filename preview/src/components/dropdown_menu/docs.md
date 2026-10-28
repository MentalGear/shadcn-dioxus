The DropdownMenu component is used to create a dropdown menu that can be triggered by a button click. It allows users to select an option from a list of items.

## Component Structure

```rust
// The dropdown menu component must wrap all dropdown items.
DropdownMenu {
    // The dropdown menu trigger is the button that will display the dropdown menu when clicked.
    DropdownMenuTrigger {
        // The content of the trigger to display inside the button.
        {children}
    }
    // The dropdown menu content contains all the items that will be displayed in the dropdown menu.
    DropdownMenuContent {
        // Each dropdown menu item represents an individual option in the dropdown menu. Arrow keys move through the items in document order.
        DropdownMenuItem {
            // The value of the item which will be passed to the on_select callback when the item is selected.
            value: "",
            on_select: |value: String| {
                // This callback is triggered when the item is selected.
                // The value parameter contains the value of the selected item.
            },
        }
    }
}
```

## Nested submenus

A `DropdownMenuContent` (or another submenu's own `DropdownMenuSubContent`)
may nest a submenu with `DropdownMenuSub`/`DropdownMenuSubTrigger`/
`DropdownMenuSubContent`/`DropdownMenuSubItem`, implementing the APG "Menu
and Menubar" pattern's submenu contract (ArrowRight/Enter/Space opens and
moves focus to the first item; Escape/ArrowLeft closes and returns focus to
the sub-trigger; the parent menu stays open). A submenu may hold plain
items but not a further nested submenu of its own (one level of nesting).

```rust
DropdownMenuContent {
    DropdownMenuSub {
        // The sub-trigger is itself an item of the enclosing menu, in its
        // keyboard order like `DropdownMenuItem`.
        DropdownMenuSubTrigger {
            "More tools"
        }
        // Only rendered while the submenu is open.
        DropdownMenuSubContent {
            DropdownMenuSubItem {
                value: "duplicate",
                on_select: |value: String| {
                    // Selecting a sub-item closes the entire menu tree,
                    // not just this submenu.
                },
            }
        }
    }
}
```

### Open on hover, or click only

By default hovering a `DropdownMenuSubTrigger` opens its submenu after a short
delay (200 ms) and leaving it closes the submenu again after a short grace
delay, so a diagonal move toward the submenu is not misread as leaving. Pass
`open_on_hover: false` to the `DropdownMenuSub` for **click activation**:

```rust
DropdownMenuSub {
    // Default is `true`.
    open_on_hover: false,
    DropdownMenuSubTrigger { "Export as" }
    DropdownMenuSubContent { /* ... */ }
}
```

The submenu then opens only on click, `Enter`, `Space` or the open arrow key,
and the pointer never opens *or* closes it: it closes on `Escape` / the close
arrow, an outside click, or focus moving elsewhere. Keyboard behaviour is the
same in both modes, and touch never hover-opens. The prop is named after Base
UI's `Menu.SubmenuTrigger` `openOnHover` (default `true`); it sits on the `Sub`
rather than the trigger because the trigger and the submenu content share the
hover timers. See the `click_only_submenu` variant.

## Checkbox and radio items

`DropdownMenuCheckboxItem` and `DropdownMenuRadioGroup` + `DropdownMenuRadioItem` are shadcn's
`DropdownMenuCheckboxItem`/`DropdownMenuRadioGroup`/`DropdownMenuRadioItem`. They render
`role="menuitemcheckbox"` / `role="menuitemradio"` with an always-present
`aria-checked`; radio items sit in a `role="group"` (name it with
`aria-label` or `aria-labelledby`). They work in a `DropdownMenuContent` and in a `DropdownMenuSubContent`.

```rust
DropdownMenuContent {
    DropdownMenuCheckboxItem {
        // Controlled: `checked` + `on_checked_change`. Uncontrolled:
        // `default_checked` -- but note the content unmounts when the menu
        // closes, so an uncontrolled item starts over on every open.
        checked: Some(show_status_bar()),
        on_checked_change: move |checked| show_status_bar.set(checked),
        // The menu stays open after a toggle by default (Base UI's
        // `closeOnClick = false`), so several items can be set in one visit.
        // `true` closes it on selection (Radix's `onSelect` behaviour).
        close_on_select: true,
        // A checkbox item has no `value`: set `text_value` to make it a
        // typeahead target.
        text_value: "Status Bar",
        "Status Bar"
    }
    DropdownMenuRadioGroup {
        aria_label: "Panel position",
        value: Some(position()),
        on_value_change: move |value| position.set(value),
        // A radio item's `value` is also its typeahead label.
        DropdownMenuRadioItem { value: "top".to_string(), "Top" }
        DropdownMenuRadioItem { value: "bottom".to_string(), "Bottom" }
    }
}
```

Click, `Enter` and `Space` all select. A disabled item cannot be selected and
is skipped by arrow keys and typeahead. The check is drawn at the inline end, in a slot the item reserves (the same `--dx-menu-item-indicator-*` construction as the sub-trigger chevron), so a long label never meets it and an unchecked item does not shift its label. See the `checkboxes` and
`radio_group` variants.

## Item parts: separator, group, shortcut, destructive, inset

Built once in `primitives/src/menu_item.rs` and shared with the other two menus, these are shadcn's `DropdownMenu` parts:

```rust
DropdownMenuContent {
    DropdownMenuGroup {
        DropdownMenuLabel { "My Account" }
        DropdownMenuItem::<String> {
            value: "profile".to_string(),
            // Items take `variant` and `inset`, like shadcn's `variant="destructive"` / `inset`.
            "Profile"
            // A text-only hint pushed to the inline end of the row; it binds no key.
            DropdownMenuShortcut { "⇧⌘P" }
        }
    }
    // A divider between groups: `role="separator"`, not focusable, skipped by arrow keys and typeahead.
    DropdownMenuSeparator {}
    DropdownMenuItem::<String> {
        value: "delete".to_string(),
        variant: MenuItemVariant::Destructive,
        "Delete"
    }
}
```

- **`variant`** (`MenuItemVariant::Default` or `Destructive`) paints the item in the theme's destructive text colour with a destructive-tinted highlight. The role stays `menuitem` and the item carries `data-variant`.
- **`inset`** reserves the leading indicator slot on an item (or `DropdownMenuLabel`) that draws none, so a menu that mixes plain and checkable items keeps its labels on one edge. It also exists on the checkbox and radio items and, where the menu has submenus, on the sub-trigger. The element carries `data-inset="true"`.
- **`DropdownMenuGroup`** is a `role="group"` with no behaviour; items inside it stay in the whole menu's keyboard order. Name it with `aria-label` when it has a visible heading.

See the `shortcuts` and `destructive` variants.

## Placement

`DropdownMenuContent` opens below its trigger, start-aligned with it, 4px away. `side` (`ContentSide::Top`, `Right`, `Bottom` or `Left`), `align` (`ContentAlign::Start`, `Center` or `End`) and `side_offset` (the gap, in px) move it. A side without room flips to the opposite one, and the side the menu was actually placed on is on the content as `data-placed-side`. See the `placement` variant.

## Direction / RTL

`DropdownMenu` accepts a `dir: Option<Direction>` prop, inherited by every `DropdownMenuSub`. Under RTL: the submenu open key is `ArrowLeft` instead of `ArrowRight`, the close key `ArrowRight` instead of `ArrowLeft` (`Escape` always closes regardless of direction), and each `DropdownMenuSubContent` opens to the left of its trigger instead of the right. See the `rtl` variant.

## Motion

The menu panel fades and scales in over `--dx-motion-duration-base` (`opacity`, `scale` from 0.95 and a 2px `translate`, never `transform`, so the anchor's centring survives) and fades out with `--dx-motion-ease-in`; item highlights use `--dx-motion-duration-fast`. Retune with those tokens. Reduced motion swaps in `--dx-motion-duration-reduced`. Nothing loops. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.
