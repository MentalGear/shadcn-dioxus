---
id: 325
title: "Heavy cross-page oracles: construct the causes away, run what remains per deploy"
status: "OPEN (owner approved 2026-10-21; first construction in flight)"
size: "M"
area: tests
updated: 2026-10-21
---

# Row 325: Heavy cross-page oracles: construct the causes away, run what remains per deploy

## Item

Heavy cross-page oracles: construct the causes away, run what remains per deploy.

## Status / trigger

**OPEN (owner approved 2026-10-21; first construction in flight)**

## Size

M

## Detail

`popup-style-isolation` opens every popup on every page because popups inherit from their DOM host; the popup-root typography reset (row 218's half, lane popup-root-reset, 2026-10-21) removes the inherited channel and `check-popup-boundary.sh` already removes the selector channel, so the spec shrinks to one popup per component. `all-routes` (16% of suite time) and `first-paint-styles` (7.7%) move to the per-deploy run of row 322 once rows 327 and 329 shrink them.
