---
id: 486
title: "Code graph explorer: SCIP-derived graph with a granularity slider (scripts/codegraph)"
status: "BACKLOG, not scheduled. Researched 2026-10-29 and 2026-10-31 at the owner's request; a prototype…"
size: "M"
area: tooling
updated: 2026-10-31
---

# Row 486: Code graph explorer

## Item

An explorer that shows the codebase as a graph: pick a component, see its shared libraries, its dependencies and its
dependents, and set the granularity with a slider (crate, component, file, item). It is a repo tool, so every dependency
it uses passes the dependency rule below and every security requirement below holds before it lands.

## Status / trigger

BACKLOG, not scheduled. Researched 2026-10-29 and 2026-10-31 at the owner's request; a prototype exists. The research,
the tool comparison and the measured numbers are in
[`research/code-graph-explorer-2026-10-31.md`](../research/code-graph-explorer-2026-10-31.md), and the prototype sources
are in [`research/code-graph-explorer/`](../research/code-graph-explorer/).

Owner decisions 2026-10-31:
1. rust-analyzer is the one analysis source. crabmap, Crabviz and every other young or little-used tool are out of the
   plan; cargo-modules is at most a manual one-off cross-check and the tool never installs or calls it.
2. A general dependency rule: no dependency with few stars or a very recent release. Defined below, applied to each
   dependency.
3. The tool follows the supply-chain playbook as a repo tool: no CDN load at run time, exact pins, minimum release age,
   no install scripts, an isolated absolute `CARGO_TARGET_DIR`, generated artefacts not committed, no network need in the
   page. Written below as requirements S1-S10.
4. The published prototype stays as it is (it loads its libraries from a CDN, for a private preview only).

## Size

M

## Detail

### What the tool is

- **Data:** `rust-analyzer scip` is the only source that resolves through `rsx!` / `#[component]` and across crates
  (8 min cold, 26 MB). On top of it come a TS import scan (a regex over about 35 files), `asset!()` / `include_str!()`
  for CSS, and spec-name links. Nothing else analyses Rust.
- **Item level:** comes from rust-analyzer itself, in one of two ways, both written in Python stdlib: (a) the SCIP
  `enclosing_symbol` and `relationships` fields (about 1 MB more data; lossy for macro-generated items, 10,905
  unresolved references measured), or (b) a script that drives rust-analyzer's own LSP call hierarchy over stdio
  (`callHierarchy/incomingCalls`), lazily per component. Which fields rust-analyzer 1.97 emits is unverified; the first
  step of the item-level work is a spike that lists them.
- **UI:** a self-built static page on Cytoscape.js with its built-in `cose` layout (compound nodes). It rolls edges up
  per level and has an ego radius, a minimum edge weight, a direction filter and a hub toggle. `concentric` and
  `breadthfirst` are built in too and are candidates for the ego views. Layout quality at the file level (about 1,400
  nodes) is measured in the smoke step; the prototype used cytoscape-fcose, which this row drops (below).

### The dependency rule (owner 2026-10-31)

**No dependency that has few stars or is very recent.** The supply-chain docs define two of the numbers and leave the rest
to a case-by-case read of owners, cadence and downloads
([`dependency-evaluation.md`](../research/dependency-evaluation.md) section 1,
[`dependency-scanners.md`](../research/dependency-scanners.md) recommendation G). Where the docs define a number, it is
used as written; every other threshold is **proposed** and awaits the owner.

