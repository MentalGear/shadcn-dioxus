# Proposal: comment metrics in the code graph explorer (row 486)

Status: M1 to M6 are built (`comment_metrics.py`, `build_graph.py`, `explorer.template.html`; how to read them is in
`README.md`). The `_walk_rust` and `_embedded_comments` names below are now `walk_rust` and `embedded_comments` in
`scripts/comment-scan.py`. N1 to N6 are not built. Where the build differs from this text (sparse fence on a log scale,
`pub mod x;` skipped by M3) `README.md` says so.

Scope: dev tool only, `scripts/codegraph/`, stdlib Python run as `python3 -I`, no dependency beyond the pinned Cytoscape
(C23 untouched), no cargo build. The tool **shows** what the gates enforce; it never enforces (no gate, not in
`run-gates.sh`). Sample numbers: measured 2026-11-03 at HEAD e71db4db, `primitives/src/` (201 `.rs` files),
script `measure.py` next to this file.

## 1. Where the data comes from (one pass, one tokenizer)

`scripts/comment-scan.py` is the repo's single comment extractor. It is imported (hyphenated name, so via
`importlib.util.spec_from_file_location`), not copied, and its internals are used read-only:

- `_walk_rust(text)` yields comment tokens (with start offset, so `///` / `//!` vs `//` is `text.startswith("///", i)`)
  and string literals; `code_of(text)` blanks comments and strings, giving the code-line count; `_embedded_comments`
  gives comments of JS/CSS held in Rust strings; `history_reason(body)` is the gate's detector verbatim.
- Wish for the row: promote `_walk_rust`, `code_of`, `history_reason` and `_embedded_comments` to a public `scan` API
  (rename-only change, the two gates keep working) so the tool does not depend on underscore names.
- SCIP (`index.scip`, already parsed by `build_graph.py`) supplies symbols per file: definitions, and the set of
  identifiers that exist.
- File to node mapping is the one `build_graph.py` already has (file -> component -> crate), so every per-file number
  rolls up by **summing the numerators and denominators**, never by averaging ratios.

Cost of the whole feature except blame and duplicates: one tokenizer pass over ~1,400 tracked files, about 1 s measured
for the 201 primitives files (Python, single core). Added to `graph.json` as `nodes[i].cm = {...}`, about 150 KB.

## 2. Metrics, ranked

Roll-up rule for every row: file = raw counts; component = sum of its files; crate = sum of components; item = lexical
span of the item (its `///` block plus the comment lines inside its body). Ratios are recomputed from the summed counts.

### Must-have

| # | Metric | Definition | Computed from | Cost | UI | Overlaps a gate |
|---|---|---|---|---|---|---|
| M1 | Comment ratio (lines) | comment lines / code lines; code line = non-blank line of `code_of()` | tokenizer | trivial | lens "comment ratio", heat on node colour; table column; outlier filter | no |
| M2 | Doc share | `///` and `//!` lines / all comment lines (the rest are `//` working notes) | tokenizer | trivial | stacked bar in the panel (doc vs note) | no |
| M3 | Doc coverage of public items | documented pub items / pub items: `pub fn/struct/enum/trait/const/type/mod` plus `pub` fields (props); documented = a `///` block directly above, attributes skipped | lexical scan of the same file text, `code_of` to skip strings; SCIP definition kinds could replace the regex later | low | lens "doc coverage" (low = hot); filter "undocumented only"; panel lists the names | **yes**: `#![warn(missing_docs)]` under clippy `-D warnings`, `check-props-docs.sh`, `check-slot-docs.sh`, `check-rustdoc`. The tool displays the same fact as a heat map; it does not re-enforce |
| M4 | History-rule hits | comment lines for which `history_reason()` returns a reason, by reason (date, sha, phrase, row/lane) | the gate's own function | trivial | lens "history hits"; counter chip in the status line; red outline if > 0 | **yes**: `check-history-comments.sh` (ratchet, baseline). Shown so a dirty tree is visible before the gate run; expected 0 |
| M5 | TODO / FIXME / HACK / XXX | comment lines with the marker (word boundary, comment tokens only, so a string `"TODO"` is not counted) | tokenizer | trivial | lens "TODOs"; panel list with line numbers | no (the repo tracks work in backlog rows, so a non-zero count is itself a finding) |
| M6 | Comment-density outliers | files whose M1 is above Q3 + 1.5 IQR (dense) or below Q1 - 1.5 IQR among files with >= 100 code lines (sparse); tests excluded from the sparse side | derived from M1 | trivial | "Comments" view: worst-files table, sort and filter; outline ring on the node | no |

### Nice-to-have

