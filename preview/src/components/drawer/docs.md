The drawer component is a panel that slides in from the edge of the screen, like the sheet component, but adds a pointer drag gesture (a Vaul-style "swipe to dismiss") on top -- drag its handle or content toward the edge it slid in from to close it, in addition to Escape or a close button. It is the same edge panel as the sheet (one recipe in the theme, one side type), plus the drag, a handle and nested stacking.

## Component Structure

```rust
Drawer {
    open: open(),
    // Which edge to slide in from. Available sides: Auto (default), Top, Right, Bottom, Left.
    // Leave it out for Auto: the bottom on a phone, a floating right panel on a wider screen.
    side: DrawerSide::Auto,
    DrawerContent {
        DrawerHandle {}
        DrawerHeader {
            DrawerTitle { "Move Goal" }
            DrawerDescription { "Set your daily activity goal." }
        }
        DrawerFooter {
            DrawerClose { "Submit" }
            DrawerClose {
                as: |attributes| rsx! {
                    Button { variant: ButtonVariant::Outline, attributes, "Cancel" }
                }
            }
        }
    }
}
```

## Sides: `Auto` is the default

`side` is an `EdgeSide` (`DrawerSide` and the sheet's `SheetSide` are aliases of it), and its default is `DrawerSide::Auto`, as in shadcn's demo:

- **`Auto`** is a **bottom** drawer below the theme's `md` breakpoint (`--dx-breakpoint-md`, 768px: a phone, where a bottom sheet is the thumb-reachable one) and a **right** drawer at and above it. It follows the viewport through one media-query listener, so a drawer opened after a resize or a rotation uses the new side; it is decided once, in Rust, and everything downstream (the `data-side` attribute, the drag axis, the handle, the slide) sees the resolved side, never `auto`.
- **`Top`, `Right`, `Bottom`, `Left`** are exactly that side at every width; the viewport never changes them.

The look follows the side. A **bottom** (or top) drawer is docked flush and rounds the two corners facing the page; the grab bar shows on the bottom one only (shadcn and Nova). A **left or right** drawer **floats**, inset from the viewport edge (`--dx-edge-inset`, 8px) with a full border and all four corners rounded; it is three quarters of the viewport wide, capped at 24rem from 640px up. The recipe is the theme's "Edge panel" section (`data-dx-surface="edge"`, `data-dx-edge="floating"`), shared with the sheet.

## Nested drawers

Put a `Drawer` inside another `DrawerContent` and it nests, as Vaul's `NestedRoot` does -- no separate component:

```rust
Drawer { open: open(), on_open_change: move |v| open.set(v),
    DrawerContent {
        DrawerTitle { "Parent" }
        Button { onclick: move |_| nested.set(true), "Open nested drawer" }
        Drawer { open: nested(), on_open_change: move |v| nested.set(v),
            DrawerContent { DrawerTitle { "Nested" } }
        }
    }
}
```

While the nested drawer is open the parent steps back (`data-nested-open="true"`: a 4% shrink toward its docked edge and a 16px push away from it, `--dx-drawer-nest-scale` and `--dx-drawer-nest-shift`); it is restored the moment the nested one starts closing. Only the top-most drawer reacts to Escape, a tap on the scrim and a drag, and the page's scroll lock stays one lock until the last drawer closes.

## Drag to dismiss

Dragging [`DrawerHandle`] or anywhere on [`DrawerContent`] itself toward `side` tracks the pointer live. Only movement toward the drawer's own closing edge dismisses it -- down for a bottom drawer (the phone default), right for the `Auto` drawer on a wide screen, up for `side: DrawerSide::Top`, and so on for `Left`; dragging the other way rubber-bands and snaps back, however far or fast, and releasing the pointer over the backdrop is not a dismissal either. Releasing past 25% of the panel's own size along the closing axis, or flicking toward the closing edge (at least 0.5px/ms over the last 100ms, after at least 10px), finishes closing the drawer; releasing short of that snaps it back open. Mouse, touch and pen behave the same. Set `dismissible: false` on `Drawer` to disable the drag gesture entirely while keeping Escape and `DrawerClose` working.

A drag never starts from an interactive descendant (a button, link, or form control), or from content that is scrolled away from the drag edge -- either is read as "interact with (or scroll) this instead," not "dismiss the drawer."

## Snap points

`snap_points` (Base UI's `snapPoints`) makes the drawer rest at partial extents instead of fully open. A `SnapPoint` is a fraction of the viewport along the drawer's axis (`Fraction(0.5)`: half the height of a top/bottom drawer, half the width of a left/right one; measured against the layout viewport) or a length (`Px(355.0)`); `Fraction(1.0)` is fully open.

```rust
Drawer {
    side: DrawerSide::Bottom,
    snap_points: vec![SnapPoint::Px(148.0), SnapPoint::Px(355.0), SnapPoint::Fraction(1.0)],
    // Optional: control the active point (default: the first of `snap_points`).
    active_snap_point: active(),
    on_active_snap_point_change: move |point| active.set(Some(point)),
    DrawerContent { DrawerHandle {} /* ... */ }
}
```

- **One resting offset.** The panel is as large as the **largest** snap point and rests translated toward its closing edge by the difference. The active point writes two custom properties on the panel (`--dx-drawer-snap`, `--dx-drawer-snap-max`); the stylesheet derives `--dx-drawer-rest` from them and the panel's `transform` reads it (the live drag adds its delta to it, the nested-drawer step-back adds to it). Nothing measures the panel to place it, and a snap change is a custom-property change that the ordinary `transform` transition settles. `data-snap` is `partial` or `full` (absent without snap points).
- **Settling.** Releasing a drag settles on the snap point **nearest the release position projected by the release velocity** (150ms of it, capped at 3px/ms): a slow drag goes to the nearest point, a fast flick may skip one, and nothing overshoots the ends. Drag toward the open side tracks 1:1 up to the largest point, then rubber-bands.
- **Closing.** Only a drag **below the lowest point** closes the drawer, by the drawer's one closing rule: past 25% of the lowest point's visible extent, or a flick (0.5px/ms, after 10px). From a higher point a drag, however far or fast, lands on the lowest point at most. Escape and `DrawerClose` close from anywhere.
- **Controlled.** `active_snap_point` controls the active point; a drag reports the new point through `on_active_snap_point_change` and a controlled value the parent does not change snaps the panel back. Uncontrolled, `default_active_snap_point` (else the first point) is where it opens, and a drawer opened again is back at it. A value that is not one of `snap_points` reads as the first.
- **Content and drag.** Below the largest point the content does not scroll (it is clipped) and a press anywhere on the panel drags toward the next point; at the largest point the content scrolls, and a press in content that is scrolled away from the top does not drag. On touch, a top/bottom drawer at its largest point lets content pan vertically, so its drag starts from the handle.
- **Keyboard.** Base UI's Drawer has no keyboard stepping between snap points, and neither has this one: Escape closes, and a control of your own (a button that sets the controlled `active_snap_point`, as the demo's Expand does) moves between points.

## Non-modal drawer

`is_modal: false` (Base UI's `modal: false`) turns the drawer into a panel beside the page, not over it: **no scrim** (`overlay` is ignored), **no top layer and no inert page**, **no scroll lock** (the page scrolls and every control stays interactive), and **no focus trap**. It is the non-modal arm of the dialog primitive, so Escape, the close button and a press (or focus moving) outside the panel still dismiss it (Base UI's default light dismissal); drag-to-dismiss, handle, snap points and the slide are unchanged. The root wrapper carries `data-dx-modal="false"` and lets pointer events through, so only the panel itself catches them. The panel keeps the dialog primitive's `role="dialog"` markup. `disable_pointer_dismissal: true` (Base UI's `disablePointerDismissal`) stops a press or focus outside the panel from dismissing it, so the first page click only acts on the page; Escape, the close button and drag-to-dismiss still close it. The non-modal panel no longer reports `aria-modal` (the page is not inert).

