"""Write ledger-eval.jsonl for the note dev-docs/research/vine-egraphs-sea-of-nodes-on-row-542-2026-11-20.md."""
import json

T = "/tmp/claude-0/-home-user/2f9af3b0-3995-5f94-a96b-6c0dfde7973a/scratchpad/tech"
PV, PG, M = T + "/pages-vine/", T + "/pages-graph/", T + "/measure/"
OUT = T + "/ledger-eval.jsonl"
rows = []


def q(file, line, text):
    return {"kind": "quote", "file": file, "line": line, "text": text}


def run(log):
    return {"kind": "run", "log": M + log}


def fact(i, claim, actual, *receipts):
    rows.append({"id": f"E-{i}", "claim": claim, "verdict": "CONFIRMED", "actual": actual, "receipts": list(receipts)})


def judge(i, claim):
    rows.append({"id": f"E-{i}", "kind": "judgement", "claim": claim, "verdict": "CONFIRMED", "actual": claim,
                 "receipts": [{"kind": "judgement"}]})


def unk(i, claim, actual):
    rows.append({"id": f"E-{i}", "claim": claim, "verdict": "UNVERIFIABLE", "actual": actual, "receipts": []})


B542 = "dev-docs/backlog/542-one-generated-item-table-read-by-the-gates.md"
# Repository facts
fact(1, "Row 542's table is generated each run and never committed.", "row 542 step 1",
     q(B542, 19, "generated on each run and never committed, so it cannot drift. Gates read it."))
fact(2, "Row 542 step 2 names the calendar handlers and the one tested helper.", "row 542 step 2",
     q(B542, 24, "and `calendar/single.rs`) and only `horizontal_day_step` (`calendar/dates.rs:235`) has a unit test."))
fact(3, "Row 542's proof requires the sleep.rs wasm-cfg call and a warm run under 10 s.", "row 542 proof",
     q(B542, 57, "it lists `sleep.rs`'s `gloo_timers` call (behind `cfg(target_family = \"wasm\")`, absent from the SCIP"),
     q(B542, 58, "it runs in under 10 s warm on the workspace; UNKNOWN stays under 2 % of functions."))
fact(4, "Row 542's kill condition is UNKNOWN above 10 % or above 60 s.", "row 542 kill",
     q(B542, 59, "if it needs type inference to classify (UNKNOWN above 10 %), or runs above 60 s, stop at row 533's count."))
fact(5, "Row 542's residue: no type information; trait or closure calls can stay UNKNOWN.", "row 542 residue",
     q(B542, 71, "There is no type information: purity is a token policy"),
     q(B542, 72, "A call through a trait or a closure parameter can stay UNKNOWN."))
fact(6, "Decisions are recorded as one pure Rust function on every renderer (2026-11-11).", "decisions.md",
     q("dev-docs/decisions.md", 36, "every behaviour's decision is one pure Rust function over typed inputs, the same"))
fact(7, "The decision kernel has no dependencies by design.", "decision-kernel/Cargo.toml",
     q("decision-kernel/Cargo.toml", 10, "No dependencies, by design"))
fact(8, "The kernel gzip budget row is 12914 bytes.", "budgets tsv",
     q("scripts/check-decision-kernel.budgets.tsv", 4, "primitives/src/kernel/decision-kernel.wasm\t12914"))
fact(9, "The 2026-11-19 evaluation lists the kernel gzip as 12,914 of 12,914.", "evaluation note",
     q("dev-docs/research/repo-evaluation-2026-11-19.md", 176, "the kernel gzip (12,914 of 12,914)"))
fact(10, "The toolchain is pinned to 1.97.0.", "rust-toolchain.toml",
     q("rust-toolchain.toml", 5, "channel = \"1.97.0\""))
fact(11, "kinds! registers two kinds, fixture and otp-filter.", "kinds.rs",
     q("decision-kernel/src/kinds.rs", 53, "kinds! {\n    \"fixture\" => fixture,\n    \"otp-filter\" => otp_filter,"))
