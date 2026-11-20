# Vine programming language: facts (fetched 2026-11-20)

Saved pages: /tmp/claude-0/-home-user/2f9af3b0-3995-5f94-a96b-6c0dfde7973a/scratchpad/tech/pages-vine/ (file names in brackets; "d_*" = vine.dev/docs pages). Ledger: ledger-vine.jsonl (V-n ids).
Access note: the GitHub API and the GitHub MCP tools were refused for this session. GitHub repo, tags, commits, org, PR pages were read through WebFetch, which returns a model summary; those summaries were NOT saved as pages and are marked (WebFetch summary). raw.githubusercontent.com files were read directly and saved.

## 0. Premise check
- The referent is confirmed: "The Vine Programming Language", vine.dev, org VineLang, repo github.com/VineLang/vine; Vine compiles to Ivy, which runs on the IVM [d_ivy-ivm_.txt, V-2]. Quote: "Vine is an experimental new programming language based on interaction nets" [docs-index.txt, V-1].
- Other things called Vine, ruled out (all from search results, one-line fetches; secondary):
  - crates.io crate `vine`: "Vine is rust framework inspired by Spring Boot" (updated 2025-10-13) [crates-vine.json, V-5].
  - github.com/stevenhuyn/vine: a fork of VineLang/vine, 0 stars (WebFetch summary).
  - github.com/Trombecher/vine: Rust-implemented language "focus on simplicity and type safety", MIT, 3 stars, early stage, planned backends include JavaScript/WebAssembly (WebFetch summary).
  - xiaoxustudio "vine-lang": interpreted language on npm (`vine-lang`), TypeScript (search result snippet only; UNVERIFIED beyond that).
  - esolangs.org "Vines" and Vine (Vue-style/other) hits: search titles only, not opened.

## 1. Identity and status
- Author: the HN thread shows "tjjfvi" saying "Author here" on 2025-02-23 [hn-43144040.txt, V-37]; he also says he was lead developer of HVM-64 [V-35]. Org page lists one public member, @tjjfvi (WebFetch summary). Other commit authors seen on main: enricozb, nilscrm (WebFetch summary of commits page).
- Organisation: GitHub org "The Vine Programming Language (VineLang)", domain-verified for vine.dev, 17 followers (WebFetch summary).
- Licence: "Apache-2.0, MIT licenses found" (WebFetch summary); LICENSE-MIT exists in repo root [raw_LICENSE-MIT.txt, V-3]. A plain LICENSE file returns 404.
- First public date: posted on Hacker News 2025-02-22 [hn-43144040.txt, V-38]; InfoQ article 2025-03-11 [infoq.txt, V-39]. Earlier history UNVERIFIED (repo creation date not read).
- Maturity: "experimental" [V-1]. No stability policy or breaking-change statement found in README or docs (the docs pages read contain none).
- Releases: GitHub tags page: "There aren’t any releases here" (WebFetch summary, V-44); crate versions in the manifests are `0.0.0` [raw_vine_Cargo.toml.txt, V-4]. No versioned releases, UNVERIFIED beyond those two sources.
- Stars/forks/issues: 484 stars, 30 forks, 9 watchers, 70 open issues, 8 open PRs, 379 commits (WebFetch summary, V-45). Search-result snippet earlier said about 386 stars (older snapshot).
- Activity: newest commits on main seen: 2026-08-24 (tjjfvi), 2026-08-02, 2026-07-14 x2, 2026-07-04 (enricozb); org page says repo last updated 2026-11-04 (WebFetch summary, V-46). Exact 12-month commit count and contributor count: UNVERIFIED (contributors graph did not load).
- Community: Discord linked from vine.dev [home.txt link list: discord.gg/bgUPV8KjDv]; "Advent of Vine 2025" repos exist under the org (WebFetch summary).

