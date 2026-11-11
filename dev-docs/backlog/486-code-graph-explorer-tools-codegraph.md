---
id: 486
title: "Code graph explorer: SCIP-derived graph with a granularity slider (scripts/codegraph)"
status: "BUILT, minimal scope (2026-11-03): scripts/codegraph/ (run.sh, build_graph.py, make_explorer.py…"
size: "S-M for the minimal scope; the rest is…"
area: tooling
updated: 2026-11-07
---

# Row 486: Code graph explorer

## Item

An explorer that shows the codebase as a graph: pick a component, see its shared libraries, its dependencies and its
dependents, and set the granularity with a slider (crate, component, file, item). It is a repo tool, so every dependency
it uses passes the dependency rule below and every security requirement below holds before it lands.

**Dev tool only (owner decision 2026-11-02).** The explorer is a developer tool that lives in the repo under
`scripts/codegraph/`, like `scripts/run-gates.sh`. It is never shipped: not part of the CLI, not in the component registry
or any `component.json`, not installable by users, and not in `run-gates.sh`. The item granularity of the slider is
optional (Detail, Scope, Optional).

## Status / trigger

BUILT, minimal scope (2026-11-03): `scripts/codegraph/` (`run.sh`, `build_graph.py`, `make_explorer.py`,
`explorer.template.html`, `smoke.js`, a pnpm folder with Cytoscape.js 3.34.3, `README.md`). Delivery route: the preferred
pnpm folder (S3), because it conflicts with no rule: `scripts/codegraph/` has its own `package.json`,
`pnpm-workspace.yaml` and lock like `playwright/` and `preview/`. The staleness key, the C09/C19 config columns and the
session-setup frozen install are in. Recommended: full radius 2 done (the ego view is a complete breadth-first search; `smoke.js` checks the radius 1 and 2 sizes
against an independent search); pairing through `component.json` done (2026-11-07): `component.json` marks a themed
component and names no primitive, so the primitive is read from the `dioxus_primitives::<module>` paths in its sources
(same-named module first, else the unique most-referenced non-helper module); preview components paired with a primitive
46 -> 60, files in paired components 552 -> 779, and the 84 themed `mod.rs` files (absent from the SCIP index, e.g.
`preview/src/components/select/mod.rs`) are nodes. Reference accuracy (2026-11-07): the 11,283 workspace references with no definition occurrence (derive and Props-builder
methods 7,378, `#[component]` types 1,111, module namespaces 1,912, macro-generated types and consts about 880) are mapped to
their owner type (9,264) or module file (2,019): unresolved 0, `rs` file edges 4,949 -> 6,628, item-level edges 16,757 ->
23,897. N1 candidates 116 -> 16 by rules R1 to R6 (README), not a name list. Optional items open. The page switches to a concentric
layout above 250 nodes because `cose` gives a hairball there (`cose` is kept up to that size, with boxes). Proof numbers
are in the README. Dev tool only (owner 2026-11-02).

Comment metrics M1 to M6 BUILT (2026-11-05, owner-approved proposal `scripts/codegraph/comment-metrics-proposal.md`): comment
ratio, doc share, pub doc coverage, history-rule hits, TODO/FIXME/HACK/XXX counts and density outliers, from one pass of
`scripts/comment-scan.py` (its `walk_rust` and `embedded_comments` are now public names, a rename only; both gates
unchanged) in `scripts/codegraph/comment_metrics.py`, summed per component and crate, stored in `graph.json`, shown as
lenses, a details-panel table, outlier rings and a Comments worst-files tab. Still a dev tool: no gate, no dependency.
Measured: primitives crate M1 0.48, M2 80.5 %, M3 97.7 %; M4 total 0 (equals `check-history-comments.sh`, 0; a planted
history comment read 1 on both); M5 total 2 (two `TODO` words in `preview/src/site`); M6 dense outliers 22 files, sparse 4
(the sparse fence is on a log scale because the plain one is negative). Rebuilding the graph with the metrics takes about 14
s. Separate staleness keys (2026-11-06): the metrics live in `comments.json` (file text only, `build_comments.py`), the graph in
`graph.json` (SCIP index); a `.rs` edit no longer re-indexes, it rebuilds the metrics and the page at once and the page
marks the index stale (changed files listed); `--force` re-indexes. `status.json` and the page's status line show the HEAD,
key and time each part was built from. Nice-to-have N1 to N6 BUILT (2026-11-07, built in the order N1, N6, N4, N2/N3, N5: defect-finding and text-only first, `git blame` last):
stale-looking backticked identifiers, comment lines and code lines of JS/CSS in Rust strings, comment words, repeated comment
blocks, per-item comment numbers (8,238 items, a Comments-tab row level; no item nodes or item edges in the graph, the slider
stays crate/component/file) and comment age vs code age (`run.sh --blame`). Details and limits: `scripts/codegraph/README.md`. Numbers and how to read them: `scripts/codegraph/README.md`.

