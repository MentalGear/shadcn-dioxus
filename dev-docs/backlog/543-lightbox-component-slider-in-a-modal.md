---
id: 543
title: "A Lightbox component: a Dialog around a Carousel, with the fly-in of row 535, fed by a list of items"
status: "BACKLOG, approved by the owner 2026-11-21: \"ok\". Not started. After row 535 (the fly-in…"
size: "M (estimate): the generic composition…"
area: components
updated: 2026-11-21
---

# Row 543: A Lightbox component

## Item

The library gets a `Lightbox`: a modal that shows one item of a list at a time and pages through the list with previous
and next, arrow keys and swipe, with a counter and an optional caption. The fly-in of [row 535](./535-lightbox-fly-in-from-the-clicked-item.md)
is optional: the shown item grows out of the clicked one and shrinks back into it. The data comes in as a list plus a
render callback for one item, so the content is not limited to images. Owner 2026-11-21, asked "should we also add a
lightbox component ? basically just a slider in a modal with the optional fly-in / out activated and a data input for the
content of the lightbox ?": yes, as a thin composition, filed as this row with zoom and a thumbnail strip as explicit
non-goals ("ok").

## Status / trigger

BACKLOG, approved by the owner 2026-11-21: "ok". Not started. After row 535 (the fly-in capability), which itself
follows [port row 456](./456-port-3-ov-overlay-core-escape-dismiss-open-focus.md); the owner's order (option A, 2026-11-19)
is unchanged. Waits for the owner's go like every other lane.

## Size

M (estimate): the generic composition (S to M), the photo wall and demo change (S), specs for the gesture, focus and
close-to-current-item cases (M, the larger part).

## Detail

**Composition, not a new primitive.** `Lightbox` is a themed component that renders the themed `Dialog`
(`preview/src/components/dialog/component.rs:63`) around the themed carousel
(`preview/src/components/carousel/component.rs:148`, `CarouselVirtualContent`) and adds the counter, caption and the
fly-in wiring of row 535. It declares both as component dependencies in its `component.json`, as `dialog` does for
`button` and `icons` (`preview/src/components/dialog/component.json:12`, `componentDependencies`). Files a lane creates:
`preview/src/components/lightbox/{component.json,component.rs,mod.rs,style.css,docs.md}` and `variants/` (a gallery of
thumbnails; a card list whose items are not images), plus the photo wall change below. No primitive changes here; row 535
owns the fly-in's primitive part.

**What the source gives (checked at `d1d1db0`).**
- Dialog: `open: ReadSignal<Option<bool>>` and `on_open_change: Callback<bool>` on the themed `Dialog`
  (`component.rs:29`, `:37`) and on the primitive `DialogRoot` (`primitives/src/dialog.rs:157`). There is no trigger part;
  a caller opens it from its own handler. Close paths: the corner X (`DialogClose`, `ctx.set_open(false)`,
  `component.rs:146`), Escape and a backdrop press. Escape is the native `<dialog>`'s `cancel`, routed through state by
  `use_dialog_close_sync` (`dialog.rs:565`); the backdrop press goes through the dismiss core
  (`Policy::NATIVE_BACKDROP`, `dialog.rs:576`); the focus trap is `showModal()`'s own. `initial_focus` and `final_focus`
  place focus (`component.rs:46`, `:51`).
- Carousel: `value: ReadSignal<Option<usize>>` controls the selected slide and `on_value_change: Callback<usize>` reports
  every change (`primitives/src/carousel/root.rs:20`, `:69`), resolved by `use_controlled` (`root.rs:191`); an effect scrolls
  the track to a changed index (`root.rs:323`). `use_carousel().selected` reads it from inside (`nav.rs:133`). The root's
  `onkeydown` pages on ArrowLeft and ArrowRight (`root.rs:485`), so the keys work while focus is inside the root.
  `CarouselVirtualContent<T>` takes `items: ReadSignal<Vec<T>>` and `render_item: Callback<(usize, T), Element>` and keeps
  `2 * radius + 1` slides mounted (`virtual_content.rs:15`, `:19`, `:44`); it renders every item on the server and on
  the first client render and narrows afterwards. A mouse or pen drag pages the track; touch scrolls natively
  (`content.rs:11`).
- The photo wall (`preview/src/components/virtual_grid/variants/photo_wall/mod.rs`): `open_photo(index)` (`:296`) sets
  `selected` and `preview_open`; a plain `Dialog` (`:469`) shows one `<img>` with a title and description and has no
  previous or next. The Lightbox replaces it.
- Gestures: no pinch, double-tap or zoom gesture exists in `primitives/src` (the only pinch mention is
  `keyboard_inset.rs:41`, which suspends its logic while the page is pinch-zoomed). The Drawer has a pointer
  drag-to-dismiss on its panel (`drawer.rs:214`, `drag_axis`): it gates where a drag may start (not on an interactive
  descendant) and hands a browser-claimed pan back as a snap-back (`drawer.rs:63`), with `touch-action: pan-x` on a
  bottom drawer so the browser keeps the orthogonal axis (`drawer/style.css:135`). It does not exist on the Dialog, so
  the Lightbox has no dismiss drag unless it builds one.
- No Carousel inside a modal Dialog is demonstrated or specced anywhere (no carousel spec or variant mentions a
  dialog); the first one is this row's proof.

