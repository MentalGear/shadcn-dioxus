# Lane rules (the brief every delegated lane gets)

The durable version of the rules that grew during the 2026-10-14/15 rounds (backlog rows 216, 221, 224). Give every
lane agent this file plus its own prompt. Terminology: a **lane** is a work stream with its own file set and worktree;
an **agent** is the Haiku or Sonnet subagent that executes it (model choice: `agent-model-fit.md`).

Session tooling (paths below default to `LANE_TGT=/home/user/tgt`):
- `scripts/lane-lock.sh [gate|lane] [--label T] -- cmd...` / `--status` -- the one build lock (a FIFO ticket queue
  in front of `LANE_SLOTS` slots, row 221); everything below takes it.
- `scripts/lane-cargo.sh [--label T] -- cargo ...` -- the one command lanes use for cargo: queue, slot target dir,
  source touch, `CARGO_BUILD_JOBS=2`, `CARGO_INCREMENTAL=0`.
- `scripts/lane-ssg.sh <worktree> <lane> [port]` -- debug SSG of a worktree inside the queue, from a fresh
  copy of the warm template, served on `port` (the server starts outside the lock).
- `scripts/gate-at.sh <commit> <log>` -- `scripts/run-gates.sh` on a pinned commit in its own worktree (class `gate`).
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

## Memory and the one build lock (row 221)
- The machine has 4 cores and 15 GB. A `preview` rustc or clippy driver is ~2.5 GB, and ten concurrent agents were
  OOM-killed. EVERY cargo or dx command goes through the queue, never bare `flock` (which wakes a random waiter):
  `scripts/lane-cargo.sh -- cargo clippy ...` (or `scripts/lane-lock.sh [gate|lane] --label T -- cmd...` for anything
  else). It takes a ticket, waits for its turn (order = class, then arrival; a `gate` goes before every waiting
  `lane`, never before a running command), then for a free slot, and runs the command with the lock fds closed in its
  own session. `scripts/lane-lock.sh --status` shows the holders and the queue with ages.
- Slots: `LANE_SLOTS` (default 2). Slot 0 uses the target dirs `$LANE_TGT/check`, `gates`, `ssg-lane`; slot 1 gets
  `<name>-s1` (`LANE_SLOT_SUFFIX`), created lazily by `scripts/lane-slot-target.sh` as a warm copy of the slot-0
  dir when that is idle (cold otherwise) and refused when free disk is below its size + 20% + 2 GB. Slot 1 is
  granted only while `MemAvailable >= LANE_MIN_AVAIL_MB` (default 5000). **While Playwright runs, set
  `LANE_SLOTS=1`**: the first two-slot proof bottomed out at 2.0 GB available with browsers up. Slot 0 also holds
  `$LANE_TGT/ssg.lock` for its whole run, so an old-style `flock ssg.lock cmd` caller and the queue exclude each other.
- A killed waiter or holder never blocks the queue (liveness = the owner still holds its ticket flock). Known gap:
  SIGKILL of the wrapper ALONE frees its slot while the command, a separate session, may run on; stop a build with
  TERM/INT (the whole group stops) or kill the group (`kill -- -<pid>`).
- Two SSG pre-renders must never share a network namespace: `dx build --ssg` fetches every route from the built server
  on the FIXED port 127.0.0.1:9999, and the second of two concurrent builds talks to the first one's server, prints
  `SSG complete` in 0.4 s with no error line and writes no page (95 of 96 routes missing, measured 2026-10-19).
  `lane-ssg.sh` therefore builds inside `scripts/lane-netns.sh` (a private loopback, `BUILD_SSG_NETNS=1`); a hand-run
  `build-ssg.sh` concurrent with another SSG build needs `BUILD_SSG_NETNS=1` too.
- A server or watcher is never started inside the lock (it would be killed with the group anyway); serve through
  `lane-ssg.sh`, or start it outside the queue. Never create another target dir by hand, never `cargo clean`.