| # | Rule | Source |
|---|---|---|
| D1 | A pinned release is at least 30 days old (`minimumReleaseAge: 43200`, pnpm enforces it at resolution) | defined: owner decision, row 359, `playwright/pnpm-workspace.yaml`, `preview/pnpm-workspace.yaml`, playbook C09/C19. Cargo-installed tools: 14 days, `dependency-scanners.md` G |
| D2 | The project's first release is at least 12 months old | proposed |
| D3 | Broad adoption: at least 1,000 GitHub stars; where stars cannot be read, at least 100,000 weekly npm downloads or 1,000,000 total crates.io downloads as the stand-in | proposed |
| D4 | Active maintenance: a release in the last 24 months, not archived or deprecated | proposed |
| D5 | Footprint: exact pin, no install script, few or no transitive dependencies, a permissive licence (the repo's licence set is MIT/Apache/ISC-class) | defined: playbook C09 (no install scripts), C03 (exact pins); `dependency-scanners.md` facts (licences) |
| D6 | An exception is the owner's, written into this row with a reason and an end date, as `minimumReleaseAgeExclude` entries are | defined: the yaml header comment above |

Evidence below is from the npm registry, `api.npmjs.org` and the crates.io API read on 2026-10-31. **GitHub stars could
not be read** (the GitHub API is not reachable from the session), so D3 is judged on downloads and marked unverified
where it rests on them.

| Dependency | Evidence | Verdict |
|---|---|---|
| rust-analyzer 1.97 | ships with rustup as a toolchain component; the playbook treats the toolchain as trusted as installed ("What we do not defend against"); stars unverified | **keep** (toolchain class) |
| Python stdlib | the prototype imports `sys`, `os`, `re`, `json`, `collections`, `itertools` and nothing else; SCIP protobuf is read with a hand-written varint reader, so no `protobuf` or `scip` package; no `subprocess`, `urllib` or socket use | **keep**; stdlib only, run as `python3 -I` |
| Cytoscape.js | first published 2012-10-18; 24.3M npm downloads in the week 2026-10-23 to 29; MIT; no runtime dependencies and no install script; eight releases between 2026-04 and 2026-10. 3.34.4 (2026-10-15) is 16 days old and fails D1; **3.34.3 (2026-09-07, 54 days) passes**. Stars unverified | **keep**, pinned to the newest release that is 30 days old when the folder is created (3.34.3 today); pnpm refuses a younger one |
| cytoscape-fcose 2.2.0 | first published 2019-04-12; 22.4M weekly downloads; MIT; last release 2023-01-17, about 45 months ago (fails D4); depends on cose-base | **fails D4**: drop; the page uses Cytoscape's built-in `cose`. Back only by a D6 exception if `cose` is measured unusable at the file level |
| cose-base 2.2.0, layout-base 2.0.1 | fcose's dependencies; last releases 2023-01-17 and 2021-06-25 (fail D4); 44.9M and 45.0M weekly downloads | **drop** with fcose |
| dependency-cruiser 18.7.0 | first published 2016; 7.9M weekly downloads; the latest release (2026-10-19) is 12 days old; 18 direct dependencies | **excluded**: the regex scan covers the TS part; not added |
| Playwright 1.60.0 | already in `playwright/package.json`, exact pin, locked, 30-day age, no install scripts | **keep** (existing dependency) |
| crabmap | first published 2026-05-28 (0.3.1 is current), 124 total downloads on crates.io | **excluded** (fails D2 and D3), owner 2026-10-31 |
| Crabviz | AGPL-3.0, VS Code extension, editor-bound; stars unverified | **excluded**, owner 2026-10-31 |
| cargo-modules | first published 2016; 256,852 total downloads; MPL-2.0 (the repo dropped its last MPL crate in row 217) | **not a dependency**; a manual one-off cross-check only, no script of the tool installs or runs it |

### Security requirements (supply-chain playbook)

Playbook: [`supply-chain/README.md`](../supply-chain/README.md). Control ids are its Controls table; T-ids its threat model.

- **S1. One rule for every dependency.** The dependency rule above applies to every library, crate, tool and package the
  tool uses. A new one needs its row of evidence in the tool's README before it is used, and the ladder first (drop, then
  replace with std or a browser API, then keep; `dependency-evaluation.md` section 1).
- **S2. The page needs no network (T9).** `explorer.html` is one self-contained file: no `<script src>`, `<link>`,
  `@import`, font or image from an `http(s)` URL, and no CDN at run time. The smoke test aborts every request that is
  not `file:` and fails on any attempt. C13 (`check-demo-remote-assets.sh`) covers the docs site and does not flag
  scripts, and `check-js-volume.sh` flags an external `<script src>` only inside its universe, which excludes
  `scripts/`; so S9 adds the tool's own check.
