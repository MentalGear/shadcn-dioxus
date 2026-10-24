The Toast component is used to display brief messages to the user, typically for notifications or alerts. The toast messages can be focused with the keyboard with the `f6` key: one handler per page, however many providers it mounts. It goes to the region of the most recent toast, pressing it again cycles through the other regions that hold toasts (wrapping), and with no toast anywhere it does nothing. Providers only register their region with that handler; they never listen themselves.

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

A toast can be swiped away with the mouse, a finger or a pen (pointer events). It dismisses after 45 px toward the edge it sits at, or a flick (at least 10 px at 0.11 px/ms or more over the last 100 ms); a shorter, slower drag springs back, and a drag the other way is resisted. The direction is the screen edge nearest the toast, so it follows wherever the stylesheet puts the toaster and flips in RTL. A drag can also go vertical, toward the nearest top or bottom edge (up for a toaster at the top, down for one at the bottom); the axis is decided by the first few pixels of the drag, and the toast moves along that axis only. A press on the close or action button is a click, never a swipe. Toasts set `touch-action: none`, so a finger that starts on a toast drags the toast, not the page. While dragging, the toast carries `data-swiping="true"` and `--toast-swipe-x` or `--toast-swipe-y`, which the stylesheet maps to `translate` (not `transform`, so the stack's `scale` is kept).

## Height follows the content

A toast is as tall as its content plus a constant padding (16 px on every side, one line or many): it has no fixed height and clips nothing. The skin marks the box that holds the content with `data-toast-body` (the preview skin's `.dx-toast-body`); the primitive measures it with one `ResizeObserver` and writes the toast's own height as `--toast-height` on its list item, and the front toast's as `--front-toast-height` on the list, so the stylesheet can do the pile's arithmetic, which CSS cannot read from the DOM.

- Collapsed, every toast behind the front one takes the front toast's height and hides its content, so the pile looks even whatever each holds (Sonner's `--front-toast-height`); only its edge peeks out, one gap above the toast in front.
- Expanded (hover or focus), each toast has its own height, shows its content again and the toasts are one gap apart, never overlapping.
- A toast that leaves or arrives takes its own height plus the gap from the pile (expanded) or only the gap (collapsed), so the others glide into the freed room whatever its height.
- A skin with no `data-toast-body` is never measured: its toasts keep their auto height and are not overlapped.

The icon (a loading toast's spinner) and the action button are centred against the content, as in Sonner; the close button stays at the top of the toast, level with the first line. `playwright/toast-height.spec.ts` covers the fit, the collapsed and the expanded pile.

## Pause on hover and focus

While the pointer is over the toaster (mouse or pen; a touch tap does not count) or focus is inside it, every countdown waits, and it resumes with the time that was left when you move away or focus leaves (WCAG 2.2.1, timing adjustable). It is the same condition that expands the pile (`:hover`, `:focus-within`), so what you can see expanded is what is held. Loading and permanent toasts have no countdown to pause.

## Enter, exit and the visible cap

The pile shows at most `visible_toasts` toasts (default 3, Sonner's `visibleToasts`); a toast past the cap stays mounted but carries `data-visible="false"`, which keeps it transparent and inert, on hover and focus too, until a slot frees. `max_toasts` still bounds how many are kept at all.

Closing a toast (close button, action, countdown, `Toasts::close`) does not unmount it at once: it gets `data-removed="true"`, becomes `inert`, stops counting toward the pile and unmounts once a 200 ms exit has finished (plus a short slack), while the stylesheet fades it out and slides it toward the toaster's edge. Its list item takes the same `data-removed="true"` and leaves the layout at once, so the toasts that stay close the gap smoothly while it fades and none of them jumps when it unmounts; a new toast's slot grows in the same way as it enters. The 200 ms is a constant in the primitive, written as `--toast-exit-ms`, which the exit transitions read, so the two cannot drift. A swipe goes through the same close: the toast flies out sideways while its slot shrinks.

Every direction comes from one place, the toaster's side (`--toast-edge` on the container: `1` at the bottom, `-1` for `data-toaster-side="top"`):

- a brand-new toast enters from the toaster's own edge (from below at the bottom, from above at the top) and a closed one leaves toward it;
- with the pile expanded, a queued toast that becomes visible because another closed appears at the far end of the pile: it slides in from the opposite side (from above for a bottom toaster) with a fade. Collapsed, it only fades in.

`playwright/toast-enter-exit.spec.ts` covers both entry cases, the exit, and the cap.

## Motion and visibility

A toast's auto-dismiss is a timer the user is waiting on, so it follows the tab and nothing else. While the tab is hidden (another tab, a minimized window) the countdown is dropped, and when you come back it starts over with the full duration, so a toast you could not see is still there (a pause by hover or focus keeps the remaining time instead). Scrolling the page never changes how long a toast stays. The visibility-gated timers for decoration (spinners, tickers, autoplay) are the opposite: they stop when merely scrolled away. See `dioxus_primitives::activity::use_document_visible`.

The entrance keyframes animate `translate` (and `scale`), never `transform`, so the stack's own `transform: scale()` stays in force underneath. The slide-in and restack transitions use `--dx-motion-duration-slow`; `duration` and `permanent` on the toast control how long it stays. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.
