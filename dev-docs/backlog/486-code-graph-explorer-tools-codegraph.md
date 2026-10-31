---
id: 486
title: "Code graph explorer: SCIP-derived graph with a granularity slider (tools/codegraph)"
status: "BACKLOG, not scheduled. Researched 2026-10-29 and 2026-10-31 at the owner's request; a prototype…"
size: "M"
area: tooling
updated: 2026-10-31
---

# Row 486: Code graph explorer

## Item

An explorer that shows the codebase as a graph: pick a component, see its shared libraries, its dependencies and its
dependents, and set the granularity with a slider (crate, component, file, item).

## Status / trigger

BACKLOG, not scheduled. Researched 2026-10-29 and 2026-10-31 at the owner's request; a prototype exists. The research,
the tool comparison and the measured numbers are in
[`research/code-graph-explorer-2026-10-31.md`](../research/code-graph-explorer-2026-10-31.md), and the prototype sources
are in [`research/code-graph-explorer/`](../research/code-graph-explorer/).

## Size

M

## Detail

- **Data:** `rust-analyzer scip` is the only source that resolves through `rsx!` / `#[component]` and across crates
  (8 min cold, 26 MB). On top of it come a TS import scan, `asset!()` / `include_str!()` for CSS, and spec-name links.
  cargo-modules is the cross-check.
- **UI:** a self-built static page on Cytoscape.js. It rolls edges up per level and has an ego radius, a minimum edge
  weight, a direction filter and a hub toggle.
- **Steps to make it a repo tool:**
  1. Move the prototype to `tools/codegraph/`.
  2. Add `scripts/codegraph.sh`: the SCIP run with an isolated absolute `CARGO_TARGET_DIR` and a staleness check
     against HEAD, then the build. The index and `graph.json` are generated, not committed.
  3. Pair components through `component.json` instead of by name.
  4. Make radius 2 a full breadth-first search.
  5. Add an item level, lazy per component, from crabmap or rust-analyzer's call hierarchy.
  6. Add a Playwright smoke test (`smoke.js`), outside `run-gates.sh`.
  7. Optional: a GraphML or Cypher export.
- **Keep in mind:** the explorer page is JavaScript in tracked HTML. Keep it under a path that `check-js-volume.sh`
  excludes, or exempt it explicitly.
- **Proof:** the explorer builds from a fresh index on the current HEAD, and the smoke test renders every level with no
  console errors.