**What the metrics turn up, pass by pass, is its own standing row:** [row 507](./507-codegraph-comment-analysis-findings.md)
("code analyzer -> comment analysis"), kept separate from this row (which is about the tool itself) so the running
list of findings and follow-ups has one home across passes.

Researched 2026-10-29 and 2026-10-31 at the owner's request; a prototype exists. The research, the tool comparison and the
measured numbers are in [`research/code-graph-explorer-2026-10-31.md`](../research/code-graph-explorer-2026-10-31.md), and
the tool is in [`scripts/codegraph/`](../../scripts/codegraph/) (the prototype sources moved there).

Owner decisions 2026-10-31:
1. rust-analyzer is the one analysis source. crabmap, Crabviz and every other young or little-used tool are out of the
   plan; cargo-modules is at most a manual one-off cross-check and the tool never installs or calls it.
2. A general dependency rule: no dependency with few stars or a very recent release. Defined below, applied to each
   dependency.
3. The tool follows the supply-chain playbook as a repo tool: no CDN load at run time, exact pins, minimum release age,
   no install scripts, an isolated absolute `CARGO_TARGET_DIR`, generated artefacts not committed, no network need in the
   page. Written below as requirements S1-S10.
4. The published prototype stays as it is (it loads its libraries from a CDN, for a private preview only).

Owner decisions 2026-11-02:
1. **Dev tool only.** See Item. Not shipped, not in the CLI, the registry or any `component.json`, and not in
   `run-gates.sh`.
2. **Scope split.** The row's scope is the minimal set in Detail, Scope (a) to (c), size S-M. The recommended items and
   the optional items are defined there too and are not part of the minimal scope.
3. The dependency verdicts and security requirements S1-S10 stand unchanged in substance; sentences that assumed the
   tool would be a gate or a shipped surface were adjusted to match decision 1.

## Size

S-M for the minimal scope; the rest is unsized.

## Detail

### What the tool is

- **Data:** `rust-analyzer scip` is the only source that resolves through `rsx!` / `#[component]` and across crates
  (8 min cold, 26 MB). On top of it come a TS import scan (a regex over about 35 files), `asset!()` / `include_str!()`
  for CSS, and spec-name links. Nothing else analyses Rust.
- **Item level (optional):** comes from rust-analyzer itself, in one of two ways, both written in Python stdlib: (a) the
  SCIP `enclosing_symbol` and `relationships` fields (about 1 MB more data; lossy for macro-generated items, 10,905
  unresolved references measured), or (b) a script that drives rust-analyzer's own LSP call hierarchy over stdio
  (`callHierarchy/incomingCalls`), lazily per component. Which fields rust-analyzer 1.97 emits is unverified; the first
  step of the item-level work is a spike that lists them.
- **UI:** a self-built static page on Cytoscape.js with its built-in `cose` layout (compound nodes). It rolls edges up
  per level and has an ego radius, a minimum edge weight, a direction filter and a hub toggle. `concentric` and
  `breadthfirst` are built in too and are candidates for the ego views. Layout quality at the file level (about 1,400
  nodes) is checked by hand during the minimal scope's proof; the prototype used cytoscape-fcose, which this row drops
  (below).

### The dependency rule (owner 2026-10-31)

