# Which agent model fits which lane or task

Terms (owner, 2026-10-14): a **lane** is a work stream with its own file set and worktree ("the overlays lane owns
`top_layer.rs`"); an **agent** is the subagent that executes a lane or a task ("the theme lane, run by a Sonnet
agent"). Never "a Haiku lane".

This page is the rules. The running evidence log that produced them (2026-10-14 to 2026-10-21) is closed and archived in
[`archive/rounds/agent-model-fit-evidence.md`](./archive/rounds/agent-model-fit-evidence.md); the running log at the
end takes one row per delegated lane.

## The standing rule (owner)

Consider a Haiku subagent first for any delegated task, after a complexity assessment (owner 2026-10-25): the two
lists below are that assessment. A task on the "good fit" list goes to Haiku; a task on the "not a fit" list goes to
Sonnet; a task on neither is judged by how much construction it needs (a known idiom to copy is Haiku, a new decision is
Sonnet). The main loop keeps review, delegation and high-level decisions, and reviews every Haiku diff. Escalate only
after a failed attempt or for unusually judgment-heavy work (a headless state machine such as the Questionnaire).
**Never substitute Opus for a Sonnet or Haiku lane** (owner 2026-10-20: when Sonnet is limited, wait, or use Haiku where
it fits below); a lane stopped by a usage limit is resumed by message with its context, which beat a fresh agent.

## Haiku is a good fit for

- Read-only inventories with a crisp, checkable output: grep sweeps, counts per file, file:line lists, "which of these
  files already do X".
- Read-only audits against an external reference when the comparison is textual (demo text, labels, prop names), as long
  as the brief names the exact files and the output format.
- Small mechanical edits with a closed list of target files, a known idiom to copy and a verifying gate (wrapping
  `:hover` sheets, demo variants, closing rows whose status already says done), reviewed by the main loop afterwards.
- Run-and-report jobs (a release suite run, an install smoke) that classify nothing: the failure triage stays Sonnet.
- Small, idiom-following component additions with their own specs WITH a reviewer and a brief that says to wait in the
  foreground for queued work.

## Haiku is not a fit for (use Sonnet)

- Debugging and root-causing: anything that needs a bisect or a new hypothesis.
- Primitives and shared code (`primitives/src/**`, the theme, `top_layer.rs`, focus and placement logic), where a wrong
  construction reaches every consumer; and a change to a shared component's event or attribute plumbing needs a
  cross-component spec run before merge, not a code read, whoever wrote it.
- Designing a gate or a spec whose job is to remove a failure class: the construction needs judgment.
- Long lanes that wait on shared resources (the build lock, other lanes): a Haiku agent tended to hand back early with
  work still queued.
- Reports that must separate "measured" from "inferred" (pass and fail tallies across reruns).

## In every lane brief, whatever the model

1. [`lane-rules.md`](./lane-rules.md) plus the lane's own prompt; the files it owns and the ones it must not touch.
2. Before handback: merge the session branch into the lane branch and run `scripts/run-gates.sh --fast`; gates that
   landed after the agent branched otherwise go red only on merge (row 221).
3. Never send the user's email, name or any account identifier to an external service; a generic User-Agent such as
   `shadcn-dioxus-research` (a research agent once put the account email in a crates.io header, row 221).
4. Every cargo or dx command through the build lock (`scripts/lane-cargo.sh`, `scripts/lane-ssg.sh`).
5. Waiting on the lock or a queued build is not a reason to hand back: block on it, in the foreground.
6. Push the branch after every commit; nothing lives only in the container (row 250).

## Whatever the model, the main loop

Reviews the diff, and re-verifies lane results that ran on a shared cargo target on the merged tree: the mtime hazard
([`lane-rules.md`](./lane-rules.md), "Shared targets judge freshness by mtime", row 216) makes any model's "clippy clean"
on a shared or copied target untrustworthy until the sources were touched. Squash a lane's wip commits on merge.

## Running log (reopened 2026-10-24)

| Date | Lane | Agent | Result |
|---|---|---|---|
| 2026-10-24 | `find-photo-wall` (row 406: a find-in-page match landed on the item's estimate; shared `primitives/src/virtual/find.rs`) | Sonnet (root cause with a real find bar under Xvfb, shared-layer fix, spec) | Root-caused by measurement, fixed in the layer so the list and the wrapping grid share the step; a Haiku agent would not fit (primitives, a debugging hypothesis). |
| 2026-10-25 | `select-base-418` (row 418: `test:changed` diffs against the nearest of `origin/main` and `origin/queue/*`; closed file list in `playwright/tools/`, verified by `select.mjs --self-test` and `check-select-css.sh`) | Haiku | Fit: one new module, a self-test section on a throwaway repo, fast gates green, a manual proof on `queue/batch-20` (0 commits ahead, 4 before). It noted its own gaps (docs outside its list, no picker mutation) instead of widening the change. |
| 2026-10-26 | `js-bridge-3-0` (row 434 step 3 lane 3.0: the generic JS bridge `ts/bridge.ts`, the named-behaviour mechanism with zero real behaviours, `primitives/src/bridge/` with its one transport seam, `js_listener.rs` rebuilt on it, `check-bridge-behaviours.sh`, 61 `bun test` cases and a real-Chromium spec; shared primitives and new gate) | Sonnet (long lane, seven mid-lane scope changes folded in) | Fit, with a caveat: the shared-primitive and new-gate work is Sonnet's tier, and the additions (batched sample, `set`, the transport seam, the board's closed tables and behaviour layer replacing the interpreter) landed because the verbs are data-driven, but the board's redesign meant rewriting about half the lane after the first commit. The lane waited on the disk/hold rule before any cargo build; a Haiku agent would not fit (primitives, gates, a design with judgement calls). |
| 2026-10-26 | `js-ratchet-434r` (row 434: JS-volume ratchet; new `scripts/check-js-volume.sh`, `scripts/lib/ratchet.py`, a shared Rust tokenizer view, docs, a comment-only sweep) | Sonnet | Fit with limits: a new gate with a mutation self-test and a merge-base compare, no cargo; the shared-code part (`comment-scan.py` refactor) was checked by diffing the old and new extractor over every `.rs` file. A Haiku agent would not fit (new gate, shared code). |
| 2026-10-26 | `js-plan-docs-r4` and `js-plan-docs-r5` (row 434: fold adversarial board rounds 4 and 5 into the step 3 plan, rewrite `js-interop.md`, an audit-id ledger, then the plan as rows 444-471; docs only, no cargo) | Sonnet | Fit: each fold landed every synthesis item with fast gates and the link check green; one stale round-3 term survived in the residue doc and was fixed in review. Judgement-heavy wording (the admission and order rules) was written into the synthesis by the main loop first, so the lane folded decisions rather than making them. |
| 2026-10-26 | adversarial board rounds 4 and 5 (row 434: six read-only reviewers, lenses admission, whole-core, coherence, order-runtime, blank-slate-execution) | Opus (inherited) | Fit: read-only, evidence-cited reports against the Dioxus 0.7.9 sources; every DESIGN finding was confirmed by a second lens as at least a borderline DETAIL, none was invented. Sonnet was not tried for board members. |
| 2026-10-26 | `carousel-touch-stop` (row 470: a pointerdown on the carousel viewport stops autoplay; one rsx handler in `primitives/src/carousel/{content,virtual_content}.rs`, doc updates, a 5-test spec) | Sonnet | Fit: a small shared-primitive edit with a closed file list; the red-before/green-after proof needed two extra debug SSG builds (about 12 min each cold, a few warm) because the lane had no template to copy. A Haiku agent would not fit (primitive, test design for touch via CDP). |
| 2026-10-28 | `baseui-positioner-research` (row 296: read the published `@base-ui/react`, `@floating-ui/dom` and `@radix-ui/react-popper` packages for positioning strategy, autoUpdate options, scroll lock and iOS code paths; read-only) | Sonnet | Fit: GitHub was blocked, so it read compiled npm tarballs, quoted file paths and kept measured facts apart from inference. Haiku might have fit the lookup, but the report had to separate verified source from inferred browser behaviour. |
| 2026-10-28 | `overlay-lab-t9` (row 296: add the document-coordinate placement test, five arms including real Floating UI, to the scratchpad device lab; no repo edits) | Sonnet | Fit: it reused the lab's existing measurement helpers instead of rewriting them, and checked both outputs with a Playwright smoke test (4 px gap held on every arm, no console errors, CDN-blocked fallback recorded). Haiku would not fit (iOS coordinate-space measurement, judgement on what to reuse). |
| 2026-10-28 | `radix-scan-overlays` (Radix vs Base UI scan of the overlay, menu and dialog primitives against `@base-ui/react` 1.9.0, the Radix packages and both shadcn registries; read-only) | Sonnet | Fit: per-component table with measured and inferred claims kept apart; one claim (the top-layer `absolute` idea) it marked untested was already measured on the device. |
| 2026-10-28 | `radix-scan-controls` (the same scan for the form controls and simple widgets; read-only) | Sonnet | Fit: it found which base-nova components are not Base UI at all; its one bug claim (valued boolean `data-*` attributes matching presence selectors) did not reproduce, since the only presence-selector user already omits false values, and the main loop downgraded it to a convention. |
| 2026-10-28 | `radix-scan-rest` (the same scan for the remaining components: registry dependency inventory; read-only) | Haiku | Fit for the inventory: correct registry lookups and evidence; it returned the report in its message instead of writing the requested file, which the main loop saved. |
| 2026-10-28 | `overlay-lab-v5` (row 296: a new reduced device lab for the page-coordinate placement rule, five cards plus a Floating UI reference; no repo edits) | Sonnet | Fit: a small fresh page that reuses the working send and copy code, with a Playwright smoke per card; Chromium cannot show the iOS keyboard, so the device run stays the owner's. |
| 2026-10-28 | `overlay-lab-v6-dismiss` (lab: outside dismiss on a tap, not on the pointerdown that starts a scroll; one function in the scratch lab) | Haiku | Fit: a closed edit with a known idiom; it proved the test catches the bug by running it on the pre-fix build first. |
| 2026-10-28 | `overlay-lab-v7-live-height` (lab: the list max-height follows the available space while scrolling, never a transform write, behind a switch) | Haiku | Fit: a closed edit with a measured check; it found on its own that a 844 px viewport never exercises the live path and added a short-viewport case to show it. |
| 2026-10-28 | `backlog-placement-and-scan` (row 477 placement rule, row 296 root cause, rows 478-482 from the Radix vs Base UI scan, the research note, STATUS and index; docs only, own worktree) | Sonnet | Fit: rows follow the house format with the device numbers and the lab reference; its worktree branched from the older base, so the lab deletion moved to the main loop. |
| 2026-10-28 | `owner-decisions-rows` (record the Select default and the stay-open/no-lock decision in rows 477-479 and STATUS) | Haiku | Fit: a closed docs edit; it also fixed three sentences the decision made stale and listed them for review. |
| 2026-10-28 | `placement-477` (row 477: rule P in the shared placement code, tap-based touch dismiss, placement gate; resumed by a second agent after a container restart) | Sonnet | Fit: shared primitive work with judgement; the JS in Rust strings fell (anchor_fallback 475 to 127 lines, scroll_clamp 168 to 69). The resuming agent root-caused the remaining reds as one spec class (acting inside a panel mid-animation), now row 483. |
| 2026-10-28 | `overlay-defaults-478a` (row 478 items 1-3 and 6: menu items stay open, Combobox Home/End and Escape, Tooltip close on press and touch gate, Carousel RTL keys) | Sonnet | Fit, after one correction: it adopted Base UI's hover delays without Base UI's safe triangle; the main loop had it revert that item. |
| 2026-10-28 | `keyboard-inset-480` (row 480: diff Base UI 1.6.0 to 1.9.0 for the keyboard inset, port what fixes a user-visible problem) | Sonnet | Fit: it corrected the scan's constant list from the source and ported two changes while keeping the JS volume shrinking. |
| 2026-10-28 | `control-gaps-482` (row 482: Slider form participation, read_only, Toggle data-pressed, Tabs activate_on_focus) | Sonnet | Fit: four primitive features with specs and demos, install check green. |
| 2026-10-28 | `scroll-lock-479` (row 479: Base UI lock method, DropdownMenu unlocked) | Sonnet | Fit: it found Base UI's plain overflow method shifts content in Chromium with a classic scrollbar and added a measured branch for it. |
| 2026-10-29 | `select-defaults-478b` (row 478 item 5 and close-on-scroll cleanup; resumed after a container restart) | Sonnet | Fit: item-aligned default with a touch fallback, no wrap, open on press, no lock; it isolated two reds as belonging to the session tip, not to its lane. |
| 2026-10-29 | `post-merge-reds` (two specs red after the 477/479 merges) | Sonnet | Fit: both root-caused as stale specs with probes, no product change. |
| 2026-10-29 | `state-attributes-481` (row 481: state attribute helper, presence attributes beside data-state, gate) | Sonnet | Fit for a wide mechanical change: 62 files, one helper, a shrink-only gate baseline for the remaining valued booleans. |
| 2026-10-29 | `row-483` (backlog row for the settle fixture) | Haiku | Fit: a closed docs edit. |
| 2026-10-29 | `settle-fixture-483` (row 483: the shared Playwright fixture waits a just-opened anchored panel out before acting inside it; the six manual settles removed) | Sonnet | Fit: shared test plumbing with a full-suite proof (3703 passed, 1 unreproduced load flake). |
| 2026-10-29 | `docs-structure-lookup` and `codegraph-research` (read-only: row 422 proposal lookup; static-analysis tools for a code graph, with a trial of rust-analyzer scip and cargo-modules) | Haiku, Sonnet | Fit: the lookup quoted the approved outline with dates; the research measured both tools on this repo instead of reasoning from docs. |
| 2026-10-31 | `docs-groups-422` (row 422: Basic / Component-Specific / Examples groups on every component page, ten new Component structure sections, the order pinned by a cargo test and a spec) | Sonnet | Fit: one shared grouping definition for build.rs and the page; it found and fixed the layout shift an open-at-mount accordion item caused (one line in the accordion primitive). |
| 2026-10-31 | `overlay-review`, `row-486-security`, `backlog-cleanup`, `dependency-rule` (read-only review of the overlay rows; docs edits applying the owner decisions) | Sonnet, Sonnet, Sonnet, Haiku | Fit: the review measured shared vs duplicated overlay code with file:line evidence; the edits verified each fact before writing it. |
| 2026-11-05 | `codegraph-metrics-486` (row 486: comment metrics M1-M6 in the code graph explorer; one tokenizer pass reusing `comment-scan.py`, lenses, a Comments tab, smoke checks) | Sonnet | Fit: a data pipeline plus UI with its own smoke test; it cross-checked M4 against the history gate and planted a line to prove the count. |
| 2026-11-05 | `gate-logs-440` (row 440: one log directory per `run-gates.sh` run, `latest` link, retention, reuse records name the real log) | Sonnet | Fit: shared tooling with a closed file set, proven by two concurrent runs on one target. |
| 2026-11-05 | `status-tidy`, `post-tidy` (stale row statuses, rows 422/474/483/492/435/390 to done, stale comment references to deleted dismiss hooks) | Haiku, Haiku | Fit: exact-match docs edits; both surfaced the one judgement call (a move blocked by a link) instead of guessing. Both wrote their own model into the trailer; the main loop recommitted. |
| 2026-11-05 | `slow-specs-495` (row 495: the two ~32 s-per-test specs end their observation window on a quiet signal, 18% and 30% faster) | Sonnet | Fit: spec-only; its injected-shift check found that a frame-count quiet window missed a timer-driven shift, which added the pending-timer signal. |
| 2026-11-05 | `drawer-scroll-491` (row 491: a field in the scrollable-content demo and a keyboard-inset spec) | Sonnet | Fit for the work, but the first agent stalled on the disk guard and the build queue; finished after a resume. No library change was needed. |
| 2026-11-05 | `pointer-epoch-475` (row 475: a bridge epoch drops a superseded drag's queued frames for every `use_move_interaction` user) | Sonnet | Partly: the fix was small, the lane stalled on the build queue and spent long on a Resizable red it could not get. |
| 2026-11-05 | `menu-dismiss-489` (rows 489 and 487 phase 2: Menubar touch commits on the tap; the menu family on the dismiss core, one focus-out policy) | Sonnet | Fit: spec-first; it found the real cause of the recorded Menubar/DropdownMenu gap (a caller `id` hid the root from the dismiss layer). |
| 2026-11-05 | `chart-tooltip-472` (row 472: the chart tooltip fades with the floating tokens) | Haiku, then Sonnet | Haiku did not fit: the CSS fade broke three no-fly-in cases whose cause was in the Rust primitive (a stale measured size during the fade-out). Sonnet found that cause and fixed it. Rule: a fade on a surface that re-places itself is primitive work, so Sonnet. |
| 2026-11-05 | `form-reset-468` (row 468: controlled Input and Textarea follow a form reset through one hook; a bridge `after_render` scope fix) | Sonnet, Sonnet | Partly: the first agent did the design but queued a build on uncompiled code and ran out of context; the second compiled first, found two ordering causes (the reset event precedes the value restore; per-field reads raced) and finished. Rule added to the lane brief: compile-check before queueing a build, and wait in the foreground. |
| 2026-11-06 | `dismiss-487-56` (rows 456 and 487: dismiss items 5 and 6 on the dismiss core) | Sonnet | Fit: a closed file list with a known idiom suited the code work; verification was held up by the disk quota, not by the model. |
| 2026-11-06 | `form-value-468b` (row 468: value changes by construction, the second half of the form-reset lane) | Sonnet | Fit: root-causing and shared-primitive design suited Sonnet, and the only real cost was the wait for disk. |
| 2026-11-06 | `desktop-drag-471` (row 471: desktop drag DataTransfer investigation, stop and report) | Sonnet | Fit: an investigation with a stop-and-report outcome, which Sonnet handled. |
| 2026-11-06 | `docs-samples-392` (row 392: docs samples, three small CSS and prose fixes) | Haiku | Fit: the three fixes were small CSS and prose edits with a closed file list, each verified by a browser run or against the component code. |
| 2026-11-06 | `demos-399a` (row 399: missing demos, first half) | Haiku | Fit: a closed file list with sibling idioms; the judgement calls (the Hidden bug, the Pagination convention) were flagged rather than decided. |
| 2026-11-06 | `icons-371b` (rows 371 and 399: missing demos, second half, with the icons work) | Sonnet | Partly: the refactor and gate work fit one lane, and the proof half was blocked by the shared disk. |
| 2026-11-06 | `quality-review-436` (row 436: code-quality review, read-only) | Sonnet | Fit: a read-only review that traced callers and checked the Dioxus sources; no finding was reproduced. |
| 2026-11-06 | `deps-371a` (row 371: dependency review) | Sonnet | Partly: the edits were simple, but the dioxus-web feature interaction and a container restart needed several debugging rounds. |
| 2026-11-06 | `prune-152` (row 152: suite prune, all-routes) | Sonnet | Fit for the root-causing, which settled the question with numbers; the per-rule isolation timings were misleading for this idea, so they overstate a rule's cost in the full run. |
| 2026-11-06 | `focus-modality-490` (row 490: focus-modality restyle) | Sonnet | Partly: the work was large and mechanical, one agent could not split it across disjoint lanes, and the shared disk and Playwright lock made verification the bottleneck. |
| 2026-11-06 | `p444-a1` (row 444: grant rule and ratchet base, sub-lane A1) | Sonnet | Fit: shared-code design with a git-fixture self-test; the real trap was the ordering of the base-versus-tree compare. |
| 2026-11-06 | `p444-b1` (row 444: TS compiler-API scanner, sub-lane B1) | Sonnet | Fit: a closed-file tool with a clear proof contract, and the compiler-API design and baseline key needed Sonnet. |
| 2026-11-06 | `p444-d0` (row 444: clippy sink-ban probe, sub-lane D0) | Sonnet | Fit: a closed, measured probe with a verifying run. |
| 2026-11-06 | `p444-f` (row 444: native-slot sink scan, sub-lane F) | Sonnet | Fit: a closed-file new-gate lane with a clear house-style model; the one judgement call was the element-to-slot table. |
| 2026-11-06 | `p444-g` (row 444: installed components, no sink, sub-lane G) | Haiku | Fit: the closed file set and the verifying gate suited Haiku; a trailer conflict was flagged rather than resolved silently. |
| 2026-11-06 | `p444-h` (row 442: platform-rule TSV reader, sub-lane H) | Sonnet | Fit: a closed file list with a clear spec. |
| 2026-11-06 | `p444-integrate` (row 444: register the gates) | Sonnet | Fit: a closed file list, a known idiom and a verifying gate; the history-comments red needed judgement but was a two-line fix. |
| 2026-11-06 | `p444-tspin` (row 444: pin typescript, C23) | Sonnet | Fit: a well-bounded task, but the TS 7 API finding needed judgement. |
| 2026-11-06 | `p444-b2` (rows 434 and 444: bridge-behaviours core) | Sonnet | Fit: a judgement-heavy gate rewrite with a self-test per rule; the cost was build-queue waits, not the editing. |
| 2026-11-06 | `p444-d` (row 434: clippy sink ban) | Sonnet | Fit: mechanical edits across a closed site list; the clippy proof and the expect-placement rules needed care. |
| 2026-11-06 | `p444-e` (row 460: check-hot-events gate) | Sonnet | Fit: a new gate with a scanner and a ratchet, and judgement on what counts as a site (element, component or struct field). |
| 2026-11-06 | `p444-a2` (row 444: what check-js-volume counts) | Sonnet | Fit: a judgement-heavy gate rewrite (the unit change, introduced-column handling, the base-total gap). |
| 2026-11-06 | `i18n-371c` (row 371: replace dioxus-i18n, P10) | Sonnet | Fit: a closed file list with a known idiom; the cost was the disk shortage blocking the SSG build. |
| 2026-11-06 | `stylelint-371d` (row 371: lift the stylelint order config, P4) | Haiku | Fit: a closed-file lift with a before-and-after diff proof and a verifying gate; the JS-volume trap needed a decision, which was taken and reported. |
| 2026-11-06 | `pagination-disabled` (row 498: Pagination disabled prop) | Sonnet | Fit: a closed file list with a known idiom, and the gate and spec define done; the cost was the build queue and disk. |
| 2026-11-07 | `firstpaint-fix` (rows 468 and 482: first-paint reds, slider edge and textarea) | Sonnet | Fit: the reversal needed a design change driven by gate feedback (the computed-access rule and the JS-volume and bridge byte budgets), which suits a Sonnet debugging lane; disk exhaustion cost the last verification build. |
| 2026-11-07 | `select-race-501` (row 501: root-cause the select-escape race) | Sonnet | Fit: it reproduced the race under load and root-caused it, then corrected its own first construction after the changed-spec run exposed the echo case. |
| 2026-11-07 | `scrollarea-497` (row 497: ScrollType::Hidden) | Sonnet | Partly: the diagnosis, fix and gates went cleanly; the SSG proof stalled on the shared disk. |
| 2026-11-07 | `prune-152b` (row 152: suite prune, profiling and single-load edits) | Sonnet | Fit for the profiling and the single-load edits; the load average of about 20 made the per-file timings coarse, so only the large effects are clear. |
| 2026-11-07 | `tidy-done` (move finished rows to done/) | Haiku | Fit: a docs-only batch with a closed row list and a mechanical link re-root; the judgement calls on rows 487, 468 and 478 stayed with the main loop. |
| 2026-11-07 | `dep-register-371` (row 371: dependency register gate) | Sonnet | Fit: a new gate with a design choice (the closure check) and a multi-file proof, the Sonnet class. |
| 2026-11-07 | `gate-logs-test-440` (row 440: run-gates concurrency test) | Sonnet | Fit: the hermetic self-test exposed that run-gates retention never pruned (an emacs-syntax `find -regex`) and fixed it with a `finished` marker; the hand-back message was empty, so the commit message is the record. |
| 2026-11-07 | `drawer-name-491` (row 491: confirm shadcn's drawer demo name) | Haiku | Fit: yes, a closed docs lane with WebFetch checks against the upstream source and a verifying gate; the name was confirmed as "Scrollable Content" and nothing was renamed. |
| 2026-11-07 | `q436-overlay` (row 436: overlay findings F1, F2, F4, F11) | Sonnet | Fit: each finding needed its reproduction chosen with care (a stretched close-hold window, an eval probe, a second Escape site), and F4's hook is shared code. It kept a JS line count flat by joining a line, which row 505 turns into a measurement fix. |
| 2026-11-07 | `q436-classes` (row 436: class findings F5, F6, F8, F9, F10, F12) | Sonnet | Fit: red test or red gate first per class found a third id reader and two more unawaited-focus sites, and three gates came with the fixes. It shortened bridge.ts names to fit a byte budget, the second instance row 505 covers. |
| 2026-11-07 | `textarea-ssr-504` (row 504: the server-rendered Textarea shows no value) | Sonnet | Fit for root-causing in the Dioxus 0.7.9 sources (SSR writes `value` as an attribute; hydration applies none); the construction needs a native-slot write the gate holds for the owner, so it is parked unmerged. |
| 2026-11-07 | `prune-152c` (row 152: suite prune, slice 5) | Sonnet | Fit: merging tests into steps was mechanical, but each merge needed every case's premise read; under load only tests and page loads are firm numbers. |
| 2026-11-07 | `codegraph-pairing-486` (row 486: pairing through component.json, radius 2 proof) | Sonnet | Fit: an extractor change with a smoke-test proof; most wall time was the 15-minute SCIP index. Its new inputs were missing from the graph key, which the main loop added. |
| 2026-11-07 | `codegraph-refs-486` (row 486: unresolved references and N1 noise) | Sonnet | Fit: bucketing 11,283 unresolved references found seven mappable classes (0 left), and six rules cut N1 from 116 hits to 16 without a name list. |
| 2026-11-07 | `stale-names-486` (row 486: six stale names in comments) | Haiku | Fit: a closed six-site comment lane with `git grep` checks and a verifying gate. |
| 2026-11-07 | `model-fit-log` (this log: rows for 2026-11-06 and 2026-11-07) | Haiku | Fit: a transcript extraction into a closed table; one lane's verdict was missing from its empty hand-back and was filled in from its commit. |
| 2026-11-07 | `demos-399c` (row 399: demo titles and focus-target demos) | Sonnet | Fit: the title mechanism needed a root cause (titles were derived from directory names only) and a gate-driven redesign of the generator. |
| 2026-11-07 | `safe-triangle-499` (row 499: a safe triangle for the hover menus) | Sonnet | Fit: clear geometry and wiring; most time went to queued builds and to making the page-clock spec deterministic. |
| 2026-11-11 | `icons-carousel-371c` (row 371: 17 carousel demo files off `dioxus-icons`) | Haiku | Fit: a closed file list, one mechanical import swap, a verifying gate. A flaky wheel-replay test under sibling load needed a rerun and a judgment call the main loop reviewed. |
| 2026-11-11 | `icons-charts-371c` (row 371: 50 chart demo files off `dioxus-icons`) | Haiku | Fit: the same mechanical swap at larger scale; one import-ordering slip was caught by rustfmt and fixed on the next pass. |
| 2026-11-11 | `icons-rest-371c` (row 371: 48 varied demo files off `dioxus-icons`) | Sonnet | Partly: dispatched to Sonnet for the file set's variety (an alias, two substring near-misses), but the lane's own report judged the task Haiku-rated once exact-identifier parsing resolved those cases; Haiku would likely have fit here too. |
