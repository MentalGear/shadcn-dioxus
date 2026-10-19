---
id: 76
title: "Dev-loop measurements and tooling (dev-wait.sh, inspect.mjs, base-url.ts)"
status: "Landed 2026-09-18 (Lane K, round 3)"
size: "S"
area: process
updated: 2026-09-18
---

# Row 76: Dev-loop measurements and tooling (dev-wait.sh, inspect.mjs, base-url.ts)

## Item

Dev-loop measurements and tooling (`dev-wait.sh`, `inspect.mjs`, `base-url.ts`)

## Status / trigger

Landed 2026-09-18 (Lane K, round 3)

## Size

S

## Detail

Lane K (commit `89f8e07`, cherry-picked from `a9f651c`) measured the actual edit-to-feedback latency on this repo's `dx serve --web` (debug, 4-core/15GB, isolated `CARGO_TARGET_DIR`) rather than assuming it, and shipped tooling built on the measurements: an RSX text edit hot-reloads in ~0.5-0.8s, but — a newly found trap — **only to already-connected tabs**; a fresh page load or navigation (including a real Playwright test) keeps serving the old wasm regardless, confirmed by a held-open page vs. two independent fresh loads a minute apart. A component `style.css` edit under `src/` costs a full ~28s rebuild (only `preview/assets/` CSS hot-reloads, ~0.9s) — CSS under `src/` is not cheap here, contrary to a natural assumption. A Rust logic edit in a primitive is ~38s; a new module is ~85s end-to-end, subject to a second newly found trap (rapid multi-file saves can log a "successful" but incomplete rebuild before a trailing one actually lands the change). `dx serve --hot-patch` fails immediately on any Rust logic edit inside the `primitives` workspace crate ("Missing rustc args for replay: 'dioxus_primitives'") and stays broken for every subsequent edit on that server, even ones touching only `preview` — not usable as this workspace's default today. Delivered: `scripts/dev-wait.sh` (blocks on the dev server's log signal rather than the `/_dioxus/build_status` websocket, which exists and is documented but has a gap — no new message on a second hot-reload-only edit in a row — so log parsing stays primary), `scripts/inspect.mjs` (direct browser inspection — a11y/DOM outline, `--eval`, `--click`/`--press`, `--axe`, `--screenshot` — against a warm server in ~1-5s, no spec file, no full suite needed), and `playwright/base-url.ts` (a `PLAYWRIGHT_BASE_URL` env override consolidating three pre-existing ad hoc `BASE_URL`/`BASE` patterns into one, default behaviour unchanged; migrated as a proof of concept in avatar/toolbar/sidebar/drag_and_drop_list.spec.ts, verified live against a non-default port). Full write-up, the complete measurement table, and the trap catalogue: [`dev-docs/dev-loop.md`](../../dev-loop.md) (not edited by this docs pass). **Partially superseded/closed 2026-09-18/19, batch 2:** the 4-spec proof-of-concept migration this row describes is now essentially complete, not just started — see row 81 (`e0bb23f`/`2cb1b06`/`3cc0881`, every remaining spec migrated) and row 80 (`238c8f4`, the CSS-hot-reload root cause this row's own measurement table first surfaced, corrected from "move the files" to "stop compiling them in")