| # | Metric | Definition | Computed from | Cost | UI | Overlap |
|---|---|---|---|---|---|---|
| N1 | Stale-looking comments | a backticked identifier in a comment that is not defined anywhere in the **same crate** (SCIP symbol names, plus identifiers in that file); reported as candidates, never as errors | SCIP symbols + regex for `` `ident` `` in comment bodies | low (SCIP already parsed) | lens "stale candidates"; panel table with line and the missing name | partial: `check-comment-refs.sh` resolves `row N` and repo paths only, not identifiers |
| N2 | JS-in-Rust-string comments | comment lines inside JS/CSS held in Rust strings (`_embedded_comments`) vs the JS code lines of those strings | tokenizer | trivial | panel row; a second number next to M1 for files with eval sites (carousel: 522 of 2,899) | related to `check-js-volume.sh` (volume only, not comments); row 434 shrinks these |
| N3 | Comment tokens | whitespace-separated words in comments, and per code line (reading load) | tokenizer | trivial | panel row; alternative size for the node | no |
| N4 | Duplicated comment blocks | blocks of >= 3 consecutive comment lines whose normalised text (lowercase, digits and identifiers collapsed) hashes equal across >= 2 files | hash map over blocks | low, O(lines) | lens "duplicates"; panel lists the sibling files; edge overlay "shares comment block" (optional) | no |
| N5 | Comment age vs code age | median days since the last change of comment lines vs code lines, by `git blame --line-porcelain` | git | **high**: blame per file, minutes for 1,400 files; run only for the selected component or behind `--blame`, cached by file blob hash | lens "comment staleness" (old comment next to new code); panel sparkline | no |
| N6 | Item-level numbers | M1 to M5 for one item (needs the optional item level) | item spans from SCIP or the LSP spike | follows row 486 item work | item node colour | follows its parent metrics |

## 3. UI contract (see `ui-sketch.html`)

- A **metric lens** selector next to the level slider recolours nodes by one scalar: dependencies (default, current
  behaviour), comment ratio, doc coverage, TODOs, history hits (later: stale candidates, duplicates). Size stays
  fan-in or code lines (toggle). Legend shows the scale and the value range of the current view, since a ratio of 0.48
  has no fixed meaning.
- Colour scale is sequential for ratios and diverging only for doc coverage around the repo median. A lens whose value is
  0 everywhere (TODOs, history hits today) says so in the legend instead of drawing a flat map.
- The details panel gets a "Comments" table for the selected node: each metric, its value, a bar against the repo
  median, and the rolled-up count of files behind it. Rows link to the file list sorted by that metric.
- A separate **Comments** view (same data, no graph): worst files by lens, filters (level, crate, only tests, min code
  lines, only undocumented pub). It is the answer to "where do I look first".
- The status line keeps the staleness key: index SHA, cold time, up to date or stale (Scope (b)); comment metrics are
  recomputed from the working tree on every `graph.json` build, not from the SCIP index, so they are always current even
  when the index is stale. Only N1 needs the index.

## 4. Measured sample (2026-11-03, HEAD e71db4db, `primitives/src/`, 201 files)

Whole tree: 62,429 code lines, 30,033 comment lines (ratio 0.48), 24,162 of them doc lines (80 %), 684 comment lines
inside Rust strings, **0** TODO/FIXME/HACK, **0** history-rule hits (the gate holds). Pub items 1,295 with 90.6 %
documented, pub fields 1,629 with 97.3 %.

| Module | Code | Comment | Ratio | Pub-item doc | Notes |
|---|---:|---:|---:|---:|---|
| chart | 10,507 | 4,362 | 0.42 | 83.1 % | |
| virtual | 5,467 | 1,288 | 0.24 | 100 % | |
| carousel | 4,149 | 2,899 | 0.70 | 100 % | 522 comment lines in JS strings |
| questionnaire | 3,389 | 437 | 0.13 | 99.0 % | sparse, mostly tests |
| calendar | 3,069 | 1,102 | 0.36 | 100 % | |
| select | 1,870 | 1,190 | 0.64 | 78.3 % | |
| combobox | 1,169 | 359 | 0.31 | 67.3 % | lowest of the big modules |

Densest files (>= 100 code lines): `carousel/content.rs` 4.88, `dropdown_menu.rs` 1.94, `menubar.rs` 1.80.
Sparsest: `listbox.rs` 0.03, `questionnaire/model_tests.rs` 0.06, `selection.rs` 0.06.

Reading of the numbers (inferred, not verified): the regex-based M3 gives 7.8 % for `lib.rs` (5 of 64 `pub mod`/`pub use`
lines), which contradicts the `missing_docs` gate being green, so module declarations need the SCIP definition kind or
a rule to skip `pub mod x;`/`pub use` before M3 is trusted there. Ratios include doc lines for `pub` prop fields,
so a prop-heavy file reads dense by construction; M6 should be read next to M2.

## 5. Recommendation

Ship M1 to M6 in the minimal scope (all one tokenizer pass, about 100 lines of Python plus the lens code); N1, N2, N3
with the item spike; N4 later; N5 only behind `--blame`. Nothing here adds a gate, a dependency, or a network need
(S2 holds), and no comment text is placed in the page beyond the line numbers and short matched words, which keeps
`graph.json` small.
