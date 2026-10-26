---
id: 445
title: "Phase 0.2: freeze the legacy scripts and sinks (script hashes, per-site sink freeze, js-fix grant)"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "S"
area: tooling
js-grant: none
adds: []
updated: 2026-10-26
---

# Row 445: Phase 0.2: freeze the legacy scripts and sinks (script hashes, per-site sink freeze, js-fix grant)

## Item

The legacy freeze (root doc section 5.3, item 12): a hash per legacy script, a per-site freeze of every remaining sink argument (delete only, never re-point), and a `js-fix:` grant for a legitimate bug fix, so that nothing legacy grows or is copied while the ports wait for their rows. Why: under scope 2 the legacy files stay an open sink until ported; the freeze is what keeps that cost from compounding.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (phase 0).

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

S

## Detail

- **Files owned:** `scripts/check-js-volume.*` and its baseline. Starts after item 0.1 merges: the two share `check-js-volume.*`.
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) (shares `check-js-volume.*`).
- **Proof:** The gate's mutation self-test (edit a frozen script, re-point a sink, raise without a grant: each red).
- **Hours (inferred):** about 4 (item 12 said 3; the table's 4 stands).
