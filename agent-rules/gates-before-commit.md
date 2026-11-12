# Gates before every commit

Read this when you are about to commit, report a change as finished, or add or change a gate.

- **Run every gate before every commit, not just the ones the change "obviously" touches** (all workflows are
  `workflow_dispatch`-only, CI freeze, row 33, so nothing else will): `CARGO_TARGET_DIR=<absolute isolated dir>
  scripts/run-gates.sh`.
- Green is **`ALL GREEN` with exit 0**; exit 3 is a partial run (`--only`, `--skip`, `--fast`), exit 1 a red gate,
  exit 2 bad usage, and none of those three is a pass. A green quotes only the tree it ran on (the summary prints HEAD
  plus a diff hash). A change that leaves a gate red is not finished.
- If you need to know what a gate checks, read [`dev-docs/gates.md`](../dev-docs/gates.md): it is **generated** from the
  gate list in `scripts/run-gates.sh` and the header of each `scripts/check-*.sh`.
- If you add a gate: write its header, register it in `run-gates.sh`, run `scripts/gates-doc.py --write`; no other doc
  is edited (`check-gates-doc.sh` fails a stale page).
- Clippy runs in three arms (default, `web`, `server`), each `--all-targets -- -D warnings`, because a plain
  `cargo clippy --workspace` never enables primitives' `web` feature or preview's `server` feature, which are what
  `dx build --ssg` ships; `--all-features` is no substitute (preview's `desktop` needs gtk, and it would hide the
  `not(web)` arm). `test-server`, `check-rustdoc` and `stylelint` are gates too; gates.md says what each one is.
- A cargo gate whose inputs are byte-identical to a stored PASS is reused, not rerun (row 320; `--no-reuse` forces it);
  the summary names each reused gate and the tree it really ran on.
- Lanes run `scripts/run-gates.sh --fast`; the main loop runs the one full run per merge batch
  ([`lane-rules.md`](../dev-docs/lane-rules.md), "Gate procedure").
