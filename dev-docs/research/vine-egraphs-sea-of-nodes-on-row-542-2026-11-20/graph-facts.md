# Graph representations and engines: facts (fetched 2026-11-20)

Saved pages: /tmp/claude-0/-home-user/2f9af3b0-3995-5f94-a96b-6c0dfde7973a/scratchpad/tech/pages-graph/ (bracket = file name there). Ledger ids G-n in ledger-graph.jsonl. Dates for releases come from the crates.io API record (`*-crate.json`). The GitHub API was not reachable from this session (access refused), so "last commit" dates are NOT established; release dates are used instead. Anything marked UNVERIFIED is from memory.

## 1. E-graphs and equality saturation
- An e-graph "efficiently represents a congruence relation over many expressions"; equality saturation repurposes it for rewrite-driven optimisation; egg adds "rebuilding" (amortised invariant restoration) [egg-paper.html, arXiv 2004.03082, POPL 2021; G-?]. Structure: union-find over e-class ids, e-classes holding sets of e-nodes, a hashcons, and a congruence invariant [wiki-egraph.html].
- Plain terms (UNVERIFIED paraphrase): keep all rewritten forms of an expression side by side in one graph, apply rules until nothing new appears or a budget is hit, then pick the cheapest form with a cost function.
- Benefit stated by Tate et al.: saturation "obviates the need to worry about optimization ordering" and enables "a global optimization heuristic that selects among fully optimized programs" [peg-eqsat.html]. Cost stated by them: "Although our equality saturation has exponential blow-up" [peg-eqsat.html]. Extraction cost, need for a term language and rules: UNVERIFIED (general knowledge); the egg paper notes e-graphs "remain unspecialized" and need "ad-hoc e-graph extensions" for non-syntactic transformations [egg-paper.html].
- egg: Rust library "for e-graphs and equality saturation"; MIT; latest 0.11.0, 2025-12-04 (previous 0.10.0 2024-12-31); 4.1M downloads; README points to egglog as an alternative [egg-readme.md, egg-crate.json]. Self-described maturity: none stated beyond README. Release cadence about yearly.
- egglog: "combines the power of equality saturation and Datalog"; MIT; latest 3.0.0, 2026-08-19 (2.0.0 2026-02-12, 1.0.0 2025-10-19); README recommends egglog-experimental for extra plugins; has a `-j` thread option [egglog-readme.md, egglog-crate.json, egglog-paper.html PLDI 2023].
- egraph-serialize: "A library to serialize e-graphs"; MIT; 0.3.0, 2025-10-19 [egraph-serialize-crate.json].
- Cranelift aegraphs: RFC says restricting union "to just node creation time allows for a much simpler data structure"; rules applied once at node creation, like Cascades (Graefe 1995); full recanonicalisation dropped because parent pointers are expensive [rfc-aegraph.md]. Mid-end status today (default on, etc.): UNVERIFIED, RFC only.
- Uses outside optimising compilers (what the e-graph holds):
  - Szalinski (PLDI 2020): CAD models; equivalent CSG expressions, to recover loops/functions from meshes [szalinski.html].
  - Ruler (OOPSLA 2021): enumerated terms and candidate rewrite rules, to infer rules [ruler.html].
  - babble (POPL 2023): programs equivalent modulo a theory across a corpus; anti-unification finds shared abstractions (library learning) [babble.html].
  - Herbie: listed under floating-point optimisation (PLDI 2015) on the community list; that it uses egg is UNVERIFIED here [awesome-egraphs.md].
  - HEC (arXiv 2506.02290): equivalence checking of source-to-source code transformations (HLS) [hec.html].
  - ftui-layout 0.10.0 (Rust, third-party, 2026-11-03): "E-graph encoding for layout constraint optimization" in a terminal UI [ftui-egraph.html, ftui-crate.json]; maturity unknown.
  - Refactoring, clone detection, lint tools, CSS tooling: nothing found in two searches. Babble is the nearest to cross-program pattern mining (does not claim clone detection).

