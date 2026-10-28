The slider component allows users to select a value from a range by sliding a handle along a track.

## Component Structure

```rust
Slider {
    value: 0.0,
    horizontal: true,
    on_value_change: |value: f64| {
        // Handle the change in slider value.
    },
}
```

For a two-thumb range selector, use `RangeSlider`:

```rust
RangeSlider {
    default_value: 20.0..80.0,
    on_value_change: |value: std::ops::Range<f64>| {
        // value.start and value.end give the two endpoints
    },
}
```

## Direction / RTL

`Slider`/`RangeSlider` accept a `dir: Option<Direction>` prop, consulted only when `horizontal: true` (a vertical slider never mirrors, matching Radix). Under RTL: a click near the left end of the track resolves to a value near `max` (not `min`); `ArrowLeft` increases the value and `ArrowRight` decreases it (the opposite of LTR); the thumb and the filled range visually anchor to the opposite edge. `ArrowUp`/`ArrowDown` are never affected. See the `rtl` variant.

## In a form

Give the slider a `name` and it submits its value with the nearest `<form>` (or the one whose id is passed as `form: "form-id".to_string()`), through a hidden input per thumb: a `Slider` submits one value under its name, a `RangeSlider` two under the same name, start first. The value is printed at the step's precision (`0.3`, not `0.30000000000000004`). A disabled slider submits nothing and an empty `name` (the default) submits nothing. The form's reset restores `default_value`; for a controlled slider it calls `on_value_change` with the default, so the owner of the value follows. See the `form` variant.

```rust
DemoForm {
    Slider { label: "Volume", name: "volume", default_value: 30.0 }
    RangeSlider { label: "Price", name: "price", default_value: 20.0..80.0 }
}
```

## Finished changes: `on_value_commit`

`on_value_change` fires on every move of a drag. `on_value_commit` fires once a change is finished: when the pointer is released after a press that moved a value (a drag, or a click on the track), and on each keyboard step that changes the value (arrow keys, Page Up/Down, Home, End). It receives the value (`f64`, or the `Range<f64>` of a `RangeSlider`). A press that leaves the value where it was, a value set from outside and a form reset do not commit.

## Keeping thumbs apart

`RangeSlider` takes `min_steps_between_thumbs: u32` (default 0, the thumbs may meet). The start thumb stops that many steps short of the end thumb and the end thumb that many steps past the start thumb; the thumbs' `aria-valuemin`/`aria-valuemax` follow.
