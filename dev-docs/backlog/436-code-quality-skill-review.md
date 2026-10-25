---
id: 436
title: "Code-quality skill review: code-review and simplify skills over the whole codebase, after the row 426-435 refactors"
status: "BACKLOG. It was in the plan next to row 365 but never had a row, so 365 ran without it. The owner…"
size: "M for the review. Each accepted finding…"
area: process
updated: 2026-10-25
---

# Row 436: Code-quality skill review

## Item

A subagent pass with the session's code-quality skills over the whole codebase. The owner asked for it before the adversarial board ("load a subagent with a code quality review skill and let it analyse the codebase"). `code-review` finds correctness bugs, each with a failure scenario. `simplify`'s criteria find reuse, simplification, efficiency and altitude cleanups.

## Status / trigger

BACKLOG. It was in the plan next to row 365 but never had a row, so 365 ran without it. The owner said on 2026-10-25: "wait for the code review etc after the refactor above". It runs once rows 426-435 have landed, so it reviews the refactored code, and it comes before row 383.

## Size

M for the review. Each accepted finding becomes its own row.

## Detail

- Four read-only reviewers in parallel on one detached worktree of the merged tree, each owning one area: `primitives/src` (Sonnet), `preview/src` (Sonnet), `scripts/` (Haiku) and `playwright/` (Haiku).
- Each reviewer invokes the `code-review` skill on its area as a path target (effort high for the Rust areas, medium for the others, findings only, no `--fix`). If the skill refuses a path target, the reviewer applies the same criteria by hand, then applies `simplify`'s criteria by reading.
- Per finding: severity, `file:line` verified, the failure scenario, the fix, and whether it is a class with a fix by construction. The report skips anything already in the backlog or the row 365 note. The output goes in `dev-docs/research/code-quality-<date>/<area>.md` and then to the owner for a decision before anything is built.
