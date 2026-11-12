# Delegation and models

Read this when you are about to delegate a task, brief a lane agent or choose a model.

- **Use subagents as much as is possible and safe: this is the default, not an optimization.** Any work that can be
  split into lanes with disjoint file sets should be, and the lanes are dispatched in ONE batch so they run
  concurrently, never one after another. The main loop scopes the lanes, reviews what comes back and runs the checks; it
  does not do the work itself. Serialize only for a real reason, and name it: two lanes would edit the same files, one
  genuinely needs another's output, or a shared resource allows a single holder (the cargo lock, one dev server, one
  port). When a resource is the constraint, still parallelize the *editing* and serialize only the build or test step
  at the end. Before dispatching, tell each lane which files it owns and which it must not touch; before starting a
  batch, capture whatever baseline the work will later be verified against, since it cannot be recovered afterwards.
- **Models** ([`dev-docs/agent-model-fit.md`](../dev-docs/agent-model-fit.md)): **every delegated task gets a complexity
  assessment, and the lowest tier that fits takes it: Haiku, then Sonnet, then Opus** (owner 2026-10-25, 2026-11-12).
  Haiku takes what the assessment rates simple: inventories and audits, run-and-report jobs, and edits with a closed
  file list, a known idiom and a verifying gate. Sonnet takes what it rates complex: debugging and root-causing,
  primitives or shared code, new gates or specs, long lanes that wait on shared resources, and reports that must
  separate measured from inferred. Opus takes what it rates judgment-heavy across a whole design: evaluating or revising
  a plan, an architecture review, an adversarial review, a question that needs a large body of material read whole. The
  main loop stays clear: it keeps review, delegation and high-level decisions, and reviews every Haiku diff. A higher
  tier is chosen for the task's complexity or after a failed attempt, never because a lower tier is rate-limited (wait,
  or use Haiku where it fits). Every brief states the owner's goal in the owner's words. Delegate debugging to Sonnet
  (reproduce, bisect, root-cause, propose the construction); the main loop reviews the evidence and decides, and runs
  no long hands-on debugging loops.
- **Every brief opens with a premise check and a stop condition.** A row's claims about the source (the files it owns,
  the sites it names, the verbs or rows it depends on, its grant) are prose and drift; the lane verifies them against
  the source before its first edit and, where one is false, stops and reports instead of working around it. Measured
  2026-11-12: a lane found row 451's missing bridge rows in under a minute this way, and row 448's "dead" sites were
  caught by a read-only check before a deletion lane was sent.
- A **lane** is the work stream (file set, worktree); the **agent** is the Haiku, Sonnet or Opus subagent that executes
  it, never "a Haiku lane". Every lane agent gets [`dev-docs/lane-rules.md`](../dev-docs/lane-rules.md) plus its own
  prompt (worktree, build lock, mtime touch, gate procedure, merge queue, web-request rules). Each delegated lane adds a
  row to the running log in `agent-model-fit.md` (model, task, whether it fit).
