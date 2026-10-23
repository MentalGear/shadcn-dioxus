---
id: 376
title: "Toast height follows its content: measured heights, an even collapsed pile, own heights expanded"
status: "Owner bug report with a screenshot, 2026-10-23, plus the question \"how should dynamic height via…"
size: "M"
area: overlays
updated: 2026-10-23
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
- **One writer of the numbers CSS cannot read.** `primitives/src/toast.rs` `TOAST_SIZE_JS`, installed once per provider through `use_js_listeners` (observers disconnected by `cleanup`): a `ResizeObserver` on every `[data-toast-body]` writes `--toast-height` (body + the toast's computed padding and border) on the toast's `li`, and the first non-closing `li`'s height becomes `--front-toast-height` on the `ol`; a `MutationObserver` follows toasts coming, going and `data-removed`. The properties sit on the `li`/`ol`, which Rust never restyles, so a re-render cannot drop them. Unmeasured (no JS, a skin without the marker) the properties are invalid and the toast is auto-sized and not overlapped.
- **Collapsed:** a toast that is not `[data-top]` takes `--front-toast-height` and its `.dx-toast-body` is `opacity: 0`; the peek offset is still the list gap (12 px). **Expanded:** `--toast-box: var(--toast-height)`, `margin-top: 0`, content shown, each toast its own height.
- **Exit and enter shrink:** `--toast-expanded` (0/1 on the container) replaces the old `--toast-footprint: 4rem`; `--toast-slot-shrink` is now per slot (`-(expanded * own height + gap)`), including the top toaster's flipped start/end. `height` joins the toast's transition list.
- **No glide from the unmeasured layout:** the script sets `data-sized` on the `li` (`pending`, `true` a frame later) and the toast's `--toast-box-transition` (height, margin-top) is `0s` until then, so the switch from unmeasured to measured never animates.
- **Icon and close (Sonner reference):** no vendored Sonner sheet is in the repo (grep of `dev-docs/` and `playwright/` finds only prose), so this follows Sonner's rules as recalled: icon (spinner) and action are centred against the content (`align-items: center`); our close is an inline "x" rather than Sonner's corner badge, so it stays top-aligned with the first line. Content also gets `min-width: 0; overflow-wrap: anywhere` so a long word wraps instead of being clipped.
- Padding 12 -> 16 px vertical (Sonner's `--padding`); a one-line toast is now ~58 px, was 64.

Files: `preview/src/components/toast/{style.css, component.rs, docs.md, variants/multiline/*, variants/order.txt}`, `primitives/src/toast.rs`, `playwright/toast-height.spec.ts`, `playwright/durations.json`.

Spec: `playwright/toast-height.spec.ts` (6 tests): fit and equal padding of the loading demo's toast; one line vs long (same padding, own heights); collapsed (back = front height, back content hidden, peek 10-14 px); expanded (own heights, no overlap, content shown); collapse again; closing the front toast.

Evidence so far: `cargo clippy -p dioxus-primitives --all-targets --features web -- -D warnings` green; fast gates 73/73 (exit 3).  a static harness (the real `style.css` and theme sheet, the real size script, DOM shaped like the skin's) in Chromium: single loading-like toast 75 px, pads 16/16, not clipped; long (title + 3 lines) behind short collapsed: both 58 px, back content opacity 0, centres 12 px apart; expanded 58 / 109 px, boxes 12 px apart, pads 16/16; collapsed again 58/58. `playwright/durations.json` carries a 30 s ESTIMATE for the new spec until a build runs it.


**Open**


- Run `LANE_SLOTS=1 scripts/lane-ssg.sh <wt> toastheight 8500`, then `pnpm run test:changed`, the red-before (spec against the batch-13 build) and `playwright/tools/durations.mjs --merge` for the new file.
- `scripts/registry-install-smoke.sh` (component.rs / style.css changed).
