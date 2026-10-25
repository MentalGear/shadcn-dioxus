`DirectionProvider` declares the reading direction once, at any ancestor, and every direction-aware component under it follows. It renders no element: it is a context boundary, like Radix's `DirectionProvider` and shadcn's `direction` page (a re-export of Base UI's).

## Component Structure

```rust
DirectionProvider { direction: Direction::Rtl,
    Tabs { /* ... */ }
}
```

Read it yourself with `use_direction(local)`: the `local` override if given, else the nearest provider's, else LTR. Every direction-aware root also takes a `dir: Option<Direction>` prop that wins over the ambient value.

## Reactive

The provider is backed by a signal. Pass a changing `direction` (a signal's value) and every descendant re-renders with the new direction, so the direction can change live (the demo above, and the RTL button on every component page, do exactly that). A consumer must read `use_direction` in its component body rather than capture the value once into its own state.

## What mirrors, and how

- **Keys.** `ArrowLeft`/`ArrowRight` mean "previous"/"next" in LTR and the opposite in RTL. `Direction::resolve_horizontal(&key)` is the one rule (`Direction::ltr_equivalent(&key)` returns the mirrored key for handlers that already match on `ArrowLeft` = previous). Tabs, radio group, toggle group, toolbar, navbar, menubar, menus (submenu open/close), slider, resizable, calendar, carousel, accordion, tag group, navigation menu and the date field segments all go through it. `scripts/check-direction-keys.sh` fails any primitive that handles a horizontal arrow without reading the direction; the reasoned allowlist records "nothing to flip" cases: the colour picker's area (a physical plane), and `move_interaction` (it produces physical deltas that slider and resizable mirror). `drawer` has no horizontal key handling (its `side` is a physical screen edge) and `command` is a vertical list.
- **Layout.** Set `dir` on a container. Component stylesheets use CSS logical properties (`margin-inline-start`, `inset-inline-end`, `text-align: start`), so layout mirrors with no per-component code; `scripts/check-css-logical-properties.sh` keeps physical properties out unless a comment says why. A rule that is physical on purpose (`data-side`, a swipe edge) carries `rtl-physical`.
- **Popups.** A popup's `side` (`top`/`right`/`bottom`/`left`) is a physical screen edge, as in Radix. Its `align` is logical: under RTL, `align: start` on a top/bottom popup hangs from the trigger's right edge and `align: end` from its left, in both the CSS anchor path and the JS positioner (`primitives/src/top_layer/anchor_fallback.rs`).

## Seeing RTL

Every component page has an RTL toggle in its header. It sets `dir="rtl"` on each demo frame and wraps each demo in a `DirectionProvider`, so any component can be tried in RTL without a hand-built variant. The `rtl` variants some components keep are fixtures for `oracle/tier3-radix/rtl.spec.ts`; `oracle/tier3-radix/rtl-toggle.spec.ts` runs a keyboard and positioning smoke of every component under the toggle.
