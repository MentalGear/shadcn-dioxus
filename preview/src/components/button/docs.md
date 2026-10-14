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