- **S3. Library files come from a lockfile, with the version and checksum recorded.** Preferred: a pnpm folder
  `scripts/codegraph/` with a `package.json` that pins `cytoscape` exactly (no `^` or `~`), its own
  `pnpm-workspace.yaml` carrying `minimumReleaseAge: 43200` and `onlyBuiltDependencies: []`, and a committed
  `pnpm-lock.yaml`; the lock's sha512 `integrity` is the recorded checksum. Installed only with
  `pnpm install --frozen-lockfile` (C10; `lane-rules.md` "Worktree and files": never `pnpm add`/`update` or `npm install`,
  worktrees symlink or frozen-install `node_modules`). `make_explorer.py` inlines
  `node_modules/cytoscape/dist/cytoscape.min.js` into the page. Why preferred: pnpm enforces the age and the no-scripts
  rule itself ("tool-enforced settings are used where the tool already enforces the property"), the lock is what
  OSV-Scanner (optional, manual) and the planned C19 gate read, and no blob lands in git.
  Alternative, if the owner prefers vendoring: exact files under `scripts/codegraph/vendor/`, each with a
  `vendored-from: <package>@<version> sha512 <npm integrity>` header (the playbook's `lifted-from` convention, C20 and
  `dependency-scanners.md` Part B item 4), a sha256 manifest that a gate checks in both directions (C07's pattern), and
  the S1 evidence in the README. Nothing scans vendored bytes for advisories, which is why it is the alternative.
- **S4. Exact pins.** Every `package.json` dependency is an exact version (C19's rule); `packageManager` is the pnpm the
  other two folders use; no `latest`, no range, no git or tarball resolution.
- **S5. Minimum release age.** `minimumReleaseAge: 43200` in the new `pnpm-workspace.yaml`, with no
  `minimumReleaseAgeExclude` entry (D6 applies to any). On the Rust side nothing is installed, so there is no cargo tool
  to age.
- **S6. No install scripts (T2, C09).** `onlyBuiltDependencies: []`. The lock holds one package (Cytoscape, no transitive
  dependencies per the registry).
- **S7. The SCIP run (CLAUDE.md "Build environment", `lane-rules.md`).** `scripts/codegraph.sh` requires an absolute,
  isolated `CARGO_TARGET_DIR` (never the repo-root `target/`; it refuses a relative or shared one as `build-ssg.sh` does),
  runs with `CARGO_INCREMENTAL=0`, analyses only this repository's tree, and executes only the build scripts and
  proc-macros the workspace already builds (rust-analyzer loads them as `cargo check` does; the workspace lock has 73
  build-script crates and 42 proc-macros, measured 2026-10-23). It runs after `cargo fetch` with `CARGO_NET_OFFLINE=true`
  (proposed; unverified that `rust-analyzer scip` honours it, so the first spike checks) and fails if `Cargo.lock` differs
  afterwards. The tool adds no Cargo dependency, so C01, C02 and C16 to C18 see no change.
- **S8. Generated artefacts are not committed.** `index.scip`, `graph.json` and `explorer.html` are written to
  `target-codegraph/` (already ignored by `target-*/`); the build refuses an output path inside tracked files.
- **S9. One gate keeps S2, S3 and the stdlib claim true.** `scripts/check-codegraph-deps.sh`, offline and deterministic
  like the other supply-chain gates: every `import` in `scripts/codegraph/*.py` is in `sys.stdlib_module_names`; the page
  template and the built-page recipe contain no `http(s)://` URL outside a reasoned allowlist; `package.json`
  dependencies are exact and are the one named package; `git ls-files` holds no `*.scip`, `graph.json` or
  `explorer.html`. It gets its header, a `run-gates.sh` entry, `scripts/gates-doc.py --write`, and a row in the playbook's
  Controls table plus the drift gate's `SUPPLY_CHAIN_SCRIPTS` list. The new `pnpm-workspace.yaml` is added to the C09
  and C19 config columns and the frozen install to `scripts/session-setup.sh` (C10).
- **S10. Scanners.** Today the repo has no advisory scanner (C16 to C18 are planned), so the new files are covered by: C09
  and C10 for the pnpm folder, the planned C19 once it lists the third `pnpm-workspace.yaml`, an optional manual
  OSV-Scanner run over `pnpm-lock.yaml`, and S9 for the rest. cargo-deny and cargo-vet, once adopted, need no change
  because `Cargo.lock` does not change. The smoke test uses the Playwright already pinned in `playwright/`.

### Steps to make it a repo tool

  1. Move the prototype to `scripts/codegraph/` and derive the repository path from the script's location (the prototype
     hardcodes it).
  2. Add `scripts/codegraph.sh`: the S7 SCIP run with a staleness check against HEAD, then the build (S8).
  3. Swap the CDN `<script>` tags for the inlined Cytoscape (S3) and the fcose layout for `cose` (dependency rule);
     measure each level and keep or replace per layout.
  4. Pair components through `component.json` instead of by name.
  5. Make radius 2 a full breadth-first search.
  6. Add the item level (rust-analyzer's SCIP fields or its call hierarchy, as above), lazy per component.
  7. Add the Playwright smoke test (`smoke.js`, resolving `playwright` from `playwright/node_modules`, scratch path
     replaced by the build output), outside `run-gates.sh`, with the S2 request check.
  8. Add the S9 gate and the S3 to S6 files.
  9. Optional: a GraphML or Cypher export, written with the stdlib.
- **Keep in mind (JS volume):** the explorer page is JavaScript in tracked HTML. `check-js-volume.sh` excludes `scripts/`
  (like `playwright/` and `dev-docs/`), so the tool lives under `scripts/codegraph/` and the page is dev tooling that
  ships nowhere and touches nothing in `primitives/` or `preview/`, which is what the bridge rule in
  [`js-interop.md`](../js-interop.md) governs. A top-level `tools/` would be inside the gate's universe and needs a
  visible `--update-baseline --allow-raise "row 486 ..."`; the volume rule says it can only shrink, so the excluded path is
  the choice.
- **Proof:** the explorer builds from a fresh index on the current HEAD, the smoke test renders every level with no
  console errors and no network attempt, `scripts/check-codegraph-deps.sh` is green, `scripts/run-gates.sh` is
  `ALL GREEN`, and `git status` shows no generated artefact.
