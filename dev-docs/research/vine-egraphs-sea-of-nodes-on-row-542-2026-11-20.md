# Vine, e-graphs, the sea of nodes and their alternatives on top of row 542: evaluation of 2026-11-20

The owner's question, asked before the next batch may start: "let's first evaluate whether to better use the vine
programming language or alternatives eg program expression grammars, e-graphs, or the sea of nodes., on top of this".
"This" is [row 542](../backlog/542-one-generated-item-table-read-by-the-gates.md), approved 2026-11-19: (1) one item table
of every function, generated each run by a `syn` binary in `scripts/attr-spread-collision-checker` and never committed;
(2) branching decisions in hooks and handlers move into pure functions returning an outcome value; (3) the code graph
explorer reads the table for syntactic facts and keeps its `rust-analyzer scip` index for typed facts. "Program
expression grammars" is read as **Program Expression Graphs** (Tate et al., POPL 2009), the representation built for
equality saturation, which sits beside e-graphs and the sea of nodes; **Parsing Expression Grammars** (Ford, POPL 2004)
is the other reading and is treated in section 4F.

**Method.** The jobs are taken from the repository's documents (section 1), each candidate is judged only against the
jobs it can apply to, and every external fact comes from pages fetched on 2026-11-20 and saved with receipts (sources at
the end). Two measurements were run on `642f304` with stdlib Python (section 2); nothing was built with cargo.
Measurements are marked **measured**; values, estimates and the recommendation are **judgement**. Effort units: S is
less than one lane (hours), M one lane, L several lanes.

## 1. The jobs a candidate would have to do here

- **(i) The language of pure logic and decisions.** Recorded: "every behaviour's decision is one pure Rust function over
  typed inputs, the same function on every renderer" ([decisions](../decisions.md), 2026-11-11). On the page it runs as the
  decision kernel, a crate with "No dependencies, by design" whose gzip budget is 12,914 bytes and whose budget row reads
  12914 (`scripts/check-decision-kernel.budgets.tsv`; the 2026-11-19 evaluation lists it as "12,914 of 12,914").
  Components are installed as source and built by the user's ordinary Rust toolchain, pinned here to `1.97.0`.
- **(ii) The representation of the generated facts.** Row 542 makes an item table: path, span, kind, length, `cfg`, test
  reachability. The open question is whether a function-internal data and control graph would earn its cost. Its proof
  requires the table to list `sleep.rs`'s `gloo_timers` call behind a wasm `cfg`, which the SCIP graph misses, and to run
  "in under 10 s warm on the workspace".
- **(iii) The engine that evaluates gate rules over facts.** `scripts/run-gates.sh` lists 111 check scripts; 71 embed
  Python in a heredoc (both measured, section 2). Two Rust-shape gates were bypassed in reproductions
  (`check-self-subscribing-effects`, `check-hooks-in-closures`; [row 537](../backlog/537-rust-shape-gates-are-text-patterns-and-have-bypasses.md));
  the third bypass is the prose gate `check-history-comments`.
- **(iv) Finding duplicated logic and shared abstractions.** Named in the documents: the two calendar key handlers
  (row 542 step 2), the menu-layer block in `dropdown_menu.rs` and `context_menu.rs` (evaluation, D-14), Tooltip and
  HoverCard (row 528, `HoverSurface`). The class recurs: `check-test-harness-controlled.sh` holds "A concern implemented once in
  `primitives/src` is used, not copied" because eight private test-harness copies and three hand-rolled controlled
  memos had grown.
- **(v) Showing a port equivalent.** Legacy JavaScript behaviours become Rust `decide` functions; today one set of vectors
  (`primitives/conformance/behaviours.json`) runs "against the Rust `decide`, once called directly and once through the
  byte route the page uses" (`decision-kernel/tests/vectors.rs`), and the witness harness of row 446 admits effects.
- **(vi) The typed questions a syntax table cannot answer.** Row 542's residue: "There is no type information"; a call
  through a trait or a closure parameter stays UNKNOWN. Step 3 keeps type-resolved facts with the compiler index (which
  clone is an `Rc` bump, which call allocates). The runtime type panics of [row 536](../backlog/536-generic-components-check-value-types-at-run-time.md)
  are a type design question, not a missing fact.
- **(vii) The kernel's own internal form.** `kinds!` registers two kinds (`fixture`, `otp-filter`); each kind is one
  `decide` over `Config`, `Snapshot` and `Event`.
