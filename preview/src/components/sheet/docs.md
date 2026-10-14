The sheet component is a panel that slides in from the edge of the screen. It can be used to display additional content, forms, or navigation menus without leaving the current page.

## Component Structure

```rust
Sheet {
    open: open(),
    // Which edge to slide in from. Available sides: Top, Right (default), Bottom, Left.
    side: SheetSide::Right,
    SheetContentClose {}
    SheetHeader {
        SheetTitle { "Edit profile" }
        SheetDescription { "Make changes to your profile here. Click save when you're done." }
    }
    SheetFooter {
        SheetClose { "Close" }
    }
}
```

## One edge panel with the drawer

`side` is a typed prop. `SheetSide` is an alias of `EdgeSide`, the one side type the drawer uses too (`DrawerSide` is its other alias), and the panel is the theme's "Edge panel" recipe in `dx-components-theme.css` (`data-dx-surface="edge"` plus the resolved `data-side`): the `<dialog>` reset, the per-side inset, size, border and slide are written once for both components. A sheet defaults to `Right`, as shadcn's does; `SheetSide::Auto` is available (bottom below the `md` breakpoint, right at and above it) but not the default. The old `"data-side"` attribute still works and overrides the prop (`Sidebar` still passes it).

Sheet follows Nova's panel padding of 0: the header, footer and body carry the inset (16px), not the panel -- so a title sits one header inset from the edge, not two.

## SheetClose with `as` prop

The `as` prop allows you to render a custom element while preserving the close behavior, similar to shadcn/ui's `asChild` pattern.

```rust
// Default: renders as <button>
SheetClose { "Close" }

// Custom element: attributes include the preset onclick handler
SheetClose {
    as: |attributes| rsx! {
        a { href: "#", ..attributes, "Go back" }
    }
}
```

## Alignment with shadcn/ui v4

This component's `style.css` was checked line-by-line against shadcn/ui v4's
`Sheet` (`SheetOverlay`/`SheetContent`/`SheetHeader`/`SheetFooter`/
`SheetTitle`/`SheetDescription`/`SheetPrimitive.Close`) and brought in line
with it, translated onto this repo's own design tokens rather than copying
Tailwind classes verbatim:

- **Overlay**: `fixed inset-0 bg-black/50` in shadcn; here the shared overlay
  scrim every modal uses. On the web build, `.dx-sheet` is a real `<dialog>`
  (`primitives/src/dialog.rs`'s modal web arm), so the scrim is its native
  `::backdrop` pseudo-element; the non-web (Blitz) arm has no such element,
  so `.dx-sheet-root` paints the same scrim for that arm.
- **Content**: Nova's `.cn-sheet-content`: `fixed z-50 flex flex-col gap-4 bg-popover text-sm shadow-lg`,
  sliding in from its `data-side` with `transition ease-in-out`, a 500ms
  open / 300ms close duration (this repo's motion scale has no exact 500ms
  step, so that one value is a literal; 300ms matches
  `--dx-motion-duration-slower` exactly). `right`/`left` are
  `inset-y-0 h-full w-3/4 border-l|border-r`, capped at `sm:max-w-sm`
  (384px) only from a 640px viewport up, matching shadcn's own
  breakpoint-gated cap rather than applying it unconditionally.
  `top`/`bottom` are `inset-x-0 h-auto border-b|border-t`.
- **Header/Footer/Title/Description**: `flex flex-col gap-0.5 p-4` /
  `mt-auto flex flex-col gap-2 p-4` / `text-foreground text-base font-medium` /
  `text-muted-foreground text-sm` (Nova's `.cn-sheet-*`). Tailwind's preflight
  resets heading sizes, which this repo has no equivalent of, so the title
  size is an explicit `--dx-text-base`.
- **Close button**: Nova's ghost `icon-sm` Button at `absolute top-3 right-3`:
  a 28px square with `min(radius-md, 12px)` corners, a 16px icon, a `muted`
  hover fill and the Button focus ring, with an `aria-label` (this repo's
  equivalent of shadcn's visually-hidden "Close" span).
- **Native `<dialog>` centering defect (shared with Drawer)**: Chromium's
  `dialog:modal` UA stylesheet ships `inset: 0; margin: auto; width/height:
  fit-content`. Overriding only the one edge a given side needs (e.g.
  `right: 0`) used to leave the UA's own `inset: 0` still supplying the
  *opposite* edge uncontested, over-constraining the box against a definite
  `width`/`height` -- the UA resolved that by centering the panel into its
  own auto margins rather than anchoring it (confirmed live: a right/left
  sheet rendered as a floating centered card, and a top/bottom sheet
  stretched to the full viewport height instead of sizing to its content).
  Fixed by resetting `margin`/`inset`/`width`/`height` to a known, non-auto
  baseline in the theme's shared Edge panel recipe (for Sheet and Drawer alike) and letting each `[data-side]` rule reopen
  (set back to `auto`) exactly the one edge it doesn't pin.

## Overlay

A modal sheet dims the page behind it with the shared overlay scrim: the dialog's own `::backdrop`, the same 10% black and 4px blur on every modal overlay. Pass `overlay: false` to keep the modality (focus trap, inert page, click-outside dismissal) without the dim and the blur; the `<dialog>` then carries `data-dx-overlay="off"`.

Closing keeps the sheet a modal -- in the top layer, at the same size and place -- until its exit animation has played, and only then closes it. That holds in every browser; no stylesheet needs an engine-specific keep-alive.

## Motion

The sheet slides in from its edge over 500 ms and out over `--dx-motion-duration-slower`; the durations are the `--dx-edge-duration-enter/-exit` slots' defaults in the theme's Edge panel recipe, and the scrim follows `--dx-overlay-duration`, `--dx-overlay-ease`, `--dx-overlay-scrim` and `--dx-overlay-blur`. The sheet stays open until its exit settles (at most 1500 ms) and only then closes, so the slide-out is always played. Its keyframes are the shared `dx-edge-in-*`/`dx-edge-out-*` pair in the theme, on `translate` like every other keyframe (no `transform`, so nothing is on the `check-anchored-keyframes.sh` allowlist). Reduced motion swaps in `--dx-motion-duration-reduced`. Nothing loops. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.
