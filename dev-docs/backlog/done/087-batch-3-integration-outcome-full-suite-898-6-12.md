---
id: 87
title: "Batch-3 integration outcome: full suite 898/6/12, 0 unaddressed regressions"
status: "Landed 2026-09-19 (batch-3 integration, HEAD 3ce62f5)"
size: "—"
area: process
updated: 2026-09-19
---

# Row 87: Batch-3 integration outcome: full suite 898/6/12, 0 unaddressed regressions

## Item

Batch-3 integration outcome: full suite 898/6/12, 0 unaddressed regressions

## Status / trigger

Landed 2026-09-19 (batch-3 integration, HEAD `3ce62f5`)

## Size

—

## Detail

**39 commits (32 lane picks + 7 integration-authored) landed on `claude/roadmap-evaluation-6eiahc`, base `548de1e`.** Gates green in an isolated `CARGO_TARGET_DIR`, run twice (once before `rtl-rust` landed, once after — only the second, final run is reported): the 7 `scripts/check-*.sh` guards (the 6 from batch 2 plus this batch's new `check-css-logical-properties.sh`), `cargo fmt --all -- --check`, `cargo check --workspace --all-targets` and `--features web`, `cargo test --workspace` **343/0/2** (up from batch-2's 326 — date-picker's own +3, rtl-rust's own +9 unit +2 doctests), `cargo clippy --workspace --tests --examples -- -D warnings`, `cd preview && npx stylelint "src/**/*.css"` (clean after a 3-file mechanical `--fix`), and all three `cargo doc` invocations. Targeted Playwright: `main-thread.spec.ts` alone (`PW_WORKERS=1`) 55/3 (row 45's own known-red set); a 22-file batch covering every RTL-touched primitive plus keyboard-matrix/menu-submenu/menu-roles/window-splitter/form-label-association/date-picker/virtual_list/preview/sidebar, **259/2/1** — the 2 reds are row 21's own known date-picker anchoring class and one isolated `Tooltip — Escape` flake (root-caused as 22-file load contention, not a regression: 4/4 in isolation, and the whole file 77/77 in isolation at the same worker count); drawer/sheet/tag_group/navigation_menu (this batch's own CSS-audit regression guards) 40/40. **SSG execution** (row 22/46's own closure, this round's headline result): a production-shape release+ssg+fullstack build, 71 routes, 0 not-found shells, a static server kept up for the rest of the round; hydration-parity **11/11** (5 of those were batch-2's own "not run" results, now real and green — the direct, positive confirmation that the Label duplicate-`class` fix, row 46, introduced no regression anywhere else either). **Full suite** (one run, `:8090` kept up throughout so hydration-parity's extended rules ran for real): 916 collected, **898 passed / 6 failed / 12 skipped / 0 flaky**, 14.2 minutes. Diffed programmatically (title-keyed set comparison against the batch-2 full-suite baseline, not eyeballed): **8 fixed** — one date-picker anchoring flake (row 21's own class, not claimed as a fix), two tests already fixed by batch-2's own tail commits (confirmed ancestors of this round's own base, so not this round's work), and hydration-parity's 5 formerly-"not-run" rules, now real and green. **3 still red**, unchanged, all previously documented: date-picker's own row-21 anchoring test, `drag_and_drop_list.spec.ts`'s pre-existing intermittent cancelled-drag case, and `top-layer.spec.ts`'s ColorPicker Rule 8 (also row 21's class). **3 "new"** — mechanically new only because the file itself is new (`main-thread.spec.ts`): the three row-45 red-by-design opens, same signature every run. **Net: zero unexplained failures** — every red and every fix traces to a cited, already-documented cause; nothing was silently re-run until green or excluded without a reason. The isolated Tooltip-Escape flake did not recur at all in this full run, confirming it as a one-off contention artifact of the narrower 22-file batch, not a standing issue — worth a note only if it recurs in a future round. Dev server (`:8080`, PID 469) ran the whole round, never restarted; the SSG static server (`:8090`) was stopped after the full suite finished; `target-integration` (9.3GB) was deleted after capturing results, disk back to ~20GB free