- **(viii) Added: making an effect visible at a function's boundary.** Row 542 gate 2 holds declared core modules pure,
  and the census classes a function EFFECTFUL when its signature or body names a reactive type. This is the job Vine's
  capability-passing IO speaks to; the brief's list does not name it separately.

**The matrix (judgement; the deciding fact in each cell).**

| Job | A Vine | B PEG / sea of nodes / RVSDG / MIR | C e-graphs (egg, egglog) | D Datalog / SQL | E compiler typed graph | F Parsing EG | G row 542 |
|---|---|---|---|---|---|---|---|
| (i) decision language | no: compiles to Ivy run by the IVM; no wasm or Rust output documented; decisions are Rust by decision | no: an IR, not a language | no | no | no | no | n/a (Rust stays) |
| (ii) fact representation | no | could: a per-function graph; no question in the documents needs one | no | no | partly: MIR has it, nightly only | no | yes: item table |
| (iii) rule engine | no | no | could: egglog as Datalog | yes | partly: typed lints | no | yes: Rust functions in the checker |
| (iv) duplicates | no | could, in principle | could: babble (library learning) | no | no | no | partly: names the calendar pair; a token pass finds all three (section 2) |
| (v) port equivalence | no | no | could: research only (HEC, for HLS) | no | no | no | no: vectors and witnesses do it |
| (vi) typed residue | no | partly: MIR, nightly | no | no | yes: SCIP occurrences, clippy `disallowed-methods` | no | no, by design |
| (vii) kernel form | no | no | could: two kinds, nothing to simplify | no | no | no | n/a |
| (viii) effects at the boundary | as an idea | no | no | no | partly | no | yes: gate 2 and the census rule |

## 2. Measurements

**Gates that read Rust (measured).** Method: `gates_rust.py` takes every `name|scripts/check-*.sh` entry of
`run-gates.sh`, reads the script and any `scripts/*.py` it names, and counts a gate as reading Rust when that text names
a `.rs` path or glob or a ripgrep Rust type. Result at `642f304`: 111 check-script entries, 71 with Python in a heredoc,
75 that read Rust source; 2 parse it through the `syn` member (`check-attr-spread-collision`, `check-public-api`; a third
hit, `check-gate-reuse`, only names the member among the roots it lists). The other 73 match text. A header read of
the first 28 of those in list order (`gates_header_read.py`, the class assigned by hand) finds 21 that decide a property
of Rust items, expressions, `rsx!` or attributes (for example `check-constant-signal`, `check-component-call`,
`check-render-clock`, `check-unawaited-focus`) and 7 that read `.rs` mainly for CSS classes or page-script text. A tighter
automatic classifier was tried and over-matched on documentation gates, so the Rust-shape count across all 73 is not
established; 21 of 28 is a sample.

**Duplicates by a token scan (measured).** Method: `clones.py`, the PMD-CPD method in stdlib Python. Over the 201
non-test `.rs` files under `primitives/src` (281,715 tokens), with comments blanked and code cut at the first
`#[cfg(test)]`, it hashes every window of 40 tokens and merges matching windows into maximal runs. Two modes: `exact`
keeps identifiers, `renamed` maps identifiers to one token and literals to another.

| Mode, minimum run | Pairs | Across two files | Lines covered | Wall time |
|---|---|---|---|---|
| exact, 120 tokens | 21 | 9 | 1,243 | 2.2 s |
| renamed, 120 tokens | 45 | 17 | 3,710 | 2.1 s |
| renamed, 60 tokens | 668 | 463 | 15,791 | 2.3 s |

All three duplicates the documents name are among the 120-token pairs: the calendar key handlers
(`calendar/range.rs:249-309` against `calendar/single.rs:191-246`, exact, two runs of 150 and 219 tokens), the menu-layer
block (`context_menu.rs:243-279` against `dropdown_menu.rs:272-312`, exact, 124 tokens) and Tooltip and HoverCard
(`hover_card.rs:36-143` against `tooltip.rs:36-142`, renamed, 151 tokens). At 60 tokens the renamed mode is mostly
`rsx!` and attribute boilerplate; none of the pairs was read for precision.

## 3. What the 2026-11-19 findings need

