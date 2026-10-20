---
id: 290
title: "Toast: exit animation, a real visible cap, and enter from the toaster's edge (Sonner parity)"
status: "LANDED 2026-10-20 (exit animation, visible_toasts cap, edge-derived enter, queued slide-in; spec toast-enter-exit)"
size: "M"
area: primitives
updated: 2026-10-20
---

# Row 290: Toast: exit animation, a real visible cap, and enter from the toaster's edge (Sonner parity)

## Item

Toasts should dismiss with an animation, show at most 3 in the pile, and fade or slide the next one in when a slot frees up, as Sonner does.

## Status / trigger

OPEN (owner review 2026-10-19). New work: the landed rows 138, 262 and 263 cover stacking, swipe, pause and F6, but none of this.

## Size

M

## Detail

- **Today:**
  - Close, the timer and `Toasts::close` all call `remove_record` (`primitives/src/toast.rs:161-169`), which unmounts the `<li>` in the same frame. There is no exit animation; only a swipe animates out.
  - Toasts at index 3 and above get `--toast-hidden` → `opacity: 0` (`toast/style.css:132,140`), but they stay mounted and reappear on hover or focus-within.
  - `max_toasts = 10` evicts the oldest toast (`toast.rs:178-180, 272-290`).
  - Every toast enters from below (`dx-toast-slide-in`), even in a top toaster.
- **Sonner** (`src/index.tsx`, `styles.css`):
  - `visibleToasts = 3`, computed as `isVisible = index + 1 <= visibleToasts`. Toasts past that stay at opacity 0 with `pointer-events: none`, **even when the pile is expanded**.
  - Dismiss sets `data-removed="true"`. CSS then slides and fades the toast out, and it unmounts after `TIME_BEFORE_UNMOUNT = 200` ms.
  - When a slot frees up, the next toast's index shifts and it fades in.
  - Toasts enter from the toaster's own edge: from above for a top toaster, from below for a bottom one.
- **Target:**
  - A `closing` state set before `remove_record`, with `data-removed` and about 200 ms before unmount.
  - A `visible_toasts` prop (default 3) emitting `data-visible`, which replaces `--toast-hidden` and stays hidden on hover.
  - A direction-aware enter keyframe.
- **Files:** `primitives/src/toast.rs`, `preview/src/components/toast/style.css`, the toast specs and `docs.md`.
- **Two entry cases** (owner 2026-10-19):
  - A brand-new toast enters at the front, from the toaster's edge, as Sonner does: from below for a bottom toaster.
  - When the stack is expanded and a toast closes, the next queued toast (index 3 becoming visible) appears at the far end of the pile. It **slides in from above with a fade**, as on shadcn.
  - That second case needs a `translate` plus `opacity` transition on `data-visible` going from false to true. Today only the opacity changes.
  - Spec both cases: one with the pile collapsed and one with it expanded (hover held).

## Update 2026-10-20

Landed. Construction: **the toaster's side is the one source of direction** (`--toast-edge`, `1` bottom / `-1` for `data-toaster-side="top"`, on the container). A new toast enters from that edge and a closed one leaves toward it (`* var(--toast-edge)`); a queued toast that becomes visible at the far end of the expanded pile arrives from the opposite side (`* -1 * var(--toast-edge)`). The restack nudge keyframes read the same variable, so a top toaster no longer nudges the wrong way.

- Primitive (`primitives/src/toast.rs`): `ToastRecord.closing` and `close_record` (marks closing, `spawn_forever` removes after `TOAST_EXIT` = 200 ms). Close button, action, countdown and `Toasts::close` use it; a swipe (already flown out) and `max_toasts` eviction remove directly. The pile index counts only non-closing toasts, so the others restack at once. New `visible_toasts` provider prop (default 3). `Toast` derives `data-visible` (index < cap) and `data-removed`/`inert` (closing) itself from the record and the provider, so a themed wrapper passes nothing; `--toast-exit-ms` is written from the Rust constant and the exit transition reads it (one source for the unmount delay).
- Stylesheet: `--toast-hidden` is now `1` only for `data-visible="false"` (also inert), and the hover/focus rules no longer force