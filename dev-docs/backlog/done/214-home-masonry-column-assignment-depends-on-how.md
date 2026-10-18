---
id: 214
title: "Home masonry column assignment depends on how much of the ~580 KB prerendered page has been parsed"
status: "LANDED 2026-10-18 (lane masonry 43898ac7), merged on the session branch; deploy pending -- was…"
size: "M"
area: tests
updated: 2026-10-18
---

# Row 214: Home masonry column assignment depends on how much of the ~580 KB prerendered page has been parsed

## Item

Home masonry column assignment depends on how much of the ~580 KB prerendered page has been parsed

## Status / trigger

**LANDED 2026-10-18 (lane masonry 43898ac7), merged on the session branch; deploy pending** -- was: **Scheduled 2026-10-15 in `dev-docs/research/next-round-plan-2026-10-15.md`** (set up, waiting for the owner's go). **OPEN 2026-10-14 (found by the row 204 lane)**

## Size

M

## Detail

Cards jump columns during progressive paint on slow devices -- the most plausible remaining cause of the owner's iPad flicker (row 204). Needs an arrival-independent column assignment (e.g. explicit column containers or CSS grid). `playwright/layout-stability.spec.ts` starts at DOMContentLoaded, so it does not cover this (documented in the spec header); a spec for it must throttle the parse (a slow-chunked response) and record from the first paint.
