---
id: 500
title: "The all-routes axe split falls back to a full dark scan on 88 of 94 routes"
status: "BACKLOG. Found by the batch verifier, 2026-11-07."
size: "S-M"
area: playwright
updated: 2026-11-07
---

# Row 500: The all-routes axe split falls back to a full dark scan on 88 of 94 routes

## Item

Row 152's axe split runs the full axe set in light and only the colour rules in dark, guarded by a structure
signature that must match between the two schemes. On a debug build the signature differs on 88 of 94 routes, so
nearly every route runs the full dark scan as well, and `all-routes` is slower than before the split (973 s against
788 s on the same machine; `dev-docs/research/batch-verify-2026-11-07.md`).

## Status / trigger

BACKLOG. Found by the batch verifier, 2026-11-07.

## Size

S-M

## Detail

**Root-cause first:** print the first differing element per route from the two signatures. Candidates the lane named:
ids that change on re-render, scheme-dependent attributes on icons or images, and the order of the two scans (dark is
scanned first on an untouched page, light after axe has run, so any DOM change axe makes shows as a difference).
**By construction:** the signature covers only what axe's structure rules read; anything scheme-specific by design
is excluded with its reason. If the premise does not hold for this site, revert the split and record why.
