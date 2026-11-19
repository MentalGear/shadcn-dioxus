# The 2026-11-19 evaluation: ledgers, logs and scripts

The evidence behind [`../repo-evaluation-2026-11-19.md`](../repo-evaluation-2026-11-19.md). [`evidence.md`](./evidence.md)
holds the reproduction recipe of every headline finding; this page explains the data files beside it.

| Path | What it is |
|---|---|
| [`evidence.md`](./evidence.md) | Recipes and recorded results: the CLI deletions, the gate bypasses, the clippy run, the wasm measurement, the soak, the size commands, the scans, the code-analyser run. |
| [`ledgers/`](./ledgers/) | The six ledgers `ledger-a.jsonl` to `ledger-f.jsonl`, committed as they were checked, with the session's scratch paths unchanged. |
| [`scratch/`](./scratch/) | The logs the ledgers' `run` receipts name and the small scripts the recipes use, laid out like the session's scratch directory (`$S`). 93 files, each under 10 KB. |

## What a ledger is

A ledger is one JSON object per line: `id`, `claim`, `verdict` (`CONFIRMED`, `PARTLY`, `REFUTED` or `UNVERIFIABLE`),
`actual` and `receipts`. The five evaluation reports (A code-analyser, B allocation audit, C TigerStyle, D code quality,
E output) were each re-derived by an independent verifier; the ledgers `ledger-a` to `ledger-e` hold the 216 claims
after that pass (146 confirmed, 53 partly, 12 refuted, 5 unverifiable; 8 are marked as opinion), `ledger-f` the 8 facts
of the filing step (which rows already cover a topic). **Where a report and a ledger disagree, the ledger wins**: a
number is used only where a ledger line confirms it, and for a `PARTLY` or `REFUTED` line the number is the ledger's
`actual`. The rule is in [`../../../agent-rules/delegation-and-models.md`](../../../agent-rules/delegation-and-models.md).

A receipt is a `quote` (file, line and the verbatim text), a `count` (a ripgrep pattern, globs, mode and the expected
number) or a `run` (the absolute path of a log whose first line is the command and the rest its unedited output). The
quote and count receipts check against the tree at commit **`0df1782`**, the commit everything was measured on. The
checker is [`scripts/report-receipts.py`](../../../scripts/report-receipts.py) (gate `scripts/check-report-receipts.sh`).

## Running the checker

```sh
git checkout 0df1782                      # or any later tree, then read the drifts below
python3 -I scripts/report-receipts.py dev-docs/research/repo-evaluation-2026-11-19/ledgers/ledger-*.jsonl --repo .
```

With the committed files as they are, every `run` receipt, and every `quote` receipt that points into the scratch
directory, names a path that exists only on the machine that wrote it (`/tmp/claude-0/-home-user-shadcn-dioxus/
2f9af3b0-3995-5f94-a96b-6c0dfde7973a/scratchpad/`, called `$S` below), so those lines come out BROKEN until the paths
are mapped. The checker wants absolute paths for `run`, so a repo-relative path cannot be committed; the mapping is one
prefix. Rewrite a temporary copy:

```sh
R=$(pwd)/dev-docs/research/repo-evaluation-2026-11-19
S=/tmp/claude-0/-home-user-shadcn-dioxus/2f9af3b0-3995-5f94-a96b-6c0dfde7973a/scratchpad
mkdir -p $TMP/ledgers && for f in $R/ledgers/ledger-*.jsonl; do sed "s#$S/#$R/scratch/#g" $f > $TMP/ledgers/$(basename $f); done
python3 -I scripts/report-receipts.py $TMP/ledgers/ledger-*.jsonl --repo .
```

The mapping is `$S/<path>` to `dev-docs/research/repo-evaluation-2026-11-19/scratch/<path>`, for example
`$S/verify-e/soak.js` is `scratch/verify-e/soak.js` and `$S/gate-sse-hostile.log` is `scratch/gate-sse-hostile.log`.
A `run` receipt is not re-run by the checker (it reports `LOGGED`: the log exists and has at least two lines); the logs
are the recorded evidence, and the recipes in `evidence.md` re-create them.

## Receipts that do not resolve from the repository

- `scratch/verify-ac/tiger/TIGER_STYLE.md` and `tiger/site.txt` (the 32 claims of ledger C that quote the
  TigerStyle source): third-party text whose licence was not checked, so it is not committed. Fetch TigerBeetle's
  `docs/TIGER_STYLE.md` and the text of tigerstyle.dev into those two paths to re-check them; the ledger quotes are the
  excerpts the rows rely on.
- `/home/user/code-analyser/README.md`, `build_graph.py`, `build_blame.py` (ledger A): a checkout of
  MentalGear/code-analyser at `374bf4a` beside the main checkout, where `scripts/codegraph.sh` looks for it.
- `/root/.cargo/registry/src/index.crates.io-*/tracing-wasm-0.2.1/src/lib.rs:346` (ledger E-16): the crate source in the
  cargo registry.
- Not kept, for size or kind: the 28 MB `index.scip`, the pickles, the raw 6 MB clippy JSON (the per-lint table is in
  `evidence.md` instead), `rows.json`, `scan2.json`, the 229 KB `rust.txt` text dump, the server log, the aborted soak
  runs, the screenshot, and the scripts that generated the ledgers.

## The known count drifts

Run on the tree that carries this directory, the checker reports three BROKEN lines. None is a wrong finding.

- **B-13** (a count of 196 against 195): the ledger line says all 195 `.expect(` calls take literals and its count
  receipt expects 195. `git grep -o '\.expect(' 0df1782 -- '*.rs'` finds 196 on the measured commit itself, so the
  ledger's count was one low when it was written; the 196th call was not inspected.
- **D-45** (118 gate entries against 117 and 111 `check-*.sh` against 110): the filing step of this session added
  `scripts/check-report-receipts.sh` and its entry in `scripts/run-gates.sh`; at `0df1782` the counts are 117 and 110.
- **D-58** (507 backlog row files against 501): rows 529 to 534 (and later rows) were filed after the measurement; at
  `0df1782` the count is 501.

Any other BROKEN line after the path mapping is a real finding about the ledger, not a drift.

## Scripts

`scratch/` keeps the scripts as they ran, with the session's absolute paths in them (`/home/user/shadcn-dioxus` as the
repository, `/tmp/claude-0/...` for outputs); edit the constants at the top before running one. They are measurement
code, not repository tooling: nothing runs them in a gate, and the history and reference gates read their comments like
any other code file under `dev-docs/`.
