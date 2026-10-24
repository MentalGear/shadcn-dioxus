---
id: 266
title: "Repo cleanup: archive historical dev-docs, trim CLAUDE.md and the big dev docs, split backlog.md"
status: "Extension part (a) LANDED 2026-10-23 (lane historyguards): the guards are in, before any file moves…"
size: "M-L"
area: process
updated: 2026-10-23
---

# Row 266: Repo cleanup

## Item

Owner (2026-10-19, thread root of "Repo cleanup and dev-doc refactor plan"): "cleaning up repo from historical stuff to
make it easy to understand and streamline. huge (dev doc) files should also be reported to me with a plan how to best
properly refactor them."

## Status / trigger

**Extension part (a) LANDED 2026-10-23 (lane historyguards): the guards are in, before any file moves** -- `scripts/check-history-comments.sh` (per-file ratchet, baseline 562 history lines in 229 files) and `scripts/check-comment-refs.sh` (every `row N` and cited repo path resolves; 254 dead references baselined, mostly the old `docs/` prefix), one extractor `scripts/comment-scan.py`, rule in [`history-in-comments.md`](../history-in-comments.md). The rest (the moves, trims, parts (b)-(d)) still waits for the Round 6 sweep: **APPROVED 2026-10-19 (owner: run it when safe, in the Round 6 sweep); starts once no feature lane, Round 4 or Round 1 work is unmerged; before row 267 and the 152b prune. Owner: the cleanup thread**

## Size

M-L

## Detail

Proposed by the cleanup thread (plan: https://claude.ai/code/artifact/19769a0d-67cd-4794-acb3-bc078458b8cb):
archive historical dev-docs (fork research, round logs, research for done rows); delete duplicates and superseded notes;
trim `CLAUDE.md` (~30 KB to ~6 KB) via a generated `dev-docs/gates.md`; rewrite `STATUS.md`; trim `plan.md`,
`conformance-harness.md`, `dev-loop.md` and `agent-model-fit.md`; split `backlog.md` into open and done; fix 36 dead
`docs/...` comment links. Runs in Round 6 after the open lanes merge, so no lane edits a file that is being moved. Owner approval 2026-10-19: "I generally approve, but make sure it is executed when it's safe and cant conflict with running lanes/agents - so maybe best in the area of the sweep". When Round 6 starts, the backlog thread messages the cleanup thread to run it.

**Extension, owner approved 2026-10-20** ("Yes" in the cleanup thread to adding no-drift playbook steps; evaluation:
https://claude.ai/code/artifact/22d5effe-b48d-4176-809f-f377acb563d2). Still gated on the Round 6 start:
- (a) **LANDED 2026-10-23 (lane historyguards; counts at the new gates' patterns, not the 1,000/340 estimate).** Guards land before any move: `scripts/check-history-comments.sh`, a per-file ratchet with a baseline TSV counting
  comment lines that carry dates, SHAs or "used to/previously/before the fix" (~1,000 lines in ~340 files at f62953a);
  and a code-comment link/ID check (every "row N" resolves, every cited repo path exists; 14 dead paths today, mostly
  the old `docs/` prefix), by extending `check-backlog-index.sh` or as a new gate.
- (b) A `dev-docs/decisions.md` index: current decision text, date, row link.
- (c) Optional, after row 267: a `cargo public-api` report for `dioxus-primitives` with a diff gate (needs nightly).
- (d) A final independent review of the resulting state by one Opus agent, judgement only.
