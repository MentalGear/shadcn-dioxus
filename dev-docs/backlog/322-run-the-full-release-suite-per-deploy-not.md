---
id: 322
title: "Run the full release suite per deploy, not per merge batch"
status: "BACKLOG (owner: maybe, 2026-10-21)"
size: "S"
area: tests
updated: 2026-10-21
---

# Row 322: Run the full release suite per deploy, not per merge batch

## Item

Run the full release suite per deploy, not per merge batch.

## Status / trigger

**BACKLOG (owner: maybe, 2026-10-21)**

## Size

S

## Detail

Per merge batch: `npm run test:changed` plus the cross-component oracles (popup-style-isolation, first-paint, hydration-parity); the 42-file interaction run on 2026-10-20 took 24 min against 66 min for the full release suite. The full suite then runs once, right before a deploy (or nightly). Depends on row 321 (one owner who runs the deploy).
