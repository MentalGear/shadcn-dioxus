---
id: 535
title: "The dialog family gets a lightbox fly-in: the clicked item grows into the modal's media and shrinks back, proven on the photo wall"
status: "BACKLOG, approved by the owner 2026-11-19 (\"yes\"). Not started. After port row 456 (the order of…"
size: "M (inferred): four S to M parts"
area: primitives
updated: 2026-11-19
---

# Row 535: a lightbox fly-in from the clicked item

## Item

The modal dialog family offers a lightbox motion: the clicked item (a photo in the virtual grid) appears to grow into
the media inside the open dialog, and on close the media shrinks back to the item. Today the dialog fades and scales from
0.95 around its own centre, whatever was clicked. The effect belongs in the library (owner 2026-11-19, asked "Does the
effect belong in the library at all?": "yes"). The one construction is below.

## Status / trigger

BACKLOG, approved by the owner 2026-11-19 ("yes"). Not started. After port row 456 (the order of 2026-11-19, option A), which rewrites the exit path its close half builds on; waits for the owner's go.

**Ordering evidence.** The close half builds on the dialog's exit path (`use_animated_open`, `primitives/src/lib.rs:897`),
which open [row 456](./456-port-3-ov-overlay-core-escape-dismiss-open-focus.md) rewrites (it owns the exit wait and
`use_animated_open`'s two placements). The row is best built after 456 lands, or with its exit half waiting for it; the
order is the owner's.

## Size

M (inferred): four S to M parts

## Detail

**Size parts.** Rect math and media-pose recipe (S), theme recipe (S), photo-wall wiring (S), specs (S to M).

**Prior art.** No existing row or note covers a shared-element transition, a lightbox, a zoom or hero transition, FLIP or
the View Transitions API (searched `dev-docs/backlog/`, its `done/` and `dev-docs/research/`, 2026-11-19). The owner said yes 2026-11-19.

**The demo the owner means.** `preview/src/components/virtual_grid/variants/photo_wall/mod.rs`: the 90,000-photo wall in
two layouts (the two-axis `VirtualGrid` of 128 px cells, `mod.rs:379`; `VirtualJustifiedGrid`, `mod.rs:354`).
Clicking a cell, or Enter or Space on the focused one, calls `open_photo(index)` (`mod.rs:296`): it sets `selected` and
`preview_open`, and a plain `Dialog` (`mod.rs:469`) shows the photo at its real aspect ratio. The cell image sits
`object-fit: contain` on a 6 px mat, or `cover` behind a switch; thumbnail and full image are the same generated
`data:image/svg+xml` `<img>` (`photo_src(index)`), so nothing is fetched. `gallery` (5,000 items, no click) is not it.

**The decision taken apart.**
- *Settled by existing rules.* Where it lives: a capability of the dialog family (the primitive owns the open state, the
  delayed `data-entering` and the exit hold), not a new component. Reduced motion: every motion sheet answers it
  (`dialog/style.css:52`, `dx-components-theme.css:969`), so the effect is absent under it. Rust first: nothing here
  needs a browser API the bridge lacks (below). No new `@starting-style` (row 516 removed it, `primitives/src/lib.rs:629`).
- *Settled by evidence.* A View Transition is the wrong mechanism here (below); FLIP as one CSS transition is cheaper.
- *Answered by the owner.* The library wants the effect (2026-11-19, "yes"). Left for the build: whether it ships as a
  primitive capability, as the construction below has it, or stays a documented recipe in the photo-wall demo only.

**Today's motion, read from the source.** `use_dialog_open_driver` (`primitives/src/lib.rs:629`) returns
`Closed`, `Entering` (the first rendered frame, a frame before `data-open`) and `Open`; the panel's entry is a
transition of `opacity` and `scale` (`dialog/style.css:29`), the exit is played while the dialog is still modal, and
`use_animated_open` (`lib.rs:897`) holds a popover-based overlay for its animations. Constraints: `check-motion-gating`
(no ungated loops or ticks; a transition is not one), `check-infinite-animations` (reduced-motion replacement for
`infinite`), `check-anchored-keyframes` (no `transform` in an anchored keyframe: this uses a transition, not a keyframe).
Row 523 (the same transition construction for edge and floating overlays) is the neighbour and does not conflict.

**The bridge.** It has no animate verb (`Method` at `bridge/mod.rs:267` is `showModal`, `close`, `scrollTo` and similar;
`Prop` at `:357` reads styles, attributes and offsets). A rectangle is read with Dioxus's own `MountedData`
`get_client_rect` (used in `slider.rs:1093`, `navigation_menu.rs:428`), which the bridge doc names as the first choice
(`bridge/mod.rs:51`). So the construction spends no bridge bytes, adds no behaviour (`check-bridge-behaviours` untouched)
and no JS (`check-js-volume` does not rise); the decision kernel is not involved.

**The virtual grid.** Cells are mounted by window only (docs `virtual_grid/docs.md:126`): the active cell is always
rendered, but a clicked cell the user scrolls away from, or a re-created node, can be gone when the dialog closes. The
photo wall's cell and image are plain rsx with no stable element handle, so the source rect must be read at open and
re-read at close, and the close needs a stored fallback.

**Web research (fetched 2026-11-19, generic client; saved under the scratchpad `web/` copies).**
- View Transitions, same document: MDN marks `Document.startViewTransition()` and `view-transition-name` "Baseline 2025,
  newly available, since October 2025" (https://developer.mozilla.org/en-US/docs/Web/API/Document/startViewTransition).
  Per engine (Chrome 111, Safari 18, Firefox 144) from knowledge, not re-extracted: MDN's table renders in script.
  Chrome's guide states Chrome 111 and the feature-detect fallback `if (!document.startViewTransition)`
  (https://developer.chrome.com/docs/web-platform/view-transitions/same-document).
- A `view-transition-name` must be unique per rendered element; a duplicate rejects `ready` and skips the transition, and
  the update callback still runs (MDN `view-transition-name`; spec https://www.w3.org/TR/css-view-transitions-1/). Fine for
  a grid if only the clicked item carries the name at the click, but the name must be moved on every open.
- The spec captures the whole document (the view-transition layer paints above the top layer) and, while rendering is
  suppressed, all pointer hit testing targets the document element: a full-page snapshot and a window with no input, which
  a modal that must take a press on its first frame (row 516's `data-entering` rule) does not want. A new
  `startViewTransition` skips the active one (`skipTransition()` abort), so it does not reverse mid-flight. Reduced motion is
  the author's media query on the `::view-transition-*` pseudo-elements (Chrome guide).
- FLIP (first, last, invert, play): from the platform as known, not re-checked; it needs two rect reads and a transform
  transition and works in every engine and on desktop webviews.

**The one construction: FLIP as a CSS transition on the dialog's media, driven by the existing open state.**
- *API a user writes.* a `LightboxSource` handle that the trigger registers on its element (`onmounted` stores its
  `MountedData`) and passes to the dialog, and a `DialogMedia` wrapper around the `<img>` that carries the pose. Without a
  registered source nothing changes.
- *Mechanism.* At open, Rust reads the source rect and the media's own rect (two `get_client_rect` calls) and writes
  `--dx-lightbox-dx`, `-dy` and `-scale` on the media through its inline style. The media's rule transitions `translate`
  and `scale` from that inverted pose (while `data-closed` or `data-entering`) to identity (`data-open`), on the same
  `--dx-overlay-duration` tokens. The media stays `visibility: hidden` until the vars are set (placement is one-shot, so
  invisible-until-placed is allowed by `js-interop.md`'s deadline table, the before-paint row, option G). The cell's image
  is hidden for the flight so there is one image, not two. Because it is a transition, a close mid-entry reverses it.
- *Why not View Transitions.* One document-wide snapshot, no input during it, no mid-flight reversal, a name to move per
  open, and `startViewTransition`'s update callback would have to await the executor's `showModal()`. FLIP reuses the
  delayed open state the family already has and needs nothing from the page.
- *Absent source, or reduced motion.* No registered source, a failed read, `prefers-reduced-motion`, or the native
  arm: no vars are written, the pose is identity, and the existing fade-and-scale runs. The recipe sits under
  `@media (prefers-reduced-motion: no-preference)`, (the same shape as the stylesheets' reduced-motion replacements).
- *Close after a scroll.* Re-read the source rect at close. If the node is gone or its rect is outside the grid window, the
  media does not fly to a stale place: it falls back to the plain exit (fade and scale), and the grid is not asked to
  scroll back. The target rect is clamped to the window so a half-visible cell shrinks into its visible part.
- *Focus and scroll lock.* Untouched: the dialog already restores focus to the opener and locks scroll in
  `use_dialog_open_driver` and `DialogContent`'s scroll lock. The source rect is read before the lock changes the layout
  (the scrollbar gutter baseline already prevents a shift), and the close's rect read happens before focus returns.
- *Cost.* Bridge bytes: 0. Gates: none new; one spec rule that the lightbox recipe uses `translate`/`scale` transitions only.
- *Desktop and native.* The desktop webview runs the same CSS and `get_client_rect` is an async IPC call there, so the
  media stays hidden until the reply (an extra frame or two at open, never a flash); web-verified, desktop inferred
  (`js-interop.md`, "Desktop and mobile caveats"). The Blitz arm has no `<dialog>` and no page: it gets the plain motion.

**Residue.** The visible photo is the image inside a letterboxed mat: `contain` and `cover` differ from the dialog's
aspect-ratio box, so the pose must come from the visible photo rect (the demo knows `aspect(index)`), and `cover` needs a
clip-path or a size crossfade or it distorts at scale. Border radius morphs only through a size-independent crop.
The dialog panel's overflow could clip a media that starts outside it (to check; it sets `overflow-y: auto` only for the
keyboard band, `dialog/style.css:166`). A second image during the flight is a cost with large photos. Firefox, Safari and the
desktop webview are unrun here (row 153). The row 516 lesson applies: a close mid-entry must reverse, never snap.

**Proof.** Extend the photo wall demo only (`photo_wall/mod.rs`, the `Dialog` at `:469`), with specs in `playwright/`:
open from an item (the media's first frame is at the item's rect, the last at the dialog's); close back to it; close after
scrolling the source out of the window (plain exit, no flight to a stale rect); `prefers-reduced-motion: reduce` (no
translate or scale transition on the media); rapid open and close (reversal, no snap, one image); keyboard open (Enter on
the focused cell). The timing specs run on a release build only (`agent-rules/build-and-fast-loop.md`); the install smoke
runs if `DialogContent` or its `style.css` change (`registry-install-smoke: OK -- N/N`).
