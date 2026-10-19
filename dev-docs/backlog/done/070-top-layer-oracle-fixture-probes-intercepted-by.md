---
id: 70
title: "top_layer oracle fixture probes intercepted by the docs Sidebar, baseline group C"
status: "CLOSED 2026-10-19 (bookkeeping: status already said done). Found + fixed 2026-09-18 (construction)…"
size: "S"
area: tests
updated: 2026-10-19
---

# Row 70: top_layer oracle fixture probes intercepted by the docs Sidebar, baseline group C

## Item

`top_layer` oracle fixture probes intercepted by the docs Sidebar, baseline group C

## Status / trigger

CLOSED 2026-10-19 (bookkeeping: status already said done). Found + fixed 2026-09-18 (construction); live verification confirmed round 2

## Size

S

## Detail

Every route now permanently mounts `DocsLayout`'s `Sidebar` (PRs #51-57 dogfooding the real Sidebar site-wide), whose real container (`.dx-sidebar-container`) is `position: fixed; z-index: var(--dx-z-local-sm)` (10), occupying a `left: 0`, 16rem-wide column at any viewport ≥768px — including chromium's default 1280×720. The `top_layer` fixture (`/component/?name=top_layer&`) places several of its own probe triggers with `position: fixed` at literal viewport edges and **no `z-index` of their own** (`z-index: auto`) — the near-bottom-edge-flip row and one inline-styled modal-anchor trigger both sit under the Sidebar's column. A `position: fixed` box with `z-index: auto` always loses hit-testing to a sibling stacking context with an explicit positive `z-index`, regardless of DOM order — so Playwright's real `.click()`/`.hover()` (not `.evaluate(el => el.click())`, which bypasses hit-testing and is what several *passing* sibling tests in the same describe blocks use) retried until the outer 5-minute test timeout, naming `.dx-sidebar-desktop`/`.dx-sidebar-group-label`/`.dx-sidebar-menu-button` as the interceptor in Playwright's own actionability log. Members: `native-dialog.spec.ts` Rule 6c/6d/6e (`#popover-modal-anchor-trigger`) and `top-layer.spec.ts` Rule 5 Tooltip/HoverCard flip tests. **Not a new mechanism** — the modal-anchor trigger's own comment already documents this fixture hitting the identical interception once before, against the navbar, fixed there by nudging `top`; this repo's "two occurrences is a class" rule applied: nudging position again would only fix this one collision and leave the mechanism free to recur against the next permanently-mounted chrome. **Construction (Lane I, commit `e17c996`):** raise the fixture's own affected wrappers to `z-index: var(--dx-z-top, 9999)` — the fixture's own existing "clear ordinary page chrome" convention (already applied to `.dx-top-layer-stack-sibling`) — instead of another position nudge; `.dx-top-layer-edge-right-col` (the opposite edge row) got the same clearance preventively, same mechanism, not observed failing at the current viewport but not guaranteed to hold at every one — one root cause subsuming both by construction rather than a per-trigger patch. Every probe's actual edge/offset geometry (what Rule 5's flip and Rule 6e's trigger-anchored placement measure) is untouched. **Not verified against a live browser by the lane** (same sandbox resource constraint as row 69) — checked only with `playwright test --list`. Live re-run of the 5 Group-C specs is confirmed: **round 2's targeted Playwright run against the integrated build found all 5 Group-C specs green** (`native-dialog.spec.ts` Rule 6c/6d/6e's `#popover-modal-anchor-trigger`, `top-layer.spec.ts` Rule 5 Tooltip/HoverCard flip tests) — fixed live, not just confirmed by `--list`. Same lane and same day as row 69 (Lane I); `pw-baseline/BASELINE-REPORT.md` Group C is the full evidence trail, including the confirming negative (sibling tests hitting a different trigger location on the identical page passed in 1.4-3.6s)
