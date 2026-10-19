The AlertDialog primitive provides an accessible, composable modal dialog for critical user confirmations (such as destructive actions).

## Component Structure

```rust
let mut open = use_signal(|| false);
rsx! {
    button { onclick: move |_| open.set(true), type: "button", "Show Alert Dialog" }
    AlertDialog { open: open(), on_open_change: move |v| open.set(v),
        AlertDialogTitle { "Title" }
        AlertDialogDescription { "Description" }
        AlertDialogActions {
            AlertDialogCancel { "Cancel" }
            AlertDialogAction { on_click: move |_| { /* destructive action */ }, "Confirm" }
        }
    }
}
```

### Components
- **AlertDialog**: Provides context, manages open state, renders the backdrop and the content panel.
- **AlertDialogTitle**: The dialog's heading.
- **AlertDialogDescription**: Additional description for the dialog.
- **AlertDialogActions**: Container for action buttons.
- **AlertDialogAction**: Main action button (e.g., confirm/delete). Closes dialog and calls optional `on_click`.
- **AlertDialogCancel**: Cancel/close button. Closes dialog and calls optional `on_click`.

## Overlay

A modal alert dialog dims the page behind it with the shared overlay scrim: the dialog's own `::backdrop`, the same 10% black and 4px blur on every modal overlay. Pass `overlay: false` to keep the modality (focus trap, inert page, click-outside dismissal) without the dim and the blur; the `<dialog>` then carries `data-dx-overlay="off"`.

Closing keeps the alert dialog a modal -- in the top layer, at the same size and place -- until its exit animation has played, and only then closes it. That holds in every browser; no stylesheet needs an engine-specific keep-alive.

## Motion

An alert dialog is not closed the instant `open` turns false. It stays open until its own exit animation and its backdrop's have settled (at most 1500 ms) and only then calls `close()`, so the exit is always played in full. The scrim and the panel share `--dx-overlay-duration` and `--dx-overlay-ease` (defaults `--dx-motion-duration-slow` and `--dx-motion-ease`). Under `prefers-reduced-motion: reduce` both durations become `--dx-motion-duration-reduced`. Nothing loops. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.

## Initial and final focus

`initial_focus` and `final_focus` take a function of the open modality (`OpenModality::{Keyboard, Mouse, Touch, Pen}`, Base UI's `initialFocus(openType)`) that returns a `FocusTarget`: `Default`, `Panel`, `Selector("#id")` or, for `final_focus`, `None`. They are inputs to the one open-focus policy (`dxOpenFocus` / `dxCloseFocus`, `primitives/src/modality.rs`), not a second code path: the function is called once per modality at render, the answers ride on the panel as `data-dx-initial-focus` / `data-dx-final-focus`, and the policy reads the entry for the modality of the open. A selector that matches nothing falls back to the built-in policy (keyboard and mouse: the first control; touch and pen: the panel, so no software keyboard rises). An explicit target is honoured as given, so return `Panel` for `Touch` yourself when it names a field.

```rust
AlertDialog {
    initial_focus: Callback::new(|m: OpenModality| match m {
        OpenModality::Touch => FocusTarget::Panel,
        _ => FocusTarget::selector("#name"),
    }),
    final_focus: Callback::new(|_| FocusTarget::Default),
}
```

`final_focus` defaults to the element that held focus when the panel opened: the trigger for a click open, the previously focused element for a programmatic open. The opener is remembered in exactly one place (`dxOpenFocus`), for every overlay kind. See the `focus_target` variant.