## Responsive Dialog to Drawer

shadcn's "Responsive Dialog": render a `Dialog` at and above the mobile breakpoint and a `Drawer` below it, with the same form inside both. The Sidebar's `use_is_mobile()` is the viewport signal (`window.innerWidth < 768`, a resize listener owned by the calling component); the demo branches on it and shares the form as one component.

## DrawerClose with `as` prop

The `as` prop allows you to render a custom element while preserving the close behavior, similar to shadcn/ui's `asChild` pattern.

```rust
// Default: renders as <button>
DrawerClose { "Close" }

// Custom element: attributes include the preset onclick handler
DrawerClose {
    as: |attributes| rsx! {
        a { href: "#", ..attributes, "Go back" }
    }
}
```

## Overlay

A modal drawer dims the page behind it with the shared overlay scrim: the dialog's own `::backdrop`, the same 10% black and 4px blur on every modal overlay. Pass `overlay: false` to keep the modality (focus trap, inert page, click-outside dismissal) without the dim and the blur; the `<dialog>` then carries `data-dx-overlay="off"`.

Closing keeps the drawer a modal -- in the top layer, at the same size and place -- until its exit animation has played, and only then closes it. That holds in every browser; no stylesheet needs an engine-specific keep-alive.

## Motion

The drawer slides in over `--dx-motion-duration-slow` and out over `--dx-motion-duration-base`; the scrim follows `--dx-overlay-duration`, `--dx-overlay-ease`, `--dx-overlay-scrim` and `--dx-overlay-blur`. While you drag, the panel follows the pointer through an inline `transform` (it composes with the slide, which is the `translate` property, because a held animation would override an inline `translate`), and a release settles with a `--dx-motion-duration-base` transition. The drawer stays open until its exit settles (at most 1500 ms) and only then closes. Reduced motion swaps in `--dx-motion-duration-reduced`. The slide is the theme's shared `dx-edge-in-*`/`dx-edge-out-*` keyframes, selected per side; the drawer sets its own durations through the `--dx-edge-duration-*` and `--dx-edge-ease-*` slots. Nothing loops. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.

## Panel padding

The panel keeps its 16px padding and gap (`--dx-edge-padding`), unlike the sheet, which follows Nova's panel padding of 0 (only the header, footer and body carry the inset). Components that put content straight into a drawer panel rely on that inset; once they pad themselves the drawer can take the recipe default too.
