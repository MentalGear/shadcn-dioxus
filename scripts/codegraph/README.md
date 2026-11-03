# Code graph explorer (backlog row 486)

A developer tool: a self-contained page that shows the codebase as a graph. Pick a component, see what it uses and what
uses it, and set the granularity with a slider (crate, component, file). It ships nowhere: not in the CLI, the registry
or any `component.json`, not in `run-gates.sh`, nothing under `primitives/`, `preview/` or `cli/`.

## Run

```
CARGO_TARGET_DIR=/home/user/tgt/codegraph LANE_SLOTS=1 scripts/codegraph/run.sh [--force] [--out DIR]
```

Open `target-codegraph/explorer.html` (git-ignored through `target-*/`) from disk. `CARGO_TARGET_DIR` must be an
absolute, isolated directory; `run.sh` refuses a relative one or the repo-root `target/`. The SCIP run takes the one
build lock (`scripts/lane-lock.sh`), so it may wait behind a gate or a build. The last line it prints names what was
reused and what was rebuilt, for example `index reused, graph reused, page reused; index key 3f2a9c0b11de; 1s`.

## Pipeline and staleness key

| Product | Made by | Key (sha256 of) |
|---|---|---|
| `index.scip` | `rust-analyzer scip` (rustup component) with `CARGO_INCREMENTAL=0`, `CARGO_NET_OFFLINE=true` | content hash (`git hash-object`) of every `*.rs` and `Cargo.toml` that `git ls-files -co --exclude-standard` lists (an uncommitted edit counts), `Cargo.lock`, the rust-analyzer version |
| `graph.json` | `build_graph.py` (Python stdlib, run as `python3 -I`) | index key, `build_graph.py`, every `*.ts` and `*.css` (the extractor's static scans) |
| `explorer.html` | `make_explorer.py` (stdlib) | graph key, `explorer.template.html`, `make_explorer.py`, the Cytoscape bundle |

Each key is stored beside its product as `<name>.key`. A changed key rebuilds that product and the ones after it; an
unchanged key reuses it. A source edit re-indexes; an extractor or page change rebuilds only the graph or the page.
`--force` re-indexes. The run fails if `Cargo.lock` changed during the SCIP run. rust-analyzer loads the workspace as
`cargo check` does, so it runs the workspace's own build scripts and proc-macros; the tool adds no Cargo dependency.

## The page

One file: the graph JSON and the Cytoscape.js bundle are inlined, so it opens from `file:` with no network request (no
`<script src>`, `<link>`, font or image from a URL). Layout is Cytoscape's built-in `cose` (compound nodes); `fcose` and its
dependencies fail rule D4 and are not used. The ego view is a complete breadth-first search to the chosen radius.

The extractor model is files as nodes (`{id, comp, items}`; `items` is the per-node metrics slot) and
`[src, dst, weight, kind]` edges, kinds `rs`, `ext`, `ts`, `asset`, `test`. Add per-node fields beside `items`.

## Dependencies and security (rule C23, requirements S1-S10)

- Python: stdlib only (`sys os re json collections`), run as `python3 -I`; SCIP protobuf is read by a hand-written varint
  reader. No `subprocess`, `urllib` or socket use.
- Cytoscape.js `3.34.3`, delivered through the pnpm folder in this directory: `package.json` pins it exactly,
  `pnpm-workspace.yaml` carries `minimumReleaseAge: 43200` and `onlyBuiltDependencies: []`, `pnpm-lock.yaml` records the
  sha512 integrity (`yfYGhRcG...kSeaQ==`). Install only with `pnpm install --frozen-lockfile` (`run.sh` does it when
  `node_modules` is missing, `scripts/session-setup.sh` does it too). Never `pnpm add` or `update`.
- C23 evidence, read 2026-11-03: D1 3.34.3 published 2026-09-07 (57 days; the newest, 3.34.4 of 2026-10-15, is younger than
  30 days and pnpm would refuse it); D2 first published 2012-10-18; D3 24,704,988 npm downloads in the week 2026-10-26 to
  2026-11-01 (stand-in for stars, which the session could not read: unverified); D4 latest release 2026-10-15, not
  deprecated; D5 MIT, no dependencies, no install script (`scripts` has no `preinstall`, `install` or `postinstall`).
- Generated files (`index.scip`, `graph.json`, `explorer.html`) stay under the git-ignored output dir; `run.sh` refuses
  an output dir that is not ignored or holds tracked files.
- The page template, the scripts and the recipe contain no `http(s)://` URL; the only ones in `explorer.html` are licence-comment text inside the unmodified Cytoscape bundle (never fetched). Check by hand: `grep -n 'https\?://' scripts/codegraph/*.py scripts/codegraph/run.sh scripts/codegraph/explorer.template.html`; `smoke.js` aborts and fails on any non-`file:` request.

## Measured (rust-analyzer 1.97.0, 2026-11-03, 4 cores)

- Cold run: SCIP 684 s (26.3 MB, 900 documents), extractor and page 2 s. 1,383 files, 254 components, 10,792 edges.
- Warm run, nothing changed: 1 s, all three products reused. Template-only change: 0 s, page rebuilt. Extractor-only
  change: 2 s, graph and page rebuilt, no SCIP. A new untracked `.rs` file changes the index key and starts the SCIP run.
- Page, opened from `file:` at 1280 px (`node scripts/codegraph/smoke.js`, playwright from `playwright/node_modules`):
  no console error, no non-`file:` request (all aborted and reported), picker, crate, component and file levels render;
  file level radius 1 for `button` is 154 nodes and radius 2 is 465 nodes. `cose` (with boxes) is the layout up to 250
  nodes; above that the ego view is a concentric ring layout by hop distance without boxes, because `cose` gives a hairball
  there. Raise the minimum edge weight to thin a big view.
