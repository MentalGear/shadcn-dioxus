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
- Commit in your worktree: Conventional Commits `type(scope): subject`, a body with the why and the evidence, and the
  attribution trailers your harness gives you.

## Memory and the one global lock (row 221)
- The machine has 4 cores and 15 GB. A `preview` rustc or clippy driver is ~2.5 GB, and ten concurrent agents were
  OOM-killed. EVERY cargo or dx command takes the one global lock and limits jobs:
  `flock $LANE_TGT/ssg.lock env CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=$LANE_TGT/check CARGO_INCREMENTAL=0 cargo ...`
  (`scripts/lane-ssg.sh` holds the same lock for its build). Never create another target dir, never `cargo clean`.
- Playwright: `--workers=2` at most, and every run under its own lock: `flock $LANE_TGT/pw.lock npx playwright test ...` (ten lanes' browsers at once would OOM). A crash, "Target closed" or browser-disconnected failure is checked against
  `dmesg | grep -i 'killed process' | tail` before it is believed.

## Shared targets judge freshness by mtime (row 216)
cargo gives a workspace crate the same unit hash in every worktree, so a lane whose files are older than another
tree's artifact silently builds or lints that other tree's code. Touch your sources INSIDE the lock before every cargo
or dx run: `find preview/src preview/assets preview/build.rs primitives/src dioxus-attributes/src -type f -exec touch {} +`.
After a build, prove your change is in the output (grep the served HTML, CSS or wasm for a string you added).

## Before handback (rows 221, 224)
1. Merge the current session branch into your lane branch (`git merge <session branch>`, no rebase). New gates may have
   landed since you branched; on a `preview/src/components/mod.rs` conflict keep both sides' variants on every line.
2. Run the FULL `scripts/run-gates.sh` (fmt, three clippy arms, `cargo test --workspace`, stylelint, every
   `check-*.sh`) under the lock, and quote its `ALL GREEN` line. A partial run is not a pass.
3. Run your own specs plus the existing specs of everything you touched on a `lane-ssg` build that you verified
   contains your change. A change to shared plumbing (a primitive, `merge_attributes`, the theme, a component other
   components compose such as Button) also needs the specs of its consumers: each lane green alone proves nothing about
   the merge (row 224).
4. Wait in the foreground for queued builds (long timeouts) instead of handing back early.

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
