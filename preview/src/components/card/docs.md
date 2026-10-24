The card component is a flexible container for grouping related content and actions. It provides a structured layout with optional header, content, and footer sections.

## Component Structure

```rust
// The Card component must wrap all card elements.
Card {
    // CardHeader contains the title, description, and optional action.
    CardHeader {
        // CardTitle displays the main heading.
        CardTitle { "Card Title" }
        // CardDescription provides supporting text.
        CardDescription { "Card description goes here." }
        // CardAction positions action elements (e.g., buttons) in the header.
        CardAction {
            Button { "Action" }
        }
    }
    // CardContent holds the main body content.
    CardContent {
        p { "Main content of the card." }
    }
    // CardFooter contains footer actions or information.
    CardFooter {
        Button { "Submit" }
    }
}
```

## Layout Notes

- When `CardAction` is present inside `CardHeader`, the header automatically switches to a two-column grid layout.

## Size

`Card { size: CardSize::Sm }` (shadcn's `size="sm"`) tightens the spacing from 16px to 12px and the title to `text-sm`. The card always carries `data-size` (`default` or `sm`); it is the typed prop's value, so a raw `"data-size"` attribute cannot disagree with it.

## Slot variables

A component that composes `Card` retunes it by setting a custom property on the card's own element, never by declaring the property on its own class: two stylesheets declaring the same property on one element are decided by which sheet loads last, and that order differs from route to route. `.dx-card` and `.dx-card-footer` read each of these with its own default, so a variable that is not set changes nothing.

| Variable | Default | Property | Read by |
| --- | --- | --- | --- |
| `--dx-card-padding-block` | `var(--dx-card-spacing)` | `padding-block` | `.dx-card` |
| `--dx-card-footer-direction` | `row` | `flex-direction` | `.dx-card-footer` |
| `--dx-card-footer-align-items` | `center` | `align-items` | `.dx-card-footer` |
| `--dx-card-footer-font-size` | `inherit` | `font-size` | `.dx-card-footer` |
| `--dx-card-footer-gap` | `normal` | `gap` | `.dx-card-footer` |

```css
/* In the composing component's own stylesheet, on the card's or footer's element: */
.dx-chart-interactive { --dx-card-padding-block: 0; }
.dx-chart-footer { --dx-card-footer-direction: column; }
```

Set the variable on the exact element rather than an ancestor: custom properties inherit, so an ancestor's value would reach every nested card too.