## 2. The model
- Interaction nets (docs' framing, author on HN): "an alternate model of computation (along the lines of the lambda calculus or Turing machines)", "fundamentally parallel" [hn-43144040.txt, quote of tjjfvi]. A net is a graph of agents joined by wires; only "active pairs" (two agents connected principal port to principal port) can be rewritten, and a rewrite touches only those two nodes and their neighbours, so rewrites are local. (General background, UNVERIFIED from memory except the docs quotes below.)
- Docs: "Two combinators of the same label annihilate, whilst two combinators of different labels commute" [d_ivy-ivm_interaction-system.txt, V-8]; base is "symmetric interaction combinators, with various extensions" [V-7]. Seven rule categories: Annihilate, Commute, Copy, Erase, Expand, Call, Branch.
- Ivy: "a low-level interaction-combinator programming language"; program = named global nets (`::name { ... }`) in a syntax based on the interaction calculus [d_ivy-ivm_ivy.txt]. IVM: "a performant interaction combinator runtime"; boots a ::main net with an IO value; has a parallel mode with worker threads and a dispatch thread [d_ivy-ivm_ivm.txt, V-9, V-10].
- Offered because of the model: parallel evaluation (`--workers`) [V-24]; deterministic interaction counts [V-32]; memory reuse, allocated=freed else "vicious circle" [V-34]; the `~` inverse operator and backwards data flow, which the docs say "is only possible due to the unique properties of interaction nets" [d_features_inverse.txt, V-18, V-19] (the page also calls itself "bad and should feel bad", niche and low-level). Author on HN: a wire is like a one-shot channel and `~` swaps its producer and consumer side [hn-43144040.txt].
- Memory: no garbage collector is mentioned in the docs; values are explicitly forked or dropped (below). "No GC" is not stated in so many words: UNVERIFIED as a claim.
- Types: Rust-influenced: structs, enums (`Option`, `Result`), tuples, generics, traits with implementation parameters, closures, methods, pattern matching, modules [d_features_types_enums.txt V-17, d_features_traits_intro.txt V-16, d_features_usual.txt]. Linearity: types are non-forkable and non-droppable by default (use exactly once); `?` droppable, `+` forkable, `*` both; compiler inserts fork/drop calls on multiple/zero use [d_features_traits_fork-drop.txt, V-15]. No borrow checker; `&` references are values representing places [d_features_references.txt]. Effects: no effect system; IO is a capability handle (pure unless given `&IO`) [V-13, V-14].
- Lacks, in docs' words: extrinsics set is fixed ("In the future, this will be extensible (#54)") [V-11]. No other "limitations" page found.

## 3. Targets and interop
- Compiler emits an Ivy file (`vine build`) [d_tooling_cli.txt, V-23]; execution is by the IVM interpreter (`vine run`). The docs describe no native or WebAssembly code generation.
- Browser/WebAssembly: not mentioned anywhere in the docs pages read or README (searched for wasm, webassembly, browser: no hits). Indirect evidence: lsp/Cargo.toml has `cfg(target_arch = "wasm32")` dependencies [raw_lsp_Cargo.toml.txt, V-31]; PR #550 "lsp: support wasm compilation target" merged 2026-06-25, stating the playground "(running on wasm)" cannot use glob, and adds "runtime hooks and an IVM runner" (WebFetch summary, V-47); a web playground exists at vine.run (examples, Run/Stop, "Breadth-First", Debug, Statistics) whose page does not say where code runs (WebFetch summary). No official statement that the IVM runs in a browser: UNVERIFIED.
- Runtime size: not stated.
- FFI: none documented. Only the built-in extrinsics (N32, F32, IO values; arithmetic, comparison, bit ops, stdout/stdin byte IO, `@seq`) [d_ivy-ivm_extrinsics.txt, V-11, V-12]; extensibility is issue #54 (V-49).
- Calling Vine from Rust as a library, or Rust from Vine: no documentation found. The repo is a Rust workspace of crates cli, ivm, ivy, lsp, util, vine, all version 0.0.0, with the compiler depending on git crate `class` [raw_Cargo.toml.txt, raw_vine_Cargo.toml.txt]; not published to crates.io under that name (the crates.io `vine` is unrelated, V-5). Use as a library: UNVERIFIED.
- IO: all via an `IO` handle, asynchronous, ordered by threading the handle [d_features_io.txt, V-13].
- Shipping as source built with an ordinary Rust toolchain: installation is `cargo install --git https://github.com/VineLang/vine --locked` [V-6], i.e. the compiler/runtime is Rust, but the Vine programs themselves compile to Ivy, not to Rust.

## 4. Tooling
- Compiler: Rust (cargo workspace) [raw_Cargo.toml.txt]. Install: cargo from git [V-6]. No release binaries documented.
- Compiler stages: cli, loader, parser, charter, resolver (type inference/checking, TIR), finder, distiller (to VIR, Stacked Flow Graph), normalizer, analyzer (reachability, dataflow), specializer, emitter [d_compiler_architecture.txt, V-20, V-22]; SFG described in [d_compiler_sfg.txt, V-21].
- CLI subcommands in source: run, build, check, test, fmt, show, repl, lsp, completion, plus hidden doc and root [raw_cli_src_vine_cli.rs.txt, V-30]. The docs describe build, run, REPL, debug mode only [d_tooling_cli.txt, d_tooling_repl.txt, d_tooling_debug.txt]. `show` appears to print formatted source (source lines read), not a net.
- Modules: Rust-like module system; third-party libraries via `--lib <path>` [d_features_modules.txt, d_tooling_cli.txt]. No package manager or registry found: UNVERIFIED.
- LSP: yes, `vine lsp` (tower-lsp); VSCode extension (publisher vinelang); Helix config with tree-sitter grammar in lsp/tree-sitter-vine [d_tooling_editor.txt, V-29]. Formatter `vine fmt` [same].
- Debugger: debug mode with `debug::log` (no-op outside debug) and `debug::error` with backtrace [d_tooling_debug.txt, V-28]. Docs generator: hidden `doc` subcommand and an "API Docs for #root" link [raw cli source; d_features_root.txt].
- Error messages: runtime error shown in docs is terse ("Error: the net did not return its `IO` handle"); compile-error examples: none read.
- CI examples: none read.

## 5. Performance
- No published benchmark against other languages found in the docs, README or HN thread (V-48; absence only).
- Docs show one illustrative statistics block: 11_106_946 interactions; sequential 63_375_794 IPS in 175 ms; 16 workers 569_167_395 IPS in 19 ms [d_ivy-ivm_statistics.txt, V-33]. Not a benchmark; program and machine not stated. Hello world: 341 interactions [d_starting_hello-world.txt].
- Memory: heap peak 784 B for that sample; "the evaluator was reusing its memory buffer very effectively. (This is very common for the IVM.)" [V-34]; default cap 4 GB [V-25].
- Authors on single-thread speed vs compiled code: nothing found. HN commenters questioned performance of `sub_min` (commenter, not author).

## 6. Ecosystem and use
- Standard library `#root`: N32, I32, F32, Char, String, List, Array, Map, IO, rng, debug [d_features_root.txt]. Examples in vine/examples/ [README]. Playground at vine.run with examples hello_world, fibonacci, primes, mandelbrot, lcs (WebFetch summary). Advent of Vine 2025 (repos advent.vine.dev, AoV-2025; WebFetch summary).
- Known users/production deployments: none found. Community: Discord; HN 203 points (2025-02-22). Teaching material: docs only; HN commenters asked for an up-front explanation of interaction nets.

## 7. Relatives
- HVM2: "massively parallel Interaction Combinator evaluator"; compilers to C and CUDA; HVM README says low-level IR "not meant for direct human usage" [rel_hvm_readme.txt, V-41]. Bend: high-level language over it; README lists targets "C, Metal, CUDA and JavaScript" and says "The JavaScript target runs on one core" [rel_bend_readme.txt, V-42].
- Inpla: "multi-threaded parallel interpreter of interaction nets", MIT, version 0.10.0 built 2022-09-17 in the README sample [rel_inpla_readme.txt, V-43].
- Per Vine's author: IVM is "architecturally similar to HVM-64"; main difference is IO (extrinsics via an IO handle vs HVM-64 built-in side-effecting nets vs HVM2 monadic IO "which requires stopping the whole program on every single IO call"); HVM/Bend "focus entirely on functional programming", Vine supports "both imperative and functional patterns" [hn-43144040.txt, V-35, V-36].
- MLIR talk by a third party (Martin Coll, 2025-04-10) lists "IVM + Vine" and "HVM2 + Bend" as prior art, and says Ivy is "syntactically similar to MLIR" [mlir.txt, V-40].

## 8. Program analysis, graph exposure
- The compiler uses graph IRs: Stacked Flow Graphs (VIR) and a reachability/dataflow analyzer [d_compiler_sfg.txt, d_compiler_architecture.txt, V-21, V-22]; Ivy is a textual serialisation of nets (named global nets with trees, pairs and wires) emitted as a file [V-23, d_ivy-ivm_ivy.txt]. That Ivy file is the artifact a tool can read. A separate tree-sitter grammar and an LSP exist [V-29].
- No documentation found of net visualisation, DOT/JSON export, or an API exposing the net/graph to other tools. `--breadth-first`/`--depth` measure parallel depth [V-26]; statistics give interaction counts [V-32]. The docs index links an "Illustrated Guide" to interaction nets (external) [docs-index.txt].

## 9. Explicit unknowns
- GitHub contributor count, exact 12-month commit count, repo creation date, any release or stability policy file (no GitHub API access; only WebFetch summaries).
- Whether the IVM or vine.run runs in the browser via WebAssembly; the size of any wasm build; the playground's architecture.
- Any Rust or JS interop beyond built-in extrinsics; library use of the `vine` crate from Rust.
- Package manager / registry; compile-error message quality; CI examples; formatter/test details beyond CLI source names.
- Published benchmarks; single-thread comparison with conventional code; GC statement.
- Production users; the contents of the vine/examples directory and the repo docs source.
- Whether "no garbage collector" is stated by the authors.
