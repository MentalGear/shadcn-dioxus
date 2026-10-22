---
id: 366
title: "Gates: run cargo test on the shipped web arm too (ssr_parity differs between the plain and server feature sets)"
status: "LANDED in lane gatetools. Found in batch 6, when one run of the same code failed on each arm in…"
size: "S: one gate step plus its reuse key."
area: tooling
updated: 2026-10-22
---

# Row 366: Gates: run cargo test on the shipped web arm too

## Item

`scripts/run-gates.sh` runs `cargo test --workspace`, which never enables preview's `server` feature or `dioxus-primitives/web`. That is the arm `dx build --ssg` ships. `ssr_parity`'s page test gives different results on the two arms: Popover and Tooltip sync the content id into the trigger's `anchor-name` in a web-only effect (`primitives/src/popover.rs:242`, `:385`), and the dropdown_menu account-switcher avatar changes load state only in the web arm. Batch 6 split the baseline per arm (`ROUTE_BASELINE_WEB`, commit 6aac0739), but the gates still only check the plain arm. This is the same class as row 139, where clippy never linted the `web` and `server` code until the three clippy arms existed.

## Status / trigger

LANDED in lane gatetools. Found in batch 6, when one run of the same code failed on each arm in turn.

## Size

S: one gate step plus its reuse key.

## Detail

Add a `test-server` cargo gate, `cargo test -p preview --features server` (or only `ssr_parity` if the cost is too high: measure both), to `run-gates.sh`'s cargo-gate list and to `cargo-inputs-hash.sh`'s command lines, so reuse covers it. Update the CLAUDE.md gate sentence and `check-gate-reuse.sh` if its expectations list the gates.

**Landed.** `test-server|cargo test -p preview --features server` sits after `test` in `run-gates.sh`; `CARGO_GATES` is now `^(clippy-|test(-server)?$|...)` (the fast-gate skip pattern in `lane-rules.md` follows), `check-gate-reuse.sh` requires it reusable, the reuse key picks its command line up from the gate list automatically. Measured on this box (slot 0/1, other lanes compiling, so noisy): `ssr_parity` alone 9 tests, 39.6 s of test time (479 s including a cold compile of the server arm in a cold slot-1 target); the full server arm 112 tests, 38.6 s of test time, 295-442 s wall including the recompile of `preview` that the feature flip forces; plain `cargo test --workspace` 335 s (1,006 + 116 + 56 + 191 tests). The full set is 1.3x the plain gate, under the 2x cut-off, so the full set was chosen; in the gate run it took 365.8 s (193-247 s with a warm tree in the red-proof run). Proof: `run-gates.sh --only '^test-server$'` PASS; with a deliberately failing `#[cfg(all(test, feature = "server"))]` test appended to `preview/src/ssr_parity.rs`, `test` stayed PASS (247 s) and `test-server` went FAIL rc=101 (193 s); reverted.
