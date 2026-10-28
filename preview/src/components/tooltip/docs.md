The Tooltip component is used to display additional information when a user hovers over an element.

## Component Structure

```rust
// The Tooltip component wraps the trigger element and the content that will be displayed on hover.
Tooltip {
    // The TooltipTrigger contains the elements that will trigger the tooltip to display when hovered over.
    TooltipTrigger {
        // The elements that will trigger the tooltip when hovered over.
        {children}
    }
    // The TooltipContent contains the content that will be displayed when the user hovers over the trigger.
    TooltipContent {
        // The side of the TooltipTrigger where the content will be displayed. Can be one of Top, Right, Bottom, or Left.
        side: ContentSide::Top,
        // The alignment of the TooltipContent relative to the TooltipTrigger. Can be one of Start, Center, or End.
        align: ContentAlign::Center,
        // The content of the tooltip, which can include text, images, or any other elements.
        {children}
    }
}
```

## Delays

`Tooltip` takes `delay` and `close_delay` in milliseconds (Radix `delayDuration`; Base UI `Tooltip.Trigger`'s `delay` / `closeDelay`):

```rust
Tooltip {
    // How long the pointer must rest on the trigger before the tooltip opens. Styled default 0.
    delay: 600,
    // How long the tooltip stays open after the pointer leaves. Default 0; above 0 the pointer can move onto the tooltip, which holds it open.
    close_delay: 400,
    TooltipTrigger { "Hover" }
    TooltipContent { "Add to library" }
}
```

**Two defaults, on purpose.** The primitive (`dioxus_primitives::tooltip::Tooltip`) defaults to Base UI's `delay` of 600 ms. This styled component defaults to `delay: 0` and `close_delay: 0`, because shadcn's styled `tooltip.tsx` ships a `TooltipProvider` whose default is `delay = 0` (Base UI flavour) / `delayDuration = 0` (Radix flavour): a shadcn tooltip opens on contact. Pass `delay: 600` for a rest-to-open tooltip.

With a delay above 0, nothing opens until it has passed, so sweeping across a toolbar flashes no tooltip, and leaving the trigger before then cancels the open. **Keyboard focus is not delayed**: focusing the trigger opens the tooltip at once, and blur or Escape close it at once. The timers belong to the component, so unmounting cancels them. There is no provider that skips the delay for the next tooltip; set `delay` on each tooltip instead.

## Touch and press

Hover is a mouse's (and a pen's): a touch pointer never opens a tooltip, and a tap does not open one through the focus it moves onto the trigger either (only keyboard focus opens a tooltip). Pressing the trigger closes an open tooltip, as Base UI's `closeOnClick` does. The tooltip sits 4 px from the trigger, shadcn's `sideOffset={4}`.

## No click-only mode

Unlike the menus (`NavigationMenu`, `Navbar`, `Menubar` and the dropdown /
context menu submenus), which take an `open_on_hover` prop to switch hover
off, a tooltip has no such option: hovering or focusing the trigger *is* what
shows it, and it is a non-interactive hint, never the only way to reach
something. For a panel opened by a click, use [Popover](/component/popover/).

## Motion

The tooltip fades in and out over `--dx-motion-duration-slow` (`opacity` only, so the anchor's `transform` centring is untouched). Retune with that token. Reduced motion swaps in `--dx-motion-duration-reduced`. Nothing loops. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.
