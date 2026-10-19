The Trace component is a quiet record of what an agent did in one turn: thinking, a tool call, a file read. A line is one muted row with an icon, a title and an optional detail; a collapsible line expands to show more.

## Component Structure

```rust
Trace {
    TraceLine {
        // Available statuses: Running, Error, Pending (none is a finished step)
        status: TraceStatus::Running,
        TraceIcon { Search {} }
        TraceTitle { "Searching the web" }
        TraceDescription { "3 results" }
    }

    TraceCollapsible {
        TraceCollapsibleTrigger {
            TraceLine {
                TraceIcon { Terminal {} }
                TraceTitle { "Ran cargo check" }
            }
        }
        TraceCollapsibleContent { "Finished dev profile in 8.37s" }
    }
}
```

- `TraceCollapsible` is uncontrolled by default. Pass `open` with `on_open_change` to drive it, for example open while a reasoning step streams and folded when it finishes.
- The detail is unmounted while closed unless you pass `keep_mounted` (or `hidden_until_found` so find-in-page can open it).

## Accessibility

- The trigger is a real `<button>` with `aria-expanded` and `aria-controls`, so Enter and Space toggle it. The chevron and `TraceIcon` are `aria-hidden`; the title carries the meaning.
- `Pending` dims the icon only, so the text keeps its 4.5:1 contrast.
- `Error` is a colour change on top of words: write what failed in the title ("Failed to read config.toml").
- A running step shimmers its title. Put `role: "status"` on the `Trace` if the update should be announced as it happens. The shimmer is paused while off-screen and off under `prefers-reduced-motion`.
- Under RTL the chevron mirrors so it still points toward the detail.
