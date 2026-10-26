# Which agent model fits which lane or task

Terms (owner, 2026-10-14): a **lane** is a work stream with its own file set and worktree ("the overlays lane owns
`top_layer.rs`"); an **agent** is the subagent that executes a lane or a task ("the theme lane, run by a Sonnet
agent"). Never "a Haiku lane".

This page is the rules. The running evidence log that produced them (2026-10-14 to 2026-10-21) is closed and archived in
[`archive/rounds/agent-model-fit-evidence.md`](./archive/rounds/agent-model-fit-evidence.md); the running log at the
end takes one row per delegated lane.

## The standing rule (owner)

Consider a Haiku subagent first for any delegated task, after a complexity assessment (owner 2026-10-25): the two
lists below are that assessment. A task on the "good fit" list goes to Haiku; a task on the "not a fit" list goes to
Sonnet; a task on neither is judged by how much construction it needs (a known idiom to copy is Haiku, a new decision is
Sonnet). The main loop keeps review, delegation and high-level decisions, and reviews every Haiku diff. Escalate only
after a failed attempt or for unusually judgment-heavy work (a headless state machine such as the Questionnaire).
**Never substitute Opus for a Sonnet or Haiku lane** (owner 2026-10-20: when Sonnet is limited, wait, or use Haiku where
it fits below); a lane stopped by a usage limit is resumed by message with its context, which beat a fresh agent.

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

## Running log (reopened 2026-10-24)

| Date | Lane | Agent | Result |
|---|---|---|---|
| 2026-10-24 | `find-photo-wall` (row 406: a find-in-page match landed on the item's estimate; shared `primitives/src/virtual/find.rs`) | Sonnet (root cause with a real find bar under Xvfb, shared-layer fix, spec) | Root-caused by measurement, fixed in the layer so the list and the wrapping grid share the step; a Haiku agent would not fit (primitives, a debugging hypothesis). |
| 2026-10-25 | `select-base-418` (row 418: `test:changed` diffs against the nearest of `origin/main` and `origin/queue/*`; closed file list in `playwright/tools/`, verified by `select.mjs --self-test` and `check-select-css.sh`) | Haiku | Fit: one new module, a self-test section on a throwaway repo, fast gates green, a manual proof on `queue/batch-20` (0 commits ahead, 4 before). It noted its own gaps (docs outside its list, no picker mutation) instead of widening the change. |
| 2026-10-26 | `js-bridge-3-0` (row 434 step 3 lane 3.0: the generic JS bridge `ts/bridge.ts`, the named-behaviour mechanism with zero real behaviours, `primitives/src/bridge/` with its one transport seam, `js_listener.rs` rebuilt on it, `check-bridge-behaviours.sh`, 61 `bun test` cases and a real-Chromium spec; shared primitives and new gate) | Sonnet (long lane, seven mid-lane scope changes folded in) | Fit, with a caveat: the shared-primitive and new-gate work is Sonnet's tier, and the additions (batched sample, `set`, the transport seam, the board's closed tables and behaviour layer replacing the interpreter) landed because the verbs are data-driven, but the board's redesign meant rewriting about half the lane after the first commit. The lane waited on the disk/hold rule before any cargo build; a Haiku agent would not fit (primitives, gates, a design with judgement calls). |
| 2026-10-26 | `js-ratchet-434r` (row 434: JS-volume ratchet; new `scripts/check-js-volume.sh`, `scripts/lib/ratchet.py`, a shared Rust tokenizer view, docs, a comment-only sweep) | Sonnet | Fit with limits: a new gate with a mutation self-test and a merge-base compare, no cargo; the shared-code part (`comment-scan.py` refactor) was checked by diffing the old and new extractor over every `.rs` file. A Haiku agent would not fit (new gate, shared code). |
| 2026-10-26 | `js-plan-docs-r4` and `js-plan-docs-r5` (row 434: fold adversarial board rounds 4 and 5 into the step 3 plan, rewrite `js-interop.md`, an audit-id ledger, then the plan as rows 444-471; docs only, no cargo) | Sonnet | Fit: each fold landed every synthesis item with fast gates and the link check green; one stale round-3 term survived in the residue doc and was fixed in review. Judgement-heavy wording (the admission and order rules) was written into the synthesis by the main loop first, so the lane folded decisions rather than making them. |
| 2026-10-26 | adversarial board rounds 4 and 5 (row 434: six read-only reviewers, lenses admission, whole-core, coherence, order-runtime, blank-slate-execution) | Opus (inherited) | Fit: read-only, evidence-cited reports against the Dioxus 0.7.9 sources; every DESIGN finding was confirmed by a second lens as at least a borderline DETAIL, none was invented. Sonnet was not tried for board members. |
