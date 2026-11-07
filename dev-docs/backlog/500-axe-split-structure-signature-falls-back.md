---
id: 500
title: "The all-routes axe split falls back to a full dark scan on 88 of 94 routes"
status: "DONE 2026-11-07 (lane axe-root-500): the split is reverted; see Detail."
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

DONE 2026-11-07 (lane axe-root-500): the split is reverted; see Detail.

## Size

S-M

## Detail

**Root-cause first:** print the first differing element per route from the two signatures. Candidates the lane named:
ids that change on re-render, scheme-dependent attributes on icons or images, and the order of the two scans (dark is
scanned first on an untouched page, light after axe has run, so any DOM change axe makes shows as a difference).
**By construction:** the signature covers only what axe's structure rules read; anything scheme-specific by design
is excluded with its reason. If the premise does not hold for this site, revert the split and record why.

**Result (lane axe-root-500, 2026-11-07).** Root cause of the 88 fallbacks: the header theme toggle (two buttons and two icons,
one `display: none` per scheme, with scheme-specific labels) and an animated Progress demo's `aria-valuenow`; not ids, not the
scan order (an axe colour scan and a light-dark-light round trip left the signature unchanged). Excluding those two with their
reasons and walking only rendered elements gave 0 of 94 fallbacks. But the premise of the split is false for this site: the
colour rules alone take as long as the full rule set (`color-contrast` is the cost of an axe scan), so the whole-file time
only moved from 862.0 s to 830.6 s (same build, 2 workers, noisy box). The split is reverted; `all-routes` is back to two full
scans per route and `durations.json` carries 862.0 s. Details and numbers: row 152, "152b slice 3".
