The button element is used to trigger actions or events in a user interface.

## Component Structure

```rust
button {
    // Global html attributes
    class: "dx-button",
    "data-style": "outline",
    "data-size": "default",
    // Children
    {children}
}
```

To render the button as a link (shadcn's "As Link", Radix's `asChild`), pass `as`. The callback gets an `AsChild`: the button's attributes, its `onclick` and other handlers included, and its children. Spread the one on the anchor and render the other inside it:

```rust
Button {
    variant: ButtonVariant::Secondary,
    size: ButtonSize::Sm,
    r#as: |el: AsChild| rsx! {
        a { href: "/login", ..el.attributes, {el.children} }
    },
    "Login"
}
```

The Secondary and Destructive hover fills step the fill toward `--dx-foreground` in the interpolation space of the theme token `--dx-hover-mix-space` (`oklch`, shadcn base-nova's), so they match shadcn's `color-mix(in oklch, ...)` hover. Retune the space once, in the theme.

## Radius

`Button { radius: ButtonRadius::Full }` (shadcn's `rounded-full`) makes a pill on a text size and a circle on an icon size; the default keeps each size's own radius. It sets the one `--dx-button-radius` slot every corner reads, so a ButtonGroup's per-position corners still win inside a group.
