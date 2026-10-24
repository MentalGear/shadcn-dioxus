The Select component is used to create a dropdown menu that allows users to select one or more options from the select groups.

## Component Structure

```rust
Select::<String> {
    value: "option1",
    item_to_label: |value: String| label_of(&value), // required: the trigger's text, read at render time
    on_value_change: |value: String| {
        // Handle the change in selected value.
    },
    SelectGroup {
        SelectGroupLabel { "Group A" }
        SelectOption::<String> {
            index: 0,
            value: "option1",
            "Option 1"
        }
    }
}
```

## Selected label

`item_to_label: Callback<T, String>` is required on `Select` and `SelectMulti` (like Base UI's `itemToStringLabel`
and `ComboboxMulti`'s). `SelectValue` and the trigger's fallback accessible name read it at render time, never the
option registry: options register in a client-only effect and come after the trigger in tree order, so a
registry-derived label is empty in the server HTML (a preselected `Select` would show its placeholder until hydration).
A required prop, not a `T: Display` default, because select values are often `&str`, numbers or enums with no
`Display`, and a default that falls back to the registry would reintroduce the empty first paint. `SelectMulti`
joins its values' labels with a comma. A label that differs from an option's own text is fine; keep them equal so the
trigger reads like the row that was picked.

## Scroll buttons

The list scrolls inside the popup (`max-height: min(18rem, var(--dx-available-height))`). `Select` and `SelectMulti`
put a `SelectScrollUpButton` first and a `SelectScrollDownButton` last in the list (shadcn's `SelectScrollUpButton` /
`SelectScrollDownButton`); a primitive-level caller places them the same way. Each is sticky at its edge, out of flow
(showing one never moves an option), hidden from assistive technology, and visible only while the list can scroll that
way (`data-visible`): a passive `scroll` listener through `js_listener.rs` reports the position. Hovering one with a
mouse scrolls the list (6 px per 16 ms tick) through `use_interval_while`, gated by `use_motion`, so a hidden tab or a
skipped card pauses it; leaving, or reaching the end, stops it. See the `scroll` variant.

`position: SelectPosition::ItemAligned` (shadcn's `position="item-aligned"`) opens the list over the trigger with the
selected option on the trigger's line, moved and then scrolled to stay inside the viewport. With no selected option
there is nothing to align: the whole list opens below the trigger and flips above it when it does not fit there and the
room above is larger, capped to that side's room (`data-side` on the list). See the `aligned` variant.

## Open focus

A click or tap opens a single `Select` on its selected option, focused and scrolled into view, the same as a keyboard open (Base UI opens on the selected index for every modality). With nothing selected, or in `SelectMulti`, a pointer open leaves focus on the listbox and ArrowDown reaches the first option. A `DropdownMenu` / `ContextMenu` does the opposite on purpose: a keyboard open focuses the first item, a pointer open highlights none.

## Size

`size: SelectSize::Sm` makes the trigger 28px tall (`--dx-control-height-sm`) instead of the default 32px (`--dx-control-height-default`), shadcn's `size="sm" | "default"` on `SelectTrigger`. It sets `data-size` on the trigger of both `Select` and `SelectMulti`. See the `sizes` variant.

## Direction / RTL

`Select`/`SelectMulti` accept a `dir: Option<Direction>` prop, emitted as `dir`/`data-direction` on the trigger and the listbox. The trigger/listbox's own keyboard navigation is vertical-only (`ArrowUp`/`ArrowDown`/`Home`/`End`), so there is no arrow-key behavior to flip. See the `rtl` variant.

## Motion

The list fades and scales in and out over `--dx-floating-duration` (the shared `dx-pop-in`/`dx-pop-out`: `opacity`, `scale` from 0.95, a 2px `translate`); option highlights use `--dx-motion-duration-fast`. Retune with those tokens. Reduced motion swaps in `--dx-motion-duration-reduced`. Nothing loops. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.
