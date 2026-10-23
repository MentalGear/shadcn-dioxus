The ScrollArea component is used to create a scrollable container for its children. It can be used to enable scrolling for content that overflows the available space.

## Component Structure

```rust
// The ScrollArea component wraps all scrollable content.
ScrollArea {
    // The direction in which the scroll area can scroll. Can be one of Horizontal, Vertical, or Both.
    direction: ScrollDirection::Vertical,
    // The content of the scrollable area
    {children}
}
```

## Direction / RTL

`ScrollArea` accepts a `dir: Option<Direction>` prop, emitted as `dir`/`data-direction` on the root. This component wraps the browser's own native scrollbar (not a custom-drawn one), so the CSS Overflow spec's own `dir`-relative behavior repositions the scrollbar to the correct physical side automatically -- no additional logic is needed. See the `rtl` variant.

## Edge fade (the `dx-scroll-fade` utility)

`fade: ScrollFade` (`None` by default; `Y`, `X`, `Both`, `Top`, `Bottom`, `Start`, `End`) fades the edges of the scroller where there is more to scroll to:

```rust
div { style: "border: 1px solid var(--dx-border); border-radius: var(--dx-radius-lg);",
    ScrollArea { height: "12rem", direction: ScrollDirection::Vertical, fade: ScrollFade::Y,
        {children}
    }
}
```

At rest the start edge is crisp and the end edge fades to hint at more content; scrolling fades the start edge in, and at the end the end edge sharpens. If the content fits there is no fade. `ScrollArea` puts the class on the element that scrolls, so the same `fade` prop works on `VirtualList` (its container) and `MessageScroller` (`MessageScrollerViewport`, which defaults to `ScrollFade::Bottom`).

### The utility

The prop is a thin wrapper over a plain-CSS utility in `assets/dx-effects.css`, ported from shadcn/ui's `scroll-fade` (MIT). Use it directly on any scroll container you own. It is pure CSS (a `mask-image` driven by `animation-timeline: scroll()`, no JavaScript, no scroll listener), and a browser without scroll-driven animations shows a static end-edge fade. (The site has no separate Utilities page yet; this section is where it is documented.)

| Class | Fades |
|---|---|
| `dx-scroll-fade` / `dx-scroll-fade-y` | both block edges of an `overflow-y` box (`ScrollFade::Y`) |
| `dx-scroll-fade-x` | both inline edges of an `overflow-x` box; follows the reading direction (`ScrollFade::X`) |
| `dx-scroll-fade-xy` | all four edges of a box that scrolls on both axes: the block and inline masks intersected, each edge only while there is content past it; the inline edges follow the reading direction (`ScrollFade::Both`) |
| `dx-scroll-fade-t` / `-b` | one physical block edge (`ScrollFade::Top` / `Bottom`) |
| `dx-scroll-fade-l` / `-r` | one physical inline edge |
| `dx-scroll-fade-s` / `-e` | one logical inline edge, mirrored in RTL (`ScrollFade::Start` / `End`) |
| `dx-scroll-fade-none` | turns it off (works in any class order) |

Custom properties, set inline or on a class of your own:

- `--dx-scroll-fade-size`: depth of every fade (default `min(12%, var(--dx-space-10))`, 40px at most).
- `--dx-scroll-fade-t-size`, `-b-size`, `-s-size`, `-e-size`: depth of one edge.
- `--dx-scroll-fade-reveal`: the scroll distance over which a fade eases in or out (default 96px).

```html
<div class="dx-scroll-fade-y" style="--dx-scroll-fade-size: 24px; max-height: 12rem; overflow-y: auto;"></div>
```

Notes:

- **Link `dx-effects.css`** once at the app root next to `dx-components-theme.css`; the `scroll_area`, `virtual_list` and `message_scroller` registry entries list it in `globalAssets`, so `shadcn-dioxus add` copies it. It is a global sheet rather than part of this component's `style.css` because it also registers the `@property` custom properties and keyframes that every fading scroller shares, so there is exactly one copy of them; without the sheet the prop adds a class that matches nothing.
- The fade is a mask on the scroller, so a border or background on the same element fades with the content. Put those on a wrapper (as the demo does) and the class on the inner scroller. A classic scrollbar stays opaque: the mask leaves its gutter out, measured by the component's own script. Only an overlay scrollbar (macOS, iOS), which floats over the content, fades with it.
- Inline fades follow the reading direction: `ScrollFade::X`, `Both` and `Start`/`End` mirror when the scroller (or an ancestor) is `dir="rtl"`. The `fade` variant has a button that flips its horizontal strip.
- The element must scroll on that axis (`ScrollDirection::Vertical` for `Y`/`Top`/`Bottom`, `Horizontal` for `X`/`Start`/`End`, `Both` for `Both`: a corner is masked by both gradients, so it is as faded as its two edges together); the animation timeline is the element's own scroll position.
