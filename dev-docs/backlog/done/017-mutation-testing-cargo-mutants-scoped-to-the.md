---
id: 17
title: "Mutation testing (cargo-mutants), scoped to the algorithmic modules that already carry unit tests (slider…"
status: "Landed 2026-08-31, PR #13 — baseline was 37% (397 survivors, ~350 genuinely unit-killable); all…"
size: "S to set up"
area: build
updated: 2026-08-31
---

# Row 17: Mutation testing (cargo-mutants), scoped to the algorithmic modules that already carry unit tests (slider…

## Item

Mutation testing (`cargo-mutants`), scoped to the algorithmic modules that already carry unit tests (slider math, calendar, virtualizer, `text_search`, selection, pointer, …)

## Status / trigger

**Landed 2026-08-31, PR #13** — baseline was 37% (397 survivors, ~350 genuinely unit-killable); all eight targeted test additions landed same day (per-file scores now 58–91%, see [`mutants-baseline.md`](../../mutants-baseline.md)), and the scoped command is a `workflow_dispatch` job, `.github/workflows/mutants.yml` (recommendation adopted: not nightly)

## Size

S to set up

## Detail

**2026-09-26 (audit correction):** baseline DONE (PR #13, 2026-08-31, see `mutants-baseline.md`); periodic re-run optional, not required. Automates what the red-first discipline does by hand — a surviving mutant is exactly the "test cannot go red" finding Phase 2 hit twice (slider 2.1, virtual_list 2.2). Do NOT scope it over component render code: those mutants are only killable by the Playwright suite, and a wasm rebuild + e2e run per mutant is economically impossible; the oracle red-first rule covers that layer instead. **2026-09-19 (batch 3, leftovers lane):** `cargo-mutants` v27.1.0 installed but not run this round (disk-space gate deferred it) — command to resume: `cargo mutants -f 'primitives/src/slider.rs' -t 60 -j 1 -o <scratch-dir>`
