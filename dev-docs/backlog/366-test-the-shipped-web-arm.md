---
id: 366
title: "Gates: run cargo test on the shipped web arm too (ssr_parity differs between the plain and server feature sets)"
status: "BACKLOG. Found in batch 6, when one run of the same code failed on each arm in turn."
size: "S: one gate step plus its reuse key."
area: tooling
updated: 2026-10-22
---

# Row 366: Gates: run cargo test on the shipped web arm too

## Item

`scripts/run-gates.sh` runs `cargo test --workspace`, which never enables preview's `server` feature or `dioxus-primitives/web`. That is the arm `dx build --ssg` ships. `ssr_parity`'s page test gives different results on the two arms: Popover and Tooltip sync the content id into the trigger's `anchor-name` in a web-only effect (`primitives/src/popover.rs:242`, `:385`), and the dropdown_menu account-switcher avatar changes load state only in the web arm. Batch 6 split the baseline per arm (`ROUTE_BASELINE_WEB`, commit 6aac0739), but the gates still only check the plain arm. This is the same class as row 139, where clippy never linted the `web` and `server` code until the three clippy arms existed.

## Status / trigger

BACKLOG. Found in batch 6, when one run of the same code failed on each arm in turn.

## Size

S: one gate step plus its reuse key.

## Detail

Add a `test-server` cargo gate, `cargo test -p preview --features server` (or only `ssr_parity` if the cost is too high: measure both), to `run-gates.sh`'s cargo-gate list and to `cargo-inputs-hash.sh`'s command lines, so reuse covers it. Update the CLAUDE.md gate sentence and `check-gate-reuse.sh` if its expectations list the gates.
