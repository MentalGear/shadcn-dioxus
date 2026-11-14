---
id: 518
title: "An edit to a gate script, a rule file or CLAUDE.md re-runs the whole release suite: the site key covers the tree minus four prefixes, not the build's inputs"
status: "BACKLOG. Take it when a batch pays a full run for a change no build reads."
size: "M (inferred)."
area: tests
updated: 2026-11-14
---

# Row 518: the suite's site key from the build inputs

## Item

Row 513's site key hashes every tracked file outside `playwright/`, `dev-docs/`, `docs/` and `primitives/ts-test/`.
`scripts/` holds the build scripts and also every gate, so a gate-only change gives a new site key and the whole
suite runs, as do `agent-rules/`, `CLAUDE.md`, `.claude/`, `.github/` and `CHANGELOG.md`.

## Status / trigger

BACKLOG. Take it when a batch pays a full run for a change no build reads.

## Size

M (inferred).

## Detail

- **The construction.** One path at a time leaves the key, each with a line in the key tool's reviewed list and a
  self-test case that no build input names it; for `scripts/`, the build scripts are named and the rest is excluded.
  The include scan and the fail-closed rules of row 513 stay.
- **Observed 2026-11-14.** `carousel-wheel-replay.spec.ts` ("momentum reaching the edge draws no band") failed once in
  a release suite that ran beside three lanes compiling and probing, and passed on the re-run through row 513's reuse
  (9 files run, 282 reused, 225 s against 4,430 s). It replays recorded wheel events in real time and carries no
  `@timing` tag. The release suite runs with no lane building beside it.