fact(12, "The shared vectors run against the Rust decide directly and through the byte route.", "vectors.rs",
     q("decision-kernel/tests/vectors.rs", 1, "//! The shared vectors (`primitives/conformance/behaviours.json`) against the Rust `decide`, once called"))
fact(13, "Two Rust-shape gates were bypassed in reproductions; the third bypass is the prose gate.", "row 537",
     q("dev-docs/backlog/537-rust-shape-gates-are-text-patterns-and-have-bypasses.md", 17,
       "`scripts/check-self-subscribing-effects.sh` exits 0 on"),
     q("dev-docs/backlog/537-rust-shape-gates-are-text-patterns-and-have-bypasses.md", 19,
       "`scripts/check-hooks-in-closures.sh` exits 0 for a hook in a helper function the closure calls"),
     q("dev-docs/backlog/537-rust-shape-gates-are-text-patterns-and-have-bypasses.md", 22,
       "`scripts/check-history-comments.sh` is a phrase list"))
fact(14, "The menu-layer block is duplicated in dropdown_menu.rs and context_menu.rs (D-14).", "evaluation note",
     q("dev-docs/research/repo-evaluation-2026-11-19.md", 298,
       "The menu-layer block is duplicated at `primitives/src/dropdown_menu.rs:278-301` and `context_menu.rs:247-270` (D-14)."))
fact(15, "Row 528 makes Tooltip and HoverCard one surface (HoverSurface).", "STATUS.md",
     q("dev-docs/STATUS.md", 74, "(`HoverSurface`: Tooltip and HoverCard as one surface"))
fact(16, "check-test-harness-controlled exists because copies of a harness and a controlled memo had grown.", "gate header",
     q("scripts/check-test-harness-controlled.sh", 5, "A concern implemented once in `primitives/src` is used, not copied"),
     q("scripts/check-test-harness-controlled.sh", 5, "Eight private `with_runtime` copies"))
fact(17, "Row 536's construction is a context generic over T.", "row 536",
     q("dev-docs/backlog/536-generic-components-check-value-types-at-run-time.md", 22,
       "Construction to evaluate: a context generic over `T`"))
fact(18, "Row 486 decision: rust-analyzer is the one analysis source; young tools are out.", "row 486",
     q("dev-docs/backlog/486-code-graph-explorer-tools-codegraph.md", 83,
       "rust-analyzer is the one analysis source. crabmap, Crabviz and every other young or little-used tool are out of the"))
fact(19, "Row 486's D2-D4: first release 12 months old; 1,000 stars or 1,000,000 crates.io downloads; a release within 24 months.", "row 486",
     q("dev-docs/backlog/486-code-graph-explorer-tools-codegraph.md", 137, "The project's first release is at least 12 months old"),
     q("dev-docs/backlog/486-code-graph-explorer-tools-codegraph.md", 138,
       "where stars cannot be read, at least 100,000 weekly npm downloads or 1,000,000 total crates.io downloads as the stand-in"),
     q("dev-docs/backlog/486-code-graph-explorer-tools-codegraph.md", 139, "Active maintenance: a release in the last 24 months"))
fact(20, "The SCIP index took 614 s cold, host cfg only.", "functional-core note",
     q("dev-docs/research/functional-core-and-graph-ssot-2026-11-19.md", 134, "SCIP: type-resolved but 614 s cold (`index.info`), file nodes, host cfg only"))
fact(21, "The checker member parses rsx with the exact dioxus-rsx of the lock and uses syn full and visit.", "Cargo.toml",
     q("scripts/attr-spread-collision-checker/Cargo.toml", 14, "the real grammar"),
     q("scripts/attr-spread-collision-checker/Cargo.toml", 19, "syn = { version = \"2.0\", features = [\"full\", \"visit\"] }"))
