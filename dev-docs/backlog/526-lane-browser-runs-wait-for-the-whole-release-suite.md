---
id: 526
title: "A lane's browser run waits for the whole release suite: the suite yields between its chunks to one bounded lane run"
status: "BACKLOG (found 2026-11-15). Not started. The owner's call: it changes the release-suite procedure…"
size: "M: the yield point in run-suite.sh, the…"
area: tests
updated: 2026-11-18
---

# Row 526: a lane's browser run waits for the whole release suite

## Item

The release suite sets the hold flag `$LANE_TGT/suite-running` for its whole run (six chunks, about 62 min), and a lane
compiles and drives no browser while it is set, so a lane's browser checks queue behind the entire suite. On 2026-11-15
lane 449 waited about six hours across two suites before it could compile, and its first browser run then found two
regressions it could have found that morning. The hold has a reason: worker contention makes the suite flaky (batch 7's
one red, hydration-parity Rule 3, reproduced only under contention), so the fix keeps the suite's chunks on a quiet
machine and shortens the wait instead.

Proposed construction: the hold is per chunk, not per suite. Between two chunks, `playwright/tools/run-suite.sh` takes
the next queued lane run from the lane-lock queue, bounded in size (a `test:changed` selection capped at a fixed number
of files, one worker, its own port and site), lets it finish, then starts the next chunk. A lane then waits at most one
chunk (about 10 min) plus its own run, and no chunk ever shares the machine. Residue: the suite's wall time grows by the
lane runs it admits, and a lane run above the cap still waits for the end of the suite.

## Status / trigger

BACKLOG (found 2026-11-15). Not started. The owner's call: it changes the release-suite procedure that stands between
main and the published site.

## Size

M: the yield point in `run-suite.sh`, the cap and the queue hand-off in the lane-lock scripts, a self-test, and the
procedure text in `dev-docs/lane-rules.md` and `agent-rules/build-and-fast-loop.md`.

## Detail

- **Evidence:** lane 449's hand-back of 2026-11-15 (its fit-log row in `dev-docs/agent-model-fit.md`), and the batch 7
  Rule 3 reproduction (2/300 under contention, 0/124 without; the fix is `settleGalleryCards` in
  `playwright/hydration.ts`).
- **Proof:** a suite run with a queued lane run between chunks stays green with the same totals as a run without one,
  and the lane run's wait is at most one chunk's wall time.
