# Round 1 lane briefs (2026-10-19)

Self-contained briefs for round 1 of [`backlog-order-2026-10-18.md`](../../research/backlog-order-2026-10-18.md) plus the round-0
bookkeeping, kept in the repo because a container reset destroyed the first run of every one of them (backlog row
250). A relaunched agent reads its section here; nothing depends on session scratch.

## Common to every lane

- Repo `/home/user/shadcn-dioxus`; your worktree `/home/user/wt/<lane>` on branch `lane/<lane>`. If the worktree is
  missing, create it: `git -C /home/user/shadcn-dioxus fetch origin && git -C /home/user/shadcn-dioxus worktree add
  /home/user/wt/<lane> origin/lane/<lane>` if that branch exists on origin, else from `origin/claude/zealous-davinci-08se9a`
  or `origin/lane/main-loop`, whichever is newer; then symlink `playwright/node_modules` and `preview/node_modules` from
  the main checkout. If it exists with uncommitted changes, commit them as a WIP commit first.
- Read `CLAUDE.md` and `dev-docs/lane-rules.md` (binding). Above all "Nothing lives only in the container": commit
  early and `git push -u origin lane/<lane>` after EVERY commit. Fast gates only ("Gate procedure"); the main loop runs
  the full gates. Default git identity; the trailers your harness gives you. Never send account identifiers anywhere.
- Builds need the warm template: wait until `/home/user/tgt/template-build.log` ends with `EXIT 0` (the main loop
  rebuilds it after a reset; if the file is missing or stale and no `build-ssg.sh` process runs, say so in your report
  instead of building cold yourself). Then `scripts/lane-ssg.sh /home/user/wt/<lane> <lane> <port>`. Playwright under
  `flock /home/user/tgt/pw.lock`, `--workers=2`, `ssg.local.config.ts` with `PLAYWRIGHT_BASE_URL` (and
  `SSG_SITE_DIR=/home/user/tgt/pub-<lane>` if needed). Never start a long-lived process inside a lock.
- Spec runs: `cd playwright && PLAYWRIGHT_BASE_URL=... flock /home/user/tgt/pw.lock npm run test:changed -- --workers=2`
  (only the specs covering your diff; lane-rules "Before handback" step 3).
- Before handback: merge `origin/main`, fast gates, push. Do not edit `dev-docs/backlog/` (except lane round0): put
  the status text for your rows in your report.
- Shared with the other session (rounds 2, 3, 5, 7): message_scroller, carousel, toast, drawer, modality code and the
  component page template / `preview/build.rs` props extraction are theirs; do not edit them.

## round0 (backlog bookkeeping; mechanical)

Edit only `dev-docs/backlog/` (+ the generated index via `scripts/backlog-index.sh`, then
`scripts/check-backlog-index.sh` must print OK). Front matter `status` must equal the Status section text; set
`updated: 2026-10-19` on every changed row; moving to done = status, then `git mv` into `done/`, fixing any relative
link the link check reports. Skip rows already done/closed and list them. Owner-approved verdicts of the 2026-10-18
review:
1. Close as done ("CLOSED 2026-10-19 (backlog review 2026-10-18): done in the code" + evidence): 10, 22 (the SSG lane is
   in playwright.yml and preview/src/main.rs; the row's cited hashes do not resolve in this clone), 23, 72 (its one
   leftover moves to row 249), 93 (closed 2026-09-25, round 8), 94, 100, 103 (a measurement note), 108, 109, 110, 147 (a
   note), 161 (landed c6a80af, 5682701, c0976a9).
2. Duplicates ("CLOSED 2026-10-19 (duplicate of row X)"): 68 -> 216, 146 -> 213, 187 -> 188 (do not edit the targets).
3. Obsolete ("CLOSED 2026-10-19 (obsolete)"): 16 (tracker only; non-shadcn candidates "not planned"), 106 (historic
   count).
4. Narrow (keep open, prefix "NARROWED 2026-10-19 (backlog review): "): 5 "only rule 8 remains, blocked on `form`
   attribute forwarding in the primitives"; 98 "local fix in (build-ssg.sh absolute-path guard); filing upstream is the
   owner's decision (row 6)"; 130 "local fix 7ad8421 in; filing upstream is the owner's decision (row 6)"; 131 "local
   fix 150e97b in (build-ssg.sh wipes public/); filing upstream is the owner's decision (row 6)"; 165 "primitives Props
   are fully documented (missing_docs on); about 63 of 334 preview wrapper props lack docs; the generated table is still
   to build"; 170 "baseline is now 138 sites in 9 sheets (was 190 in 12)"; 218 keep its text, add the prefix plus
   "remaining: the typography reset for popup roots and the variant-slot gate".