## 2. egglog and Datalog engines
- Datalog+EqSat: egglog abstract: "Like Datalog, it supports efficient incremental execution, cooperating analyses, and lattice-based reasoning. Like EqSat, it supports term rewriting, efficient congruence closure, and extraction" [egglog-paper.html].
- Soufflé: "similar to Datalog", used "for analysis problems"; translates to parallel C++; licence UPL-1.0 [souffle-readme.md]. Fact loading (files) and performance scale: UNVERIFIED.
- CodeQL: "a language and toolchain for code analysis" [codeql-about.html]; Rust support status and licence: UNVERIFIED.
- Glean: "a system for working with facts about source code" (Meta) [glean-site.html]; scale numbers UNVERIFIED.
- ascent: Datalog-like logic language embedded via macros; MIT; 0.8.1, 2026-08-29 (0.8.0 2025-03-02) [ascent-readme.md, ascent-crate.json].
- crepe: Datalog-like procedural macro; MIT/Apache-2.0; 0.2.0, 2025-12-14 (0.1.8 2023-03-19) [crepe-readme.md, crepe-crate.json].
- datafrog: "lightweight Datalog engine intended to be embedded", "no runtime"; Apache-2.0/MIT; newest version 2.0.1 dated 2019-01-02 on crates.io [datafrog-readme.md, datafrog-crate.json]. That Polonius uses it: UNVERIFIED (not on the saved page).
- Documented lint-style use over a codebase: treeedb 0.1.0 generates Soufflé facts from tree-sitter parse trees [treeedb.html]; CodeQL is by design a query system for code. No documented Rust-lint use of ascent/crepe/datafrog found.
- SQL route: DuckDB community extension sitting_duck exposes tree-sitter ASTs of 28 languages as SQL tables [sitting-duck.html]; Datadog static analysis uses tree-sitter query plus JavaScript rule functions, not SQL [datadog-sa.html]. No documented project doing Rust lint rules in SQLite/DuckDB found.

## 3. Program Expression Graphs; Parsing Expression Grammars
- Names: Tate et al. (POPL 2009) = "Program Expression Graphs"; Ford (POPL 2004) = "Parsing Expression Grammars" [peg-eqsat.html, ford-peg.html]. Both abbreviate to PEG.
- PEG (Tate): "a complete representation of a function, making it unnecessary to keep the original control flow graph"; "referentially transparent"; "no intermediate variables"; E-PEG = PEG plus equality edges [peg-eqsat.html]. Theta/eval/pass node names: UNVERIFIED (not on saved page). Shortcoming named by the authors: loop unrolling/reindexing [peg-eqsat.html]. Journal version LMCS 2011 [peg-eqsat.html].
- Maintained implementation today: none found; the Cornell page is a research page (original implementation was in the Peggy optimiser, UNVERIFIED). Rust `rvsdg` crate 0.2.0 (2026-04-24) exists but is unrelated to Tate et al. [rvsdg-crate.html].
- Relation to RVSDG: RVSDG is "a data flow centric IR where nodes represent computations, edges represent computational dependencies, and regions capture the hierarchical structure" (arXiv 1912.05036, 2019) [rvsdg.html]. Relation to e-graph mid-ends: arXiv 2602.16707 (Feb 2026, rev. Aug 2026) "E-Graphs as a Persistent Compiler Abstraction" [arx-2602.html].
- Parsing Expression Grammars: "recognition-based formal foundation for describing machine-oriented syntax" [ford-peg.html]. Rust crates: pest 2.9.6 (2026-11-10), peg 0.8.6 (2026-05-04), chumsky 0.14.2 (2026-11-17; crates.io max_version string shows 1.0.0-alpha.8) [crate jsons]. chumsky is a parser-combinator library "for humans with powerful error recovery"; PEG-likeness UNVERIFIED. Use: parsing text into syntax trees.

## 4. Sea of nodes
- Definition: "a graph representation of single-static assignment (SSA) ... that combines data flow and control flow, and relaxes the control flow from a total order to a partial order"; used in HotSpot, LibFirm, GraalVM; formerly V8 TurboFan [wiki-son.html]. Click and Paleczny 1995 paper: no usable saved copy (Oracle URL returned an error page; third-party PDF not text-extractable); UNVERIFIED beyond Wikipedia.
- V8 blog "Land ahoy: leaving the Sea of Nodes", published 25 March 2025: Turbofan "is famously one of the few large-scale production compilers to use Sea of Nodes"; since almost 3 years the team moved to a CFG IR "which we named Turboshaft". Reasons quoted: "It's too complex. Effect and control chains are hard to understand, leading to many subtle bugs."; "It's too limited. Too many nodes are on the effect and control chain"; "Compiling is too slow. State-tracking is expensive"; "it has more downsides than upsides, at least as far as JavaScript and WebAssembly are concerned"; ~3x more L1 dcache misses than the CFG IR (up to 7x in some phases), est. up to 5% of compile time [v8-son.html]. Status: JS backend of Turbofan and all of WebAssembly use Turboshaft; two parts of Turbofan still use SoN [v8-son.html line 6].
- Teaching implementation: SeaOfNodes/Simple (Java; chapter-based; x86-64, RISC-V, ARM64 backends) [simple-son.md]; last activity UNVERIFIED. Maintained Rust implementation: none found; crates.io has an `rvsdg` crate (2026-04-24, 27 downloads at fetch), not sea of nodes.
- RVSDG: see 3. MLIR: "a compiler intermediate representation with similarities to traditional" SSA three-address IRs [mlir.html]. Cranelift IR: Cranelift mid-end uses aegraph (section 1); its IR is CFG with block parameters (UNVERIFIED, page fetched did not confirm).