**No dependency that has few stars or is very recent.** The supply-chain docs define two of the numbers and leave the rest
to a case-by-case read of owners, cadence and downloads
([`dependency-evaluation.md`](../research/dependency-evaluation.md) section 1,
[`dependency-scanners.md`](../research/dependency-scanners.md) recommendation G). Where the docs define a number, it is
used as written. Owner decision 2026-10-31: the rule is the general rule for every new dependency in the repo (crates,
npm packages, vendored libraries, cargo-installed tools), not only this tool's. D2-D4 were proposed and are now
**approved by the owner 2026-10-31**; D1 and D5 were already repo rules; D6 is the owner's. The rule lives in the
supply-chain playbook, section [Choosing a new dependency](../supply-chain/README.md#choosing-a-new-dependency-owner-2026-10-31)
(control C23), and this row applies it.

| # | Rule | Source |
|---|---|---|
| D1 | A pinned release is at least 30 days old (`minimumReleaseAge: 43200`, pnpm enforces it at resolution) | defined: owner decision, row 359, `playwright/pnpm-workspace.yaml`, `preview/pnpm-workspace.yaml`, playbook C09/C19. Cargo-installed tools: 14 days, `dependency-scanners.md` G |
| D2 | The project's first release is at least 12 months old | approved by the owner 2026-10-31 |
| D3 | Broad adoption: at least 1,000 GitHub stars; where stars cannot be read, at least 100,000 weekly npm downloads or 1,000,000 total crates.io downloads as the stand-in | approved by the owner 2026-10-31 |
| D4 | Active maintenance: a release in the last 24 months, not archived or deprecated | approved by the owner 2026-10-31 |
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
| Playwright 1.60.0 | already in `playwright/package.json`, exact pin, locked, 30-day age, no install scripts | **keep** (existing dependency; used only by the optional smoke test) |
| crabmap | first published 2026-05-28 (0.3.1 is current), 124 total downloads on crates.io | **excluded** (fails D2 and D3), owner 2026-10-31 |
| Crabviz | AGPL-3.0, VS Code extension, editor-bound; stars unverified | **excluded**, owner 2026-10-31 |
| cargo-modules | first published 2016; 256,852 total downloads; MPL-2.0 (the repo dropped its last MPL crate in row 217) | **not a dependency**; a manual one-off cross-check only, no script of the tool installs or runs it |

### Security requirements (supply-chain playbook)

Playbook: [`supply-chain/README.md`](../supply-chain/README.md). Control ids are its Controls table; T-ids its threat model.

- **S1. One rule for every dependency.** The dependency rule above applies to every library, crate, tool and package the
  tool uses. A new one needs its row of evidence in the tool's README before it is used, and the ladder first (drop, then
  replace with std or a browser API, then keep; `dependency-evaluation.md` section 1).
- **S2. The page needs no network (T9).** `explorer.html` is one self-contained file: no `<script src>`, `<link>`,
  `@import`, font or image from an `http(s)` URL, and no CDN at run time. The optional smoke test aborts every request that
  is not `file:` and fails on any attempt. C13 (`check-demo-remote-assets.sh`) covers the docs site and does not flag
  scripts, and `check-js-volume.sh` flags an external `<script src>` only inside its universe, which excludes `scripts/`;
  so S9 adds the tool's own check. Until the optional gate exists, the page is checked by hand (Scope, Proof).
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
- **S7. The SCIP run (CLAUDE.md "Build environment", `lane-rules.md`).** `scripts/codegraph/run.sh` requires an absolute,
  isolated `CARGO_TARGET_DIR` (never the repo-root `target/`; it refuses a relative or shared one as `build-ssg.sh` does),
  runs with `CARGO_INCREMENTAL=0`, analyses only this repository's tree, and executes only the build scripts and
  proc-macros the workspace already builds (rust-analyzer loads them as `cargo check` does; the workspace lock has 73
  build-script crates and 42 proc-macros, measured 2026-10-23). It runs after `cargo fetch` with `CARGO_NET_OFFLINE=true`
  (proposed; unverified that `rust-analyzer scip` honours it, so the first spike checks) and fails if `Cargo.lock` differs
  afterwards. The tool adds no Cargo dependency, so C01, C02 and C16 to C18 see no change.
- **S8. Generated artefacts are not committed.** `index.scip`, `graph.json` and `explorer.html` are written to
  `target-codegraph/` (already ignored by `target-*/`), with the staleness key beside `index.scip` (Scope (b)); the build
  refuses an output path inside tracked files.
- **S9. The checks behind S2, S3 and the stdlib claim.** Owner decision 2026-11-02: the tool is not in `run-gates.sh`, so
  these checks are not a gate by default. They are: every `import` in `scripts/codegraph/*.py` is in
  `sys.stdlib_module_names`; the page template and the built-page recipe contain no `http(s)://` URL outside a reasoned
  allowlist; `package.json` dependencies are exact and are the one named package; `git ls-files` holds no `*.scip`,
  `graph.json` or `explorer.html`. Until the owner adopts the optional gate (Scope, Optional), they are checked by hand at
  review. If adopted, the gate takes a header, the usual `scripts/gates-doc.py --write` and a row in the playbook's
  Controls table plus the drift gate's `SUPPLY_CHAIN_SCRIPTS` list; its `run-gates.sh` registration is a separate owner
  decision, because decision 1 keeps the tool out of `run-gates.sh`. The new `pnpm-workspace.yaml` is added to the C09 and
  C19 config columns and the frozen install to `scripts/session-setup.sh` (C10), as part of Scope (c).