fact(22, "The explorer's project config uses the gates' comment tokenizer scripts/comment-scan.py.", ".codegraph/project.py",
     q(".codegraph/project.py", 8, "TOKENIZER = \"scripts/comment-scan.py\""))
fact(23, "clippy disallowed-methods runs on stable in the root and kernel clippy.toml.", "clippy.toml files",
     q("clippy.toml", 6, "disallowed-methods = ["), q("decision-kernel/clippy.toml", 7, "disallowed-methods = ["))
fact(24, "The census counts spawn and document:: calls as effectful.", "functional-core note",
     q("dev-docs/research/functional-core-and-graph-ssot-2026-11-19.md", 60,
       "a runtime call such as `spawn` or `consume_context`, `document::`"))
fact(25, "Workflows are workflow_dispatch-only (CI freeze).", "run-gates.sh",
     q("scripts/run-gates.sh", 4, "is `workflow_dispatch`-only (CI freeze, dev-docs/backlog.md row 33), so nothing else runs these."))
# Measurements
fact(26, "run-gates.sh lists 111 check scripts; 71 embed Python in a heredoc; 75 read Rust source; 2 parse it through the syn member; 73 match text.",
     "gates_rust.py at 642f304 (check-gate-reuse is a false syn hit: it only lists the member)", run("gates_rust.log"))
fact(27, "Of the first 28 text-pattern gates, 21 decide a property of Rust code and 7 read .rs for CSS classes or page-script text (hand class).",
     "gates_header_read.py", run("gates_header_read.log"))
fact(28, "Exact token clones, 120 tokens, primitives/src: 21 pairs, 9 across files, 1,243 lines, 2.2 s.", "clones.py exact 120",
     run("clones-exact-120.log"))
fact(29, "Renamed token clones, 120 tokens: 45 pairs, 17 across files, 3,710 lines, 2.1 s.", "clones.py renamed 120",
     run("clones-renamed-120.log"))
fact(30, "Renamed token clones, 60 tokens: 668 pairs, 463 across files, 15,791 lines, 2.3 s.", "clones.py renamed 60",
     run("clones-renamed-60.log"))
fact(31, "The token scan reports the calendar handler pair, the menu-layer block and Tooltip/HoverCard among 120-token pairs.",
     "range.rs:249-267~single.rs:191-209 (150), range.rs:272-309~single.rs:209-246 (219), context_menu.rs:243-279~dropdown_menu.rs:272-312 (124) in exact; hover_card.rs:36-143~tooltip.rs:36-142 (151) in renamed",
     run("clones-exact-120.log"), run("clones-renamed-120.log"))
# External facts
fact(32, "Vine is experimental and based on interaction nets.", "vine.dev/docs",
     q(PV + "docs-index.txt", 5, "Vine is an experimental new programming language based on interaction nets"))
fact(33, "Vine compiles to Ivy; vine build outputs an Ivy file.", "vine.dev docs",
     q(PV + "d_ivy-ivm_.txt", 5, "Vine compiles to Ivy, a low-level interaction-combinator programming language."),
     q(PV + "d_tooling_cli.txt", 6, "You can build a Vine program with vine build, which outputs an Ivy file."))
fact(34, "Vine's crates carry version 0.0.0.", "vine/Cargo.toml", q(PV + "raw_vine_Cargo.toml.txt", 4, "version = \"0.0.0\""))
fact(35, "The IVM has a fixed set of extrinsics.", "extrinsics page",
     q(PV + "d_ivy-ivm_extrinsics.txt", 9, "Currently, there is a fixed set of extrinsics built-in to IVM."))
fact(36, "Vine installs by cargo install --git.", "installation page",
     q(PV + "d_starting_installation.txt", 6, "cargo install --git https://github.com/VineLang/vine --locked"))
fact(37, "In Vine a function not passed an IO handle is pure.", "io page",
     q(PV + "d_features_io.txt", 20, "Any function that does not get passed a reference to an IO handle is pure"))
