---
id: 82
title: "Batch-2 integration outcome: full suite 775/12/11, net 0 unaddressed regressions"
status: "Landed 2026-09-19 (batch-2 integration, HEAD f7987aa)"
size: "—"
area: process
updated: 2026-09-19
---

# Row 82: Batch-2 integration outcome: full suite 775/12/11, net 0 unaddressed regressions

## Item

Batch-2 integration outcome: full suite 775/12/11, net 0 unaddressed regressions

## Status / trigger

Landed 2026-09-19 (batch-2 integration, HEAD `f7987aa`)

## Size

—

## Detail

**19 commits (12 lane picks + 7 integration-authored) landed on `claude/roadmap-evaluation-6eiahc`, base `cbae695`.** Gates green in an isolated `CARGO_TARGET_DIR`: the 6 `scripts/check-*.sh` guards (4 pre-existing + the 2 new ones from row 79), `cargo fmt --all -- --check`, `cargo check --workspace --all-targets` and `--features web`, `cargo test --workspace` 326/0/2, `cargo clippy --workspace --tests --examples -- -D warnings`, `cd preview && npx stylelint "src/**/*.css"`, and all three `cargo doc` invocations. Targeted Playwright (15 lane-owned spec files) 227/8/1 after row 81's fixes — the one remaining red is ColorPicker Rule 8 (row 21's own known anchor-freeze class). **Full suite (one run, not re-run after the two post-full-suite fixes): 775 expected, 12 skipped, 11 unexpected.** Diffed programmatically (title-keyed set comparison, not eyeballed) against the pre-batch-2 baseline (`pw-round3-full`): **2 fixed** — `oracle/tier2-html/top-layer-ink.spec.ts`'s light/dark "select" cases, already fixed on `cbae695` before this batch started (matching the prior round's own "fixed in #59" note; nothing in batch 2 touched them). **8 known, unchanged:** date-picker anchoring — **both titles named explicitly**, per row 21's own updated exclusion list — "anchors the popup next to its trigger, not viewport-centered or offset" and "offset to trigger is unchanged after scrolling (CSS anchor tracks scroll natively)", the second reclassified into row 21's class this round rather than treated as new; the 5 `oracle/hydration-parity.spec.ts` cases (`ECONNREFUSED 127.0.0.1:8090` — no SSG server started for this batch, by design, "not run" rather than "failed"); ColorPicker Rule 8 (row 21); `drag_and_drop_list.spec.ts`'s "cancelled mouse drag" case (confirmed still intermittent on re-run in isolation, 2/3 passed, same avatar-initials text-drift signature its baseline name implies — not a regression, no batch-2 lane touches `drag_and_drop_list.rs` or its spec). **3 new, all fixed and re-verified before this row was written** — row 81's three timing fixes (`escapesClip`'s animation wait, the drawer short-drag pause, the sortable-table R1 wait), each re-run 10/10 in isolation after its fix, with the owning file passing in full 2x more. **Net effect: 0 unaddressed regressions.** The full suite was deliberately NOT re-run after the two post-full-suite-run fixes (`4f05f75`, `f7987aa`) — each fix's own isolated re-run (10/10, full file green twice) is the verification, not a second full-suite pass, matching the same disk-pressure coordination that deleted `target-integration` early to free space for concurrent batch-3 lanes (6.0GB freed, disk 5-6GB → 11GB free, before the full run finished; all cargo-gate evidence was already captured by that point). **New dx-serve trap, added to `dx-serve-hot-reload.md` as trap 6** (cross-referenced from `dev-loop.md`): near-simultaneous saves to two different files can make the watcher drop the rebuild entirely — no `Hotreloading:`, no `Build completed`, the process idles at ~2% CPU — rather than merely mis-time it (distinct from `dev-loop.md`'s existing traps 4 and 5). A trivial re-save of either file recovers it; observed once, on the shared dev server during integration — not restarted, per this task's own non-negotiable rule, but nudged with a trivial add/remove blank-line edit, which fired a rebuild ~20s later that picked up every pending change
