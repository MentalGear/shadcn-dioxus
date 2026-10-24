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
            r#as: |el: AsChild| rsx! { Button { variant: ButtonVariant::Outline, attributes: el.attributes, {el.children} } },
            "Cancel"
        }
        Button { r#type: "submit", "Save changes" }
    }
}
```

## Close button

`Dialog` renders its own close button, an `X` at the top end of the panel, as shadcn's `DialogContent` does (`showCloseButton`, default on). It is the real `Button` (`ghost`, `icon-sm`) named "Close", inside a wrapper that only places it. Pass `show_close_button: false` to leave it out (Escape and a click on the scrim still dismiss the dialog), as in the "No close button" example.

The button is the **last** child of the panel, in shadcn's DOM order. A dialog opened from the keyboard focuses its first control (see `use_dialog_open_driver`), so a dialog with a form lands on the first field and never on the X; a dialog with no other control lands on the X. A pointer or touch open focuses the panel itself.

`DialogFooter { show_close_button: true }` appends an outline "Close" button to the footer, as shadcn's. `DialogClose` is the close behaviour on its own, shadcn's `DialogClose`: a `<button type="button">` that closes the enclosing dialog, or any element through `r#as` (its `AsChild` carries the click handler in `attributes` and the label in `children`), the way `SheetClose` takes `as`.

## Header and footer

`DialogHeader` is a column with a `0.5rem` gap. `DialogFooter` cancels the panel's padding (`-mx-4 -mb-4`) to draw a `muted/50` band with a top border and the panel's bottom corners; it is a reversed column below `sm` (the primary action on top) and a row aligned to the end from `sm` up. A title and description placed directly in the panel (no header) keep their own, tighter pull-back.

## Overlay

A modal dialog dims the page behind it with the shared overlay scrim: the dialog's own `::backdrop`, the same 10% black and 4px blur on every modal overlay. Pass `overlay: false` to keep the modality (focus trap, inert page, click-outside dismissal) without the dim and the blur; the `<dialog>` then carries `data-dx-overlay="off"`.

Closing keeps the dialog a modal -- in the top layer, at the same size and place -- until its exit animation has played, and only then closes it. That holds in every browser; no stylesheet needs an engine-specific keep-alive.

## Motion

A dialog is not closed the instant `open` turns false. It stays open (in the top layer, at the same size and place) until its own exit animation and its backdrop's have settled, at most 1500 ms, and only then calls `close()`, so any exit animation you write on the dialog or its `::backdrop` is played in full. The scrim and the panel share `--dx-overlay-duration` and `--dx-overlay-ease` (defaults `--dx-motion-duration-slow` and `--dx-motion-ease`); retune them in one place, or pass `overlay: false` for no scrim. Keep exit keyframes to `transform`, `translate` and `opacity`. Under `prefers-reduced-motion: reduce` both durations become `--dx-motion-duration-reduced`. Nothing here loops, so there is nothing to pause while unseen. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.

## Initial and final focus

`initial_focus` and `final_focus` take a function of the open modality (`OpenModality::{Keyboard, Mouse, Touch, Pen}`, Base UI's `initialFocus(openType)`) that returns a `FocusTarget`: `Default`, `Panel`, `Selector("#id")` or, for `final_focus`, `None`. They are inputs to the one open-focus policy (`dxOpenFocus` / `dxCloseFocus`, `primitives/src/modality.rs`), not a second code path: the function is called once per modality at render, the answers ride on the panel as `data-dx-initial-focus` / `data-dx-final-focus`, and the policy reads the entry for the modality of the open. A selector that matches nothing falls back to the built-in policy (keyboard and mouse: the first control; touch and pen: the panel, so no software keyboard rises). An explicit target is honoured as given, so return `Panel` for `Touch` yourself when it names a field.

```rust
Dialog {
    initial_focus: Callback::new(|m: OpenModality| match m {
        OpenModality::Touch => FocusTarget::Panel,
        _ => FocusTarget::selector("#name"),
    }),
    final_focus: Callback::new(|_| FocusTarget::Default),
}
```

`final_focus` defaults to the element that held focus when the panel opened: the trigger for a click open, the previously focused element for a programmatic open. The opener is remembered in exactly one place (`dxOpenFocus`), for every overlay kind. See the `focus_target` variant.

## Non-modal and pointer dismissal

`is_modal: false` renders a plain panel beside the page (no top layer, no trap, no inert page). It reports `role="dialog"` but no `aria-modal`: the attribute is derived from the arm in one helper, so only a modal panel ever tells assistive tech the page behind it is inert. By default a press or focus outside the panel dismisses it. `disable_pointer_dismissal: true` (Base UI's `disablePointerDismissal`) turns that off: an outside press only acts on the page (on a modal, a backdrop click no longer closes it). Escape and the dialog's own close controls still close it.
