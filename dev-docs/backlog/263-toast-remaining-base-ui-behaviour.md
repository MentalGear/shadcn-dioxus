---
id: 263
title: "Toast: pause on hover/focus, vertical swipe, legacy keyframes on translate"
status: "BACKLOG (left open by row 138, 2026-10-19)"
size: "S-M"
area: primitives
updated: 2026-10-19
---

# Row 263: Toast: pause on hover/focus, vertical swipe, legacy keyframes on translate

## Item

The three Base UI Toast behaviours row 138 did not take, plus the old keyframes still on `transform`.

## Status / trigger

**BACKLOG (left open by row 138, 2026-10-19)**

## Size

S-M

## Detail

(1) Auto-dismiss pauses while the pointer is over the toaster or focus is inside it (Base UI does; WCAG 2.2.1 timing
adjustable), resuming with the remaining time, through the existing keyed `ToastTimer`. (2) Vertical swipe for top/bottom
toasters (row 138's swipe is horizontal only; direction is still derived from the toaster position). (3) The legacy
`dx-slide-up-*` and `dx-toast-slide-in` keyframes still animate `transform` and sit on `check-anchored-keyframes.sh`'s
ALLOW list: move them to `translate` and delete both allowlist entries in the same change. Specs per item.