fact(38, "Vine types are linear by default.", "fork-drop page",
     q(PV + "d_features_traits_fork-drop.txt", 17, "its values must be used exactly once."))
unk(39, "Vine documents WebAssembly output, an FFI or library use from Rust.",
    "Absent from the pages read (ledger-vine V-49 UNVERIFIABLE); absence is not proof.")
unk(40, "Vine's GitHub stars and releases.", "Only a summarised GitHub page was read (ledger-vine V-44, V-45 UNVERIFIABLE).")
fact(41, "Program Expression Graphs are a complete, referentially transparent representation of a function.", "Tate et al. page",
     q(PG + "peg-eqsat.html", 18, "PEGs are a complete representation of a function, making it unnecessary to keep the original control flow graph."))
fact(42, "The PEG authors note exponential blow-up of their equality saturation.", "Tate et al. page",
     q(PG + "peg-eqsat.html", 20, "Although our equality saturation has exponential blow-up"))
unk(43, "No maintained Program Expression Graph implementation exists.", "Not found by the 2026-11-20 search (graph-facts.md section 3); absence only.")
fact(44, "Sea of nodes is an SSA graph representation.", "Wikipedia",
     q(PG + "wiki-son.html", 14, "A sea of nodes is a graph representation of single-static assignment (SSA)"))
fact(45, "V8 left the sea of nodes; reasons: too complex, too limited, too slow; more downsides than upsides.", "v8.dev 2025-03-25",
     q(PG + "v8-son.html", 5, "Published 25 March 2025"),
     q(PG + "v8-son.html", 153, "It’s too complex. Effect and control chains are hard to understand, leading to many subtle bugs."),
     q(PG + "v8-son.html", 154, "It’s too limited."),
     q(PG + "v8-son.html", 155, "Compiling is too slow."),
     q(PG + "v8-son.html", 40, "it has more downsides than upsides, at least as far as JavaScript and WebAssembly are concerned"))
fact(46, "An rvsdg crate 0.2.0 exists on crates.io.", "crates.io", q(PG + "rvsdg-crate.html", 2, "\"max_version\":\"0.2.0\""))
fact(47, "Flowistry installs nightly-2023-08-25 with rustc-dev.", "Flowistry README",
     q(PG + "flowistry-readme.md", 188, "rustup toolchain install nightly-2023-08-25 -c rust-src -c rustc-dev -c llvm-tools-preview"))
fact(48, "rustc_public does not exist on crates.io.", "crates.io API",
     q(PG + "stablemir-crate.json", 2, "crate `rustc_public` does not exist"))
fact(49, "egg: 0.11.0, 4,143,745 downloads.", "crates.io",
     q(PG + "egg-crate.json", 2, "\"max_version\":\"0.11.0\""), q(PG + "egg-crate.json", 2, "\"downloads\":4143745"))
fact(50, "egglog combines equality saturation and Datalog; 3.0.0; 190,546 downloads; first published 2023-10-31.", "README, crates.io",
     q(PG + "egglog-readme.md", 15, "which combines the power of equality saturation and Datalog."),
     q(PG + "egglog-crate.json", 2, "\"max_version\":\"3.0.0\""), q(PG + "egglog-crate.json", 2, "\"downloads\":190546"),
     q(PG + "egglog-crate.json", 2, "\"created_at\":\"2023-10-31T18:42:44.148502Z\""))
fact(51, "babble learns libraries with e-graphs modulo a theory.", "arXiv 2212.04596",
     q(PG + "babble.html", 17, "uses e-graphs and equality saturation to compactly represent the space of programs equivalent modulo the theory"))
fact(52, "HEC checks equivalence of code transformations by equality saturation (HLS).", "arXiv 2506.02290",
     q(PG + "hec.html", 3, "HEC: Equivalence Verification Checking for Code Transformation via Equality Saturation"))
