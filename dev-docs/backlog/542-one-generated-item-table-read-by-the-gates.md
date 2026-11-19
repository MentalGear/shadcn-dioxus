---
id: 542
title: "One generated item table of the workspace, read by the gates; handler decisions as pure outcome functions"
status: "BACKLOG, steps 1 and 2 approved by the owner 2026-11-19. Not started, not scheduled; the order in…"
size: "S-M for the table and the first gate; S…"
area: tooling
updated: 2026-11-19
---

# Row 542: one generated item table of the workspace

## Item

Two steps, approved by the owner 2026-11-19 ("yes for 1 and 2"):

1. **The item table.** A `syn` binary in the existing workspace member `scripts/attr-spread-collision-checker` (it already
   parses Rust and `rsx!` and produces the public-API snapshot) writes one table of every function in the workspace: path,
   span, kind (component, hook, pure, effectful, unknown), length, `cfg`, and whether a unit test reaches it. The table is
   generated on each run and never committed, so it cannot drift. Gates read it.
2. **Handler decisions as pure outcome functions.** The branching decisions inside hooks and `rsx!` handlers move into pure
   functions that return an outcome value, and the handler applies it. The pattern exists: `KeyOutcome`
   (`QuestionnaireState::key_down`), `DateSegmentEffect` (`date_segment_key_effects`), `ReorderCommand` (`command_for_key`).
   The two calendar keyboard handlers go first: they duplicate each other (`RangeCalendar`'s `onkeydown`, about 60 lines,
   and `calendar/single.rs`) and only `horizontal_day_step` (`calendar/dates.rs:235`) has a unit test.

Step 3, the code graph explorer reading the table, is not approved and nothing of it is built; it is the open question
below.

## Status / trigger

BACKLOG, steps 1 and 2 approved by the owner 2026-11-19. Not started, not scheduled; the order in "Next" waits for the
owner. Step 3 is an open question for the owner.

## Size

S-M for the table and the first gate; S per handler for step 2. An estimate from the note, nothing built.

## Detail

**What it takes over.** The function-length gate of [row 533](./533-tigerstyle-slice-kernel-and-function-length.md) (the
per-crate shrink-only count, primitives 17 and cli 5) and the `syn` checker of
[row 537](./537-rust-shape-gates-are-text-patterns-and-have-bypasses.md) (the Rust-shape gates that are text patterns) are
consumers of this table and are built here, once. Row 533 keeps the kernel lints, the assertions, the const assertions and
the `dx_input` cap; row 537 keeps `check-history-comments`, which reads prose, and the CI-freeze note.

**The gates that read the table, in order.**
1. The long-function count (row 533's gate), per crate, shrink-only, through `scripts/lib/ratchet.py`.
2. A purity gate on declared core modules (a TSV, shrink-only for exceptions): `chart/engine`, `virtual/layout`,
   `data_view`, `date_picker/segment`, `questionnaire/model`, `state_attrs` are already pure.
3. Shrink-only counts of handler lines (1,685 today) and of hooks no unit test reaches.
4. The two text-pattern gates of row 537 (`check-self-subscribing-effects`, `check-hooks-in-closures`) re-expressed over the
   table.

**First lane, proof and kill condition.** Step 1 alone for primitives and cli, plus the long-function count on it.
- **Proof:** it reproduces 17 and 5; it agrees with the 60 hand-read functions on at least 57 of 60 and with the 17 long
  ones on 17 of 17; it lists `sleep.rs`'s `gloo_timers` call (behind `cfg(target_family = "wasm")`, absent from the SCIP
  graph); it runs in under 10 s warm on the workspace; UNKNOWN stays under 2 % of functions.
- **Kill:** if it needs type inference to classify (UNKNOWN above 10 %), or runs above 60 s, stop at row 533's count.

**The census that motivates it (measured 2026-11-19 on `b7b83ed`; method in the note).** A stdlib token scanner over
non-test Rust, a signal read counted as effectful.
- 54.1 % of non-component functions in primitives are pure, by a scanner whose measured error is 5 of 60 hand-read
  functions, all understating purity (a name collision, one `Debug` impl); the share is a lower bound.
- 3,342 lines of hooks and 1,685 lines of event handlers are reachable only through a `VirtualDom` or Playwright; 9 of 147
  hook names appear in any unit test.
- 136 of 657 body lines in 12 components read whole (21 %) are a decision that could be a pure function.
- 10 of the 17 long non-component functions in primitives are already pure, which is why the length count alone does
  little for the pure-core question.

**Residue.** There is no type information: purity is a token policy, and reading a signal counts as effectful, so a
context method that only reads stays shell. A call through a trait or a closure parameter can stay UNKNOWN. The runtime
type panics (row 536) and the overlay memory growth (row 534) are untouched. The 136-line figure rests on 12 components
and may not generalise. The purity gate guards modules, not single functions inside mixed files.

Source: [`research/functional-core-and-graph-ssot-2026-11-19.md`](../research/functional-core-and-graph-ssot-2026-11-19.md),
sections 3 to 5. That note orders the work as four steps (table, gates, shell rule, analyser); this row's steps 1 and 2 are
the note's steps 1 and 3, its step 2 (the gates) is the gate half taken over from rows 533 and 537 above, and step 3 here is
the note's step 4.

## Open question for the owner: the code graph explorer

Step 3, not approved, nothing built. The explorer (`MentalGear/code-analyser`, a separate repository this session could
only read) takes its edges from one `rust-analyzer scip` index. It finds items itself with regular expressions
(`comment_metrics.py:40-44`: `_PUB_ITEM = re.compile(r"^\s*pub\s+(?:(?:async|unsafe|const)\s+)*(fn|struct|enum|trait|const|static|type|mod)\s+(\w+)")`
and `_ANY_ITEM = re.compile(...)`). It drops the index's occurrence ranges when it builds the graph (`build_graph.py:40-45`:
the occurrence loop reads field 2, the symbol, and field 3, the roles, and keeps `d['occ'].append((sym,roles))`; the range
field is never read).

**The construction put forward, instead of "the analyser reads a second graph": each fact comes from the source that can
know it.**
- **Type-resolved facts stay with the compiler index.** The explorer keeps the occurrence ranges it drops today. That
  gives per-function allocation call counts that tell a `String::clone` from an `Rc::clone` (the index holds 752
  `format!`, 412 `String::clone`, 319 `Vec::clone` and 46 `Rc::clone` occurrences, each with file and line), so row 486's
  decision stands as written.
- **Syntactic facts come from this repo's item table.** Length, kind, `cfg` and reachability by a unit test reach the
  explorer through the project config it already loads (`.codegraph/project.py` already supplies the comment tokenizer),
  replacing the explorer's own regular-expression item scan: one derivation fewer than today, not one more. The table
  contributes no edges, so two graphs cannot disagree.

**Value.** The explorer would show per function what the gates enforce, and rank functions by typed allocation calls. It
stays a dev tool and the gates carry the enforcement, so the value is moderate. It is worth doing only after the table has
passed its first-lane proof. **Effort:** S in `MentalGear/code-analyser`, S here for the project hook (estimates).

**What the owner decides.** [Row 486](./486-code-graph-explorer-tools-codegraph.md), line 83, reads "rust-analyzer is the
one analysis source. crabmap, Crabviz and every other young or little-used tool are out". By its wording that rules out
young third-party analysis tools; whether a table from this repo's own checker counts against it is the owner's reading to
make. Until then step 3 is not filed work.
