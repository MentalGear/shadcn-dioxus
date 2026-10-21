---
id: 254
title: "Stale Cargo.lock in the detached crates (attr-synth, carousel-lab) rewritten by lane spec runs"
status: "FIXED 2026-10-20 (gate check-cargo-locks.sh; both locks refreshed)"
size: "S"
area: process
updated: 2026-10-20
---

# Row 254: Stale Cargo.lock in the detached crates (attr-synth, carousel-lab) rewritten by lane spec runs

## Item

`playwright/oracle/attr-synth/Cargo.lock` and `labs/carousel-lab/Cargo.lock` went stale on main when 98e7ca1b dropped `dioxus-sdk-time` from `primitives/`, because nothing in the gates builds those detached crates. Any lane whose spec run built attr-synth (hydration-parity Rule 4c) then carried a rewritten lock as an unexplained side effect.

## Status / trigger

**FIXED 2026-10-20 (gate check-cargo-locks.sh; both locks refreshed)**

## Size

S

## Detail

Seen twice on 2026-10-20 (lanes theme-touch and initial-checked), so it was a class, not a slip. Construction: `scripts/check-cargo-locks.sh` (in `run-gates.sh`, described in CLAUDE.md) resolves every committed `Cargo.lock` against its manifest with `cargo metadata --locked --offline` and fails when cargo would rewrite it; a dependency merely missing from the cache is not a failure; no network. Verified red on main's two stale locks, green after `cargo metadata --offline` refreshed them (attr-synth: `dioxus-sdk-time` -> `gloo-timers`/`tokio`; carousel-lab: also drops `palette`/`num-integer`, which primitives put behind features).