5. New rows (shape of `245-*.md`, status "BACKLOG (2026-10-19)"): 247 "avatar-loading spec has a 450 ms floor too tight
   for this machine" (S, tests; verify5: 445.5 ms vs 450 in the full run and both reruns; widen or measure relative to
   the fallback delay); 248 "Select renders its placeholder until mount, so a Select with a default value shifts layout
   at hydration" (S-M, components; the data_table page-size Select caused CLS 0.0131 > 0.01; fix in the primitive by
   rendering the default value's label in SSG and the first client render); 249 "NavigationMenuIndicator (the leftover
   of closed row 72)" (S, components; check whether shadcn's base navigation-menu still ships an Indicator first).
One commit, push, report.

## lock (rows 221, 216, 92)

Rebuild this design (built and proven once, then lost unpushed):
1. FIFO ticket queue replacing bare `flock`: `scripts/lane-lock.sh [gate|lane] [--label T] -- cmd...` and `--status`.
   Each waiter flocks its own ticket file (temp name, locked, renamed into `$LANE_TGT/lockq`); liveness = owner still
   holds the ticket flock, so killed waiters/holders never block. Order (class, arrival), gate before lane; only the
   head takes a slot. `LANE_SLOTS` (default 2), each slot its own lock file; the command gets `LANE_SLOT` and
   `LANE_SLOT_SUFFIX` ("" for slot 0, "-s1" otherwise) and runs with lock fds CLOSED; TERM/INT/HUP stop it and its
   children. Slot 1 only while MemAvailable >= `LANE_MIN_AVAIL_MB` (5000). Slot 0 also holds the legacy
   `$LANE_TGT/ssg.lock` for its whole run (`LANE_LEGACY_LOCK`; empty disables), so old-style `flock ssg.lock` callers
   and slot 0 exclude each other. Document: SIGKILL of the wrapper alone frees the slot while its command may run on.
