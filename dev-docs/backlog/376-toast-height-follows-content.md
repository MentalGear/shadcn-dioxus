---
id: 376
title: "Toast height follows its content: measured heights, an even collapsed pile, own heights expanded"
status: "Owner bug report with a screenshot, 2026-10-23, plus the question \"how should dynamic height via…"
size: "M"
area: overlays
updated: 2026-10-24
---

# Row 376: toast height follows its content

## Item

A toast with a title and a two-line description (the loading demo, dark mode) sat tight against its top and bottom border: `.dx-toast` was a fixed `height: var(--dx-space-16)` with `overflow: hidden`, and the stack overlap (`margin-top: calc(-1 * var(--dx-space-16))`) and the exit footprint were the same constant.

## Status / trigger

Owner bug report with a screenshot, 2026-10-23, plus the question "how should dynamic height via multiline be handled?". Answered the way Sonner does it, adapted to this stack's in-flow layout (no absolute positioning).

## Size

M

## Detail

Construction (one size model, `preview/src/components/toast/style.css` head comment):

- **Height is content plus a constant padding.** `.dx-toast` is `display: block`, `box-sizing: border-box`, `padding: var(--toast-pad)` (16 px, Sonner's); its content row is `.dx-toast-body` (`data-toast-body`), the one box whose height is the content's.
- **One writer of the numbers CSS cannot read.** `primitives/src/toast.rs` `TOAST_SIZE_JS`, installed once per provider through `use_js_listeners` (observers disconnected by `cleanup`): a `ResizeObserver` on every `[data-toast-body]` writes `--toast-height` (body + the toast's computed padding and border) on the toast's `li`, and the first non-closing `li`'s height becomes `--front-toast-height` on the `ol`; a `MutationObserver` follows toasts coming, going and `data-removed`, and measures a new toast in the callback that sees it inserted (the observer's first callback comes after the next frame's rAF callbacks, so a frame would lay the pile out without the new toast's height). The properties sit on the `li`/`ol`, which Rust never restyles, so a re-render cannot drop them. Unmeasured (no JS, a skin without the marker) the properties are invalid and the toast is auto-sized and not overlapped.
- **Collapsed:** a toast that is not `[data-top]` takes `--front-toast-height` and its `.dx-toast-body` is `opacity: 0`; the peek offset is still the list gap (12 px). **Expanded:** `--toast-box: var(--toast-height)`, `margin-top: 0`, content shown, each toast its own height.
- **Exit and enter shrink:** `--toast-expanded` (0/1 on the container) replaces the old `--toast-footprint: 4rem`; `--toast-slot-shrink` is now per slot (`-(expanded * own height + gap)`), including the top toaster's flipped start/end. `height` joins the toast's transition list.
- **No glide from the unmeasured layout:** the script sets `data-sized` on the `li` (`pending`, `true` a frame later) and the toast's `--toast-box-transition` (height, margin-top) is `0s` until then, so the switch from unmeasured to measured never animates.
- **Icon and close (Sonner reference):** no vendored Sonner sheet is in the repo (grep of `dev-docs/` and `playwright/` finds only prose), so this follows Sonner's rules as recalled: icon (spinner) and action are centred against the content (`align-items: center`); our close is an inline "x" rather than Sonner's corner badge, so it stays top-aligned with the first line. Content also gets `min-width: 0; overflow-wrap: anywhere` so a long word wraps instead of being clipped.
- Padding 12 -> 16 px vertical (Sonner's `--padding`); a one-line toast is now ~58 px, was 64.

Files: `preview/src/components/toast/{style.css, component.rs, docs.md, variants/multiline/*, variants/order.txt}`, `primitives/src/toast.rs`, `playwright/toast-height.spec.ts`, `playwright/durations.json`.

Spec: `playwright/toast-height.spec.ts` (6 tests): fit and equal padding of the loading demo's toast; one line vs long (same padding, own heights); collapsed (back = front height, back content hidden, peek 10-14 px); expanded (own heights, no overlap, content shown); collapse again; closing the front toast.

Evidence (debug SSG build of the branch, 100 routes, chromium, 2 workers, 2026-10-24):

- `toast-height.spec.ts` 6/6 on the branch, 0/6 on the parent `35f1435f` (the multiline demo added to the parent's tree for the comparison).
- Every `toast*.spec.ts` on the first build: 56 passed, 2 failed 3/3, both green on the parent: `toast-stack-motion` "a newly added toast ..." (one frame laid the pile out with the new toast unmeasured; fixed in the size script, above) and `toast-swipe` "RTL ..." (the drag pressed `x + 24`, the close button in RTL, now on the toast's middle row; the helper presses at the toast's centre). After the fix: 58/58.
- `CHANGED_RANGE=35f1435..HEAD pnpm run test:changed`: 479 passed, 8 skipped, 4 failed: `top-layer.spec.ts` :919, :962, :1516, red on the parent build too, and `toast-stack-motion` :166, green 5/5 alone (load).
- Measured, parent -> branch: the loading demo's toast 64 px with 12 px of content overflowing and 0.5 px pads -> 77 px, pads 16/16, no overflow; a title plus three wrapped lines 64 px with 21 px clipped -> 113 px expanded, pads 16/16; a one-line toast 64 -> 59 px; collapsed, the long toast behind a short one takes the front's 59 px with its content hidden; expanded gap 12 px both.
- `playwright/durations.json`: 17.1 s for the new spec, from that run.
- `scripts/registry-install-smoke.sh --only toast`: OK, 1/1 installed with dx 0.7.9, cargo check green (52 s).
- Fast gates 73/73 (exit 3).
