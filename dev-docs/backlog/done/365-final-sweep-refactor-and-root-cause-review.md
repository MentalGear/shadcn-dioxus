---
id: 365
title: "Final sweep: refactor review with the deconstruct-to-root and solve-by-construction exercise, proposals to the owner"
status: "DONE (review). Proposals filed as rows 426-435."
size: "M for the review. Each accepted…"
area: process
updated: 2026-10-25
---

# Row 365: Final sweep: refactor review and root-cause exercise

## Item

Once every other row has landed, sweep the whole repo for refactors. Deconstruct each recurring problem or duplicated concern to its root cause, find the solve-by-construction model for it (as row 360's collection-model evaluation did), and bring the owner proposals before building anything.

## Status / trigger

DONE (review). Proposals filed as rows 426-435.

## Size

M for the review. Each accepted proposal becomes its own row.

## Detail

- **Sources:** the gates' allowlists and baselines (each entry is a known exception), duplicated helpers across `primitives/src` and `preview/src/components`, backlog rows closed as "instance fixed, class not swept", and the lane reports in `dev-docs/agent-model-fit.md`.
- **Per finding:** the invariant at stake, where it is broken or duplicated (`file:line`), the construction that makes the class unable to recur, what it subsumes and what it does not, and the gate or test that keeps it closed.
- **Output:** a design note under `dev-docs/research/`, plus one decision card per proposal for the owner. Nothing is built before the owner's answer.

## Outcome (2026-10-25)

The sweep note is `dev-docs/research/final-sweep-2026-10-25.md` (a read-only review of `queue/batch-26` at `de2725bd`; nothing built, no cargo run). The owner approved all ten proposals on 2026-10-25 ("All ten" on the decision card), P10 included, which reverses row 53's keep-separate decision. They are filed as rows 426-435: P1 426 (scrim and visually-hidden recipe), P2 427 (spec page and read helpers), P3 428 (`..props` wrappers), P4 429 (gate kit), P5 430 (derived manifests), P6 431 (test harness and `use_controlled`), P7 432 (anchor attribute), P8 433 (derived slot variables), P9 434 (JS in TypeScript), P10 435 (one menu family; P10a lands first). Originally filed BACKLOG 2026-10-22 from the backlog thread ("final sweep, check for refactors and also run there the deconstruction until root and solve-by-construction exercise, then come back to me with proposals"), running last in Round 6 after 266, 267 and 355.
