The Tabs component is used to create a tabbed interface, allowing users to switch between different views or sections of content.

## Component Structure

```rust
// The Tabs component wraps all tab triggers and contents and orders them based on their index.
Tabs {
    // The TabList component contains all the tab triggers
    TabList {
        // The TabTrigger component is used to create a clickable tab button that switches the active tab.
        TabTrigger {
            // The index of the tab trigger, used to determine the focus order of the tabs.
            index: 0,
            // The value of the tab trigger, which must be unique and is used to identify the active tab.
            value: "tab1",
            // The contents of the tab trigger button
            {children}
        }
    }
    // The TabContent component contains the content that is displayed when the corresponding tab is active.
    TabContent {
        // The index of the tab content, used to determine the focus order of the tabs.
        index: 0,
        // The value of the tab content, which must match the value of the corresponding TabTrigger to be displayed.
        value: "tab1",
        // The content of the tab, which is displayed when the tab is active.
        {children}
    }
}
```

## Variants

`Tabs` takes `variant: TabsVariant`. `Default` is the muted pill with a raised active tab inside a bordered card. `Line` is shadcn's `line` variant: a transparent list with a 4px gap, no card around the panel, and the active tab carries a 2px bar that fades in (an underline when horizontal, a bar on the panel side when vertical; the docs site's DEMO/CODE tabs use it). `Ghost` is the former name of `Line` and renders identically (`data-variant="line"`). See the `line` variant.

(shadcn puts `variant` on `TabsList`; here it is on `Tabs`, because the panel's frame follows it too.)

## Orientation

`Tabs` takes `horizontal: bool`, `true` by default (shadcn's `orientation="horizontal"`). `horizontal: false` is shadcn's `orientation="vertical"`: the list becomes a column beside the panels (`flex-col`, as tall as its triggers, triggers full width and start-aligned), and the root reports `data-orientation="vertical"`. The list always carries `aria-orientation` (`horizontal` or `vertical`).

Keyboard follows WAI-ARIA: in a horizontal list `ArrowLeft`/`ArrowRight` move focus (swapped under RTL) and `ArrowUp`/`ArrowDown` do nothing; in a vertical list `ArrowUp`/`ArrowDown` move focus (never affected by direction) and `ArrowLeft`/`ArrowRight` do nothing. `Home`/`End` work in both, and focus wraps unless `roving_loop` is false. Focus only moves; `Enter` or `Space` activates the focused tab, as in the rest of this library. The vertical `line` bar sits on the end edge, the side that faces the panel, in RTL too. See the `vertical` and `vertical_line` variants.

```rust
Tabs { horizontal: false, variant: TabsVariant::Line, default_value: "a".to_string(), /* ... */ }
```

**Behaviour change:** `horizontal` used to default to `false` (vertical) in the primitive, although the styled component always rendered a row; it now defaults to `true`, so a `Tabs` that omitted it is a horizontal tab list with `ArrowLeft`/`ArrowRight`. Pass `horizontal: false` for the old keyboard.

## Sizes

`Tabs` takes `size: TabsSize` (`Sm`, `Default`, `Lg`), emitted as `data-size` on the root. The list height follows the theme's shared control-height scale (`--dx-control-height-sm` / `-default` / `-lg` = 28 / 32 / 36px, the same ladder as `Button`, `Input` and `Select`), so a `Sm` tab list lines up with a `Sm` button. `Default` is shadcn's 32px list and is unchanged. `Sm` sets the triggers in `text-xs` with a 14px icon; `Lg` keeps `text-sm` and widens the trigger padding. Both variants (`Default` and `Line`) take every size, and a Tabs nested in another Tabs' panel keeps its own size. See the `sizes` variant.

```rust
Tabs { size: TabsSize::Sm, default_value: "a".to_string(), /* ... */ }
```

## Panel padding

`TabContent` has no padding of its own (shadcn's Nova `TabsContent` is `text-sm` and nothing else; the padding that used to be 2rem was this library's). Put padding on the content, or on the panel with `padding`, or let a `Card` inside it bring its own. The default (bordered) variant still draws a 1px card-coloured frame around the panel, so text placed straight in it wants about `var(--dx-space-4)` of padding; the `line` variant has no frame.

**Breaking change:** a `TabContent` that relied on the 2rem padding now sits flush against its frame. Add `padding: "var(--dx-space-8)"` to restore the old look.

## Direction / RTL

`Tabs` accepts a `dir: Option<Direction>` prop (defaulting to the nearest `DirectionProvider`, or LTR). Under RTL, `TabTrigger`'s `ArrowLeft`/`ArrowRight` roving focus swaps: `ArrowLeft` moves to the *next* tab, `ArrowRight` to the *previous* one (matching Radix's shared `RovingFocusGroup` behavior). See the `rtl` variant.

## Searchable inactive tabs

By default inactive panels are unmounted (an empty `hidden` element), so find-in-page (Ctrl+F) cannot see their text and a `#fragment` link cannot open them. Pass `hidden_until_found: true` on `Tabs` to keep every panel's content mounted and mark inactive panels `hidden="until-found"`. The browser then searches them, and when a match is inside an inactive panel it reveals the panel and its tab becomes the active one: `aria-selected` and `data-state` follow, and `on_value_change` is called with that tab's value (a controlled `value` should follow it). See the `until_found` variant: try Ctrl+F for the word in its hint.

```rust
Tabs {
    default_value: "tab1".to_string(),
    hidden_until_found: true,
    // ...TabList and TabTrigger as above...
    TabContent { index: 1usize, value: "tab2".to_string(), "Text that Ctrl+F can find while inactive" }
}
```

Cost: the content of **every** tab is mounted and, on a server-rendered page, hydrated, so effects, timers and charts inside inactive tabs all run, and all of it is in the server-rendered HTML. That is why it is opt-in; leave it off for heavy panels.

### Browser support

| Engine | `hidden="until-found"` | `beforematch` |
| --- | --- | --- |
| Chrome / Edge 102+ | Yes | Yes |
| Firefox | 148+ (139 to 147: reveals, may scroll to the wrong place) | 139+ |
| Safari | 26.2+, partial: reveals the content but does not scroll to the match | 26.2+ |
| Older engines | Treated as plain `hidden`: the content stays closed and unsearchable, exactly as without the prop | n/a |

Nothing breaks where it is unsupported: the content is simply not found by find-in-page, as it is by default.

### Accessibility

Inactive `until-found` panels are hidden from the accessibility tree exactly like `hidden`: screen readers only reach the active panel, as with plain tabs. Find-in-page is the one way to reach an inactive panel without selecting its tab; it is exposed as soon as it activates.

### Styling

`TabContent` carries `hidden` when inactive (`hidden="until-found"` with the opt-in). A stylesheet must not set `display: none` on an `until-found` panel: that overrides the browser rule that keeps its text searchable. The shipped stylesheet skips `[hidden="until-found"]` and takes such a panel out of flow at zero size, so the inactive panels add no empty cards or gaps.

## Motion

Trigger colours and the `line` variant's bar change over `--dx-motion-duration-base` with `--dx-motion-ease-standard`; the panels switch instantly. Retune with those tokens. Reduced motion swaps in `--dx-motion-duration-reduced`. Nothing loops. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.
