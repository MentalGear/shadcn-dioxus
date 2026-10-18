---
id: 230
title: "Attachment first demo has no image"
status: "LANDED 2026-10-18 (lane demos 187dc853), merged on the session branch; deploy pending -- was…"
size: "S"
area: docs
updated: 2026-10-18
---

# Row 230: Attachment first demo has no image

## Item

Attachment first demo has no image

## Status / trigger

**LANDED 2026-10-18 (lane demos 187dc853), merged on the session branch; deploy pending** -- was: **Scheduled 2026-10-18 in `dev-docs/research/next-round-plan-2026-10-15.md` (lane demos)** -- was: **BACKLOG (owner 2026-10-17)**

## Size

S

## Detail

`research/docs-audit-2026-10-17.md` §1: ours is 3 file chips; shadcn's is an image group, an uploading file with a Spinner, a done code file -- our `composite` variant already has that content -> make it the main demo. Owner question answered: the group does NOT use our Carousel; it is its own scroll-snap strip with `dx-scroll-fade-x` (`attachment/component.rs:313-326`, `style.css:229-250`), as in shadcn (no carousel there either).
