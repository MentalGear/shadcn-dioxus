# Claude Code notes for this repository

Short on purpose: each rule has one home, linked here. A rule that belongs in a linked doc goes there, not here; a new
gate never needs an edit to this file.

## Session start

- **Read [`dev-docs/STATUS.md`](./dev-docs/STATUS.md) first** (current round, what is merged and deployed, what runs,
  the next round, owner inputs pending; the main loop keeps it current), then [`dev-docs/README.md`](./dev-docs/README.md)
  for the map of the docs. The backlog (`dev-docs/backlog.md` row N) is one file per row under `dev-docs/backlog/`; the
  index is generated: edit the row file, run `scripts/backlog-index.sh`, commit both.
- A fresh remote container is bootstrapped by the SessionStart hook (`scripts/session-setup.sh`: dx 0.7.9, wasm32, dx
  tools, `pnpm install --frozen-lockfile`, `cargo fetch`; ~8 s). If `dx` is missing, run the script by hand; never
  `cargo install dioxus-cli` (~21 min) or hand-seed the dx tools.

## Delegation

- **Use subagents as much as is possible and safe: this is the default, not an optimization.** Any work that can be
  split into lanes with disjoint file sets should be, and the lanes are dispatched in ONE batch so they run
  concurrently, never one after another. The main loop scopes the lanes, reviews what comes back and runs the checks; it
  does not do the work itself. Serialize only for a real reason, and name it: two lanes would edit the same files, one
  genuinely needs another's output, or a shared resource allows a single holder (the cargo lock, one dev server, one
  port). When a resource is the constraint, still parallelize the *editing* and serialize only the build or test step
  at the end. Before dispatching, tell each lane which files it owns and which it must not touch; before starting a
  batch, capture whatever baseline the work will later be verified against, since it cannot be recovered afterwards.
- **Models** ([`dev-docs/agent-model-fit.md`](./dev-docs/agent-model-fit.md)): Sonnet subagents for delegated work
  (implementation, research, validation runs, debugging); the main loop keeps review, delegation and high-level
  decisions. Haiku fits read-only inventories and audits and small mechanical edits for a lane with a closed file list
  and a verifying gate; it does not fit debugging, primitives or shared code, new gates or specs, or long lanes that wait
  on shared resources. Escalate to a stronger model only after repeated failure or for unusually judgment-heavy work;
  never substitute Opus for a Sonnet or Haiku lane. Delegate debugging to Sonnet too (reproduce, bisect, root-cause,
  propose the construction); the main loop reviews the evidence and decides, and runs no long hands-on debugging loops.
- A **lane** is the work stream (file set, worktree); the **agent** is the Haiku or Sonnet subagent that executes it,
  never "a Haiku lane". Every lane agent gets [`dev-docs/lane-rules.md`](./dev-docs/lane-rules.md) plus its own prompt
  (worktree, build lock, mtime touch, gate procedure, merge queue, web-request rules). The evidence log of lane outcomes
  is closed; lanes add no rows to `agent-model-fit.md`.

## Rules for every change

- **When the same problem shows up more than once, stop patching instances.** Two or more occurrences is a class, not a
  coincidence: deconstruct down to the root cause, then propose the best-fit fix *by construction*, one that makes the
  whole class unable to recur, rather than fixing the instances one at a time. Say plainly which instances the
  construction subsumes and which (if any) it does not.
- **Run every gate before every commit, not just the ones the change "obviously" touches** (all workflows are
  `workflow_dispatch`-only, CI freeze, row 33, so nothing else will): `CARGO_TARGET_DIR=<absolute isolated dir>
  scripts/run-gates.sh`. Green is **`ALL GREEN` with exit 0**; exit 3 is a partial run (`--only`, `--skip`, `--fast`),
  exit 1 a red gate, exit 2 bad usage, and none of those three is a pass. A green quotes only the tree it ran on (the
  summary prints HEAD plus a diff hash). A change that leaves a gate red is not finished.
  - What every gate checks is [`dev-docs/gates.md`](./dev-docs/gates.md), **generated** from the gate list in
    `scripts/run-gates.sh` and the header of each `scripts/check-*.sh`. **A new gate: write its header, register it in
    `run-gates.sh`, run `scripts/gates-doc.py --write`; no other doc is edited** (`check-gates-doc.sh` fails a stale page).
  - Clippy runs in three arms (default, `web`, `server`), each `--all-targets -- -D warnings`, because a plain
    `cargo clippy --workspace` never enables primitives' `web` feature or preview's `server` feature, which are what
    `dx build --ssg` ships; `--all-features` is no substitute (preview's `desktop` needs gtk, and it would hide the
    `not(web)` arm). `test-server`, `check-rustdoc` and `stylelint` are gates too; gates.md says what each one is.
  - A cargo gate whose inputs are byte-identical to a stored PASS is reused, not rerun (row 320;
    `--no-reuse` forces it); the summary names each reused gate and the tree it really ran on.
  - Lanes run `scripts/run-gates.sh --fast`; the main loop runs the one full run per merge batch
    ([`lane-rules.md`](./dev-docs/lane-rules.md), "Gate procedure").
- **Install check:** run `scripts/registry-install-smoke.sh` whenever a `component.json` or a component's
  `component.rs`/`mod.rs`/`style.css` changes, or a component is added or renamed (not in `run-gates.sh`: a cold check
  takes minutes and several GB; green is `registry-install-smoke: OK -- N/N components installed`).
- **Build environment** ([`lane-rules.md`](./dev-docs/lane-rules.md), "Build environment"): every `CARGO_TARGET_DIR` is an
  **absolute, isolated** path, never the shared repo-root `target/` (a relative one breaks `dx build --ssg`, a shared one
  contaminates base paths); build the SSG only through `scripts/build-ssg.sh`; lane cargo runs with `CARGO_INCREMENTAL=0`;
  lanes that share or copy a target touch their sources before every cargo or dx run; worktrees symlink or
  `pnpm install --frozen-lockfile` the `node_modules`, never `pnpm add`/`update` or `npm install`.
- **The fast loop** ([`lane-rules.md`](./dev-docs/lane-rules.md), "The fast loop", owner 2026-10-21): iterate on `dx
  serve` (own warm target copy, own port, `scripts/dev-wait.sh` after each edit), verify with the diff-based
  `cd playwright && pnpm run test:changed` on one `lane-ssg` build, and leave the full suite to the merge batch. Before
  trusting a `dx serve` result, read [`dev-docs/dx-serve-hot-reload.md`](./dev-docs/dx-serve-hot-reload.md): a plain
  HTTP 200 does not prove your edit is served, and a new component directory needs `preview/build.rs` rewritten, not
  touched. Functional specs may run on debug SSG builds; timing specs only on release
  ([`dev-loop.md`](./dev-docs/dev-loop.md), "Running tests").
- **Commits:** use the `commit-guide` skill when it is available; when it is not loaded (e.g. this remote environment),
  say so once and follow the convention already used in this repo: Conventional Commits `type(scope): subject`, a body
  that explains the why and the evidence, the attribution trailers. Deviate only for a valid concern, and state it.
- Never write feedback about the default "commit nag" stop hook in the chat. When the hook flags uncommitted or
  unpushed changes that belong to an in-flight agent or are otherwise intentionally staged for a later commit, continue
  silently; act on it only when the changes are genuinely finished work that should be committed and pushed.
- Whenever a preview deploy is ready (Pages publish succeeded), always post the site link in the chat:
  https://mentalgear.github.io/shadcn-dioxus/ (with the main commit it was built from).
