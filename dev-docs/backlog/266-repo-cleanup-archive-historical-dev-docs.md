---
id: 266
title: "Repo cleanup: archive historical dev-docs, trim CLAUDE.md and the big dev docs, split backlog.md"
status: "BACKLOG (owner request 2026-10-19; plan approval pending; scheduled with the Round 6 sweep). Owner…"
size: "M-L"
area: process
updated: 2026-10-19
---

# Row 266: Repo cleanup

## Item

Owner (2026-10-19, thread root of "Repo cleanup and dev-doc refactor plan"): "cleaning up repo from historical stuff to
make it easy to understand and streamline. huge (dev doc) files should also be reported to me with a plan how to best
properly refactor them."

## Status / trigger

**BACKLOG (owner request 2026-10-19; plan approval pending; scheduled with the Round 6 sweep). Owner: the cleanup thread**

## Size

M-L

## Detail

Proposed by the cleanup thread (plan: https://claude.ai/code/artifact/19769a0d-67cd-4794-acb3-bc078458b8cb):
archive historical dev-docs (fork research, round logs, research for done rows); delete duplicates and superseded notes;
trim `CLAUDE.md` (~30 KB to ~6 KB) via a generated `dev-docs/gates.md`; rewrite `STATUS.md`; trim `plan.md`,
`conformance-harness.md`, `dev-loop.md` and `agent-model-fit.md`; split `backlog.md` into open and done; fix 36 dead
`docs/...` comment links. Runs in Round 6 after the open lanes merge, so no lane edits a file that is being moved.
