---
id: 353
title: "Carousel Next arrow fades 0.5 to 1 after hydration (first-paint flicker)"
status: "LANDED 2026-10-21 (claude/first-pose): CarouselContext::next_disabled treats an unregistered slide…"
size: "S-M"
area: components
updated: 2026-10-21
---

# Row 353: Carousel Next arrow fades 0.5 to 1 after hydration (first-paint flicker)

## Item

Carousel Next arrow fades 0.5 to 1 after hydration (first-paint flicker).

## Status / trigger

LANDED 2026-10-21 (claude/first-pose): `CarouselContext::next_disabled` treats an unregistered slide count (0) as unknown, so Next is enabled at opacity 1 in the SSR markup and on every frame after hydration; a one-slide carousel settles to disabled after registration (fade 1 to 0.5, rare). Spec in playwright/first-pose.spec.ts, red before (server HTML had `disabled`).

## Size

S-M

## Detail

Next is server-rendered `disabled` because the slide count is only known after the items register in a post-mount effect (`can_scroll_next(selected, count)` is false for `count == 0`, `primitives/src/carousel.rs:499`; the `count` comment near line 6708 says the same). Once registered it enables and fades 0.5 to 1 over `--dx-motion-duration-slow` (200 ms): a first-paint flicker, the same class as row 344 (decide the initial pose during render).

Fix by construction, not by waiting in the spec: either the server supplies the slide count (a `count` prop or a count derived from the rendered children) so the first render is already enabled, or the first registration skips the transition. Spec: Next is enabled and at opacity 1 on the SSR markup (JS off) and never animates after hydration.
