---
id: 506
title: "The release suite failed twice on main b78c231e; neither failure is confirmed real or confirmed a flake"
status: "OPEN, 2026-11-11. Found running the deploy after merging PR #170. The suite run…"
size: "S (to re-run both cleanly and read the…"
area: tooling
updated: 2026-11-11
---

# Row 506: two unconfirmed release-suite failures block the deploy

## Item

`scripts/deploy-preview.sh` on main `b78c231e` (PR #170) ran the full release suite once and failed two tests. A
second, narrower attempt to settle whether either is real hung rather than answering the question. The site is not
deployed from this commit; the live preview still serves the older `85c05466`.

## Status / trigger

OPEN, 2026-11-11. Found running the deploy after merging PR #170.

**The suite run** (`/home/user/tgt/release-suite/run-b1c062864c55c02d4d611be6784fa78f93108e3f`, 6 chunks, 3 workers):
4100 passed, 2 failed, 18 skipped. Chunks 1-4 and 6 took 569-686s each; chunk 5 took 1754s, about 3x the others, and
the kernel log for that window shows `TCP: request_sock_TCP: Possible SYN flooding on port 127.0.0.1:4830` (the
suite's own test server). Disk ran out during an unrelated side task later in the same session (a `cargo build`
while this run's own warm-tree copy was still on disk), which is evidence the box was tight on resources that day,
not evidence about this specific run, since the disk was not yet exhausted while the suite itself ran.

- **`tag_group.spec.ts:156` "Delete works for non-selectable removable tags"**: failed with a 1,200,000 ms (20
  minute) test timeout, then "Target page, context or browser has been closed" -- consistent with the browser or
  node process dying under load (an OOM kill, a wedged server under the SYN-flood symptom above), not a locator
  finding the wrong thing. A direct re-run (serving a freshly built release copy on a different port, no sibling
  load) hung the same way **twice in a row**, each attempt killed after it sat unresponsive past its own 20-minute
  timeout with no further progress logged. That rules out "this test is simply slow under load" (a quiet machine
  reproduced the hang too) without yet pointing at a cause: it could be the test harness's re-run path (the same
  long-lived dev server across repeat-each), a real hang in the primitive under a specific input this test hits, or
  something about this container page server. Not resolved.
- **`chart_tooltip_follow.spec.ts:571` "Narrow chart (phone): where neither side has room the tooltip pins beside
  its point on the roomier side, inside the chart"**: failed on a plain, synchronous geometry assertion (`category 1:
  not over its point`, `t.x`/`t.width` computed from measured DOM rects) -- not a timeout, not a crash. Nothing in
  PR #170 touches the chart tooltip's positioning code (`primitives/src/chart/components/tooltip.rs`, which has its
  own flip/clamp logic, not the shared `anchor_fallback.rs`/`js_listener.rs` row 436 touched); `git log` over the
  PR's range confirms no commit touches `primitives/src/chart/**`. That makes it very unlikely this PR caused it, but
  "unlikely" here is inference from the diff, not a red-on-base-then-green-on-head proof: the retry that would have
  supplied that proof (rerun this one test, isolated) never got past the tag_group hang above to reach it.

**Left undecided:** whether either failure recurs on a clean attempt uncomplicated by the hang; whether the hang
itself is this repo's bug or an environment one; whether the chart tooltip assertion is a genuine edge-case gap in
the "neither side has room" branch of the flip/clamp logic. **The deploy is parked here rather than forced through**
(`DEPLOY_SUITE=skip:<reason>` was available and not used): a skip reason is for an emergency fix, and this is routine
backlog work the next session can pick up with a clean machine and the time to let both reruns finish.

## Size

S (to re-run both cleanly and read the result); unknown until then whether a real fix follows.

## Detail

- First step: re-run the full release suite once more, alone, nothing else active on the box, and read whether both
  failures recur, recur differently, or are gone. If the hang recurs in isolation, it stops being a load story and
  becomes this repo's own bug to find (start from `tag_group.spec.ts:156`'s own body and `fixtures.ts`'s locator
  wrapper, which is where the stack trace points).
- If `chart_tooltip_follow.spec.ts:571` still fails alone, read `primitives/src/chart/components/tooltip.rs`'s
  "neither side has room" branch against the assertion's own math (`room = max(px - chart.x, chart.x + chart.width -
  px) - OFFSET`) for an off-by-one or a rounding case the narrow-phone width exposes.
- Proof when closed: the release suite green on the exact commit being deployed, or (if a real bug is found and
  fixed) a red-on-base reproduction of it plus the fix, same as any other row.
