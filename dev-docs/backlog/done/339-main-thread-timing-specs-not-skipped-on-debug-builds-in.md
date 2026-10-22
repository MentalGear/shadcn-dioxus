---
id: 339
title: "main-thread timing specs are not skipped on debug builds in test:changed runs"
status: "LANDED 2026-10-21 (lane test-tools): build-ssg.sh stamps a debug public dir with build-profile.txt…"
size: "S"
area: tests
updated: 2026-10-21
---

# Row 339: main-thread timing specs are not skipped on debug builds in test:changed runs

## Item

main-thread timing specs are not skipped on debug builds in test:changed runs.

## Status / trigger

**LANDED 2026-10-21 (lane test-tools): build-ssg.sh stamps a debug public dir with build-profile.txt; base-url.ts buildProfile() reads it from SSG_SITE_DIR or the served site, so every config skips main-thread on a lane build**

## Size

S

## Detail

Seen 2026-10-21 (lane selector's browser run, and lanes popup-root-reset and charts-page before): `oracle/tier2-html/main-thread.spec.ts` ran on a debug SSG through `npm run test:changed` and produced 11 false reds, although CLAUDE.md says the Playwright configs ignore it on known-debug builds (`PW_BUILD_PROFILE`, read off the build when unset). Find why the detection does not apply on `ssg.local.config.ts` / `lane-ssg.sh` builds (profile not exported, not readable from the served build, or the guard missing from that config) and make it hold by construction for every config; a debug run must report the timing specs as skipped, not failed.
