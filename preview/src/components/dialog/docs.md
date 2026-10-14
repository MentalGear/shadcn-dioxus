The dialog component can be used to display additional information or actions related to an item. It is a simple modal dialog that can be opened and closed by the user.

## Component Structure

```rust
// The dialog component must wrap all dialog elements.
Dialog {
    // The open prop determines if the dialog is currently open or closed.
    open: open(),
    // Header: the title and description in a column (shadcn's DialogHeader).
    DialogHeader {
        DialogTitle { "Item information" }
        DialogDescription { "Here is some additional information about the item." }
    }
    // Footer: the action row, a muted band across the foot of the panel.
    DialogFooter {
        DialogClose {
            r#as: |attributes| rsx! { Button { variant: ButtonVariant::Outline, attributes, "Cancel" } },
        }
        Button { r#type: "submit", "Save changes" }
    }
}
```

## Close button

`Dialog` renders its own close button, an `X` at the top end of the panel, as shadcn's `DialogContent` does (`showCloseButton`, default on). It is the real `Button` (`ghost`, `icon-sm`) named "Close", inside a wrapper that only places it. Pass `show_close_button: false` to leave it out (Escape and a click on the scrim still dismiss the dialog), as in the "No close button" example.

The button is the **last** child of the panel, in shadcn's DOM order. A dialog opened from the keyboard focuses its first control (see `use_dialog_open_driver`), so a dialog with a form lands on the first field and never on the X; a dialog with no other control lands on the X. A pointer or touch open focuses the panel itself.

`DialogFooter { show_close_button: true }` appends an outline "Close" button to the footer, as shadcn's. `DialogClose` is the close behaviour on its own, shadcn's `DialogClose`: a `<button type="button">` that closes the enclosing dialog, or any element through `r#as` (the attributes carry the click handler), the way `SheetClose` takes `as`.

## Header and footer

`DialogHeader` is a column with a `0.5rem` gap. `DialogFooter` cancels the panel's padding (`-mx-4 -mb-4`) to draw a `muted/50` band with a top border and the panel's bottom corners; it is a reversed column below `sm` (the primary action on top) and a row aligned to the end from `sm` up. A title and description placed directly in the panel (no header) keep the tighter pull-back of the previous release.

## Overlay

A modal dialog dims the page behind it with the shared overlay scrim: the dialog's own `::backdrop`, the same 10% black and 4px blur on every modal overlay. Pass `overlay: false` to keep the modality (focus trap, inert page, click-outside dismissal) without the dim and the blur; the `<dialog>` then carries `data-dx-overlay="off"`.

Closing keeps the dialog a modal -- in the top layer, at the same size and place -- until its exit animation has played, and only then closes it. That holds in every browser; no stylesheet needs an engine-specific keep-alive.

## Motion

A dialog is not closed the instant `open` turns false. It stays open (in the top layer, at the same size and place) until its own exit animation and its backdrop's have settled, at most 1500 ms, and only then calls `close()`, so any exit animation you write on the dialog or its `::backdrop` is played in full. The scrim and the panel share `--dx-overlay-duration` and `--dx-overlay-ease` (defaults `--dx-motion-duration-slow` and `--dx-motion-ease`); retune them in one place, or pass `overlay: false` for no scrim. Keep exit keyframes to `transform`, `translate` and `opacity`. Under `prefers-reduced-motion: reduce` both durations become `--dx-motion-duration-reduced`. Nothing here loops, so there is nothing to pause while unseen. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.
