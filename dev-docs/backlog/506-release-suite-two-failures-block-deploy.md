---
id: 506
title: "The release suite failed twice on main b78c231e: one stale locator that waited 20 minutes, one racy geometry read"
status: "DONE 2026-11-12: fixed in PR #175 (merge b69c006d), and the full release suite on main edb71de0 (PR…"
size: "S, done in the lane; the full release…"
area: tooling
updated: 2026-11-12
---

# Row 506: the two release-suite failures on b78c231e, explained and bounded

## Item

`scripts/deploy-preview.sh` on main `b78c231e` (PR #170) ran the full release suite once and failed two tests; a narrower
re-run of the first hung twice. The site was not deployed from that commit. Both failures are test-side, neither is a
product bug, and the suite now bounds a hang like the first in minutes.

## Status / trigger

DONE 2026-11-12: fixed in PR #175 (merge `b69c006d`), and the full release suite on main `edb71de0` (PR #176) is green
on the exact tree deployed: 4106 passed, 0 failed, 0 flaky, 14 skipped; site published by deploy commit `c34eda92`.
Neither failure was a product bug or an environment fault. The chunk wall limit added here read six lines of the shard
table as one number and turned the first deploy attempt red in five seconds; PR #176 reads one line and refuses a
limit that is not a plain number of seconds. The class behind the stale locator (a demo found by its title) is closed
by row [512](./512-specs-locate-variants-by-slug.md).

**`tag_group.spec.ts:156` "Delete works for non-selectable removable tags"** (measured, reproduced on a release build of
main `bdb41d2a`, whose code equals `b78c231e` plus docs commits, served by the pre-change `serve-spa.py`):

- Not a hang of the browser, the server or the component. `statesVariant()` located the demo by its heading,
  `getByRole("heading", { name: "states" })`. Row 399's title audit (`d892fb1e`, PR #170, 2026-11-11) renamed that demo's
  heading to "Disabled and non-selectable" (`variants/states/title.txt`), so the locator matched nothing. A click on a
  locator that matches nothing retries until the test timeout, and this file set that timeout to 20 minutes
  (`LOAD_TIMEOUT`, written for a cold `dx serve` build) inside a `serial` describe, where it overrides the CLI
  `--timeout`. Reproduced: the worker sat at 0.8 % CPU with no connection open to the server and a healthy server
  (`curl` 200 in 1.4 ms) for 6+ minutes until killed. With each step logged and the click bounded to 8 s the call log
  reads `waiting for locator(... getByTestId('tag-group-nonselectable').getByRole('row', { name: 'alpha' }))`; the page's
  variants are `["Multi", "Disabled and non-selectable"]` and the group is present under the second. The "Target page,
  context or browser has been closed" of the original report is the runner closing the browser when the 20 minutes ended,
  which is why the load, OOM and SYN-flood readings could not explain the isolated re-runs.
- `serial` mode skips the rest of the file after the first failure, and the sibling test `Delete keeps selected tags that
  do not have a remove button` uses the same locator (it would fail the same way).
- Why the file was serial: it shares no state; the comment said "parallel navigations contend with the preview webServer
  build", a `dx serve` concern that does not exist on the served SSG build. The whole file is 13 tests and 20 s of test
  time in `playwright/durations.json`.

**`chart_tooltip_follow.spec.ts:571` "Narrow chart (phone): where neither side has room ..."** (measured):

- Not reproduced alone: 20 of 20 passed (`--repeat-each=20`, 2 workers). Reproduced once under load: the 16 chart spec
  files at `--workers=3 --repeat-each=2`: 573 passed, 1 failed, this test, at the `not over its point` assertion.
- Cause: the tooltip stays `data-state="open"` while the pointer moves from one category to the next, so
  `toHaveAttribute("data-state", "open")` and `expectInside` are satisfied by the previous category's box, and the
  one-shot `box(tooltip)` after them can read that box before the move lands. A diagnostic copy of the loop (6 runs x 6
  categories x 6 passes = 216 reads, 3 workers) compared each immediate read with the position 250 ms later: 3 of 216
  immediate reads differed (for example x 66 immediate, x 118.6 at rest, `data-placed` already `true`), and the at-rest
  position was beside its point in all 216. So `chart/components/tooltip.rs`'s flip/clamp shows no gap here (from 216 rest
  reads at one phone width, not a proof for every width); the assertion read before rest.