- **S10. Scanners.** Today the repo has no advisory scanner (C16 to C18 are planned), so the new files are covered by: C09
  and C10 for the pnpm folder, the planned C19 once it lists the third `pnpm-workspace.yaml`, an optional manual
  OSV-Scanner run over `pnpm-lock.yaml`, and S9 for the rest (by hand until the optional gate exists). cargo-deny and
  cargo-vet, once adopted, need no change because `Cargo.lock` does not change. The optional smoke test would use the
  Playwright already pinned in `playwright/`.

### Scope

**Minimal (the row's scope), size S-M:**

- **(a) Run and build.** `scripts/codegraph/run.sh` runs `rust-analyzer scip` with an isolated absolute
  `CARGO_TARGET_DIR` (S7); the stdlib-only Python extractor, run as `python3 -I`, writes `graph.json`; `make_explorer.py`
  builds `explorer.html` into the git-ignored `target-codegraph/` folder (S8), which the developer opens locally. The
  prototype moves from `research/code-graph-explorer/` to `scripts/codegraph/`, and the repository path comes from the
  script's location instead of being hardcoded.
- **(b) Staleness check (owner: keep it).** The ~8 min SCIP run is skipped when the indexed tree has not changed since the
  last index. The cached `index.scip` is keyed on the Rust sources as they are in the working tree: the content hashes
  (`git hash-object`) of the `*.rs` and `Cargo.toml` files that `git ls-files -co --exclude-standard` lists, so an
  uncommitted edit counts; plus `Cargo.lock` and the rust-analyzer version. The key is stored beside the index. When only
  the extractor or the page template changed, only `graph.json` and `explorer.html` are rebuilt. `--force` re-indexes.
  Refined by the owner's follow-up: the index key is the tree it was built from, a later `.rs` edit marks the index stale in the
  page instead of re-indexing (the comment metrics, which need no index, are rebuilt at once); `--force` or a missing index builds it.
- **(c) Libraries.** Cytoscape.js is pinned through the repo's package setup per C23 and S1-S10 (S3 to S6), inlined into
  the page, and loaded from no CDN. The new `pnpm-workspace.yaml` goes in the C09 and C19 config columns and the frozen
  install in `scripts/session-setup.sh` (C10). fcose is replaced by Cytoscape's built-in `cose` (dependency rule);
  each level is measured by hand at file level and the layout is kept or replaced per level.

**Recommended (not in the minimal scope):**

- **Pair through `component.json`.** Pair a themed component with its primitive through `component.json` instead of by
  name; the name match misses files such as `preview/src/components/select/mod.rs`.
- **Full radius 2.** Radius 2 is a complete breadth-first search; the prototype only follows the strongest 80/30
  neighbours.

**Optional (not in the minimal scope, not sized):**

- **Item level** from rust-analyzer itself, lazily per component, in one of the two ways under What the tool is. The
  first step is a spike that lists the fields rust-analyzer 1.97 emits.
- **Playwright smoke test** outside `run-gates.sh`: `smoke.js`, resolving `playwright` from `playwright/node_modules`, with
  the scratch path replaced by the build output; it renders every level and runs the S2 request check (every non-`file:`
  request aborted and reported).
- **GraphML or Cypher export**, written with the stdlib.
- **`check-codegraph-deps` gate (S9)**, a proposal only; the owner decides whether it exists.

**Keep in mind (JS volume):** the explorer page is JavaScript in tracked HTML. `check-js-volume.sh` excludes `scripts/`
(like `playwright/` and `dev-docs/`), so the tool lives under `scripts/codegraph/` and the page is dev tooling that
ships nowhere and touches nothing in `primitives/` or `preview/`, which is what the bridge rule in
[`js-interop.md`](../js-interop.md) governs. A top-level `tools/` would be inside the gate's universe and needs a
visible `--update-baseline --allow-raise "row 486 ..."`; the volume rule says it can only shrink, so the excluded path is
the choice.

**Proof (minimal scope):**

- `scripts/codegraph/run.sh` on the current HEAD builds `index.scip`, `graph.json` and `explorer.html` in
  `target-codegraph/` from a fresh index.
- A second run with no source change reports the cached index and skips rust-analyzer; a `.rs` edit, committed or not,
  rebuilds the metrics and the page and marks the index stale; an extractor or page-only change rebuilds graph and page without SCIP; `--force` re-indexes.
- `Cargo.lock` is unchanged after the run; `git status` shows no generated artefact; the page has no `http(s)` URL
  outside the allowlist; opened from `file:` it renders every level with no console errors and no network request,
  checked by hand in a browser.
- The commit that lands the tool passes `scripts/run-gates.sh` as `ALL GREEN` (repo rule). The tool itself is not a gate.
