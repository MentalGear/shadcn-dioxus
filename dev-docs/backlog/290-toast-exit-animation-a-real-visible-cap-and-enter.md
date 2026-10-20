---
id: 290
title: "Toast: exit animation, a real visible cap, and enter from the toaster's edge (Sonner parity)"
status: "OPEN (owner review 2026-10-19). New work: the landed rows 138, 262 and 263 cover stacking, swipe…"
size: "M"
area: primitives
updated: 2026-10-19
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
