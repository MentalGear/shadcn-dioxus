---
id: 68
title: "Cross-worktree cargo build-directory hazard"
status: "CLOSED 2026-10-19 (duplicate of row 216)"
size: "—"
area: process
updated: 2026-10-19
---

# Row 68: Cross-worktree cargo build-directory hazard

## Item

Cross-worktree cargo build-directory hazard

## Status / trigger

**CLOSED 2026-10-19 (duplicate of row 216)**

## Size

—

## Detail

This session's `~/.cargo/config.toml` sets `build.target-dir` to one shared directory so parallel git-worktree lanes reuse compiled dependencies (`evaluation-draft.md`'s Environment notes) — intended as a speed optimization, but the sharing is not safe: cargo's own output-caching keyed the shared directory to another lane's build-script `OUT_DIR` and test binary, and **served them as if they were the querying lane's own**, not just slower-but-correct. **First found by Lane D (Resizable):** its own `cargo check -p preview`/`cargo test -p dioxus-primitives` runs in the shared directory silently picked up another lane's `preview` build-script `OUT_DIR` (containing `data_table`, a component Lane D never touched) and a foreign `dioxus-primitives` test binary — persistently, not a one-off flake. **Independently observed by three more lanes** (their own hand-backs each note re-verifying in an isolated `CARGO_TARGET_DIR` once this was known: Lane B/Drawer — "the shared one proved to leak state across concurrent worktrees"; Lane H/typeahead and Lane G/portal-toggle-calendar — "the shared build directory was found to serve cross-worktree-contaminated artifacts"; Lane C/Navigation Menu also switched to an isolated dir for the same reason) **and by integration round 1 itself**, the other direction: round 1's own gate run used the shared directory against its own instruction to use an isolated one, so its `cargo test --workspace`/`clippy` results are flagged in `round1-report.md` as "to be re-confirmed in round 2's isolated dir" rather than trusted outright. **Rule, stated once so it does not recur a fifth time:** one `CARGO_TARGET_DIR` per worktree, always — the shared-directory speed optimization is incompatible with concurrent lanes and should not be reintroduced without a mechanism that actually isolates build-script output and test binaries per worktree, not just per crate
