---
id: 320
title: "Skip the cargo gates when the Rust tree is unchanged since the last green full gate"
status: "BACKLOG (owner approved 2026-10-21)"
size: "S-M"
area: process
updated: 2026-10-21
---

# Row 320: Skip the cargo gates when the Rust tree is unchanged since the last green full gate

## Item

Skip the cargo gates when the Rust tree is unchanged since the last green full gate.

## Status / trigger

**BACKLOG (owner approved 2026-10-21)**

## Size

S-M

## Detail

fmt, the three clippy arms, `cargo test` and `check-primitives-features` are ~20 of the full gate's ~22 min. A CSS-, spec- or docs-only merge batch (most of batch A's fixes, the triage merge) re-ran them for nothing: about 2 of the 4 full gates on 2026-10-20. Construction: `run-gates.sh` records, per green run, a hash of the cargo inputs (every `*.rs`, `Cargo.toml`, `Cargo.lock`, `build.rs`, `*.ts` bundled into primitives, the toolchain) and reuses the cargo results for an identical hash, printing that it did so; any other change re-runs them. The summary must still say which tree each result came from, so `ALL GREEN` keeps meaning the quoted tree.
