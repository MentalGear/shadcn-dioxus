---
id: 365
title: "Final sweep: refactor review with the deconstruct-to-root and solve-by-construction exercise, proposals to the owner"
status: "BACKLOG (owner 2026-10-22, backlog thread: \"final sweep, check for refactors and also run there the…"
size: "M for the review. Each accepted…"
area: process
updated: 2026-10-22
---

# Row 365: Final sweep: refactor review and root-cause exercise

## Item

Once every other row has landed, sweep the whole repo for refactors. Deconstruct each recurring problem or duplicated concern to its root cause, find the solve-by-construction model for it (as row 360's collection-model evaluation did), and bring the owner proposals before building anything.

## Status / trigger

BACKLOG (owner 2026-10-22, backlog thread: "final sweep, check for refactors and also run there the deconstruction until root and solve-by-construction exercise, then come back to me with proposals"). Runs last in Round 6, after the cleanup (266), the file split (267) and the Rust style items (355).

## Size

M for the review. Each accepted proposal becomes its own row.

## Detail

- **Sources:** the gates' allowlists and baselines (each entry is a known exception), duplicated helpers across `primitives/src` and `preview/src/components`, backlog rows closed as "instance fixed, class not swept", and the lane reports in `dev-docs/agent-model-fit.md`.
- **Per finding:** the invariant at stake, where it is broken or duplicated (`file:line`), the construction that makes the class unable to recur, what it subsumes and what it does not, and the gate or test that keeps it closed.
- **Output:** a design note under `dev-docs/research/`, plus one decision card per proposal for the owner. Nothing is built before the owner's answer.
