---
id: 322
title: "Run the full release suite per deploy, not per merge batch"
status: "LANDED 2026-10-23 (branch claude/playwright-suite-infra): scripts/deploy-preview.sh requires a…"
size: "S"
area: tests
updated: 2026-10-23
---

# Row 322: Run the full release suite per deploy, not per merge batch

## Item

Run the full release suite per deploy, not per merge batch.

## Status / trigger

**LANDED 2026-10-23 (branch `claude/playwright-suite-infra`): `scripts/deploy-preview.sh` requires a green release suite of the exact tree; merge batches run the gates, `test:changed` and the cross-component oracles**

## Size

S

## Detail

Per merge batch: `npm run test:changed` plus the cross-component oracles (popup-style-isolation, first-paint, hydration-parity); the 42-file interaction run on 2026-10-20 took 24 min against 66 min for the full release suite. The full suite then runs once, right before a deploy (or nightly). Depends on row 321 (one owner who runs the deploy).

## Update 2026-10-23: built

Row 321 (one merge owner) has landed, so the dependency is met. `scripts/deploy-preview.sh` now refuses to build until a
release-suite pass is stamped for `HEAD^{tree}` (`$LANE_TGT/release-suite/<tree>.pass`, clean worktree outside
`docs/`). With no stamp it makes one: a no-base-path release build in a copy of `release-template`, `serve-spa.py`,
`playwright/tools/run-suite.sh` (6 chunks, 3 workers), a stamp only when every chunk reported and nothing failed; a red
stops the deploy and lists the failing titles. `--suite-only` runs just that step (ahead of a deploy, or nightly);
`DEPLOY_SUITE=skip:<reason>` is the loud emergency exit. `dev-docs/lane-rules.md` "Merge queue" steps 2 and 4 now say
per batch: full gates, `CHANGED_RANGE=origin/main...HEAD pnpm run test:changed`, `popup-style-isolation`,
`first-paint-styles`, `oracle/hydration-parity`. Checked: the verdict parser on hand-made reports, `check-lane-target.sh`
(the target refusals still come first). Not yet run end to end: that is the next deploy's ~70 min.
