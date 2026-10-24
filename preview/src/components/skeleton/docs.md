The Skeleton component is used to display a placeholder preview of content before the data gets loaded. This helps improve perceived performance and provides users with a visual indication that content is being loaded.

## Component Structure

```rust
Skeleton {
    // Accepts all GlobalAttributes, commonly used with style for sizing
    style: "width: 15rem; height: 1rem;",
}
```

## Motion

The pulse is an `opacity` animation that loops forever (2 s). It is paused whenever the skeleton is outside the viewport (plus a 200px pre-roll), inside a skipped `content-visibility` subtree or in a hidden tab, and it resumes where it stopped. `Skeleton` calls `dioxus_primitives::activity::use_motion()` and spreads `motion.attributes()` on its root; the theme's single `[data-dx-motion="paused"]` rule does the pausing.

To change or remove the pulse, override `animation` on `.dx-skeleton`. There is no reduced-motion rule for it yet, so add `@media (prefers-reduced-motion: reduce) { .dx-skeleton { animation: none; } }` to your copy if your audience needs it. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.

## Slot variables

A component that composes `Skeleton` (such as `SidebarMenuSkeleton`) sizes it by setting a custom property on the skeleton's element, never by declaring `width`, `height`, `flex` or `border-radius` on its own class: two stylesheets declaring the same property on one element are decided by which sheet loads last, and that order differs from route to route. `.dx-skeleton` reads each of these with its own default, so a variable that is not set changes nothing. The box is zero specificity, so a class or an inline `style` that sizes the skeleton still wins.

| Variable | Default | Property |
| --- | --- | --- |
| `--dx-skeleton-width` | `auto` | `width` |
| `--dx-skeleton-height` | `auto` | `height` |
| `--dx-skeleton-flex` | `0 1 auto` | `flex` |
| `--dx-skeleton-radius` | `var(--dx-radius-md)` | `border-radius` |
