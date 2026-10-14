The HoverCard component can be used to display additional information when a user hovers over an element. It is useful for showing tooltips, additional details, or any other content that should be revealed on hover.

## Component Structure

```rust
// The HoverCard component wraps the trigger element and the content that will be displayed on hover.
HoverCard {
    HoverCardTrigger {
        // Anything inside the trigger (icon, text, rich markup) acts as the hover target.
        {children}
    }
    HoverCardContent {
        side: ContentSide::Bottom,
        align: ContentAlign::Start,
        {children}
    }
}
```

## Delays

`HoverCard` takes `delay` and `close_delay` in milliseconds (Radix `openDelay` / `closeDelay`; Base UI `PreviewCard.Trigger`'s `delay` / `closeDelay`):

```rust
HoverCard {
    // How long the pointer must rest on the trigger before the card opens. Default 600 (Base UI).
    delay: 10,
    // How long the card stays open after the pointer leaves the trigger or the card. Default 300 (Base UI).
    close_delay: 100,
    HoverCardTrigger { "Hover Here" }
    HoverCardContent { "..." }
}
```

Leaving the trigger before `delay` cancels the open. After the pointer leaves, the card closes `close_delay` later unless the pointer enters the card (or the trigger) again, so it can cross the gap between them. **Keyboard focus is not delayed**: focusing the trigger opens the card at once, and blur or Escape close it at once. The timers belong to the component, so unmounting cancels them.

## No click-only mode

Unlike the menus (`NavigationMenu`, `Navbar`, `Menubar` and the dropdown /
context menu submenus), which take an `open_on_hover` prop to switch hover
off, a hover card has no such option: hovering (or focusing) the trigger *is*
what opens it, and its content is supplementary -- never the only way to reach
something. For a panel opened by a click, use [Popover](/component/popover/).

## Motion

The card fades in and out over `--dx-motion-duration-fast` (`opacity` only, so the anchor's `transform` centring is untouched). Retune with that token. Reduced motion swaps in `--dx-motion-duration-reduced`. Nothing loops. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.
