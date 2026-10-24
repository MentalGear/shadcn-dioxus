The message scroller is the scroll container for a chat transcript. It owns *scroll* and nothing else: where the thread opens, whether it follows a reply that is still arriving, where a new turn lands, keeping your place while older messages load above you, and a button to get back to the latest message. It does not own messages, models, transport or persistence; the rows are whatever you put inside a `MessageScrollerItem` (here, `Message`, `Bubble` and `Marker`).

It is a port of shadcn/ui's `message-scroller`. The one rule behind every behavior: **never move the reader against their intent.**

## Component Structure

```rust
MessageScrollerProvider { auto_scroll: true,
    MessageScroller {
        MessageScrollerViewport {
            MessageScrollerContent {
                for message in messages() {
                    MessageScrollerItem {
                        key: "{message.id}",
                        message_id: "{message.id}",
                        scroll_anchor: message.is_user,
                        Message { /* ... */ }
                    }
                }
            }
        }
        MessageScrollerButton {}
    }
}
```

`MessageScroller` fills its parent, so place it in a container with a fixed or constrained height.

- `MessageScrollerProvider` is the headless root. It owns the scroll controller and the behavior props, and renders no element.
- `MessageScroller` is the frame. It lays out the viewport and the jump button.
- `MessageScrollerViewport` is the scrolling element. It is `role="region"` with `aria-label="Messages"` and `tabindex="0"`, so keyboard users can scroll it.
- `MessageScrollerContent` holds the rows. It is `role="log"` with `aria-relevant="additions"`: a new row is announced, token-by-token text growth inside a row is not.
- `MessageScrollerItem` wraps every direct child of the content. A row can be a message, a marker, a typing indicator, a separator, or a "load earlier" row.
- `MessageScrollerButton` scrolls to the end (or, with `direction: MessageScrollerDirection::Start`, the start).

## Following the live edge

With `auto_scroll`, the transcript follows new content while the reader is at the live edge. Scrolling away releases it, whether by the wheel, touch, the scroll keys (arrows, Page Up/Down, Home/End, Space) or by dragging the scrollbar, and so does an explicit jump to a message. New chunks then arrive without moving the reader. The jump button, or `scroll_to_end`, re-engages following.

Following is off by default: nothing moves unless you ask it to.

## Anchoring a turn

