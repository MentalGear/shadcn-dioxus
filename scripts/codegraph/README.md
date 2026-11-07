# Code graph explorer (backlog row 486)

A developer tool: a self-contained page that shows the codebase as a graph. Pick a component, see what it uses and what
uses it, and set the granularity with a slider (crate, component, file). It ships nowhere: not in the CLI, the registry
or any `component.json`, not in `run-gates.sh`, nothing under `primitives/`, `preview/` or `cli/`.

## Run

```
CARGO_TARGET_DIR=/home/user/tgt/codegraph LANE_SLOTS=1 scripts/codegraph/run.sh [--force] [--blame] [--out DIR]
```

Open `target-codegraph/explorer.html` (git-ignored through `target-*/`) from disk. `CARGO_TARGET_DIR` must be an
absolute, isolated directory; `run.sh` refuses a relative one or the repo-root `target/`. The SCIP run takes the one
build lock (`scripts/lane-lock.sh`), so it may wait behind a gate or a build. The last line it prints names what was
reused and what was rebuilt, for example `index reused, graph reused, page reused; index key 3f2a9c0b11de; 1s`.

## Pipeline and staleness keys

The graph depends on the SCIP index; the comment metrics depend only on file text. They are separate products with
separate keys, so a source edit updates the metrics at once while the (8 minute) index stays as it was and is shown as stale.