Judgement, from the findings in [the evaluation](./repo-evaluation-2026-11-19.md): the text-gate bypasses need parsed
facts (job iii); hot-path allocation hidden from unit tests and logic reachable only through Playwright need decisions
pulled out as pure functions (row 542 step 2); the missing wasm-`cfg` edge needs a reader that sees every `cfg` branch
(job ii); the menu-layer duplicate needs detection (job iv); the runtime type panics need a context generic over `T`
(row 536), which no analysis replaces; the CLI path traversal needs input validation (row 529). None of these needs data
or control dependence inside a function, a term rewriting engine, or a new language.

## 4. The candidates

**A. Vine.** Facts: "Vine is an experimental new programming language based on interaction nets"; "Vine compiles to Ivy",
and "vine build ... outputs an Ivy file" that the IVM interpreter runs; the crate versions are `0.0.0`; "there is a fixed
set of extrinsics built-in to IVM"; installation is `cargo install --git`. No WebAssembly output, foreign function
interface or Rust library use is documented (absence in the pages read; repository stars and releases came only from a
summarised GitHub page and are unknown here, so row 486's adoption rules D2-D4 cannot be applied). As the language of
decisions (job i) it fails on three facts, each of which decides alone: decisions are recorded as Rust functions, users
build installed sources with a Rust toolchain, and the kernel must stay dependency-free under a zero-headroom budget. As ideas (job
viii): "Any function that does not get passed a reference to an IO handle is pure", and types are linear by default
("its values must be used exactly once"). In Rust here the first idea holds only partly, because `spawn`,
`document::eval` and thread-locals are free functions that need no handle, so a signature alone does not prove
purity; the body scan of the census does. That makes the useful form a placement rule, not a type change: an outcome
function lives in a declared core module, and gate 2 then holds it to the census rule. Linearity has no stable
Rust lint and no finding behind it. Value here: low as a language, small but real as the placement rule.
Effort: none for the rule. Finding prevented: none directly. Leaves: everything typed.

**B. Program Expression Graphs, the sea of nodes, RVSDG, MIR.** Facts: "PEGs are a complete representation of a
function, making it unnecessary to keep the original control flow graph", built for equality saturation, whose authors
note "exponential blow-up"; the 2026-11-20 search found no maintained implementation. A sea of nodes "is a graph
representation of single-static assignment (SSA)"; V8, which used one, wrote on 25 March 2025 that it is "too complex.
Effect and control chains are hard to understand, leading to many subtle bugs", "too limited", that "Compiling is too
slow", and that "it has more downsides than upsides, at least as far as JavaScript and WebAssembly are concerned"; no
Rust implementation was found. RVSDG has a Rust crate, `rvsdg` 0.2.0. Rust's own function-internal graph, MIR, is reached
through `rustc_private`: Flowistry installs `nightly-2023-08-25` with `rustc-dev`, and `rustc_public` "does not exist" on
crates.io. Judgement: the questions in the documents are positional (a hook under `if`, a tracked read and a write in one
effect closure, a `match` inside an `rsx!` handler), and a `syn` visitor answers them from the tree. Building an SSA or
PEG form from `syn` output means re-implementing name resolution, macro expansion and types; reading MIR means a nightly
toolchain beside the pinned `1.97.0` and the host or wasm `cfg` of one compile. V8's reasons are an optimiser team's;
for a team that only queries facts they say the effect and control chains are the costly part, and no query here needs
them. Value: none found. Effort: L. Risk: a second truth beside the compiler's. Finding prevented: none.

**C. E-graphs and equality saturation.** Facts: egg is "a library for e-graphs and equality saturation" (0.11.0, 4,143,745
downloads); egglog "combines the power of equality saturation and Datalog" (3.0.0, 190,546 downloads, first published
2023-10-31); babble finds common structure in a corpus of programs "equivalent modulo the theory"; HEC applies equality
saturation to equivalence of code transformations in high-level synthesis. Per job: (iii) egglog as a rule engine is
below row 486's D3 stand-in (1,000,000 crates.io downloads where stars cannot be read; stars were not readable) and its
strength, congruence closure and extraction, is not what a gate rule needs. (iv) Library learning is documented on
benchmark corpora (judgement: research here); the token scan already finds the three named duplicates in 2.2 s, and the
hard part, choosing the shared abstraction (`HoverSurface`), is a design call row 528 made by hand. (v) Equivalence of a
legacy JavaScript behaviour and a Rust `decide` needs both in one term language with a JavaScript semantics: research,
while the shared vectors already test both routes. (vii) Two kinds, nothing to simplify. Value: none here now. Effort: L.

**D. Datalog and SQL over the table.** Facts: ascent (849,950 downloads) and crepe (859,597) are below the D3 stand-in;
datafrog's newest version, 2.0.1, is dated 2019-01-02, outside D4's 24 months; Soufflé compiles to parallel C++ under
UPL-1.0. Judgement: the class "each of 73 gates scrapes the source its own way" is removed by the one extraction, not by
the language its rules are written in. Gate rules here are mostly local (a call in a position); the recursive ones are
reachability (a hook reached through a helper, row 537) and purity through same-file callees, which a worklist of a
few dozen lines in the checker computes. Plain Rust functions over the table keep one binary, one dependency set and
types. Value of an engine: low until recursive rules multiply. Effort: M plus a register row for an engine. Leaves: all
typed facts.

**E. The compiler's typed graph.** Facts: Dylint lints are libraries with `rustc_private = true`; Charon emits "simplified
MIR bodies of functions" and is installed through its own rustup toolchain; Flowistry needs a 2023 nightly; the SCIP index
took 614 s cold and sees the host `cfg` only; clippy `disallowed-methods` already runs on stable in the root and kernel
`clippy.toml`. Judgement per residue item: which clone is deep and which call allocates come from SCIP occurrences with
their ranges (row 542 step 3, approved); a typed ban that a method name decides goes to clippy `disallowed-methods`, as
row 537 already names; purity through a trait call stays UNKNOWN, and row 542's kill condition (UNKNOWN above 10 %)
measures whether that matters; the runtime type panics need row 536's `Context<T>`, not a fact. Dylint or a MIR tool
would add a second, nightly toolchain to the gate run for no measured gap. Value: what is already approved. Effort: 0
beyond step 3.

**F. Parsing Expression Grammars.** Fact: they are a "recognition-based formal foundation for describing machine-oriented
syntax". Judgement: Rust is parsed by `syn`, `rsx!` by the exact `dioxus-rsx` the build uses ("the real grammar", the
checker's `Cargo.toml`), and comments by `scripts/comment-scan.py`, which the explorer already shares. No job needs another
grammar. Value: none.

**G. Row 542 as approved.** It does jobs ii, iii and viii and part of iv; its proof and kill condition already measure
the one open risk (UNKNOWN through type-free resolution). It does not detect duplicates, and nothing in it says where
an outcome function lives.

| | Value here | Practicality | Effort | Local minimum or second truth | Leaves |
|---|---|---|---|---|---|
| A Vine (language) | none | no Rust or wasm output documented; against the recorded Rust decision | L | a second language | all |
| A Vine (idea) | small | a placement rule, gate 2 checks it | 0 | none | linearity |
| B PEG, sea of nodes, MIR | none found | nightly or a re-built front end | L | second truth | nothing in the findings |
| C e-graphs | none now | egglog below D3; library learning is research | L | high | iv to a design call |
| D Datalog / SQL | low | engines below D3 or D4 | M | low | typed facts |
| E typed graph | as approved | SCIP 614 s, so not a gate; clippy on stable | 0 | none | trait calls UNKNOWN |
| F Parsing EG | none | not needed | - | - | - |
| G row 542 | high (2026-11-19) | stable, in an existing member | S-M | low | duplicates, typed residue |

## 5. The construction (judgement)

**Row 542 unchanged in its steps, its first lane and the owner's order (option A), with two additions.**

1. **Outcome functions live in declared core modules (with step 2, no new code).** Each pure outcome function step 2
   extracts goes into a module listed in gate 2's core TSV, so gate 2 holds it to the census rule: no reactive type, no
   runtime call. This is Vine's "no handle, no effect" made checkable with what row 542 already builds. *Trigger:* the
   first step-2 lane (the calendar handlers). *First step:* the calendar decision goes to a core module beside
   `horizontal_day_step` and gate 2 lists it. *Proof:* gate 2 turns red on a fixture that adds a `Signal` read to that
   module. *Kill:* if more than a quarter of the step-2 functions need an exception in the TSV, the rule is wrong for
   this code; keep the count only.
2. **A duplicate count read from the same run (after the table passes its proof).** The checker hashes token windows
   of the bodies it already parses (exact mode, 120 tokens, the method of section 2) and a gate holds the number of
   cross-file pairs shrink-only through `scripts/lib/ratchet.py`. *Trigger:* met by the measurement: 9 exact cross-file
   pairs today, among them the menu-layer block and the calendar handlers. *First step:* one lane, S. *Proof:* it
   reports the three named pairs; a hand read of 20 reported pairs finds at least 15 real copies; it adds under 1 s
   to the table's run. *Kill:* fewer than 15 of 20 real, or the count is blocked by `rsx!` boilerplate twice in a
   month; then it stays a report.

**Order:** row 542's first lane as planned; addition 1 rides on step 2; addition 2 follows the table's proof, beside
gates 1-4. **Residue:** typed questions stay with SCIP and clippy (step 3); purity through trait calls stays UNKNOWN; the
runtime type panics stay row 536's; equivalence of ports stays with the vectors and witnesses; choosing an abstraction
for a found duplicate stays a design call.

**Not adopted, with the deciding fact.** Vine as a language: decisions are Rust by the 2026-11-11 decision, and Vine
documents no Rust or wasm output. Program Expression Graphs and the sea of nodes: no question here needs function-internal
dependence, and the only Rust source of one, MIR, needs nightly. egglog and babble: egglog's 190,546 downloads are below
D3's stand-in, and the named duplicates fall to a 2-second token scan. A Datalog engine: ascent and crepe are below D3,
datafrog outside D4; revisit only if more than five gate rules need recursion over the table. Dylint, Charon, Flowistry:
nightly toolchains against the pinned stable one. Parsing Expression Grammars: no unparsed language in scope.

## 6. Direct answers (judgement)

- **Vine.** Not as a language here: it is experimental, compiles to an interaction-net file run by its own interpreter,
  and documents no Rust or WebAssembly output, while decisions are recorded as Rust functions built by the user's Rust
  toolchain. Its one transferable idea, purity visible as the absence of an effect handle, is taken as addition 1.
- **Program Expression Graphs.** They are an optimiser's representation for equality saturation, with no maintained
  implementation found, and no question this repository asks needs data or control dependence inside a function. Not
  adopted; the item table and SCIP cover the facts.
- **E-graphs.** Documented elsewhere for optimisation, rule inference, CAD and library learning; here the candidate jobs
  are met more cheaply (duplicates by a token scan, equivalence by shared vectors) and egglog falls below the
  dependency rule's adoption bar. Not adopted.
- **The sea of nodes.** V8 left it in 2025 as too complex, too limited and too slow for its compiler, and this team
  writes no optimiser. Not adopted.

**Effect on the plan.** Row 542's steps, proof, kill condition and the owner's order (option A) are unchanged; the two
additions attach to step 2 and to the gates after the table's proof. The waiting batch can start as it is.

## Sources (all fetched 2026-11-20)

- Vine: https://vine.dev/docs/, https://vine.dev/docs/ivy-ivm/, https://vine.dev/docs/ivy-ivm/extrinsics,
  https://vine.dev/docs/features/io, https://vine.dev/docs/features/traits/fork-drop, https://vine.dev/docs/tooling/cli,
  https://raw.githubusercontent.com/VineLang/vine/main/vine/Cargo.toml.
- Program Expression Graphs and equality saturation: https://www.cs.cornell.edu/~ross/publications/eqsat/.
- Sea of nodes: https://v8.dev/blog/leaving-the-sea-of-nodes, https://en.wikipedia.org/wiki/Sea_of_nodes.
- E-graphs: https://crates.io/api/v1/crates/egg, https://crates.io/api/v1/crates/egglog,
  https://raw.githubusercontent.com/egraphs-good/egglog/main/README.md, https://arxiv.org/abs/2212.04596 (babble),
  https://arxiv.org/abs/2506.02290 (HEC).
- Datalog: https://crates.io/api/v1/crates/ascent, https://crates.io/api/v1/crates/crepe,
  https://crates.io/api/v1/crates/datafrog.
- Typed graphs: https://raw.githubusercontent.com/trailofbits/dylint/master/README.md,
  https://raw.githubusercontent.com/AeneasVerif/charon/main/README.md,
  https://raw.githubusercontent.com/willcrichton/flowistry/master/README.md, https://crates.io/api/v1/crates/rustc_public,
  https://crates.io/api/v1/crates/rvsdg.
- Parsing Expression Grammars: https://bford.info/pub/lang/peg/.

## Not established

Vine's repository activity, stars and releases (only a summarised GitHub page was read); whether the IVM runs in a
browser; GitHub stars of egglog, ascent and crepe, so D3 is judged on its download stand-in; the precision of the clone
pairs (none read beyond the three named); the Rust-shape share of all 73 text gates (23 of 28 sampled); the run time of
the `syn` table and of a duplicate pass inside it (unbuilt).
