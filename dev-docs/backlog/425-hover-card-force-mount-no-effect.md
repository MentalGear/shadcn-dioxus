---
id: 425
title: "HoverCard `force_mount` has no effect: the card is still rendered only while open"
status: "BACKLOG. Found by row 393's docs lane (2026-10-25). The prop docs describe the current behaviour…"
size: "S"
area: components
updated: 2026-10-25
---

# Row 425: HoverCard `force_mount` has no effect

## Item

`force_mount` only skips an early return before the hooks in `primitives/src/hover_card.rs`; `use_animated_open`'s `render()` still gates the DOM, so the content is not in the DOM while closed. Radix's `forceMount` keeps it mounted, hidden, for animation libraries and SEO.

## Status / trigger

BACKLOG. Found by row 393's docs lane (2026-10-25). The prop docs describe the current behaviour until this lands.

## Size

S

## Detail

- Decide `force_mount` once for every primitive that has it (check how the other `force_mount` props reach `use_animated_open`) so that a component cannot accept the prop and ignore it. Then make HoverCard follow that rule, or drop the prop if the owner prefers.
- A spec checks the closed card is in the DOM and hidden with `force_mount`.
