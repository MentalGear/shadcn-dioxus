# Lane rules (the brief every delegated lane gets)

The durable version of the rules that grew during the 2026-10-14/15 rounds (backlog rows 216, 221, 224). Give every
lane agent this file plus its own prompt. Terminology: a **lane** is a work stream with its own file set and worktree;
an **agent** is the Haiku or Sonnet subagent that executes it (model choice: `agent-model-fit.md`).

Session tooling (paths below default to `LANE_TGT=/home/user/tgt`):
- `scripts/lane-ssg.sh <worktree> <lane> [port]` -- debug SSG of a worktree under the global build lock, from a fresh
  copy of the warm template, served on `port`.
- `scripts/gate-at.sh <commit> <log>` -- `scripts/run-gates.sh` on a pinned commit in its own worktree.
- Setup for a new container: `CARGO_TARGET_DIR=$LANE_TGT/template scripts/build-ssg.sh debug` once (the warm
  template), `mkdir -p $LANE_TGT`.

## Worktree and files
- Work ONLY in your own git worktree, on its own branch (`lane/<name>`). Never touch the main checkout or another
  lane's worktree. Do not push. Symlink `playwright/node_modules` and `preview/node_modules` from the main checkout
  (never `npm install`).
- Edit ONLY the files your prompt says you own. If you must touch another file, stop and say so in the report.
- Commit with the default git identity (never `-c user.email`, `GIT_AUTHOR_*`/`GIT_COMMITTER_*` or a worktree config
  identity: GitHub shows any other committer as Unverified). Commit in your worktree: Conventional Commits `type(scope): subject`, a body with the why and the evidence, and the
  attribution trailers your harness gives you.

## Memory and the one global lock (row 221)
- The machine has 4 cores and 15 GB. A `preview` rustc or clippy driver is ~2.5 GB, and ten concurrent agents were
  OOM-killed. EVERY cargo or dx command takes the one global lock and limits jobs:
  `flock $LANE_TGT/ssg.lock env CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=$LANE_TGT/check CARGO_INCREMENTAL=0 cargo ...`
  (`scripts/lane-ssg.sh` holds the same lock for its build). Never start a long-lived process (a server, a watcher)
  inside `flock ... cmd`: it inherits the locked file and holds the lock for every lane after its parent exits. Serve
  through `lane-ssg.sh`, or start the server outside the lock with `3>&- 9>&-`. Never create another target dir, never `cargo clean`.
