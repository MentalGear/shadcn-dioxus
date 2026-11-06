---
id: 266
title: "Repo cleanup: archive historical dev-docs, trim CLAUDE.md and the big dev docs, split backlog.md"
status: "Left: deleting the stale remote branches (the owner said go on 2026-10-26, but this environment's…"
size: "M-L"
area: process
updated: 2026-10-26
---

# Row 266: Repo cleanup

## Item

Owner (2026-10-19, thread root of "Repo cleanup and dev-doc refactor plan"): "cleaning up repo from historical stuff to
make it easy to understand and streamline. huge (dev doc) files should also be reported to me with a plan how to best
properly refactor them."

## Status / trigger

**Left: deleting the stale remote branches (the owner said go on 2026-10-26, but this environment's proxy refuses remote deletes, so the owner runs the command from their own machine with `/mnt/project-files/handoff/merged-remote-branches-2026-10-26.txt`), part (d).** (c) landed by construction (below); (d) is the final independent review by one Opus agent, which row 365 now carries. The main cleanup landed in batch 15: `CLAUDE.md` 47 KB to 8 KB, generated `dev-docs/gates.md` (`check-gates-doc.sh`), the `check-rustdoc` ratchet, the archive moves (living docs 2.4 MB to 0.55 MB), `decisions.md`, `guides/rust-conventions.md`, the index split into `backlog.md` and `backlog-done.md`, and the guards `check-history-comments.sh` and `check-comment-refs.sh`.

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
- (c) **DONE by construction.** The goal is "a public-API change is visible and deliberate", not `cargo public-api` (which needs a nightly rustdoc JSON and a new tool under rule C23). `primitives/public-api.txt` is a sorted text snapshot of every item reachable from `primitives/src/lib.rs` through `pub mod` and `pub use` (fns with signatures, structs with their `pub` fields and `#[props(..)]` attributes, enums with variants, traits, consts, aliases, macros, inherent methods and trait impls of the public types; each `cfg` is an annotation, so the file does not depend on build features). It is written by the `public-api-snapshot` bin of the `attr-spread-collision-checker` crate (`syn` only, plus `quote`, already locked), and `scripts/check-public-api.sh` is red when the tree's snapshot differs from the committed one, printing the diff and the one command that updates it (`--update`); the update is the deliberate act a reviewer reads in the diff. A body, a private item or a comment does not move it; a trailing comma is not API. Limits, stated in the tool's header: items a `macro_rules!` generates (`bridge::EventKind`, `bridge::EventField`) appear as their `use` line only. `--self-test` plants a `pub fn` (one added line), a private fn (no change) and runs the tool twice (identical). Registration in `run-gates.sh` is the main loop's.
- (d) A final independent review of the resulting state by one Opus agent, judgement only.