## 5. Graphs of a Rust program exposed to tools
Cells "macros"/"wasm-only" are UNVERIFIED unless a source is cited; "wasm-only" = code under `cfg(target_arch="wasm32")` when run on a host: for cfg-driven tools it is seen only if the cfg is enabled for the run (UNVERIFIED general).
- rustc MIR: statement/data-flow/CFG per body; rustc-internal, nightly (`rustc_private`). Sees expanded macros (MIR is post-expansion); wasm-only code only when compiled with that target (UNVERIFIED).
- rustc_public (ex Stable MIR): "SemVer compliant API" over MIR; docs on nightly; crate "planned" for crates.io, lookup returned "does not exist" 2026-11-20 [rustc-public.html, rustc-public-start.html, stablemir-crate.json].
- rust-analyzer SCIP/LSIF, `ra_ap_*`: ra_ap_hir 0.0.363 (2026-11-16, MIT/Apache-2.0) [ra-hir-crate.json]; SCIP/LSIF output and granularity: UNVERIFIED (saved pages did not cover). Stable toolchain use of ra_ap crates: UNVERIFIED.
- rustdoc JSON: `rustdoc-types` 0.61.0 (2026-07-29) mirrors the format [rustdoc-types-crate.json]; item-level (signatures, no bodies); generating it needs nightly `-Z unstable-options` (UNVERIFIED).
- syn: syntax only, stable, macro invocations kept as unexpanded token streams (UNVERIFIED); crate record saved [syn-crate.json]. tree-sitter-rust: syntax only; crate updated 2026-03-27 [treesitter-rust.json]; does not expand macros (UNVERIFIED).
- ast-grep 0.51.3 (2026-11-16, MIT): "code structural search, lint, rewriting" by AST patterns [astgrep.html]. Semgrep: Rust "Generally available" [semgrep-rust2.html]. Both syntax-level; see tree-sitter caveats.
- dylint 6.4.0 (2026-11-10): "Rust linting tool, similar to Clippy" running lints from dynamic libraries [dylint-readme.md]; typed, post-expansion HIR/MIR; needs a pinned nightly (UNVERIFIED on page). clippy_utils: same nightly dependence (UNVERIFIED).
- Charon: extracts item types, functions, "simplified MIR bodies" to `.llbc` JSON; OCaml bindings; used by Aeneas [charon-readme.md]; it installs via rustup with its own pinned toolchain [charon-readme.md lines 71-75].
- Flowistry 0.5.44 (2025-09-23, MIT): information flow / slicing in Rust [flowistry-readme.md, flowistry-crate.json]. Paralegal (OSDI 2025): privacy/security static analyser; builds against nightly "pinned in rust-toolchain.toml" [paralegal-readme.md].
- cargo-call-stack 0.1.16 (2024-10-28): needs nightly; "Last tested nightly: 2023-11-13" [callstack-readme.md]. cargo-modules 0.28.0 (2026-10-18, MPL-2.0): module tree, `--cfg-test` option [modules-readme.md]. cargo-geiger 0.14.0 (2026-10-12): counts unsafe usage; its library is "Unversioned and highly unstable" [geiger-readme.md].

## 6. Purity / effects for Rust
- Keyword Generics Initiative: group under the lang team to solve function colouring "through the type system" for async, const and other modifier keywords (progress report 2023-02-23, `?async` style syntax) [kwgen-blog.html]. Current (2026) status: UNVERIFIED (initiative page 404; project-goals page not retrievable).
- no-panic crate exists (crates.io record updated 2026-07-18) [noalloc.html]; a purity classifier for Rust functions, `#[no_alloc]`-style crates, clippy purity lints: nothing established. `const fn` as a purity proxy: UNVERIFIED.

## 7. Unknowns
GitHub activity dates (API refused); Click and Paleczny paper text; PEG node kinds (theta/eval/pass); any maintained PEG implementation; Cranelift aegraph default status; rust-analyzer SCIP/LSIF details; rustdoc JSON stability; per-tool macro and wasm32 behaviour; Soufflé/CodeQL/Glean scale and Rust support; Polonius's use of datafrog; effects/keyword-generics state in 2026; Rust purity/allocation tools; documented SQL-based Rust code analysis; documented e-graph refactoring/clone/lint/CSS tools.