**Server** (measured): `scripts/serve-spa.py`'s `ThreadingHTTPServer` listened with the stdlib backlog of 5. Three bursts
of 400 simultaneous connections: `ListenOverflows` +4,580, `Possible SYN flooding on port` in `dmesg`, 149 of 400
requests failed in the second burst, worst connect 28.9 s (SYN retries). With `request_queue_size = 1024`: three bursts
of 400, 0 failures, worst 0.42 s, `ListenOverflows` unchanged. A page load of the preview issues 42 requests over 42
connections on the HTTP/1.0 handler and 18 connections with `protocol_version = "HTTP/1.1"` (measured); that change is
not made, because with the backlog fixed nothing fails on 1.0 and keep-alive adds idle server threads.

## Size

S, done in the lane; the full release suite on the merged tree is the main loop's.

## Detail

What changed (all under `playwright/` and `scripts/serve-spa.py`, no product code):

- `tag_group.spec.ts`: the states demo is found by the test id it contains, not by its title; no `serial`, no 20 minute
  timeout (the config's 90 s per test applies; the whole file sums to 20 s in `durations.json`, so no test of it exceeds
  20 s, a margin of at least 4.5x); `gotoHydrated` uses the test's timeout.
- `chart_tooltip_follow.spec.ts`: `expectInsideBesidePoint` polls the inside-the-chart and beside-the-point conditions
  until two reads 100 ms or more apart agree on the box, instead of one read after `open`.
- `scripts/serve-spa.py`: `request_queue_size = 1024` (the kernel caps it at `net.core.somaxconn`, 4096 here).
- `playwright/tools/run-suite.sh`: each chunk runs under `--global-timeout` = 3 x the chunk's expected wall time (its
  summed table seconds / workers, at least 900 s; 6 chunks and 3 workers give 2,142 s against the 569-686 s the 2026-11-11
  chunks took; `--limit-factor` overrides the 3). A chunk that hits it still writes its JSON report, with the unfinished
  tests `skipped` and two `errors` ("Timed out waiting Ns for the test suite to run"); `suite_verdict` in
  `scripts/deploy-preview.sh` counts a report with `errors` as a missing chunk, so the run is red and the lock is freed.
  Checked on a scratch spec: 1 passed, 2 skipped, 2 errors.
- Not set, on purpose: `actionTimeout` / `navigationTimeout` in the configs. The config's per-test timeout (90 s) already
  caps every action and navigation of a test without its own `setTimeout`; the tests with one are the sweeps
  (`computed-style-snapshot` 90 min, `layout-stability` and `charts-hydration-stability` 45 min,
  `demo-forms-no-navigation` 30 min, `docs-structure` 10 min), whose slowest single action was not measured, so a
  per-action bound would be a guess that can create the flakes it is meant to prevent. The chunk limit bounds them. No
  retries were added (an owner decision).

Proof (served release build of main `bdb41d2a`, new `serve-spa.py`): `tag_group.spec.ts` and
`chart_tooltip_follow.spec.ts` `--repeat-each=3` at 3 workers: 135 passed; the 16 chart files plus `tag_group.spec.ts`
`--repeat-each=2` at 3 workers (the load that failed before): 598 passed, 0 failed; `all-routes`, `layout-stability`,
`carousel` at 2 workers: 170 passed (10.0 min).

Open: the full release suite on the merged tree (the deploy's run), then the deploy. Any other spec that finds a demo
by its title (`getByRole("heading", { name })` on a variant title) breaks the same way when a title is renamed; the
2026-11-11 suite found only this one.
