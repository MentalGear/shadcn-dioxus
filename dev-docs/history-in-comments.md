# History in comments: the rule and its ratchet

**The codebase describes only its current state.** A comment says what the code is and why it is that way. What it
used to be, which bug found it, who asked and on which day is history, and history lives in the backlog row (and the
archive) and is linked, not retold. Owner goal, backlog [row 266](./backlog/266-repo-cleanup-archive-historical-dev-docs.md).

Why: a retold history goes stale the day the next fix lands, is the bulk of what a reader has to skip, and gets copied
into the next comment. A link stays true.

## Write it this way

| Instead of | Write |
|---|---|
| `// used to be 3px; changed to 2px on 2026-10-19 after the clipping report` | `// 2px: the ring must not clip inside an overflow:hidden card (backlog row 312)` |
| `// previously this took the first child; now it reads the registered item` | `// reads the registered item, so a wrapper element cannot be mistaken for the first child` |
| `// fixed in 98e7ca1b` | `// (backlog row 245)` -- the row carries the commit |
| `// no longer uses the shared timer` | say what it uses; the row says what it stopped using |

Three moves, in this order: **archive** the story into the backlog row or a `dev-docs/research/` note, **link** it
(`dev-docs/backlog.md row N`, or a path), **fold** the lasting part into the comment as a present-tense reason. A comment
that only retells has nothing to fold: delete it.

What stays: the reason for a non-obvious choice (it is present tense), a pinned upstream revision a file was ported from
(`aria-practices 7e4034b`, `shadcn-ui/ui@295a1f1` are provenance, the gate leaves them alone), and a quote of an
upstream changelog marked `history-ok: <reason>`.

## How the ratchet works

Two gates, one comment extractor (`scripts/comment-scan.py`: Rust `//` `///` `//!` `/* */`, CSS `/* */`, TS/JS, shell `#`;
a marker inside a string is not a comment). Both are in `scripts/run-gates.sh` and run their own `--self-test`.

- **`scripts/check-history-comments.sh`** counts, per file, the comment lines that carry a 2026-08+ date, a 7-40 char
  commit SHA, or a phrase (`used to`, `previously`, `before the fix`, `formerly`, `old behaviour`, `was changed/renamed`,
  `we now`, `now uses/reads/lives/owns`, `no longer uses/needs/renders`, `since removed`). The counts are in
  `scripts/check-history-comments.baseline.tsv` (`path<TAB>count`). The gate **fails** when a file's count rises, when a
  new file has a non-zero count, and when a count **fell** while the baseline still allows the old number (run
  `scripts/check-history-comments.sh --update-baseline` to lock the drop in). Shrink-only: `--update-baseline` refuses
  to raise a count unless `--allow-grow` is given and the commit says why.
- **`scripts/check-comment-refs.sh`** checks that every `row N` in a comment resolves to `dev-docs/backlog/NNN-*.md` or
  `done/NNN-*.md`, and that every cited repo path exists. The dead ones that predate the gate (mostly the old `docs/`
  prefix, which is the deployed site now; the project docs are `dev-docs/`) are in
  `scripts/check-comment-refs.baseline.tsv` (`file<TAB>kind<TAB>ref`), shrink-only the same way.

Skipped: `dev-docs/` (history belongs there), `docs/`, `*.md`, the vendored APG reference tree, generated
`primitives/src/js/*.js`, lock files, and the gates' own sources. Not seen: CSS or JS that sits inside a Rust string, and
Python files.

## Day to day

- The gate went red on your change: `scripts/check-history-comments.sh` prints the file and its hit lines with the reason
  (`[date]`, `[used to]` ...). Rewrite the line as above. If the count of a file you cleaned fell, run
  `--update-baseline` and commit the TSV with the change.
- `scripts/check-history-comments.sh --inventory` lists every counted line (`file:line  reason  text`), the worklist
  for the sweep that archives, links and folds.
- A line that is deliberately history (a quoted changelog, a date the code needs): end it with
  `history-ok: <reason>`. A dead citation that is deliberate: `ref-ok: <reason>`. Both need a reason a reviewer can judge.
- A false positive (a pattern that fires on a non-history sentence) is a bug in `scripts/comment-scan.py`: tighten the
  pattern and add the sentence to the gate's `--self-test`, do not mark it.
