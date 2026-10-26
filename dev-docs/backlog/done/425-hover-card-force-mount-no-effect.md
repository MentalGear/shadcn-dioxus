---
id: 425
title: "HoverCard `force_mount` has no effect: the card is still rendered only while open"
status: "DONE 2026-10-25. Found by row 393's docs lane. The prop and component docs state the contract."
size: "S"
area: components
updated: 2026-10-25
---

# Row 425: HoverCard `force_mount` has no effect

## Item

`force_mount` only skips an early return before the hooks in `primitives/src/hover_card.rs`; `use_animated_open`'s `render()` still gates the DOM, so the content is not in the DOM while closed. Radix's `forceMount` keeps it mounted, hidden, for animation libraries and SEO.

## Status / trigger

DONE 2026-10-25. Found by row 393's docs lane. The prop and component docs state the contract.

## Size

S

## Detail

- Decide `force_mount` once for every primitive that has it (check how the other `force_mount` props reach `use_animated_open`) so that a component cannot accept the prop and ignore it. Then make HoverCard follow that rule, or drop the prop if the owner prefers.
- A spec checks the closed card is in the DOM and hidden with `force_mount`.

## Resolution

- `HoverCardContent` was the only component with a `force_mount` prop (no other primitive or styled component has one, and none passes it to `use_animated_open`), so there was no shared rule to fix: the class is this one prop. It had an early return before the hooks that its own default (`true`) never took, and `use_animated_open`'s `render()` alone decided the DOM.
- Now `force_mount` (default `false`, Radix's `forceMount`) mounts the card as a dormant instance while it is closed: `hidden`, `data-state="closed"`, never in the top layer. `render()` still decides when the card is shown (open, and its exit animation); `shown` keys the instance, so the card is remounted at each edge and leaves the top layer by replacement, as an unmounted card does. The early return is gone, so no hook is skipped.
- Spec: `playwright/hover-card-force-mount.spec.ts` on the new `force_mount` demo, with a no-`force_mount` control. Unit test: `hover_card::tests` (SSR).
