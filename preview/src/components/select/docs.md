The Select component is used to create a dropdown menu that allows users to select one or more options from the select groups.

## Component Structure

```rust
Select::<String> {
    value: "option1",
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

## Size

`size: SelectSize::Sm` makes the trigger 28px tall (`--dx-control-height-sm`) instead of the default 32px (`--dx-control-height-default`), shadcn's `size="sm" | "default"` on `SelectTrigger`. It sets `data-size` on the trigger of both `Select` and `SelectMulti`. See the `sizes` variant.

## Direction / RTL

`Select`/`SelectMulti` accept a `dir: Option<Direction>` prop, emitted as `dir`/`data-direction` on the trigger and the listbox. The trigger/listbox's own keyboard navigation is vertical-only (`ArrowUp`/`ArrowDown`/`Home`/`End`), so there is no arrow-key behavior to flip. See the `rtl` variant.

## Motion

The list fades and scales in and out over `--dx-floating-duration` (the shared `dx-pop-in`/`dx-pop-out`: `opacity`, `scale` from 0.95, a 2px `translate`); option highlights use `--dx-motion-duration-fast`. Retune with those tokens. Reduced motion swaps in `--dx-motion-duration-reduced`. Nothing loops. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.
