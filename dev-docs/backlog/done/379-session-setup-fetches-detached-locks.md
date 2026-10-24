---
id: 379
title: "Session setup fetches every committed Cargo.lock, so check-cargo-locks resolves offline in a fresh container"
status: "LANDED in batch 14: scripts/session-setup.sh step 5 runs cargo fetch --locked for the root…"
size: "S."
area: tooling
updated: 2026-10-24
---

# Row 379: session setup fetches the detached crates' locks

## Item

`scripts/check-cargo-locks.sh` resolves every committed `Cargo.lock` with `cargo metadata --locked --offline`. The detached crates `labs/carousel-lab` and `playwright/oracle/attr-synth` have their own locks, and `scripts/session-setup.sh` only fetched the root workspace, so in a fresh container their dependencies were missing from the cache and the gate could only report "not cached" for them.

## Status / trigger

LANDED in batch 14: `scripts/session-setup.sh` step 5 runs `cargo fetch --locked` for the root workspace and for each detached crate.

## Size

S.

## Detail

See the Status section and the batch 14 commits of the lane.
