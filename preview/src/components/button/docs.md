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

To render the button as a link (shadcn's "As Link"), pass `as` and render the anchor yourself. The anchor takes the button's attributes and handlers:

```rust
Button {
    variant: ButtonVariant::Secondary,
    size: ButtonSize::Sm,
    r#as: move |attrs: Vec<Attribute>| rsx! {
        a { href: "/login", ..attrs, "Login" }
    },
}
```

The Secondary and Destructive hover fills step the fill toward `--dx-foreground` in the interpolation space of the theme token `--dx-hover-mix-space` (`oklch`, shadcn base-nova's), so they match shadcn's `color-mix(in oklch, ...)` hover. Retune the space once, in the theme.

## Radius

`Button { radius: ButtonRadius::Full }` (shadcn's `rounded-full`) makes a pill on a text size and a circle on an icon size; the default keeps each size's own radius. It sets the one `--dx-button-radius` slot every corner reads, so a ButtonGroup's per-position corners still win inside a group.

## Slot variables

A component that composes `Button` retunes it by setting a custom property on the button's own element, never by declaring the property on its own class: two stylesheets declaring one property on one element are decided by which sheet loads last, and that order differs from route to route. `.dx-button` and its variants read each of these with their own default, so a variable that is not set changes nothing.

| Variable | Default | Property |
| --- | --- | --- |
| `--dx-button-background` | the variant's fill | `background-color` at rest (hover and `aria-expanded` keep the variant's own fill) |
| `--dx-button-color` | the variant's foreground | `color` at rest |
| `--dx-button-border-color` | `transparent` (the Outline variant's edge colour) | `border-color` at rest (the focus and invalid rings still win) |
| `--dx-button-transition-property` | `all` | `transition-property` |
| `--dx-button-line-height` | `1.25rem` (`1rem` in the `xs` size) | `line-height` |

```css
/* In the composing component's own stylesheet, on the button's element: */
.dx-message-scroller-button { --dx-button-background: var(--dx-background); }
```

Set the variable on the button's own element rather than an ancestor: custom properties inherit, so an ancestor's value would reach every nested button too.
