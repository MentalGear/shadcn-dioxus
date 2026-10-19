The Toast component is used to display brief messages to the user, typically for notifications or alerts. The toast messages can be focused with the keyboard with the `f6` key.

## Component Structure

```rust
// The Toast provider provides the toast context to its children and handler rendering any toasts that are sent.
ToastProvider {
    // Any child component can consume the toast context and send a toast to be rendered.
    button {
        onclick: |event: MouseEvent| {
            // Consume the toast context to send a toast.
            let toast_api = consume_toast();
            toast_api
                .error(
                    "Critical Error".to_string(),
                    ToastOptions::new()
                        .description("Some info you need")
                        .duration(Duration::from_secs(60))
                        .permanent(false),
                );
        },
        "Show Toast"
    }
}
```

## Handles: `update` and `close`

`show` (and `success`, `error`, `warning`, `info`, `loading`) returns a `ToastId`. Keep it to change or dismiss that toast later:

```rust
let id = toast_api.info("Syncing...".to_string(), ToastOptions::new().permanent(true));
toast_api.update(id, ToastUpdate::new().toast_type(ToastType::Success).title("Synced").permanent(false));
toast_api.close(id);
```

The toast is updated in place (same element, same place in the stack). A `ToastId` is an opaque newtype from one monotonic counter: it cannot be built from a number, never repeats, and cannot be mixed up with another toast's. `update` and `close` on a toast that is already closed do nothing (`update` returns `false`), so a late callback can never touch a different toast.

## Loading and promise

`ToastType::Loading` (`toast_api.loading(..)`) shows a spinner and is never auto-dismissed while it is loading; move it on with `update`. `promise` does that for a future:

```rust
toast_api.promise(
    save(),
    PromiseMessages::new("Saving...", |v| format!("{v} saved"), |e| format!("Failed: {e}")),
);
```

One toast goes loading -> success or error in place. Its live region is `role="status"` (polite) and, for an error, `role="alert"` (assertive); the text change is announced once and never a second toast. The future runs to completion even if the component that started it unmounts; if the user closed the toast first the settling update is a no-op.

## Action button

`ToastOptions::new().action("Undo", |_| ..)` adds a real `Button` to the toast. It is in the tab order (F6 focuses the region, Tab reaches it), activates with click, Enter or Space, runs the callback and dismisses the toast. A custom skin reads it with `use_toast_action()` or renders the primitive's `ToastActionButton`.

## Stacking and swipe

Several toasts collapse into a pile; hovering or focusing the stack (or F6) expands it. Under `prefers-reduced-motion: reduce` there is no entrance animation and no restack transition (the pile still expands, instantly).

A toast can be swiped away with the mouse, a finger or a pen (pointer events). It dismisses after 45 px toward the edge it sits at, or a flick (at least 10 px at 0.11 px/ms or more over the last 100 ms); a shorter, slower drag springs back, and a drag the other way is resisted. The direction is the screen edge nearest the toast, so it follows wherever the stylesheet puts the toaster and flips in RTL. A press on the close or action button is a click, never a swipe. While dragging, the toast carries `data-swiping="true"` and `--toast-swipe-x`, which the stylesheet maps to `translate` (not `transform`, so the stack's `scale` is kept).

## Motion and visibility

A toast's auto-dismiss is a timer the user is waiting on, so it follows the tab and nothing else. While the tab is hidden (another tab, a minimized window) the countdown is dropped, and when you come back it starts over with the full duration, so a toast you could not see is still there. Scrolling the page never changes how long a toast stays. The visibility-gated timers for decoration (spinners, tickers, autoplay) are the opposite: they stop when merely scrolled away. See `dioxus_primitives::activity::use_document_visible`.

The slide-in and restack transitions use `--dx-motion-duration-slow`; `duration` and `permanent` on the toast control how long it stays. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.
