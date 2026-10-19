---
id: 263
title: "Toast: pause on hover/focus, vertical swipe, legacy keyframes on translate"
status: "LANDED 2026-10-19 (lane toast2: pause on hover/focus, vertical swipe, keyframes on translate)"
size: "S-M"
area: primitives
updated: 2026-10-19
---

# Row 263: Toast: pause on hover/focus, vertical swipe, legacy keyframes on translate

## Item

The three Base UI Toast behaviours row 138 did not take, plus the old keyframes still on `transform`.

## Status / trigger

**LANDED 2026-10-19 (lane toast2: pause on hover/focus, vertical swipe, keyframes on translate)**

## Size

S-M

## Detail

(1) Auto-dismiss pauses while the pointer is over the toaster or focus is inside it (Base UI does; WCAG 2.2.1 timing
adjustable), resuming with the remaining time, through the existing keyed `ToastTimer`. (2) Vertical swipe for top/bottom
toasters (row 138's swipe is horizontal only; direction is still derived from the toaster position). (3) The legacy
`dx-slide-up-*` and `dx-toast-slide-in` keyframes still animate `transform` and sit on `check-anchored-keyframes.sh`'s
ALLOW list: move them to `translate` and delete both allowlist entries in the same change. Specs per item.

## Update 2026-10-19

Landed in lane `toast2`, all three items.

(1) Pause: the provider reports `hover || focus-within` of its region (the very condition the stylesheet expands the pile on) through a `listen` script into a `paused` signal in `ToastCtx` (a touch tap is not a hover). `ToastTimer` counts the remaining time down in 50 ms ticks through `use_interval_while(visible && !paused)`, so a pause keeps the remaining time with no clock (hidden tab still restarts in full, as before). Still keyed by duration and revision, so `update` restarts it. Spec `toast-pause.spec.ts`: no hover -> gone after 3 s; hover past the duration holds it, leaving resumes with the remaining ~1.5 s (present at +0.6 s, gone by +2 s: a full restart would fail); focus holds, blur resumes.
(2) Vertical swipe: `TOAST_SWIPE_JS` now keeps a dismiss direction per axis (left/right edge by viewport half as before, top/bottom likewise) and takes the axis from the first 3 px of movement; writes `--toast-swipe-x` or `--toast-swipe-y` (mapped to `translate`); `touch-action` went from `pan-y` to `none` so a finger drag can go either way. New demos `pause` and `swipe_top` (a provider with `style: inset-block: ... auto` puts the toaster at the top). Spec `toast-swipe-vertical.spec.ts`: top toaster swipes up, bottom swipes down, wrong direction springs back, slow short drag follows then springs back, horizontal still dismisses.
(3) `dx-slide-up-even/odd` and `dx-toast-slide-in` now animate `translate` (and `scale`) instead of `transform`, no longer restating the stacking scale; both `check-anchored-keyframes.sh` ALLOW entries are deleted in the same change.

A top toaster is the stylesheet modifier `data-toaster-side="top"` (flips the pile through `--toast-list-direction`); an inline `style` on the provider was lost after the first toast (not chased, outside this row). docs.md updated. Evidence: 37 `toast[-.]` specs (13 new across the three files) + 22 `listener-inventory` tests green on a lane-ssg build containing the change; all static gates green. Nothing remains open.
