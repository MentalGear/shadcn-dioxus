The Badge is a component designed to display small, distinct labels that help highlight important content or status indicators. Perfect for use cases like notifications, status labels, or categorization.

## Component Structure

```rust
Badge {
    // Available variants: Primary, Secondary, Destructive, Outline, Ghost, Link
    variant: BadgeVariant::Primary,
    {children}
}
```

To make the badge a link, pass `as` and render the element yourself, with the children inside it:

```rust
Badge {
    r#as: move |attrs: Vec<Attribute>| rsx! {
        a { href: "#link", ..attrs,
            "Open Link"
            ArrowUpRight {}
        }
    },
}
```
