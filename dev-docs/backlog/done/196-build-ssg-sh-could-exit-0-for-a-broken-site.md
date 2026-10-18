---
id: 196
title: "build-ssg.sh could exit 0 for a broken site"
status: "LANDED 2026-10-14"
size: "S (done)"
area: build
updated: 2026-10-14
---

# Row 196: build-ssg.sh could exit 0 for a broken site

## Item

`build-ssg.sh` could exit 0 for a broken site

## Status / trigger

**LANDED 2026-10-14**

## Size

S (done)

## Detail

dx 0.7.9 prints `ERROR ssg error: ... IncompleteMessage` for a route that failed to render and still exits 0; one cold build wrote only `index.html` and `ssg-css-bundle` then failed after the fact (three lanes hit some form of it). `build-ssg.sh` now runs `set -Eeuo pipefail`, reads dx's status from `PIPESTATUS`, and `scripts/ssg-verify.sh` requires `SSG complete`, no `ssg error:`, and a fresh complete `index.html` for every `Rendering <route> for SSG` route cross-checked against `CATALOG`; one loud retry on transport-level errors only. Gate `check-ssg-verify.sh` runs its 14-case self-test. Hypothesis for the drops: all ~93 requests in one millisecond to a cold debug server.