| Product | Made by | Key (sha256 of) |
|---|---|---|
| `index.scip` | `rust-analyzer scip` (rustup component) with `CARGO_INCREMENTAL=0`, `CARGO_NET_OFFLINE=true` | taken when it is built: content hash (`git hash-object`) of every `*.rs` and `Cargo.toml` that `git ls-files -co --exclude-standard` lists (an uncommitted edit counts), `Cargo.lock`, the rust-analyzer version. Built when missing or with `--force`; `index.files` keeps the hashes it was built from and `index.info` its HEAD, time and cold seconds |
| `graph.json` | `build_graph.py` (Python stdlib, run as `python3 -I`) | the index key as stored, `build_graph.py`, every `*.ts` and `*.css` (the extractor's static scans) |
| `comments.json` | `build_comments.py` + `comment_metrics.py` (stdlib) | content hash of every `*.rs`, `*.ts`, `*.css`, `*.html`, `*.js`, `*.mjs` and `*.txt`, `comment_metrics.py`, `build_comments.py`, `scripts/comment-scan.py`. No index involved |
| `blame.json` | `build_blame.py` (stdlib) from cached `git blame` output | only with `--blame`: `comments.key`, HEAD, `build_blame.py`; reused while that key matches, else an empty `{"files":{}}` |
| `status.json` | `run.sh` | not keyed; rewritten each run: HEAD, time and key each part was built from, whether the index is stale and which Rust/Cargo files differ from the tree it was built from |
| `explorer.html` | `make_explorer.py` (stdlib) | graph key, comments key, `explorer.template.html`, `make_explorer.py`, the Cytoscape bundle, `status.json` |

Each key is stored beside its product as `<name>.key`. A changed key rebuilds that product and the ones after it; an
unchanged key reuses it. **A `.rs` or `Cargo.toml` edit does not re-index**: the metrics and the page are rebuilt from the
current tree, the index is reused and the page marks it stale. `--force` re-indexes (and so rebuilds everything), and so does
a missing index. The last line `run.sh` prints names what was reused and rebuilt, for example
`index reused, STALE (...), graph reused, comments rebuilt, page rebuilt; index key 3f2a9c0b11de, tree key 9c01b7e4a2d5; 16s`.
The run fails if `Cargo.lock` changed during the SCIP run. rust-analyzer loads the workspace as `cargo check` does, so it runs the
workspace's own build scripts and proc-macros; the tool adds no Cargo dependency.

The page's status line shows the inputs of each part: the graph (the HEAD, key, cold seconds and time of the index), a chip
`index up to date` or `index STALE: N Rust/Cargo files changed since it was built` (hover for the file names; the graph's edges
and item counts are then from the older tree), and the comment metrics (the working tree HEAD, key and time they were built
from, independent of the index).

## The page

One file: the graph JSON and the Cytoscape.js bundle are inlined, so it opens from `file:` with no network request (no
`<script src>`, `<link>`, font or image from a URL). Layout is Cytoscape's built-in `cose` (compound nodes); `fcose` and its
dependencies fail rule D4 and are not used. The ego view is a complete breadth-first search to the chosen radius.

The extractor model is files as nodes (`{id, comp, items}`; `items` is the definitions count) and `[src, dst, weight, kind]` edges, kinds `rs`, `ext`, `ts`, `asset`, `test`. Add per-node fields beside
`items`.

## Comment metrics (M1 to M6)

The tool shows what the comment gates enforce; it enforces nothing (no gate, not in `run-gates.sh`). One pass of the repo's
comment tokenizer (`scripts/comment-scan.py`: `walk_rust`, `code_of`, `embedded_comments`, `history_reason`, the same
functions the history gate calls) runs per file in `comment_metrics.py`, called by `build_comments.py`, over the working tree. The file set is the one the history gate reads (`tracked_files`: tracked, minus its skip lists),
restricted to the graph's nodes. Per file `comments.json` stores raw counts `cm` in the order of `fields`
(`code com doc pubs doced fields fdoced todo hist js jscode words stale dup`), the detail lists `d` (undocumented pub names, TODO and
history-hit lines) and `t:1` for a test file. The page sums numerators and denominators to component and crate, so a ratio
is always the ratio of the sums, never an average of ratios. A Rust file gets every field; a TS, CSS or other scanned file
gets only the TODO and history counts (its code lines are not tokenized, a ratio over it would mean nothing).

| # | Metric | Definition |
|---|---|---|
| M1 | comment ratio | comment lines / code lines. A comment line is a line holding part of a `//`, `///`, `//!` or `/* */` token (blank `//` lines included); a code line is a non-blank line of `code_of(text, keep_strings=True)`, so a line of a string literal is code. JS or CSS comments inside Rust strings are not in the ratio (counted apart as `js`) |
| M2 | doc share | `///` and `//!` comment lines / all comment lines; the rest are `//` working notes |
| M3 | pub doc coverage | documented / (pub items + pub fields). Items: `pub fn struct enum trait const static type mod` (async, unsafe, const prefixes allowed); `pub(crate)` and friends are not matched; `pub mod x;` is skipped because x's own `//!` header documents it. Documented = a `///` line directly above, attributes (also multi-line) and `//` note lines skipped. Lexical, on the comment-free code view: `missing_docs` under clippy, `check-props-docs.sh`, `check-slot-docs.sh` and `check-rustdoc` stay the authority |
| M4 | history-rule hits | comment lines (and embedded-string comment lines) for which `history_reason()` returns a reason, over every scanned node file; the same function and file universe as `check-history-comments.sh`, so on a clean tree both read 0 |
| M5 | TODO / FIXME / HACK / XXX | comment lines with the word (word boundary, comment text only, so the string `"TODO"` is not counted); line numbers in the panel |
| M6 | density outliers | per level (file, component, crate) over entities with at least 100 code lines: dense = M1 above Q3 + 1.5 IQR; sparse = M1 below Q1 - 1.5 IQR computed on ln(M1 + 0.02). The plain sparse fence is negative (M1 is skewed and bounded at 0) and flags nothing, so the log scale is used on that side only. A sparse entity made only of test files is not flagged. Quartiles use linear interpolation |

### Nice-to-have metrics (N1 to N6)

Built in this order, by value over cost: **N1** first (it finds comments that name something that no longer exists, the
one metric that points at a defect, and it needs only file text), **N6** (the item level: it narrows a dense or bare file to
the item, and reuses the M1 to M5 machinery), **N4** (repeated comment blocks: copy-paste a reader can act on), **N2** and **N3**
(trivial counters from the same pass), **N5** last (it needs `git blame`, minutes, so it runs only with `--blame`).
All but N5 are in `comments.json` and come from the one tokenizer pass; none needs the SCIP index.

| # | Metric | Definition and limits |
|---|---|---|
| N1 | stale-looking identifiers | a backticked identifier in a Rust comment that occurs in no code of the repo (code views of every Rust file, comments blanked and string contents kept, plus the text of TS, JS, CSS, HTML and TXT node files). Candidates, never errors. Shape: `foo`, `Foo`, `foo()`, `a::b::C`, `Vec<T>`; names of three or more characters. Skipped: lowerCamelCase names, names ending in `_`, and every name on a line that cites an upstream (Base UI, Radix, Sonner, React, `.mjs`/`.ts`/`.js` files, `window.`, `document.` and the like), because those name another library's API. The test is repo-wide, not per crate, so a name that exists in another crate does not show; the list is mixed (a real leftover such as a removed function sits beside a std or dioxus name the repo never uses) |
| N2 | JS or CSS held in Rust strings | comment lines (`js`) and code lines (`jscode`: source lines of the string with text on them, minus the lines that carry a comment) of a literal the comment scanner reads as JS or CSS; the panel shows `js` of `js + jscode` |
| N3 | comment words | whitespace-separated words in comment lines; the panel shows words per code line; node size can be comment words |
| N4 | repeated comment lines | comment lines inside a run of 3 or more consecutive lines (with at least 45 letters in the 3-line window) whose text, lower-cased with digits and backticked spans collapsed, also occurs in another file; the panel lists the line range and up to three sibling files. Prop descriptions that a themed component repeats from its primitive are repeats by this definition |
| N5 | comment age vs code age | `git blame --line-porcelain` per tracked node file (about 2 minutes for the tree, cached by blob hash in `blame/`), line ages in days; the lens and the table use the mean age of comment lines minus the mean age of code lines (positive = the comments are older than the code), summed over lines so a component is a true mean; the panel shows the per-file medians. A shallow clone dates every older line at its boundary commit. Built only with `run.sh --blame` (an existing `blame.json` is reused while its key matches) |
| N6 | items | every Rust item (`fn struct enum trait union type static mod const impl macro_rules!`, any visibility) with its lexical span: its `///` block through its closing brace or semicolon, found on the comment-and-string-free code view. Per item: code, comment, doc, TODO, history and word counts, `pub` and `documented`. A nested item is counted in its parent too. The Comments tab has Rows: items (worst items, same ranks and filters, outlier fences over items with 20+ code lines), and a file's panel lists its 10 largest items. There are no item nodes or item edges in the graph: the slider stays crate, component, file |

How to read them: a lens recolours the nodes (Metric lens: comment ratio, doc coverage with low = hot, TODOs, history hits;
`dependencies` is the crate colour of before). The ramp is `--heat-lo` to `--heat-hi`; a ratio of 0.48 has no fixed meaning,
so the legend names the scale and the value range of the current view, and a lens that is 0 everywhere (TODOs, history
hits on a clean tree) says so in a banner instead of drawing a flat map. A node with no value (no Rust code, or no pub
items) is grey. A ring marks an M6 outlier at the current level (dense: warm, sparse: cool; off for the dependencies
lens). Node size can be items, code lines or uniform. Select a node: the details panel has a Comments table (value, a bar
with a tick at the repo value, the repo number beside it), the lists behind the counts and a button to the Comments tab.
The status chips at the top carry the repo totals; the history chip turns warm above 0. The **Comments** tab is the worst
list: rows are files, components or crates, ranked by sparsest, densest, undocumented pub items, TODOs or history hits,
filtered by minimum code lines, crate, tests (exclude, include, only), a path substring, only undocumented, only
outliers; a row opens its file or component in the graph. Read M6 next to M2: a prop-heavy file is dense by construction
because every `pub` field has a doc line.

The metrics live in `comments.json` (`{fields, files: {path: {cm, d, t}}}`), not in `graph.json`: the page joins them to the nodes by file
id, so they are rebuilt from the working tree on every key change, whatever state the index is in. A node that is not in `comments.json`
(an external crate, a file of another type) has no metrics. No comment text goes into the page except line numbers and the
matched marker word or history reason.

## Dependencies and security (rule C23, requirements S1-S10)

- Python: stdlib only (`sys os re json collections importlib`), run as `python3 -I`; SCIP protobuf is read by a hand-written varint
  reader. The scripts here use no `subprocess`, `urllib` or socket; the loaded `scripts/comment-scan.py` (stdlib, the repo's
  shared comment extractor) runs `git ls-files` through `subprocess` to list the files it reads, and nothing else.
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
- Comment metrics (2026-11-05, `node scripts/codegraph/smoke.js`): `build_comments.py` takes about 15 to 26 s (the tokenizer
  pass over 1,387 files; the SCIP part is unchanged). `graph.json` is 396 KB. Primitives crate: comment ratio 0.48, doc
  share 80.5 %, pub doc coverage 97.7 %; repo totals 0 history-rule hits (the gate reads 0) and 2 TODO words; file-level M6
  fences sparse < 0.002 and dense > 0.96 over 276 files with 100+ code lines: 22 dense, 4 sparse outliers (densest
  `primitives/src/carousel/content.rs` 4.88, `dropdown_menu.rs` 1.92, `menubar.rs` 1.80, `context_menu.rs` 1.70,
  `carousel/helpers.rs` 1.60; the 4 sparse files have no comment at all).
