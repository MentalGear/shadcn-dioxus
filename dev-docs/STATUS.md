# Status: where we are

**Living page. The main loop updates it at every merge batch, deploy and round change; if the date below is old, treat
the page as stale and check `git log` and the flag files named here.** What each batch contained is in `git log` and the
backlog row files, not here: this page says only what is deployed, what runs, what is next and what waits for the owner.

Last updated: **2026-11-19**. Several sessions work in parallel; who holds what is on the `claims` branch (`CLAIMS.md`).
Preview site: https://mentalgear.github.io/shadcn-dioxus/ serves main `19d63c14`, built from `5b5adef1` (PRs #186-#197;
release suite 4204 passed, 0 failed, 0 flaky, 19 skipped on that exact tree, the `@timing` cases deactivated per row 494). Check
`git log origin/main -1` for the exact tip rather than trusting a SHA frozen here.

Session `session_01MsL6RCeq7QSe81nNBiuKP1` owned merges and deploys through PR #173 and is archived; session
`session_01MsL6RCa7c4QWmqNtUYQ8RE` merged and deployed the batches of 2026-11-12 to 2026-11-16 (through PR #197) and
stopped there at the owner's word ("finish phase 0, then wait and report"). A new
session reads this page, then takes the first open row of "Next" below; while "Next" says its lanes wait for the
owner's go, the session reports what it found and waits for that go instead of starting them. Each row file under [`backlog/`](./backlog/)
says what to build, which files, its proof and its size. The rules are [`../CLAUDE.md`](../CLAUDE.md) and
[`lane-rules.md`](./lane-rules.md); the owner's decisions are [`decisions.md`](./decisions.md).

Session `session_01MsL6RDL4Y2Pxs9N6sNdZF1` (2026-11-19) ran the repository evaluation (the code-analyser tool, the
allocation audit, TigerStyle, the built output and the code quality), made the receipt rule standing with its checker and
gate (`scripts/report-receipts.py`), and filed rows 529-542 (535, a lightbox fly-in for the
dialog family, is approved by the owner and not scheduled) with the note
[`research/repo-evaluation-2026-11-19.md`](./research/repo-evaluation-2026-11-19.md) and its evidence directory. Its work is
on branch `claude/sleepy-archimedes-f3i7ze` and is **not merged to main**. Row **529** (the CLI deletes outside the
components root) is proposed as the first of the new rows because it is a destructive bug; the owner set the order of the new rows on
2026-11-19 (option A, in "Next" below), no lane has started, and every lane waits for the owner's go. The same session evaluated a functional core and a graph as the single source of truth
([`research/functional-core-and-graph-ssot-2026-11-19.md`](./research/functional-core-and-graph-ssot-2026-11-19.md)); its
steps 1 and 2 are row 542 (approved), and its step 3 (the code graph explorer reads the item table) is approved too, to be
built after the table passes its proof (owner 2026-11-19: "ok"). On 2026-11-20 the owner held the batch ("Not yet") for an evaluation of the Vine
language, Program Expression Graphs, e-graphs and the sea of nodes on top of row 542
([`research/vine-egraphs-sea-of-nodes-on-row-542-2026-11-20.md`](./research/vine-egraphs-sea-of-nodes-on-row-542-2026-11-20.md)):
it adopts none of them, leaves row 542 and the order as they are, and puts two additions to row 542 forward that wait
for the owner.

## Merge queue

One session owns merges to main and deploys (the owner names it; the backlog thread did it through batch 29). Lanes push
a fast-gated branch (`scripts/run-gates.sh --fast`) and a QUEUE row on the `claims` branch; the queue owner merges a
batch of 4-5 into `queue/batch-N`, runs ONE full `scripts/run-gates.sh`, the release suite and
`scripts/registry-install-smoke.sh` on the merged tree, opens and merges the PR, and deploys once
(`scripts/deploy-preview.sh`, which runs the full Playwright suite on a release build). Procedure:
[`lane-rules.md`](./lane-rules.md), "Merge queue". Since 2026-11-12 the merging session has delegated its own lanes and
assembled each `queue/batch-N` from their `lane/*` branches without QUEUE rows (PRs #186-#198); `CLAIMS.md` on the
`claims` branch was last written on 2026-11-12 and holds no current lane.

The no-base-path validation tree (`$LANE_TGT/release-template`) was removed on 2026-11-07 to free disk for lane builds;
the next deploy first rebuilds it with `scripts/lane-target.sh warm-release` (a cold release build).

## Merged and deployed

Merge queue batches 1-29 are on main (PRs #121-#155); PRs #156-#165 followed from session
`session_01MsL6RCeq7QSe81nNBiuKP1` and are deployed (main `d040554a`, release suite 4159 passed, 0 failed). PRs #166-#168 are deployed (main `f38662dc`, deploy `85c05466`, release suite 4192 passed, 0 failed). PRs #170-#176 are deployed (main `edb71de0`, deploy `c34eda92`, release suite 4106 passed, 0 failed). PRs #177-#180 are deployed (main `3caa4f2c`, release suite 4108 passed, 0 failed). PR #182 is deployed (main `ce2cf865`, release suite 4108 passed, 0 failed). PR #185 is deployed (main `8b4326ad`, release suite 4142 passed, 0 failed). PRs #186-#197 are deployed (main `19d63c14`, built from `5b5adef1`, release suite 4204 passed, 0 failed): row 513 (#187), batch 5 (rows 447, 450, 516, 520 and the owner's fixes of 2026-11-14; #189), batch 6 (rows 519, 521, 522, 525; #191), batch 7 (row 448's `otpFilter`; #194) and batch 8 (row 449's page executor; #196), each followed by its deploy PR. What each batch carried is in `git log` and the row files under
[`backlog/done/`](./backlog/done/).

## Next

The owner's order of 2026-11-07 (refining 2026-10-26): **phase 0.1 of the JavaScript plan (row 444) is finished as far as
its artefacts exist (on main, PR #167); before the plan continues, every row that would be needed whether or not the
port happens comes first**, then the rest of the plan, then the final review. Rows in one step are independent unless a
row says otherwise, so a step runs as parallel lanes (one verifier per batch, `lane-rules.md`).

**Before the plan continues (port-independent)**

1. Rows 436, 451, 454, 506 and 511 are done, and the first half of row **448** (the native-arm focus trap is deleted) is
   on main (PR #182). Rows **509** (the Rust decision contract) and **446** (the witness harness, on top of 509) are on
   main with the batch of 2026-11-14, with a fix to the volume gate's `ffi` pattern; each row lists what stays open.
   Rows **450** (the bridge counters; its desktop half waits for a machine with `webkit2gtk`) and **447** (the order
   runtime as far as its file list allows) are on main with the second batch of 2026-11-14, with the owner's fixes of
   that day: docs group headings, Combobox group labels, the chart tooltip fade and the pie padding, row **516**
   (overlay entry motion without `@starting-style`) and row **520** (a carousel drag release registers its slide at
   release). Row **515** holds what row 447 leaves to the rows that own the consumers. Row **513** (the release suite
   reuses a spec file's green result) is on main.
   Rows **519**, **521**, **522** and **525** landed in batch 6 (PR #191), row **448** (`otpFilter`) in batch 7 (PR
   #194) and row **449** (the page executor's popover kind; its modal kind moved to row 456) in batch 8 (PR #196), so
   **every phase 0 row has landed, three of them in part** (step 7 below says what each leaves and where it went).
   **Next lanes, waiting for the owner's go** (owner 2026-11-14: "finish phase 0, then wait and report"): row
   **528** (`HoverSurface`: Tooltip and HoverCard as one surface, about 5 h, the first of
   [row 519](./backlog/done/519-anchored-panel-recipe-one-hook.md)'s "Further candidates"), with row **523** (an edge
   and floating close mid-entry) in the same lane, then row **517** (the chart tooltip on the real `Tooltip`, approved
   2026-11-14). The owner set the rest of the order on 2026-11-19 (option A, "A"): the same batch also takes row **529**
   (the CLI deletes outside the components root; first of the new rows, because it is a destructive bug reachable from one
   typed argument), row **530** (the transactional install, which changes the same functions as 529), row **542**'s first
   lane (the item table and its first gate, the long-function count, with the owner's approval of 2026-11-19), row **532**
   (output budgets) and row **533**'s kernel items (kernel lints and assertions); their file sets are disjoint from the
   lanes above, so they run in parallel. These lanes still wait for the owner's go. After them, step 8's ports in yield
   order.
2. Done, merged in PR #170 (2026-11-11): the deploy blockers of PR #168 (rows 468, 501); row **436**'s overlay and
   class groups (F1, F2, F4, F5, F6, F8, F9, F10, F11, F12), each reproduced red first (CHANGELOG.md has the
   user-visible ones: F1, F10, F11); **499** (safe triangle; CHANGELOG.md too); **152** slices 4-5; **371**'s
   dependency register gate (C23 enforced) and the demo-site icon deduplication (115 files moved off `dioxus-icons`
   onto the lifted central library; row 371's own stale-name fix too); **440**'s concurrent run-gates self-test;
   **486**'s pairing and resolved references; **399**'s `title.txt` override, the small-word case rule and its 78
   renames; **488** (no repro; RTL guard spec); **491** (name confirmed); **505**'s grant (PR #169) and the
   readable-name revert it covers.
   Done, merged in PR #171 (2026-11-11, on top of #170): **486**'s N1 rules and **507**'s first comment-analysis pass
   over `primitives` (two more stale names fixed; three follow-ups left open, below).
   PRs #172-#173 (2026-11-11, docs-only): two review passes over #170/#171's own row and status text, each one
   finding real staleness the previous pass's fixes had themselves introduced or missed (a row's opening sentence
   not updated when later work in the same row superseded it, byte counts stale after a later commit touched the
   same file, STATUS.md's own "main is at `<sha>`" line going stale the moment another PR merged). If a future pass
   finds the same class again, the fix belongs in the process (what makes a row's own first sentence go stale
   unnoticed), not in a fourth review pass.
   Done 2026-11-12 (the batch after PR #174): rows 440, 491, 499 and 503 moved to done; **510** (the spec selector
   follows component-to-component edges); **445** (the legacy freeze, taken early because it only guards); the
   grant-key refusal (16 open rows named a JavaScript-volume column the gate no longer has; all converted to bytes).
3. Row 436 is finished (PR #182): the delayed-task helper, per-window state and `menu_root::use_root_id`.
4. Open elsewhere: **504** (the hydrated Textarea shows no value; a fix was built on `lane/textarea-ssr-504`, which
   was never pushed and is gone; the row's Detail has the findings to rebuild it, and the fix needs the owner's call on a
   native-slot grant), **505**'s token-counting construction
   itself (the grant only covered the one readable-name revert), **371**'s remaining owner call (a D6 exception for
   `dioxus-icons`'s ~70 decorative-only demo icons vs. lifting those too), **152** (the prune continues on other
   files), **387** (the standing docs review), **507** (the standing comment-analysis backlog; pass 1's three
   follow-ups are open) and **496** (incremental gates, on an idle machine).
5. Owner and hardware: the iPhone checks (rows 028, 030, 295, 296, 305, 476, 477, 479, 480), Android (473), a desktop
   build (023), Firefox and Safari (153), and the owner's open questions (004/033 CI, 266 remote branches, 372 part 1,
   412-415, 469).

**The JavaScript plan** (row **434**; approved 2026-10-26 with scope 2). The whole plan is recorded, not only its first
step: the decided plan is [`research/sync-js-root-2026-10-26.md`](./research/sync-js-root-2026-10-26.md) (with
[`research/sync-js-residue-2026-10-26.md`](./research/sync-js-residue-2026-10-26.md) for the three feel-critical
ports), its working form is [`js-interop.md`](./js-interop.md), and every item has a row:

7. Phase 0, the construct and two proof ports: **508** (the decision-kernel proof: verdict `KERNEL`, 2026-11-12,
   upheld by an adversarial review with one blocking condition) and **509** (the Rust decision contract, which carries that
   condition), **444** (gates), **445** (legacy freeze), **446** (witness harness), **447** (order runtime), **448**
   (proof port A), **449** (proof port B) and **450** (bridge stats); all landed between 2026-11-07 and 2026-11-16.
   Row 450 stays open for its desktop latency session, which waits for a machine with `webkit2gtk` (row 23).
   Three items landed in part, and each remainder has a row. Item 0.4 (row 449) built the executor's popover kind only:
   its modal kind and exit wait go to row **456** (they run inside the open-focus policy that row ports),
   `indeterminate` and `defaultChecked` to row **458**, and the paint-safe gate's watchdog, the discovery of nodes that
   enter a declared pending state and the stale `placed` removal to row **452**, which builds that gate (the plan,
   section 2.2: "the watchdog belongs to the gate, not to the executor"). Item 0.5 (row 450) waits for a desktop
   machine. Item 0.7 (row 447) leaves its consumers to row **515**. Row 448's `prevent` witness waits for an rsx
   `onbeforeinput` on Dioxus 0.8 (row **525**).
8. The ports, as ordinary rows by yield (lines of JavaScript removed per hour, highest first; a row whose dependencies
   have not landed waits, and each row names them): **451** (3.SC, built 2026-11-12) and **454** (3.OB, built
   2026-11-12; five legacy scripts left the baseline, with 451 the volume gate's total fell from 508,767 to 482,329),
   then **452** (3.PL, 27/h), **453** (3.PV, 22/h), **455** (3.GS, 15/h), **456** (3.OV, 9/h; row **535**, the lightbox fly-in, follows it because its close half builds on the exit path that row
   rewrites), **457** (3.VS, 6/h),
   **458** (3.FC, 5/h), **461** (3.PW) and **459** (3.SL, which closes a blocking-listener class rather than removing
   lines); **460** (3.HV) and **462** (roving focus) only if phase 0's witnesses call for them.
9. The measured group, one port at a time with the old version switchable, the owner measuring on real devices after
   each: **442** (desktop proof harness) and **443** (measurement kit), **463** (freeze), **464** (carousel), **465**
   (message scroller), **466** (wheel band), then **467** (remove the old scripts per the owner's verdict).
   Trigger on the plan's last step: **494** re-measures the `@timing` cases (deactivated since 2026-11-02, row 493) on a
   release build and re-enables the timing pass.

**Last**

9a. Before the final review, in this order: rows **536** (generic components check value types at run time), **540**
    (one controlled prop type), **541** (`use_previous` writes signals inside a memo), **538** (deny the allocation lints
    after cleaning), **539** (hot-path allocation sites) and **531** (installed files carry no internal references), with
    what stays in row **537** (the prose gate `check-history-comments` and the CI-freeze note; its parsed-Rust gates are
    built in row 542). They edit many component files that the port rewrites, so mid-port they would conflict, and 536 and
    540 change the public API. Row 542's step 2 (handler decisions as pure outcome functions, the two calendar handlers
    first) and its step 3 follow the item table's proof.
10. **383**, the final adversarial review of the finished library. Parked behind the plan: **438** (Blitz renderer).

## Owner inputs pending

- Optional rows, each needing the owner's approval before anyone adopts or schedules it (owner 2026-10-24), not part of
  the adopted order: 412 (docs-site CSP meta tag), 413 (property tests and Miri), 414 (Kani/Aeneas proof spike), 415
  (GitHub Actions pinned to SHAs).
- For the measured ports (row 443): which devices the owner has (Mac, iPhone and iOS version, Windows or Android).
- Row **534** (overlay memory growth per open/close cycle) is not scheduled and comes last in every ordering; it starts
  only when the owner asks or a measured page shows memory that matters (a soak over thousands of opens, a content-heavy
  overlay).
- Row 469: start or drop (pre-hydration typing: autofill, back/forward form restoration, fast typing on a slow phone).
- iPhone re-tests: rows 28, 30, 295 and 305 (307 is closed; 296 is fixed by 477, landed and deployed, and closes with 477 after the owner's iPhone check).
- Owner question from the Radix-versus-Base-UI scan (2026-10-28): (3) `as` (Radix `asChild` semantics, row 395) against
  base-nova's `render` prop for badge, item, sidebar, breadcrumb, attachment, bubble and marker; the main loop recommends
  keeping `as`.
- Decided 2026-10-28: Select defaults to item-aligned (row 478); overlays stay open while scrolling and Menubar does not
  lock (rows 477, 479).
- Enable private vulnerability reporting on the repository ([`SECURITY.md`](../SECURITY.md) names it).
- Remote branch deletion: the owner said go (2026-10-26), but this environment's proxy refuses deletes; the owner runs
  the command from their own machine (the list of 262 merged branches was written in an
  earlier container to `/mnt/project-files/handoff/merged-remote-branches-2026-10-26.txt` and is not in the repository;
  `git branch -r --merged origin/main` lists the merged branches again; the unmerged ones are kept).
- CI back on (rows 4, 33): every workflow except `mutants.yml` is deleted, so reactivation means a new workflow that
  runs `scripts/run-gates.sh`.
- Decided 2026-11-14 (owner: "ok"): the grant columns of rows 509 and 446 (in the rows' front matter) and recording
  their raises; the hot-events `mounted` entry names row 514 (the per-row handle) and row 451 is closed; row 513 (reuse of release-suite results across spec-only changes) is approved.
- Decided 2026-11-14 (owner: "OK", "you can handle js raises from now on yourself"): the `bridge::boot` allowlist row
  and the grants of rows 450 and 447 (in the rows' front matter); JavaScript size raises are the main loop's from here
  ([`decisions.md`](./decisions.md)). Rows 516 (overlay entry motion) and 517 (the chart tooltip on the Tooltip's
  layer, hide on scroll) come from the owner's reports of the same day.
- Decided 2026-11-14 (owner): rows 519 (one anchored panel hook for the non-menu overlays, the hover timers in their
  own module; before row 449), 517 (the chart tooltip on the `Tooltip`) and 520 (a carousel drag release registers its
  destination at the release) are approved. The audit behind 519:
  [`research/overlay-layering-audit-2026-11-14.md`](./research/overlay-layering-audit-2026-11-14.md).
- Decided 2026-11-14 (owner): the chart tooltip without a fade-in was Firefox; row 516's construction removes the
  dependency ([`decisions.md`](./decisions.md)).
- Waits for the owner's go to start (both approved 2026-11-14; the go lifts the run limit): row 528 (`HoverSurface`,
  with row 523 in its lane) and then row 517, with the batch of 2026-11-19 beside them (rows 529, 530, 542's first lane,
  532, 533's kernel items; see "Next"). Optional, each the owner's call: row
  **526** (a lane's browser run waits for the whole release suite) and row **527** (gallery cards move by their
  content-visibility estimate when a script scrolls one into view).
- Settled by construction (main loop, 2026-11-18): no upstream request for `ToggleData` / `CancelData` payloads, since a
  payload cannot carry the ordering the nonce rule provides (row 525 has the argument); row 448's `prevent` witness is
  admitted when Dioxus 0.8's `onbeforeinput` moves the cancel to rsx (row 525).
- Row 508's verdict is `KERNEL`, so row 509 opens without a further decision. Loading the
  kernel inside the desktop webview stays unproven until a desktop build exists (row 23).

## Rules and decisions in force

[`decisions.md`](./decisions.md) lists the current owner decisions with their dates and rows;
[`lane-rules.md`](./lane-rules.md) is the lane brief (gate procedure, build lock, merge queue); the repo-wide rules are in
[`../CLAUDE.md`](../CLAUDE.md).

## Keeping this page current

Edit the section that changed, bump "Last updated", commit with the change that made it true. Row-level state belongs in
the row files under [`backlog/`](./backlog/), not here.
