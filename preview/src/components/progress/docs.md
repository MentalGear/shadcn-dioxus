The Progress component is used to display the progress of a task or operation. It can be used to indicate loading states, file uploads, or any other process that takes time to complete.

## Component Structure

```rust
Progress {
    // The current progress value (0 to max
    value: 0.5,
    // The maximum value of the progress (default is 100.0)
    max: 1.0,
    // Optional: a ProgressLabel and a ProgressValue, shown above the track
    {children}
}
```

## Label and value

`ProgressLabel` is the visible name and `ProgressValue` the percent readout, as in shadcn. Both go inside `Progress`: the root is a flex-wrap row, so the label sits at the inline start and the value at the end of the first line, above the full-width track. The bar is named by the label automatically (`aria-labelledby`, set once the label mounts; your own `aria_label`/`aria_labelledby` on `Progress` wins). `ProgressValue` renders `value / max` as a whole-number percent (`56%`), nothing while the progress is indeterminate, and is `aria-hidden` because `aria-valuenow` already carries it.

```rust
Progress { value: 56.0,
    ProgressLabel { "Upload progress" }
    ProgressValue {}
}
```

The track is its own element (`.dx-progress-track`, 4px, rounded) inside the root, so a `Progress` without a label or value is just the bar.

## Motion

Only the indeterminate bar loops: a `translateX` sweep, 1 s, forever. It is paused whenever the bar is off-screen or the tab is hidden (`dioxus_primitives::activity::use_motion_when`); a determinate bar subscribes to nothing. The determinate fill animates `width` for 250 ms, a literal in `progress/style.css`.

To change them, override `animation` on `.dx-progress-indicator` (indeterminate) or its `transition` (determinate). Reduced motion is handled for the indeterminate bar only: under `prefers-reduced-motion: reduce` the sweep stops (the theme's one rule for every looping component) and `progress/style.css` draws the full track in diagonal stripes instead of a still half-width bar that would read as 50%. The determinate fill's 250 ms width transition has no reduced-motion rule; add one to your copy if you need it. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.
