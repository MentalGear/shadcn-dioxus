---
id: 338
title: "hydration-parity.spec builds attr-synth with cargo at load time, outside the build lock"
status: "BACKLOG (found 2026-10-21)"
size: "S"
area: tests
updated: 2026-10-21
---

# Row 338: hydration-parity.spec builds attr-synth with cargo at load time, outside the build lock

## Item

hydration-parity.spec builds attr-synth with cargo at load time, outside the build lock.

## Status / trigger

**BACKLOG (found 2026-10-21)**

## Size

S

## Detail

Seen three times on 2026-10-20/21 (lanes popup-root-reset, selector twice): merely loading `playwright/oracle/hydration-parity.spec.ts` -- `playwright test --list`, or any `test:changed` run that includes it, which the broad set always does -- runs `cargo build --release` for `playwright/oracle/attr-synth` outside `scripts/lane-lock.sh`, an unqueued ~3 min build that competes with gate and lane builds and writes `playwright/oracle/attr-synth/target`. Fix by construction: build attr-synth only through the lock (`scripts/lane-cargo.sh`, into an absolute isolated target dir) in a Playwright globalSetup or a pre-step of `test-changed.sh`/`run-suite.sh`, never at module load; the spec reads the built binary and fails with a clear message if it is missing. Check every spec for other load-time side effects at the same time.
