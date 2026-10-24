---
id: 325
title: "Heavy cross-page oracles: construct the causes away, run what remains per deploy"
status: "LANDED 2026-10-24 (branch claude/first-paint-per-deploy): the popup half (row 259, batch A), the…"
size: "M"
area: tests
updated: 2026-10-24
---

# Row 325: Heavy cross-page oracles: construct the causes away, run what remains per deploy

## Item

Heavy cross-page oracles: construct the causes away, run what remains per deploy.

## Status / trigger

**LANDED 2026-10-24 (branch `claude/first-paint-per-deploy`): the popup half (row 259, batch A), the `all-routes` half (row 327, batch 1), and `first-paint-styles`, whose per-route sweep now runs only in the deploy's suite.**

## Size

M

## Detail

`popup-style-isolation` opens every popup on every page because popups inherit from their DOM host; the popup-root typography reset (row 218's half, lane popup-root-reset, 2026-10-21) removes the inherited channel and `check-popup-boundary.sh` already removes the selector channel, so the spec shrinks to one popup per component. `all-routes` (16% of suite time) and `first-paint-styles` (7.7%) move to the per-deploy run of row 322 once rows 327 and 329 shrink them.

**Update 2026-10-21:** the popup half landed with batch A -- row 259 (lane popup-root-reset 419a2526): one zero-specificity popup-root rule resets 98 inherited properties and `check-popup-root-reset.sh` guards it; `popup-style-isolation` shrank from 85 tests / 592 s to 22 tests / 150 s (one popup per component under a hostile parent) while the old full matrix still passes 89/89 on the new build. Left: `all-routes` and `first-paint-styles` (rows 327 and 329, Round 4 thread).

**Update 2026-10-24: `first-paint-styles` (branch `claude/first-paint-per-deploy`).** Its per-route causes were already
constructed away: a late stylesheet by `scripts/ssg-css-bundle.mjs` (every project sheet of dx's asset manifest in one
render-blocking bundle; the build aborts when a head is not "four globals, then the bundle"; `check-css-delivery.sh` keeps
the construction), a first-frame markup difference by `cargo test -p preview ssr_parity` (row 329). What only a browser sees
(the `CLIENT_MEASURED` components, and the end-to-end proof on every route) is now the deploy's: sections 2 and 3 run on the
four `SUBJECTS` (field, select, combobox, tag_group) unless `FIRST_PAINT_ROUTES` says otherwise, and
`scripts/deploy-preview.sh` already passes `FIRST_PAINT_ROUTES=all` to the release suite. No new switch was needed.
Listed tests: 117 -> 18 for a lane, a merge batch or a broad `test:changed`; 176 with `FIRST_PAINT_ROUTES=all` (unchanged).
`dev-docs/lane-rules.md` "Merge queue" step 2 says so. `all-routes` stays in the per-batch BROAD set: its causes (axe,
overflow, forced colours) are not constructed away.
