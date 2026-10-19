---
id: 10
title: "Phase 5 — collision detection"
status: "CLOSED 2026-10-19 (backlog review 2026-10-18): done in the code"
size: "S (flip, done) + S (JS-fallback shift…"
area: theme
updated: 2026-10-19
---

# Row 10: Phase 5 — collision detection

## Item

**Phase 5** — collision detection

## Status / trigger

**CLOSED 2026-10-19 (backlog review 2026-10-18): done in the code**

## Size

S (flip, done) + S (JS-fallback shift, done) + S (vertical/size clamp, done) + S (5.2, ContextMenu point-anchor, done 2026-09-14)

## Detail

Keep the CSS clamp (5.3) regardless — still pending, see `recommended-implementations.md` §6. Two adoptable ideas from the comparison, not yet taken: bits-ui's debounced nested-lock cleanup (~24ms, survives same-tick unmount/remount races), and Radix's right-click carve-out for backdrop dismiss
