The accordion component is used to display collapsible content panels for presenting information in a limited amount of space. It allows users to expand and collapse sections to view more or less content.

## Component Structure

```rust
// The accordion component must wrap all accordion items.
Accordion {
    // Each accordion item contains both a trigger the expanded contents of the item.
    AccordionItem {
        // The trigger is used to expand or collapse the item.
        AccordionTrigger {}
        // The content that is shown when the item is expanded.
        AccordionContent {}
    }
}
```

`AccordionContent` supplies shadcn's Nova spacing itself (`text-sm`, no top padding, 10px below), so content needs no padding of its own. The trigger is `text-sm font-medium` with 10px of vertical padding and a 16px chevron in `muted-foreground`. Content that adds its own bottom padding (for example 1rem) leaves a larger gap than Nova.

## Multiple open items

By default opening an item closes the one that was open. Pass `allow_multiple_open: true` on `Accordion` (shadcn's `multiple`) and each item toggles on its own; `default_open: true` on an `AccordionItem` starts it open (shadcn's `defaultValue`). See the `multiple` variant.

```rust
Accordion { allow_multiple_open: true,
    AccordionItem { default_open: true,
        AccordionTrigger { "Notification Settings" }
        AccordionContent { "..." }
    }
    AccordionItem {
        AccordionTrigger { "Privacy & Security" }
        AccordionContent { "..." }
    }
}
```

## Controlled

Give the `Accordion` a `value` (Radix's `value`) to own the open items yourself: it is the list of open item values, an item is named by its `AccordionItem` `value`, and a click on a trigger only calls `on_value_change` with the new list, so you write it back. Without `value` the accordion keeps its own state, exactly as before; `default_value` seeds that state, next to `default_open` on an item. With `allow_multiple_open: false` the list holds at most one value, and an empty list means everything is closed. See the `controlled` variant: two buttons outside the accordion change the signal and the accordion follows.

An item without a `value` is named by its position as a decimal string (`"0"`, `"1"`, ...), so give items a `value` as soon as the list can reorder or change.

```rust
let mut open = use_signal(|| vec!["shipping".to_string()]);
rsx! {
    Accordion {
        value: Some(open()),
        on_value_change: move |next: Vec<String>| open.set(next),
        AccordionItem { value: Some("shipping".to_string()),
            AccordionTrigger { "Shipping" }
            AccordionContent { "..." }
        }
        AccordionItem { value: Some("returns".to_string()),
            AccordionTrigger { "Returns" }
            AccordionContent { "..." }
        }
    }
}
```

Controlled, `default_open` and `default_value` are ignored, and a find-in-page reveal (`hidden_until_found`) goes through `on_value_change` too, so a caller that does not write the value back keeps the item closed. An item's own `on_change` still follows its open state.

## Chevron placement and hover

Pass `chevron: AccordionChevron::Start` on `Accordion` to put the chevron before the label (default `AccordionChevron::End`, shadcn's layout). It points right when closed and down when open, instead of down and up. The prop is emitted as `data-chevron` (`start` / `end`) on the root and on every trigger, so a custom trigger can read it from its own element.

A collapsed trigger does not underline on hover; an open one does. That is a deliberate deviation from shadcn, which underlines every trigger.

```rust
Accordion { chevron: AccordionChevron::Start,
    AccordionItem {
        AccordionTrigger { "Installation" }
        AccordionContent { "..." }
    }
}
```

## Searchable closed content

By default closed items are unmounted after their close animation, so find-in-page (Ctrl+F) cannot see their text and a `#fragment` link cannot open them. Pass `hidden_until_found: true` on `Accordion` to keep every item's content mounted and mark it `hidden="until-found"` once it is closed. The browser then searches it, and when a match is inside a closed item it reveals the content and the accordion opens that item: `on_change` is called with `true`, and with `allow_multiple_open: false` (the default) the open item closes, exactly as if the trigger had been clicked. See the `until_found` variant: try Ctrl+F for the word in its hint.

```rust
Accordion {
    hidden_until_found: true,
    AccordionItem {
        AccordionTrigger { "Returns" }
        AccordionContent { "Text that Ctrl+F can find while closed" }
    }
}
```

Opening and closing by trigger animate the height as usual, and the content is re-hidden once the close animation finishes. Opening because of a match is not animated, because the browser has already scrolled to the match at full size. `AccordionContent` exposes `data-closing` (only while the close animation runs) and `data-revealed` (open because of a match) for styling this; key a close animation on `data-closing`, not on `data-open="false"`, which is also how a closed panel looks at rest.

Cost: every item's content is mounted (and, on a server-rendered page, hydrated) while closed, and it is in the server-rendered HTML. That is why it is opt-in.

### Browser support

| Engine | `hidden="until-found"` | `beforematch` |
| --- | --- | --- |
| Chrome / Edge 102+ | Yes | Yes |
| Firefox | 148+ (139 to 147: reveals, may scroll to the wrong place) | 139+ |
| Safari | 26.2+, partial: reveals the content but does not scroll to the match | 26.2+ |
| Older engines | Treated as plain `hidden`: the content stays closed and unsearchable, exactly as without the prop | n/a |

Nothing breaks where it is unsupported: the content is simply not found by find-in-page, as it is by default.

### Accessibility

Closed `until-found` content is hidden from the accessibility tree exactly like `hidden`: screen readers do not read it while the item is closed, and the trigger's `aria-expanded` stays `false`. Find-in-page is the one way to reach it without activating the trigger; it is exposed as soon as it opens, and `aria-expanded` follows.

## Motion

A panel's height opens and closes with `dx-accordion-open` / `dx-accordion-close` over `--dx-motion-duration-slower`, and the chevron rotates with the same duration. A panel revealed by find-in-page (`hidden_until_found`) skips the animation by design, so the browser can scroll to the match inside it. Retune with the token. Reduced motion swaps in `--dx-motion-duration-reduced`. Nothing loops. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.