2. Wiring: `lane-ssg.sh` builds via a `--locked-build` mode inside the queue and starts its server outside it with fds
   closed; `gate-at.sh` uses class `gate` (keep the per-commit worktree and the tested-HEAD check, exit 9); new
   `scripts/lane-cargo.sh` (queue + touch + `CARGO_BUILD_JOBS=2` + `CARGO_INCREMENTAL=0` + the slot's target dir); new
   `scripts/lane-slot-target.sh` creates slot >= 1 target dirs lazily as warm copies when idle (else cold) and refuses
   when free disk < size + 20% + 2 GB.
3. Row 216: `scripts/lane-touch.sh <target> [--force]` touches the workspace sources unless the target's `.tree-owner`
   stamp names this worktree; called by build-ssg.sh (`BUILD_SSG_TOUCH=always|auto|never`), run-gates.sh,
   lane-cargo.sh, lane-ssg.sh, gate-at.sh; `lane-target.sh create` clears the stamp.
4. Row 92: moot since row 219; correct only the stale comment in `preview/build.rs`.
5. Gate `scripts/check-lane-lock.sh` (run-gates.sh + its own CLAUDE.md clause), deterministic, < 10 s, 8 scenarios:
   FIFO with a gate jumping lane waiters; SIGKILLed holder; SIGKILLed waiter; distinct `LANE_SLOT` with 2 slots; the
   memory guard; no leaked lock fd in a long-lived child; TERM stops the command; exit status and slot env pass
   through; legacy ssg.lock interop both ways.
6. Docs: lane-rules lock section + "Gate procedure" rule 2 to the new commands (and `LANE_SLOTS=1` while Playwright
   runs: the earlier proof bottomed at 2.0 GB available); the CLAUDE.md lane bullet.
Proof: `gate-at.sh <HEAD>` then two `lane-ssg.sh` builds right after; `--status` shows gate first and arrival order;
`free -m` every 15 s; `dmesg | grep -i 'killed process'` clean; a reverted marker string found in both builds.
Report the status text for rows 221, 216, 92.

## baseline-red (rows 242, 115)

Re-apply (diagnosed before, lost unpushed; re-verify against current code first):
1. `date-picker.spec.ts` natural-language "picking a day writes the formatted date": stale, date-dependent (it clicks
   a fixed day that can be the already-selected day, which deselects it) -> click a day other than the selected one.
2. `oracle/tier2-html/scroll-main-thread.spec.ts` test 2: stale (BlockPlayer's tick is motion-gated, the card was never
   scrolled into view) -> `scrollIntoViewIfNeeded` on the player card first.
3. `scroll-fade.spec.ts:305` MessageScroller fade after prepend: stale (content-visibility makes scrollHeight
   non-monotonic) -> wait for more `message-scroller-item` rows; keep the st > 96 and faded-edge assertions. Spec only.
4. `first-paint-styles` `/component/data_table/`: real (CLS 0.0131; the page-size Select trigger shows "Select an
   option" until mount, then "5") -> `preview/src/components/data_table/component.rs` passes a trigger rendering the
   page size so SSG, hydrated and settled text agree. (Primitive-level class: row 248, not here.)
5. Row 115: `combobox.spec.ts:134` -> an in-page MutationObserver before Enter records (state, option names); assert
   >= 1 closed state, every closed state holds only the filtered option, last state gone. `oracle/tier1-apg/menu-submenu.spec.ts`
   hover-intent tests (DropdownMenu.Sub, ContextMenu.Sub) -> a `hoverAndTimeOpen` helper stamping mouseover and the
   aria-expanded flip with performance.now, asserting only the >= 180 ms lower bound. Carousel: no change.
Proof: red-before where reproducible, green-after; data_table, select, combobox, menu-submenu specs green; 20 repeats of
the two flakes green. Report status text for rows 242, 115.

## suite-speed (row 152, speed half 152a)

Owner: "do the test suite optimizations / subagents procedure first so the rest goes faster". Facts: the full suite
(~145 spec files, ~3480 tests) took ~55 + ~51 min in two shards on a debug SSG (4 cores, 16 GB), ~21 min on release once.
1. Measure first: one full run with the JSON reporter on a debug build; the 30 slowest files; time by category; where
   time is wasted (fixed sleeps, per-test loads that could be per-file, networkidle on external fonts, retries, serial
   groups). Commit `playwright/durations.json`.
2. Speed without losing coverage: release-build validation if build cost < saving (measure both); duration-balanced
   sharding (< 45 min per shard); workers within memory (measure Chromium RSS; 3 if it fits beside one cargo build);
   block external requests in config/setup; per-file page reuse where specs only navigate. Owns `playwright/*.config.ts`,
   `playwright/package.json` scripts, new `playwright/tools/`; spec edits only to remove fixed sleeps/networkidle in the
   slowest files (fixtures.ts and the goto migration are lane specs2's; the baseline-red specs are theirs).
3. A changed-files spec selector (`scripts/select-specs.sh` or `playwright/tools/select.mjs`): diff range -> covering
   specs (component -> its specs, primitive -> consumers' specs, theme/main.css -> a broad set); document it in
   lane-rules "Before handback" step 3.
4. A slow-spec budget check (fast, no browser) on `durations.json` if robust (into run-gates + its CLAUDE.md clause);
   else document the budget.
Proof: before/after full-suite wall time on the same build, same titles and pass/fail set.

## specs2 (rows 164, 241)

1. Row 164: specs race hydration (row 109). Find `gotoHydrated`; count bare `page.goto` spec files and legacy `?name=`
   routes. Construction that cannot regress (choose, say why): the shared fixture (`playwright/fixtures.ts`) overrides
   `page.goto` to wait for hydration on same-origin SSG routes, or a gate forbids a bare `page.goto(` without
   `goto-ok: <reason>` (run-gates + CLAUDE.md clause). Migrate `?name=` routes to path routes (route table in
   `preview/src/main.rs`) or prove they resolve. Keep check-playwright-fixture-import.sh green; include the specs rounds
   2-3 added.
2. Row 241: all-routes, demo-forms-no-navigation and popup-style-isolation scrape block rows from the TEXT of
   `preview/src/components/mod.rs`; a cfg'd-out mirror is kept there and `preview/build.rs` `check_block_mirror` guards
   it. Make the specs read the source of truth (variants/ + order.txt, or a JSON the build writes into the SSG output),
   then delete the mirror and `check_block_mirror` (touch only those parts of build.rs).
Not yours: the playwright configs (suite-speed), the baseline-red specs beyond the mechanical goto change.
Proof: the gate (if any) red on a seeded bare goto; every edited spec green; the three block specs green without the
mirror. Report status text for rows 164, 241.
