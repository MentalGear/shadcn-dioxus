The Badge is a component designed to display small, distinct labels that help highlight important content or status indicators. Perfect for use cases like notifications, status labels, or categorization.

## Component Structure

```rust
Badge {
    // Available variants: Primary, Secondary, Destructive, Outline, Ghost, Link
    variant: BadgeVariant::Primary,
    {children}
}
```

To make the badge a link, pass `as` (Radix's `asChild`). The callback gets an `AsChild`: spread its `attributes` on your element and render its `children` inside it, so the badge's own children stay where you wrote them:

```rust
Badge {
    r#as: |el: AsChild| rsx! {
        a { href: "#link", ..el.attributes, {el.children} }
    },
    "Open Link"
    ArrowUpRight {}
}
```
