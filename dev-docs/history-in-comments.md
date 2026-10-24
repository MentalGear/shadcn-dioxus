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

Two gates, one comment extractor (`scripts/comment-scan.py`: Rust `//` `///` `//!` `/* */`, CSS `/* */`, TS/JS (regex
literals skipped), Python `#` and docstrings, shell `#`; a marker inside a string is not a comment, a Python triple-quoted
string that is not a docstring is data, and a shell heredoc body is data unless it is code: an interpreter on the command
line, a `PY`/`JS` delimiter or a `> file.py` target; the one exception to "a marker inside a string is not a comment" is
JS or CSS held in a Rust string literal, whose own comments are read, below). Both are in `scripts/run-gates.sh` and run their own `--self-test`.

- **`scripts/check-history-comments.sh`** counts, per file, the comment lines that carry a 2026-08+ date (ISO,
  numeric `02/03/2026`, or a month name in a dating frame: `since March`, `as of October`, `in October 2026`; a bare
  `May 2026` or `in November` is calendar content and stays), a 7-40 char commit SHA, or a phrase (`used to` including
  `which used to`, `previously` and `originally` unless they follow a determiner like `the previously focused element`,
  `before/after the fix`, `formerly`, `the old engine/test/way`, `was changed/renamed/removed/added` unless a `by <agent>`
  follows, `we now`, `now uses/reads`, `no longer X` unless a condition word (`that`, `once`, `when`, `is`) precedes it,
  `since removed`, `back when`, `before/since/until row N`, `row N landed`, `this round`, `round5`, `batch 13`,
  `lane/p5tabs`, `the STYLE-DELIVERY lane`, `found twice`, `the first version`, `an earlier note`, and the retold-work forms of round 2: a lane or session told in the code (`this
  lane`, `this session`, `this fix`, `the task brief`, `$S/...`), how a finding was made (`confirmed live`, `found by
  execution`, `red-then-green`, `pre-fix`, `without the fix`), a status header (`STATUS AT WRITE TIME`, `UPDATE (`), past
  framing (`exactly as before`, `renamed from`, `the original asserted`, `historical defect`, `before Phase 1.1`) and who
  asked (`owner report`, `bug report`, `PR #47`), plus a month-only date `2026-08`). The counts are in
  `scripts/check-history-comments.baseline.tsv` (`path<TAB>count`). The gate **fails** when a file's count rises, when a
  new file has a non-zero count, and when a count **fell** while the baseline still allows the old number (run
  `scripts/check-history-comments.sh --update-baseline` to lock the drop in). Shrink-only: `--update-baseline` refuses
  to raise a count unless `--allow-grow` is given and the commit says why.
- **`scripts/check-comment-refs.sh`** checks that every `row N` in a comment resolves to `dev-docs/backlog/NNN-*.md` or
  `done/NNN-*.md`, and that every cited repo path exists. The dead ones that predate the gate (mostly the old `docs/`
  prefix, which is the deployed site now; the project docs are `dev-docs/`) are in
  `scripts/check-comment-refs.baseline.tsv` (`file<TAB>kind<TAB>ref`), shrink-only the same way.
- **`scripts/check-history-names.sh`** is the same rule for NAMES (row 384): a Playwright `test(`/`test.describe(`/
  `describe(`/`it(`/`test.step(` title, a Rust `#[test]` function name and a spec file name must not carry a date
  2026-08+, `regression` / `(regression guard)`, `round 5` / `batch 13`, `user-reported` / `owner's`, `pre-fix` /
  `before the fix`, `the old behaviour`, `no longer`, `follow-up` or a `(... lane)` tag, nor anything
  `comment-scan.py`'s `history_reason` flags (`used to`, `previously`, `now uses`). A name has no baseline: a hit fails,
  the only exemption is a reasoned `ALLOW` entry in `scripts/history-names.py`, and a stale entry fails. A title says
  what the test asserts in the present tense; `row N` pointers stay (links). `--inventory` lists the hits.

Skipped: `dev-docs/archive/` (history belongs there; code files elsewhere under `dev-docs/` are scanned), `docs/`,
`*.md`, the vendored APG reference tree, generated `primitives/src/js/*.js`, lock files, and the gates' own sources.

### What the scanner reads and what it leaves alone

- **JS and CSS in Rust strings.** A raw or normal string literal that reads as code is scanned for the `//` and `/* */`
  comments inside it (an `eval!` script, a `style { r#"..."# }` sheet) and reported at its Rust line; `\n`, `\"` and a
  `\` line continuation are unescaped first. "Reads as code" is a statement or block shape plus a JS word (`const`,
  `function`, `=>`, `document` ...) for JS, a `{ prop: value }` rule plus a `/* */` for CSS (a CSS-only string yields only
  its block comments). A URL (`https://x`) or a prose string with a `//` in it is data, and so is the text around a comment.
- **A path is a link.** `dev-docs/research/round3-2026-10-05.md` is masked before the rules run: it is neither "this
  round" nor a date. A `$S/...` pointer is not a path, it is a dead scratchpad reference, and is flagged.
- **Provenance is not history.** A full SHA in a pinned-upstream frame (`radix-ui/primitives` commit `<sha>`, `<sha>` of
  `w3c/aria-practices`, `<path> @ <sha>`, `pinned`/`upstream` before it) needs no `history-ok` marker.
- **"no longer" as a state.** `Duplicate no longer matches`, `no longer applies once the element ...`, `(no longer
  `exists`)`, `whose titles no longer match`, `an API its primitive no longer has` describe what the code handles.
  Instrument `used to exclude/filter/skip ...` stays unless `that/which used to`.
- Not adopted, on purpose: bare `pre-existing` (a baseline's entries are pre-existing by design), `is gone` (a state: "the
  dialog is gone"), `carried over from`, `started as`, `was never ...`, a bare `this slice` (a pie slice), `measured live`,
  a bare `lane` (the SSG lane is a test target), `PR #N` after `upstream` (provenance). Each has a must-not-flag sentence in
  the self-test.

## Day to day

- The gate went red on your change: `scripts/check-history-comments.sh` prints the file and its hit lines with the reason
  (`[date]`, `[used to]` ...). Rewrite the line as above. If the count of a file you cleaned fell, run
  `--update-baseline` and commit the TSV with the change.
- `scripts/check-history-comments.sh --inventory` lists every counted line (`file:line  reason  text`), the worklist
  for the sweep that archives, links and folds.
- A line that is deliberately history (a quoted changelog, a date the code needs): end it with
  `history-ok: <reason>`. A dead citation that is deliberate: `ref-ok: <reason>`. Both need a reason a reviewer can judge.
- A false positive (a pattern that fires on a non-history sentence) is a bug in `scripts/comment-scan.py`: tighten the
  pattern and add the sentence to the `MUST_NOT_FLAG` table of the gate's `--self-test` (a missed form goes in
  `MUST_FLAG`, an extractor case in `EXTRACTION_CASES`), do not mark it.
