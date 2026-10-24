# Which agent model fits which lane or task

Terms (owner, 2026-10-14): a **lane** is a work stream with its own file set and worktree ("the overlays lane owns
`top_layer.rs`"); an **agent** is the subagent that executes a lane or a task ("the theme lane, run by a Sonnet
agent"). Never "a Haiku lane".

This page is the rules. The running evidence log that produced them (2026-10-14 to 2026-10-21) is closed and archived in
[`archive/rounds/agent-model-fit-evidence.md`](./archive/rounds/agent-model-fit-evidence.md); lanes add no rows.

## The standing rule (owner)

Sonnet subagents do delegated work (implementation, research, validation runs, debugging); the main loop keeps review,
delegation and high-level decisions. Escalate to a stronger model only when a task has repeatedly failed or is unusually
judgment-heavy (a headless state machine such as the Questionnaire). **Never substitute Opus for a Sonnet or Haiku lane**
(owner 2026-10-20: when Sonnet is limited, wait, or use Haiku where it fits below); a lane stopped by a usage limit is
resumed by message with its context, which beat a fresh agent.

## Haiku is a good fit for

- Read-only inventories with a crisp, checkable output: grep sweeps, counts per file, file:line lists, "which of these
  files already do X".
- Read-only audits against an external reference when the comparison is textual (demo text, labels, prop names), as long
  as the brief names the exact files and the output format.
- Small mechanical edits with a closed list of target files, a known idiom to copy and a verifying gate (wrapping
  `:hover` sheets, demo variants, closing rows whose status already says done), reviewed by the main loop afterwards.
- Run-and-report jobs (a release suite run, an install smoke) that classify nothing: the failure triage stays Sonnet.
- Small, idiom-following component additions with their own specs WITH a reviewer and a brief that says to wait in the
  foreground for queued work.

## Haiku is not a fit for (use Sonnet)

- Debugging and root-causing: anything that needs a bisect or a new hypothesis.
- Primitives and shared code (`primitives/src/**`, the theme, `top_layer.rs`, focus and placement logic), where a wrong
  construction reaches every consumer; and a change to a shared component's event or attribute plumbing needs a
  cross-component spec run before merge, not a code read, whoever wrote it.
- Designing a gate or a spec whose job is to remove a failure class: the construction needs judgment.
- Long lanes that wait on shared resources (the build lock, other lanes): a Haiku agent tended to hand back early with
  work still queued.
- Reports that must separate "measured" from "inferred" (pass and fail tallies across reruns).

## In every lane brief, whatever the model

1. [`lane-rules.md`](./lane-rules.md) plus the lane's own prompt; the files it owns and the ones it must not touch.
2. Before handback: merge the session branch into the lane branch and run `scripts/run-gates.sh --fast`; gates that
   landed after the agent branched otherwise go red only on merge (row 221).
3. Never send the user's email, name or any account identifier to an external service; a generic User-Agent such as
   `shadcn-dioxus-research` (a research agent once put the account email in a crates.io header, row 221).
4. Every cargo or dx command through the build lock (`scripts/lane-cargo.sh`, `scripts/lane-ssg.sh`).
5. Waiting on the lock or a queued build is not a reason to hand back: block on it, in the foreground.
6. Push the branch after every commit; nothing lives only in the container (row 250).

## Whatever the model, the main loop

Reviews the diff, and re-verifies lane results that ran on a shared cargo target on the merged tree: the mtime hazard
([`lane-rules.md`](./lane-rules.md), "Shared targets judge freshness by mtime", row 216) makes any model's "clippy clean"
on a shared or copied target untrustworthy until the sources were touched. Squash a lane's wip commits on merge.