- Playwright: `--workers=2` at most, and every run under its own lock: `flock $LANE_TGT/pw.lock npx playwright test ...` (ten lanes' browsers at once would OOM). A crash, "Target closed" or browser-disconnected failure is checked against
  `dmesg | grep -i 'killed process' | tail` before it is believed.

## Shared targets judge freshness by mtime (row 216)
cargo gives a workspace crate the same unit hash in every worktree, so a lane whose files are older than another
tree's artifact silently builds or lints that other tree's code. `scripts/lane-touch.sh <target> [--force]` touches the
sources, unless the target's `.tree-owner` stamp already names this worktree; `lane-cargo.sh`, `lane-ssg.sh`,
`gate-at.sh`, `run-gates.sh` (before its first cargo gate) and `build-ssg.sh` (`BUILD_SSG_TOUCH=always|auto|never`) call
it, and `lane-target.sh create` clears the stamp. After a build, prove your change is in the output (grep the served HTML, CSS or wasm for a string you added).

## Before handback (rows 221, 224; gate step changed 2026-10-18)
1. Merge the current session branch into your lane branch (`git merge <session branch>`, no rebase). New gates may have
   landed since you branched; on a `preview/src/components/mod.rs` conflict keep both sides' variants on every line.
2. Push your branch (rule 1 of "Nothing lives only in the container"), then run the FAST gates without the lock and quote their summary line (owner decision 2026-10-18, see "Gate
   procedure" below): `CARGO_TARGET_DIR=$LANE_TGT/check scripts/run-gates.sh --skip
   '^(clippy-|test$|check-primitives-features$|check-attr-spread-collision$)'` -- every listed gate PASS, exit 3
   (partial by design). That is every static `check-*.sh`, fmt and stylelint, in seconds. Do NOT run the full
   `run-gates.sh`: the main loop runs it once on the merged tree.
3. Run your own specs plus the existing specs of everything you touched on a `lane-ssg` build that you verified
   contains your change, with ONE command: `cd playwright && PLAYWRIGHT_BASE_URL=http://127.0.0.1:<port> flock
   /home/user/tgt/pw.lock npm run test:changed -- --workers=2` (`tools/test-changed.sh`: runs exactly the specs
   `tools/select.mjs` maps your diff to, on `ssg.local.config.ts`; `CHANGED_RANGE=A..B` for another range,
   `CHANGED_MATRIX=1` to add the route-matrix specs; "no spec covers the diff" means docs/gates/tooling only).
   Quote its summary line in your report. Do not pick the specs by hand: `node playwright/tools/select.mjs --explain` lists the specs
   that cover your diff (default range `origin/main...HEAD` plus uncommitted files; `--range A..B`, `--files f...`).
   Map: a component directory -> the specs about that component (file named for it, or naming its page); a primitive
   -> the specs of every component that uses it, directly or through another primitive (more than 30 components =
   shared plumbing, the broad set is added); theme CSS, `main.rs`, `eager_head.rs`, `index.html`, `components/mod.rs` ->
   the broad set (every spec that names more than 12 components, i.e. the route and subject matrices
   `hydration-parity`, `dx-class-migration`, `keyboard-matrix`, ..., plus the cheap cross-cutting guards listed in
   `BROAD_EXTRA`); a changed spec -> itself; a changed helper -> the specs importing it; `fixtures.ts`, `base-url.ts`
   or a config -> every spec. A single-component change leaves the matrix specs out (add `--matrix`, or run them with
   `--grep <component>`). Run the list with
   `PW_SPECS_FILE=<file> npx playwright test --config=suite.local.config.ts` (`select.mjs > <file>`) under
   `flock $LANE_TGT/pw.lock`. The full suite is the main loop's, run as duration-balanced chunks
   (`playwright/tools/run-suite.sh`, "Running the suite" below). A change to shared plumbing (a primitive, `merge_attributes`, the theme, a component other
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
2. **The lock is first-come, first-served, with merge gates first.** `scripts/lane-lock.sh` is a FIFO ticket queue
   (row 221): order is (class, arrival), `gate-at.sh` queues as class `gate` and so goes before every waiting lane
   build; bare `flock` woke a random waiter. Lanes still take the lock only for a build they need now (never "just
   in case"), through `scripts/lane-cargo.sh` / `lane-ssg.sh`.
3. **Batch size follows the lock's throughput: 4-5 lanes per batch.** One lock finishes about three full gate runs
   or six lane builds an hour; more lanes than that do not finish sooner, they wait longer (and hold more disk and
   memory). Split a larger round into consecutive batches with disjoint files; the next batch's editing can start
   while the previous batch's merge gates run.

## Running the suite (row 152a, measured 2026-10-19)
Facts, all from one debug SSG build of one tree on this 4-core / 15 GB box shared with other lanes' cargo builds
(`playwright/durations.json` is the per-file table, `meta.release_total_seconds` the release comparison):
- The full suite is 3,752 tests in 183 files (+68 release-only timing tests): 14,168 s of test time, i.e. **121 min** at 2
  workers on a debug build, **82 min** on a release build of the same tree (-32 %; 3,820 tests, 9,776 s). The cost is
  total work, not a few giants: `all-routes` (517 tests, 16 %), `first-paint-styles` (8 %), `layout-stability` (6 %),
  `popup-style-isolation` (4 %) and `carousel` (4 %) are 38 % of it; axe-titled tests are 16 %; the median test is
  2.7 s (debug) / 1.5 s (release), 101 tests over 10 s are 21 % of the time. The 298 fixed `waitForTimeout` calls sum to 227 s of literal
  milliseconds (under 2 % even if every one ran once) and most are "stays closed for N ms" negatives, so they were left alone.
- **Release vs debug**: release runs the suite 1.47x faster and also runs the timing specs (`main-thread`,
  `scroll-main-thread`), but a cold release build took 37-50 min here (9 min for a debug build from the warm template).
  Use release for the main loop's one full run per merge batch (saves ~40 min of `pw.lock` per run and makes the timing
  specs valid), not for a lane's own spec runs.
- **Workers**: 2 is the lane cap. Measured Chromium RSS: 2 workers peak 3.8 GB (mean 1.7 GB); 3 workers peak 4.3 GB (p95
  3.2 GB), lowest `MemAvailable` 4.4 GB beside other lanes' builds, so 3 fits next to one cargo build. 3 workers ran the
  suite 1.4x faster (7,264 s -> ~5,060 s inside the lock, with the balanced chunks and the 404 fix below) but added two
  load flakes (`layout-stability` dark and `first-paint-styles` `/component/empty/`, both green alone at 2 and 3 workers).
  So: 3 workers for the main loop's full run when no build is queued; lanes stay at 2.
- **Chunks, not one run**: one run holds `pw.lock` for 1.5-2 h and every other lane's specs wait behind it (they did).
  `playwright/tools/run-suite.sh --out DIR --chunks N --workers W` cuts the specs with `shard.mjs` (longest-first onto
  the lightest shard, weights from `durations.json`: the 6 chunks came out at 728-968 s each, the 8 count-balanced
  chunks of the baseline at 618-1180 s) and takes the lock once per chunk, so lanes interleave. Playwright's own
  `--shard` balances by test count and was not used. Refresh the table after a full run:
  `node playwright/tools/durations.mjs DIR/chunk-*.json --write playwright/durations.json` (`--merge` for a partial run;
  a new spec needs an entry or `scripts/check-slow-spec-budget.sh` fails).
- **Not-found routes**: `python3 -m http.server` answers an unknown URL with a bare 404, so `theme-picker-phone`'s two
  not-found routes (4 tests) hung to the 90 s timeout (6 min of the 121 and four reds). `scripts/serve-spa.py` serves
  `404.html` with status 404 as GitHub Pages does, `scripts/emit-404.sh` now also accepts a no-base-path build, and
  `lane-ssg.sh` emits the page and serves through it. Serving a different build by hand: `scripts/emit-404.sh <public>`
  then `python3 scripts/serve-spa.py <public> <port>`.
- **External requests**: none to block. Visiting all 95 prerendered routes in a fresh Chromium makes exactly two
  third-party requests, the deliberate `httpbin.org/delay/3600` and `invalid-url.example` of the avatar demos that
  `avatar-loading.spec.ts` asserts on; the fonts are self-hosted.
- Where the rest of the time is (not changed here; follow-ups for the owners of those files): `all-routes` loads each
  route x scheme three times (axe, 390 px overflow, forced-colors) as three tests, `first-paint-styles` and the route
  matrices (`touch-*`, `main-thread`, `dx-class-migration`) each re-navigate every route; a worker-scoped page or one
  visit-each-route fixture (`fixtures.ts`) would cut two of the three hydrations per route in `all-routes` (a navigation-plus-hydrate is ~1.8 s on debug).

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