- Playwright: `--workers=2` at most, and every run under its own lock: `flock $LANE_TGT/pw.lock npx playwright test ...` (ten lanes' browsers at once would OOM). A crash, "Target closed" or browser-disconnected failure is checked against
  `dmesg | grep -i 'killed process' | tail` before it is believed.

## Shared targets judge freshness by mtime (row 216)
cargo gives a workspace crate the same unit hash in every worktree, so a lane whose files are older than another
tree's artifact silently builds or lints that other tree's code. Touch your sources INSIDE the lock before every cargo
or dx run: `find preview/src preview/assets preview/build.rs primitives/src dioxus-attributes/src -type f -exec touch {} +`.
After a build, prove your change is in the output (grep the served HTML, CSS or wasm for a string you added).

## Before handback (rows 221, 224; gate step changed 2026-10-18)
1. Merge the current session branch into your lane branch (`git merge <session branch>`, no rebase). New gates may have
   landed since you branched; on a `preview/src/components/mod.rs` conflict keep both sides' variants on every line.
2. Push your branch (rule 1 of "Nothing lives only in the container"), then run the FAST gates without the lock and quote their summary line (owner decision 2026-10-18, see "Gate
   procedure" below): `CARGO_TARGET_DIR=$LANE_TGT/check scripts/run-gates.sh --skip
   '^(clippy-|test$|check-primitives-features$|check-attr-spread-collision$)'` -- every listed gate PASS, exit 3
   (partial by design). That is every static `check-*.sh`, fmt and stylelint, in seconds. Do NOT run the full
   `run-gates.sh`: the main loop runs it once on the merged tree.
3. Run your own specs plus the existing specs of everything you touched on a `lane-ssg` build that you verified
   contains your change. A change to shared plumbing (a primitive, `merge_attributes`, the theme, a component other
   components compose such as Button) also needs the specs of its consumers: each lane green alone proves nothing about
   the merge (row 224).
4. Wait in the foreground for queued builds (long timeouts) instead of handing back early.

## Nothing lives only in the container (owner decision 2026-10-19)
The container can come back as a FRESH CLONE (2026-10-19: `/home/user/tgt`, `/home/user/wt`, every worktree, every
unpushed lane commit and every agent transcript were gone; lost were a finished lane, three half-done lanes and the
main loop's bookkeeping). So:
1. **Push your lane branch after every commit**: `git push -u origin lane/<name>` (WIP commits included -- commit
   before any long build or test run so nothing is only in the working tree). The main loop deletes `lane/*` from
   origin after the merge. Never push anywhere else.
2. Briefs, rules and procedures live in the repo (this file, the plan docs), never only under `/home/user/tgt`;
   scratch output there is disposable and must be reproducible.
3. The main loop backs up its own unpushed (not yet gated) commits the same way, on `lane/main-loop`.

## Gate procedure (owner decision 2026-10-18; why: `dev-docs/backlog.md` row 221)
One exclusive build lock serialises every compile, a full `run-gates.sh` holds it for 15-21 min, and a round of ten
lanes that each ran it (plus reruns after a red) queued about four hours of lock time. Three rules:
1. **Lanes run the fast gates; the full gates run once per merge batch.** A lane's compile is proven by its own
   `lane-ssg` build and its specs; the main loop merges a batch of lanes and runs the ONE full `run-gates.sh`
   (three clippy arms, `cargo test`, the cargo-building checks) on the merged tree through `scripts/gate-at.sh`,
   fixes small reds itself (a clippy nit, a format line) and sends a real red back to its lane. Nothing is pushed
   without that full green on the exact commit pushed. The late-landing-gate risk the old rule covered (row 224) is
   caught by the merged-tree run, and by each lane merging the session branch before its fast gates.
2. **The lock is first-come, first-served, with merge gates first.** `flock` hands the lock to a random waiter, so a
   lane ready to hand back could lose to one mid-work for an hour. Until a ticket queue replaces bare `flock` (row
   221), the main loop starts merge-batch gates only when no lane build is queued behind them, and lanes take the
   lock only for a build they need now (never "just in case").
3. **Batch size follows the lock's throughput: 4-5 lanes per batch.** One lock finishes about three full gate runs
   or six lane builds an hour; more lanes than that do not finish sooner, they wait longer (and hold more disk and
   memory). Split a larger round into consecutive batches with disjoint files; the next batch's editing can start
   while the previous batch's merge gates run.

## Conventions the gates enforce
Listeners in eval JS through `primitives/src/js_listener.rs` `listen`; passive scroll listeners; motion through
`use_motion` / `use_interval_while`; no `transform` in component `@keyframes`; state recipes and focus rings from the
theme tokens; destructive text through `--dx-destructive-text`; no cross-component property overrides (compose through
slot variables); every `:hover` inside `@media (hover: hover) and (pointer: fine)`; descendant selectors on generic
elements end with `:not(:where([popover], dialog) *)`; overlays bounded in specs with `fixedViewportBox(page)`; demo
controls located by role and position inside the demo frame, never by demo copy (row 220); geometry measured only at
rest (row 222).

## Web requests
Never send the user's email, name or any account identifier to an external service (User-Agent, headers, query
strings, payloads). Use a generic User-Agent such as `shadcn-dioxus-research`.

## Report (<= 40 lines)
What changed (files), the root cause or construction and which class it removes, the commands run with their results
(pass/fail counts), commit hashes, and anything left open or any file outside your set you needed.
