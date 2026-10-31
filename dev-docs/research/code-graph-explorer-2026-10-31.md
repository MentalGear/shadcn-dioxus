# Code graph explorer: tools, data model and prototype (2026-10-31)

Backs row 486. Two research rounds (2026-10-29 and 2026-10-31) on the owner's request: the best static-analysis tool
(CLI first, a UI self-built later) to see the codebase as a graph. You select a component, see its shared libraries and
dependencies, and set the granularity with a slider (crate, component, file, item). Measured results are marked
**measured**; everything else comes from the cited sources and is unverified unless stated.

## Verdict

Owner decisions 2026-10-31: one analysis source (rust-analyzer), and a dependency rule (no dependency with few stars or a
very recent release; thresholds and per-dependency verdicts in [Dependency rule](#dependency-rule-applied) below and in
row 486).

1. **Data source: `rust-analyzer scip`, the only source.** It is the only candidate that resolves symbols through
   proc-macro and RSX expansion (`rsx!`, `#[component]`) and across the workspace crates, and one index carries every
   level. It ships with rustup.
2. **Cross-check: cargo-modules 0.27, manual and one-off only.** It gives a module-level DOT graph in seconds. It has no
   file level, no weights and no call edges, and it analyses one crate per run. Version 0.28 needs rustc 1.98. The tool
   never installs or runs it.
3. **Item-level edges, if wanted: from rust-analyzer itself.** The SCIP `enclosing_symbol` / `relationships` fields, or
   rust-analyzer's own LSP call hierarchy driven by a Python stdlib script. crabmap and Crabviz are excluded by the
   dependency rule, owner 2026-10-31 (Crabviz only used the same call hierarchy through an editor).
4. **TypeScript:** the import scan (a regex over about 35 files). dependency-cruiser is excluded, owner 2026-10-31.
   **CSS and assets:** scan `asset!()` / `include_str!()`.
5. **UI: self-built on Cytoscape.js**, with compound nodes, the built-in `cose` layout and a hand-written roll-up per
   level. The prototype used cytoscape-fcose; it fails the maintenance threshold and is dropped. No existing tool
   combines a granularity slider with Rust that sees through macros.

## Measured on this repository

- **`rust-analyzer scip`** (rust-analyzer 1.97, workspace root, isolated `CARGO_TARGET_DIR`):
  - 489 s cold; output 26 MB with 897 documents, 236,576 occurrences and 36,580 symbol infos.
  - About 1,700 ERROR log lines for macro-generated items, so item-level data is lossy; the file and module levels
    are reliable.
  - 4,663 file-to-file reference edges, 1,007 of them from `preview` to `primitives`.
  - Example: `preview/src/components/select/component.rs` resolves into `primitives/src/select/*`, `collection.rs`,
    `listbox.rs`, `state_attrs.rs`, `direction.rs` and `top_layer/*`.
  - Many edges point at `primitives/src/lib.rs`; the explorer treats it as a hub.
- **cargo-modules 0.27.0:**
  - Module-only graph of `primitives`: 1,139 nodes and 953 `uses` edges in 7.9 s warm.
  - With functions, traits and types included: 7,864 nodes and 5,589 edges.
- **Prototype graph** (index from 2026-10-29):

  | Level | Nodes | Edges |
  |---|---|---|
  | crate | 9 | 11 |
  | component | 250 | 2,899 |
  | file | 1,372 | 10,716 |

  The 10,716 file-level edges are 4,663 Rust references, 4,615 to external crates, 1,088 TS imports, 200 asset
  references and 150 test links.
  - **Select ego network, component level, radius 1:** 36 components and 238 edges. The strongest edges are
    primitives:select to `selectable` (108), `lib` (82) and `selection` (46), and preview:select to primitives:select
    (69).
  - **Not resolved:** 10,905 references into macro-generated items. 53,613 core/std/alloc references are dropped on
    purpose.

## Tools compared

| Tool | Granularity | Rust / our data | Verdict |
|---|---|---|---|
| rust-analyzer `scip` | files, symbols, references, enclosing symbol, impl relations | yes, macro-aware, workspace-wide (measured); SCIP protobuf (schema Apache-2.0); no incremental mode, about 8 min per refresh | **the only source** (owner 2026-10-31); item level from its SCIP fields or its LSP call hierarchy |
| cargo-modules 0.27/0.28 (MPL-2.0) | crate, module, item; `owns`/`uses` | rust-analyzer based; DOT | manual one-off cross-check only; not a dependency of the tool |
| crabmap 0.2.0 (MIT) | fn/type/module/file; `crabmap serve` web UI | syn + cargo metadata, optional rust-analyzer call hierarchy and MIR; JSON/DOT/Mermaid; no SCIP | **excluded by the dependency rule, owner 2026-10-31** (first published 2026-05-28, 124 total crates.io downloads on 2026-10-31) |
| Crabviz (AGPL-3.0, VS Code) | file overview, per-function call hierarchy | any LSP server; editor-bound | **excluded by the dependency rule, owner 2026-10-31** |
| rustdoc JSON | items, impls, re-exports | unstable format, nightly, no reference edges | no |
| cargo tree / cargo-depgraph / cargo-deps | crates only | depgraph archived, deps unmaintained | no |
| cargo-call-stack | whole-program fn calls | nightly, fat LTO, embedded focus | no |
| CodeQL | security database | Rust GA since 2.23.3; wrong shape for this | no |
| GitHub stack-graphs | name resolution | archived 2025-09 | no |
| Sourcetrail forks (NumbatUI, petermost PR #73) | symbol/file graph | no SCIP; Rust indexer is a syn-based PR; "unstable" | no |
| CodeCharta | city map with metrics | no SCIP/GraphML importer found | size/churn maps only |
| SciTools Understand, CppDepend (Rust beta) | architecture / DSM graphs | commercial | no |
| CodeScene, Sonargraph, Lattix, Structure101 | hotspots / DSM | Rust support missing or unverified | no |
| Sourcegraph | precise navigation from SCIP | no graph view or graph API found | consumer only |
| GitDiagram, Swark, CodeViz | one LLM-drawn diagram | not data-faithful | no |
| Graph-RAG tools (code-review-graph, Code-Graph-RAG, treeloom, Codebase-Memory) | query graphs for agents, Neo4j/SQLite | tree-sitter based, Rust unconfirmed | export target later |
| dependency-cruiser (MIT) | TS/JS modules, folder `collapse` | JSON/DOT/Mermaid | **excluded by the dependency rule, owner 2026-10-31** (the regex scan covers about 35 files; 18 direct dependencies) |

**UI libraries** (versions from the npm registry, 2026-10-31):
- **Cytoscape.js:** compound nodes, built-in `cose`, `concentric` and `breadthfirst` layouts, comfortable to a few
  thousand elements. The latest, 3.34.4, was released 2026-10-15 (too young for the 30-day rule); 3.34.3 (2026-09-07) is
  the newest eligible. The prototype pins 3.33.1.
- **cytoscape-fcose 2.2.0** with cose-base 2.2.0 and layout-base 2.0.1: last releases 2023-01-17 and 2021-06-25; dropped
  (maintenance threshold). The prototype uses it.
- Not selected, and not evaluated against the rule: **Sigma 4.0.4 with graphology 0.26.0** (WebGL, only needed to show all
  ~13,500 items at once), **elkjs 0.12.1** (layered layout), **React Flow (@xyflow/react 12.12)** (needs React and manual
  layout), **D3 edge bundling** (no ego or filter interaction). Any of them would have to pass the rule first.

## Dependency rule applied

Owner 2026-10-31: no dependency that has few stars or is very recent. The thresholds are in
[row 486](../backlog/486-code-graph-explorer-tools-codegraph.md) (D1 to D6). D1 (30 days) and the no-install-script,
exact-pin footprint come from the supply-chain docs
([`supply-chain/README.md`](../supply-chain/README.md), [`dependency-scanners.md`](./dependency-scanners.md)); the docs
define no star, project-age or maintenance number, so D2 (first release 12 months old), D3 (1,000 stars, or 100,000
weekly npm / 1,000,000 total crates.io downloads where stars cannot be read) and D4 (a release in 24 months) are
**proposed**. GitHub stars could not be read from the session, so adoption is judged on downloads (**measured**, npm and
crates.io APIs, 2026-10-31) and marked unverified.

| Dependency | Measured | Verdict |
|---|---|---|
| rust-analyzer | rustup component, toolchain trusted as installed | keep |
| Python stdlib (`build_graph.py`, `make_explorer.py`, `scipdump.py`) | imports: `sys`, `os`, `re`, `json`, `collections`, `itertools`; protobuf read by a hand-written varint reader | keep, stdlib only |
| Cytoscape.js | since 2012-10-18; 24.3M weekly npm downloads; MIT; no dependencies; 3.34.4 is 16 days old, 3.34.3 is 54 | keep, pin 3.34.3 |
| cytoscape-fcose, cose-base, layout-base | 22.4M, 44.9M and 45.0M weekly downloads; last releases 2023-01-17, 2023-01-17, 2021-06-25 | fail the maintenance threshold; use built-in `cose` |
| dependency-cruiser | since 2016; 7.9M weekly; 18.7.0 released 2026-10-19; 18 direct dependencies | excluded; regex scan |
| Playwright 1.60.0 | existing, locked dependency | keep |
| crabmap | first published 2026-05-28; 124 total downloads | excluded |
| cargo-modules | since 2016; 256,852 total downloads; MPL-2.0 | manual cross-check only |

## Data model

- **Levels** (the slider is the roll-up depth):
  - **L0 crate:** taken from the SCIP package.
  - **L1 component:** each `primitives/src/<x>` and `preview/src/components/<x>`. Pair the two halves by name, or
    better by `component.json`.
  - **L2 file:** SCIP documents.
  - **L3 item:** SCIP symbols with `enclosing_symbol`, or rust-analyzer's call hierarchy (no third-party tool).
- **Edge kinds:** `ref` (use, call or type mention, weighted by occurrence count), `impl`/`type-def`, `ext` (external
  crate), `ts-import`, `asset` (CSS and other assets) and `test` (spec to component).
- **Roll-up:** every edge becomes an edge between its endpoints' ancestors at the chosen level. Weights are summed and
  self-edges dropped.
- **Hubs:** components with 100 or more distinct neighbours, plus any `lib.rs`, are hidden by default.

## Prototype

- **Sources:** [`code-graph-explorer/`](./code-graph-explorer/).
  - `build_graph.py`: SCIP plus the scans, written to `graph.json`.
  - `make_explorer.py`: inlines the graph into the template.
  - `explorer.template.html`: the page.
  - `smoke.js`: Playwright smoke test.
  - `scipdump.py`: the round-one SCIP reader.
- **Rebuild:**
  ```
  CARGO_TARGET_DIR=<absolute isolated dir> rust-analyzer scip . --output index.scip
  python3 -I build_graph.py index.scip graph.json
  python3 -I make_explorer.py explorer.template.html graph.json explorer.html
  ```
- **What the page has:**
  - a component picker and the level slider (crate, component, file)
  - ego radius 1-2, a minimum edge weight and the direction (uses, used by, both)
  - a hub toggle, crate boxes and edge-kind switches
  - tap for the top neighbours, double-tap to re-centre
  - light and dark themes; usable at phone width
- **Published privately for the owner on 2026-10-31.** The page loads Cytoscape, cytoscape-fcose, cose-base and layout-base
  from jsDelivr, which a private preview allows; the repo version has no run-time load and bundles one pinned library
  (row 486, S2 and S3). The three Python files import only the standard library.
- **Limits:**
  - No item level yet (about 1 MB more data).
  - Radius 2 expands only from the strongest neighbours.
  - Components are paired by name, not `component.json`.
  - The index has to be rebuilt by hand.
  - The repository path is hardcoded in `build_graph.py` and the scratch path in `smoke.js`.

## Sources

https://github.com/sourcegraph/scip ; https://github.com/rust-lang/rust-analyzer/pull/17900 ;
https://github.com/rust-lang/rust-analyzer/issues/23299 ; https://github.com/regexident/cargo-modules ;
https://docs.rs/crate/crabmap/0.2.0 ; https://github.com/chanhx/crabviz ; https://github.com/quarkslab/NumbatUI ;
https://github.com/petermost/Sourcetrail/pull/73 ; https://codecharta.com/docs/overview/analysis ;
https://docs.scitools.com/graphs/Requires_and_Required_By.html ; https://github.com/github/stack-graphs ;
https://github.com/rust-lang/rust/issues/76578 ; https://github.com/sverweij/dependency-cruiser ;
https://github.com/AeneasVerif/charon ; https://github.com/tirth8205/code-review-graph ;
https://github.com/vitali87/code-graph-rag ; https://github.com/treeloom/treeloom ;
https://repowise.dev/blog/comparisons/best-codebase-visualization-tools ; https://registry.npmjs.org ; https://api.npmjs.org/downloads ; https://crates.io/api/v1/crates