**API a user writes (sketch).** The item type is a generic parameter carried by the context, as rows
[536](./536-generic-components-check-value-types-at-run-time.md) and [540](./540-one-controlled-prop-type.md) require, so
a part with another type fails to compile and nothing downcasts:

```rust
Lightbox::<Photo> {
    items: photos,                         // ReadSignal<Vec<Photo>>
    render_item: |(i, p): (usize, Photo)| rsx! { img { src: p.url, alt: p.alt } },
    caption: |(i, p): (usize, Photo)| p.caption,  // optional
    open: open(), on_open_change: move |o| open.set(o),
    index: index(), on_index_change: move |i| index.set(i),
    fly_in: Some(source),                  // optional, row 535's source handle
}
```

`LightboxContext<T>` holds the items and the render callbacks; the counter and caption parts read it, so a part of the
wrong `T` has no context. `open` and `index` use the controlled-prop encoding the family has when the lane starts: the
`ReadSignal<Option<_>>` of `Dialog` and `Carousel` today, or `Controlled<T>` if row 540 landed first. The Lightbox adds
no third encoding. The index is controlled by the caller so the grid can follow it (below).

**Version 1.** Previous and next buttons, arrow keys, swipe on touch and drag on mouse and pen (the Carousel's), a counter
("3 / 90 000"), an optional caption, close by X, Escape and backdrop press. The fly-in is optional: with a registered
source the shown item opens from the clicked one and closes into the item of the slide currently shown, and without a
source, with a source that is not on screen at close, or under reduced motion, the plain exit runs (row 535's rule).
Large lists mount through `CarouselVirtualContent`, so the DOM keeps about 21 slides.

**The four hard parts and their constructions.**
1. *Closing to a different item.* The Lightbox reports the current index through `on_index_change`, so the caller always
   knows the shown item. The photo wall passes `index` back into the grid's active cell, which the grid keeps rendered
   even outside its window (`virtual_grid/docs.md:126`); the fly-in's close re-reads that cell's rect and, when the cell
   is not mounted or outside the grid window, falls back to the plain exit (row 535). One index, owned by the caller.
2. *Gestures.* One owner per axis. The horizontal axis belongs to the Carousel (native scroll on touch, its drag on
   mouse and pen); the Lightbox sets no `touch-action` and no pointer handler on the panel, so nothing competes. Version 1
   has no swipe-to-dismiss: closing is X, Escape and backdrop press. A later vertical dismiss would reuse the Drawer's axis
   gate and `touch-action` split (`drawer.rs:214`, `drawer/style.css:135`); it is not built here.
3. *Focus and keys.* The arrow keys page only while focus is inside the Carousel root (`root.rs:485`), and `showModal()`
   traps focus in the dialog and keeps Escape (`dialog.rs:565`). The Lightbox sets `initial_focus` to the Carousel root so a
   keyboard open lands where the arrows work, and leaves `final_focus` to the Dialog's default (the opener). Escape closes
   the Lightbox even from a slide; the specs prove both keys from one open.
4. *Image loading.* During the fly-in the media shows the item the grid already has (the thumbnail, the same element size),
   and the full image replaces it when it has loaded. `render_item` receives the item and decides which source it uses; the
   Lightbox does not fetch. In the photo wall both are the same generated `data:` URL, so the swap is a no-op there; a
   demo with a separate thumbnail proves the swap.

**Non-goals of version 1.** Pinch and double-tap zoom: no pinch or zoom gesture exists in the primitives, so it is a
gesture engine of its own. A thumbnail strip: a second selector over the same index, deferred until the first
version is in use. Video: the render callback can return anything, but playback and its controls are not designed here.
Deep links: a URL per slide needs router integration the library does not have for overlays. Swipe-to-dismiss: see part 2.

**Dependencies.** Row 535 supplies the fly-in capability (the source handle and the media pose); row 535 waits for row
456, which rewrites the exit path its close half builds on. Until 535 lands, a lane can build the Lightbox without the
`fly_in` prop, but the photo wall proof needs it.

**Photo wall.** Row 543 owns the photo wall change (`photo_wall/mod.rs` and `virtual-grid-photo-wall.spec.ts`): the
Lightbox replaces the preview dialog and the cell registers its source. Row 535's lane ends at the capability.

**Proof.** Specs by name, in `playwright/lightbox.spec.ts` and the photo wall spec: open from an item; step to the next
and previous slide with the buttons; close back to the current slide's item (not the opened one); close after the source
scrolled out of the window (plain exit, no flight to a stale rect); arrow keys page while Escape still closes; Tab
stays inside the dialog; swipe on touch and drag on mouse; `prefers-reduced-motion: reduce` (no translate or scale
transition on the media); a 90 000-item list through the virtualised content (about 21 slides mounted, the counter
right, a jump to the end). Timing specs run on a release build only (`agent-rules/build-and-fast-loop.md`). The install
smoke runs for the new component (`registry-install-smoke: OK -- N/N`), and a unit test builds the context with one `T`
and shows a part of another `T` does not compile (compile-fail), as row 536 asks.

**Residue.** A Carousel in a modal Dialog is unproven; the focus hand-off and the arrow keys under the dialog's
focus trap may need an adjustment in the Carousel root. The Carousel's virtual window re-anchors only once a gesture is
over, so a jump far from the current slide shows the window's re-centre. The fly-in's cover and contain mismatch of
row 535 applies to the first frame. Size M is an estimate and mostly specs.