fact(53, "ascent 849,950 and crepe 859,597 crates.io downloads.", "crates.io",
     q(PG + "ascent-crate.json", 2, "\"downloads\":849950"), q(PG + "crepe-crate.json", 2, "\"downloads\":859597"))
fact(54, "datafrog's newest version 2.0.1 dates from 2019-01-02.", "crates.io",
     q(PG + "datafrog-crate.json", 2, "\"max_version\":\"2.0.1\""),
     q(PG + "datafrog-crate.json", 2, "\"created_at\":\"2019-01-02T19:42:28.496801Z\""))
fact(55, "Souffle translates Datalog to parallel C++ under UPL-1.0.", "Souffle README",
     q(PG + "souffle-readme.md", 15, "Efficient translation to parallel C++ of Datalog programs"),
     q(PG + "souffle-readme.md", 8, "License-UPL--1.0"))
fact(56, "Dylint lint libraries set rustc_private = true.", "Dylint README",
     q(PG + "dylint-readme.md", 10, "Dylint is a Rust linting tool, similar to Clippy."),
     q(PG + "dylint-readme.md", 201, "rustc_private = true"))
fact(57, "Charon emits simplified MIR bodies and installs its toolchain through rustup.", "Charon README",
     q(PG + "charon-readme.md", 29, "simplified MIR bodies of functions, source information for each item"),
     q(PG + "charon-readme.md", 72, "rustup will automatically download and install the proper packages"))
fact(58, "Parsing Expression Grammars are a recognition-based foundation for machine-oriented syntax.", "Ford page",
     q(PG + "ford-peg.html", 27, "Parsing Expression Grammars (PEGs) provide"))
unk(59, "GitHub stars of egglog, ascent and crepe.", "Not readable on 2026-11-20 (GitHub API refused); D3 judged on its download stand-in.")
# Judgements
for i, c in enumerate([
    "Jobs (i)-(viii) as listed are the jobs the documents support; the matrix cells are judgements on the facts above.",
    "None of the 2026-11-19 findings needs function-internal data or control dependence, a rewriting engine or a new language.",
    "Vine fails as the decision language on three independent facts: decisions recorded in Rust, users build sources with Rust, the kernel is dependency-free at zero headroom.",
    "Vine's handle idea holds only partly in Rust because spawn, document::eval and thread-locals need no handle; the useful form is a placement rule checked by gate 2.",
    "The documented questions are positional and a syn visitor answers them; an SSA or PEG built from syn output would re-implement a front end, and MIR needs nightly.",
    "V8's reasons apply to optimiser teams; no query here needs effect or control chains.",
    "egglog's strength (congruence closure, extraction) is not what a gate rule needs; library learning is research here; port equivalence via e-graphs needs a JavaScript semantics.",
    "The class 'each gate scrapes source its own way' is removed by one extraction, not by the rule language; plain Rust over the table suffices until recursive rules multiply.",
    "Dylint or a MIR tool would add a nightly toolchain to the gate run for no measured gap; typed residue stays with SCIP (step 3) and clippy disallowed-methods.",
    "No job needs a grammar beyond syn, dioxus-rsx and comment-scan.py.",
    "Value, practicality, effort (S/M/L) and risk ratings in the candidate table.",
    "Construction: row 542 unchanged plus (1) outcome functions in declared core modules and (2) a shrink-only cross-file duplicate count after the table's proof, with the stated triggers, first steps, proofs and kills.",
    "Revisit a Datalog engine only if more than five gate rules need recursion over the table.",
    "Direct answers: Vine, Program Expression Graphs, e-graphs and the sea of nodes are not adopted; row 542's order is unchanged and the waiting batch can start as it is.",
], start=60):
    judge(i, c)

with open(OUT, "w", encoding="utf-8") as fh:
    for r in rows:
        fh.write(json.dumps(r, ensure_ascii=False) + "\n")
print(len(rows), "lines ->", OUT)