Mark the row that starts a turn with `scroll_anchor` (usually the user's message). When a new anchor is appended, the viewport scrolls it near the top, keeping `scroll_previous_item_peek` pixels of the previous row visible above it, and the reply streams into the room below. Once the reply fills the viewport the reader is back at the live edge and `auto_scroll` takes over. Anchoring is not tied to a role: any row can be an anchor.

## Opening a saved thread

`default_scroll_position` picks where the transcript opens on its first non-empty render: `End` (the default), `Start`, or `LastAnchor`: the user's latest message with its reply below it, falling back to the end when that turn already fits.

The prop is live. Change it on a mounted provider (it is a `ReadSignal`, so a signal or a plain value both work) and the transcript applies the new opening position to the thread that is already on screen, with no remount; the `opening_position` example does exactly that from its tabs. Only the first paint is guarded (below): a later change never hides the viewport again.

### Avoiding a flash on load

A scroll container always opens at the top, so a server-rendered transcript would show its oldest messages and then jump. When the position is `End` or `LastAnchor`, the root and viewport render `data-pending-scroll` (identically on the server and on the hydrating client) and the styled viewport stays hidden until the position is applied. The attribute is removed for an empty transcript too, and a few seconds after load as a failsafe, so a transcript can never stay hidden. The guard covers the first paint only and never hides the viewport again, not even when `default_scroll_position` later changes.

## Loading earlier messages

Older messages prepended above the current ones do not move what you are reading: the viewport keeps the first visible row where it is (`preserve_scroll_on_prepend`, on by default). Give rows a stable `message_id`, so the scroller has a specific row to hold.

## Virtualization

A long thread should not lay out and paint thousands of rows that nobody can see. `virtualize` on the provider picks how the scroller keeps off-screen rows cheap:

```rust
MessageScrollerProvider { virtualize: Virtualization::ContentVisibility, /* ... */ }
```

- `Virtualization::None`: every row is fully rendered all the time. Use it for short threads, or when a row's content cannot tolerate being skipped.
- `Virtualization::ContentVisibility` (the default): rows stay in the DOM and the browser skips layout and paint for the ones that are off-screen (`content-visibility: auto`, with `contain-intrinsic-size: auto` so it remembers each row's last rendered height). Because the rows are real DOM, **the accessibility tree and text selection keep reaching every message**, including the ones scrolled far out of view, and so does the browser's find-in-page where the engine supports it (see below). The scroller never skips three groups of rows: the rows at the live edge (so the end geometry is exact while a reply streams), the row that holds focus, and the rows that hold the ends of the current selection. It marks them with `data-keep-rendered`, which the stylesheet excludes from skipping. Tune the first-render height guess with the `--dx-message-scroller-row-estimate` custom property (default `10rem`; a chunk's guess is its rows' guesses plus the gaps between them).

**Long threads: render the rows through `MessageScrollerRows`.** The browser does per-frame work for every element it may skip, so skipping each row separately costs main-thread time proportional to the row count on every scroll frame: 20.3 ms per frame for the 2,000-row example below (release build, headless Chromium, 4-core VM), against a 16.7 ms frame budget. `MessageScrollerRows` groups the rows in chunks of 20 and the browser skips each chunk as one element, which holds the same scroll to 3.7 ms per frame (measured on a heavily loaded machine, which includes the scroller's own controller work; the spec asserts under 8 ms on a release build). Use it in place of a `for` loop of `MessageScrollerItem`s; the scroller treats the chunks as transparent, so following the stream, anchoring, prepend preservation and jumping to a message work as before, and the chunk holding the live edge, focus or an end of the selection is the one that stays rendered:

```rust
MessageScrollerContent {
    MessageScrollerRows {
        count: messages.len(),
        render_row: move |i: usize| rsx! {
            MessageScrollerItem { key: "{messages[i].id}", message_id: "{messages[i].id}",
                /* the row */
            }
        },
    }
}
```

Chunks are grouped by absolute position, so when you prepend older messages, lower `start` by the number prepended (`MessageScrollerRows { start: -(prepended as isize), .. }`, accumulating) and every existing row stays in its chunk and keeps its DOM node. Rows written directly as children of the content still work and keep the per-row behaviour: fine for a few hundred rows.

**Find-in-page support.** Skipped content stays findable in Chromium and in Firefox 125 and later, and in Safari from Safari 26. In Safari 18 through 25 the browser's find-in-page does **not** find text inside a skipped (off-screen) row (WebKit bug 283846): there the text is still in the DOM, the accessibility tree and any selection, but Cmd+F will not match it until the row has been scrolled into view. If your users depend on find-in-page across a long thread on those versions, use `Virtualization::None`, or give them a search of your own built on `scroll_to_message`.

Mounting only a window of rows is a possible future option, but it is not part of this API and not planned for now: unmounted rows are not in the DOM, so native find-in-page, screen-reader history and a selection that spans messages could not reach them, and an app would have to provide its own search. `ContentVisibility` keeps the DOM, the accessibility tree and selection whole (and find-in-page where supported, as above), and is the right size for hundreds to low thousands of turns.

The `long` example below keeps 2,000 rows in the DOM, rendered through `MessageScrollerRows`: only the chunks in view are laid out, and following a reply still lands exactly at the end. In browsers that support it, find-in-page still locates text in a row far off-screen.

## Commands

`use_message_scroller()` returns commands you can call from any descendant of the provider:

```rust
let scroller = use_message_scroller();
scroller.scroll_to_end(ScrollBehavior::Smooth);
scroller.scroll_to_start(ScrollBehavior::Auto);
scroller.scroll_to_message("m42", ScrollToMessageOptions::default());
scroller.scroll_to_message(
    "m42",
    ScrollToMessageOptions {
        align: ScrollAlign::Center,
        behavior: ScrollBehavior::Instant,
        scroll_margin: Some(24.0),
    },
);
```

`ScrollBehavior` is `Auto` (jump, or follow the viewport's CSS `scroll-behavior`), `Instant` (jump now whatever that CSS says) or `Smooth` (animated; `Smooth` becomes `Auto` under `prefers-reduced-motion`). `scroll_margin` is the margin on the aligned edge for that one call; `None` uses the provider's `scroll_margin`. `align` is `Start`, `Center`, `End` or `Nearest`.

**The commands are fire-and-forget and return nothing.** They travel to the scroll controller over a one-way channel and run there on a later turn of the event loop, so a `bool` ("did that row exist?") could only be a guess made before the answer exists. A return value that can lie is worse than none, so there is none. Observe the outcome instead: `use_message_scroller_visibility` lists the target among the visible rows (and makes its turn current) once the jump has landed. A jump to an id that is not in a populated transcript does nothing; in an empty transcript it is queued until rows mount and replaces the opening position.

## Reading scroll state

`use_message_scroller_scrollable()` is a signal of which edges still have content (`Scrollable { start, end }`), fed by a message only when the value changes, never per scroll event or per token. While you are following the stream, `end` stays `false`, so a "new messages" affordance does not flicker on every chunk.

## Tracking the reader's position

`use_message_scroller_visibility()` is a signal of `Visibility { current_anchor_id, visible_message_ids }`, for a table of contents, an "active turn" highlight or a "you are here" outline:

```rust
let visibility = use_message_scroller_visibility();
let current = visibility().current_anchor_id; // Option<String>: the turn being read
```

- `current_anchor_id` is the `message_id` of the last `scroll_anchor` row whose top has reached the reading line: the top of the viewport plus `scroll_margin` plus `scroll_previous_item_peek`. It stays set after that turn scrolls off the top, so a table of contents does not flicker between turns; it is `None` before the first anchor is reached.
- `visible_message_ids` lists, in document order, the rows with a `message_id` that are at least partly inside the viewport and below the reading line. A row hidden under the previous-turn peek is not listed, which matches what the reader can read.
- It costs nothing until used. The controller starts an `IntersectionObserver` only when the first component calls the hook and stops it when the last one unmounts, and it reports only when the value changes, never per scroll event or per streamed token. A transcript that never calls the hook runs no observer.
- The `visibility` example shows a header that follows the turn being read.

## Animating new messages

Entrances are plain CSS, not a scroller feature: put an entrance class on the row's own element (the `Message` inside the `MessageScrollerItem`) and it plays once when the row mounts. `dx-effects.css` ships three, ported from shadcn's presets, next to the theme's `dx-fade-in`, `dx-pop-in` and `dx-drop-in`:

- `dx-enter-spring-bounce`: rises 12px and settles from a 0.96 scale with a spring overshoot (a CSS `linear()` easing; no script).
- `dx-enter-blur-fade`: fades in from a 4px blur while rising 6px.
- `dx-enter-slide-side`: slides in 18px from the inline-end edge (flipped in RTL; the distance is `--dx-enter-slide-x`).

They animate only `opacity`, `scale`, `translate` and `filter` (never `transform`, and never height, margin or padding), so the row's measured height is identical on every frame and turn anchoring keeps working while it plays. They play only under `prefers-reduced-motion: no-preference`; with reduced motion the row appears at once. Store the preset with the row when it is sent (as the `animation` example does) so changing it later does not replay rows already on screen. `slide-up` and `scale-fade` are not shipped: the theme's `dx-drop-in` and `dx-fade-in` cover them.

## Styling

The root and viewport carry these attributes, written by the controller rather than by Rust, so a later render never overwrites them:

- `data-scrollable`: `"start"`, `"end"`, `"start end"`, or absent.
- `data-autoscrolling`: present while a programmatic scroll to the end runs.
- `data-pending-scroll`: present until the opening position is applied.

The button carries `data-active` (`"true"` when there is content in its direction) and is `inert` otherwise.

## Edge fade

`MessageScrollerViewport` takes `fade: ScrollFade`, which puts the matching `dx-scroll-fade-*` class on the viewport, the element that actually scrolls (never the root frame). The default is `ScrollFade::Bottom`, Nova's end-edge hint; `ScrollFade::Top` suits a thread that opens at its newest messages and loads history upward, `ScrollFade::Y` fades both edges and `ScrollFade::None` adds nothing.

```rust
MessageScrollerViewport { fade: ScrollFade::Top,
    MessageScrollerContent { .. }
}
```

- The fade is a CSS scroll-driven mask on the viewport (see the `dx-scroll-fade` utility in the ScrollArea docs), so it follows the viewport's scroll offset: it is unaffected by `content-visibility` rows and by prepended history, and a transcript that fits has no fade. A browser without scroll-driven animations gets a static edge fade.
- It needs `dx-effects.css` linked once at the app root next to the theme; `message_scroller`'s `component.json` lists it in `globalAssets`, so `shadcn-dioxus add message_scroller` copies it. It cannot live in this component's `style.css`: the file also holds the `@property` registrations and keyframes every scroller shares, and a second copy would be a second set of declarations to keep in step.
- The mask also dissolves the viewport's own border and background, so keep those on a wrapper (the demos do) and let the viewport be the inner element.
- Examples: the `fade` variant (history above fades; the buttons switch edge and prepend earlier messages).

## Notes

- Rows are always real elements (see Virtualization), so the accessibility tree sees the whole transcript, and find-in-page does too in browsers that search skipped content.
- The styled viewport has a bottom edge fade from `dx-effects.css` (the `fade` prop's default, `ScrollFade::Bottom`). Without that file the fade is simply absent. See Edge fade below.
- On a renderer without `document::eval` (native/Blitz) the shell renders as plain overflow: no auto-follow, no flash guard, and an inert button.

## Motion

The jump button slides, scales and fades with `--dx-motion-duration-slow` when it shows and a literal `0.4s` when it hides; the transcript is revealed once its scroll position has been applied (the flash guard). Reduced motion swaps in `--dx-motion-duration-reduced`. Nothing loops, so nothing is paused while the scroller is unseen. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.

## Not reproduced from shadcn/ui

Not reproduced: the `slide-up` and `scale-fade` animation presets as separate classes (the theme's `dx-drop-in` and `dx-fade-in` are the same motion), and a boolean return from the scroll commands (they are fire-and-forget by design; see Commands). A composer whose actions sit in a full-width bottom row is `InputGroupAlign::BlockEnd` (see the Input Group docs).

## Agent chat

The Agent Chat demo is upstream's `chat-demo` agent turn in a scroller: a reasoning `Trace` that is open and shimmering while it runs, a tool-call line, an answer with a `CodeBlock`, and a `Checkpoint` after the question whose Restore drops the answer.
