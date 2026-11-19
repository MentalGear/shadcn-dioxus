---
id: 537
title: "The Rust-shape gates are text patterns: three reproduced bypasses and the construction behind them"
status: "BACKLOG, filed 2026-11-19. Not started. check-js-listeners.sh matches only addEventListener\\s*\\(…"
size: "M-L as filed (a syn checker for two…"
area: tooling
updated: 2026-11-19
---

# Row 537: The Rust-shape gates are text patterns

## Item

Three gates that decide a property of Rust code read it as text and each has a reproduced bypass (recipes and recorded
output: [`evidence.md`](../research/repo-evaluation-2026-11-19/evidence.md), section 2):

- `scripts/check-self-subscribing-effects.sh` exits 0 on `use_effect(move || { let v = count.cloned(); count.set(v + 1); })`
  (`Signal::cloned` is a tracked read); the `count()` form exits 1.
- `scripts/check-hooks-in-closures.sh` exits 0 for a hook in a helper function the closure calls, a hook under `if` and
  a hook under `for` (the tree calls a hook in a loop at `primitives/src/input_otp.rs:403`); a hook directly in the closure
  exits 1.
- `scripts/check-history-comments.sh` is a phrase list: "We switched this from a script to the bridge." and "Earlier this
  returned None." exit 0; "Previously it returned nil" exits 1.

The class: a pattern over text passes whatever the pattern does not spell. Construction to evaluate: one `syn`-based
checker for the Rust-shape gates (the workspace already has `scripts/attr-spread-collision-checker`, a `syn` tool), which
sees tracked reads by method, hooks by call graph and position, and comments as tokens; or clippy `disallowed_methods` for
the cases a method name decides (`Signal::cloned` inside `use_effect`). The history gate is prose and stays a heuristic;
its fix is a clearer rule, not a parser.

## Status / trigger

BACKLOG, filed 2026-11-19. Not started. `check-js-listeners.sh` matches only `addEventListener\s*\(` (`ADD_RE`); its
bypass was read, not run. The workflows are `workflow_dispatch`-only (`scripts/run-gates.sh:3-4`, the CI freeze of
[row 33](./033-3-browser-playwright-on-every-push-pr-sharded.md) and [row 4](./004-first-3-browser-ci-run-treat-as-calibration-not.md)),
so no gate runs unless a person or lane runs it; a sounder gate is worth little until that is settled.

**Taken over (2026-11-19).** The `syn` checker for the Rust-shape gates (`check-self-subscribing-effects`,
`check-hooks-in-closures`) is built by [row 542](./542-one-generated-item-table-read-by-the-gates.md) as consumers of its
generated item table. This row keeps `check-history-comments`, which reads prose, and the CI-freeze note.

## Size

M-L as filed (a `syn` checker for two gates is M; folding the others in is a larger step). The checker is now built in row 542, so what stays here is the history gate's clearer rule and the CI-freeze note, an S estimate.

## Detail

- **Proof:** each reproduced bypass becomes a red fixture of its gate; the existing self-tests stay green; the tree has
  no new hit, or each hit is a real defect fixed in the same change.
- Source: [`research/repo-evaluation-2026-11-19.md`](../research/repo-evaluation-2026-11-19.md), section 4.
