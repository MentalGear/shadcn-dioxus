---
id: 326
title: "Lanes iterate with dx serve hot reload and verify on an SSG build"
status: "BACKLOG (owner approved 2026-10-21, if reliable and safe)"
size: "S"
area: process
updated: 2026-10-21
---

# Row 326: Lanes iterate with dx serve hot reload and verify on an SSG build

## Item

Lanes iterate with dx serve hot reload and verify on an SSG build.

## Status / trigger

**BACKLOG (owner approved 2026-10-21, if reliable and safe)**

## Size

S

## Detail

Each SSG build in the lock queue cost several minutes plus waiting on 2026-10-20. Lanes doing visual work iterate on `dx serve` following `dev-docs/dx-serve-hot-reload.md` (confirm the latest edit is served before trusting it; no grepping minified CSS; no `elementFromPoint` after a `pointer-events: none` fix) and run their final specs on one SSG build. Write it into `dev-docs/lane-rules.md` after one lane has used it without a false result.
