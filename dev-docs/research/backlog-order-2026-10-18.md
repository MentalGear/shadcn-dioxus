> **Superseded.** The current order is [`../STATUS.md`](../STATUS.md), "Next" (owner 2026-10-26: every row that can land before the JavaScript plan goes first). This page is the 2026-10-18 order, kept for its reasoning.

# Backlog order (owner 2026-10-18: "they all need to be done: reorder them"; "do the test suite optimizations / subagents procedure first so the rest goes faster")

Every open row is in scope. Order follows the gate procedure (`dev-docs/lane-rules.md`, "Gate procedure"): rounds of
4-5 lanes with disjoint files, one full gate run per merge batch, the mandatory merged-tree validation, then a deploy.
The picking page is https://claude.ai/artifact/JqmDaJi6Vu7wLxN71gY6Vo (same order).

Principles: (1) what makes every later round faster or its validation trustworthy goes first -- the lane procedure (lock queue, second slot) and the test-suite SPEED half of row 152 (152a), per the owner; (2) the planned next
round (213) keeps its place; (3) changes to shared primitives come before the components and docs that read them;
(4) the PRUNE half of the test-suite review (152b: delete, merge, re-tier specs) comes last, after the rounds that add specs; (5) held items close the list.

| Round | Lanes (rows) | Why here |
|---|---|---|
| 0 (main loop, with this deploy) | close the 18 rows the backlog review found done/duplicate/obsolete and narrow the 10 partly-done ones | bookkeeping, no code |
| 1 Procedure, suite speed, test health | **lock** 221 (ticket queue with merge gates first; free ~6 GB and add a second build slot) + 216 (touch in `lane-target.sh`/`build-ssg.sh`) + 92 (build.rs rerun line); **suite-speed** 152a (measure where the ~2 h debug / ~21 min release suite time goes; run validation on a release build; sharding and worker count within memory; a changed-files spec selector for lanes; a slow-spec budget gate); **baseline-red** 242 (four specs red on the deployed preview) + 115 (residual flakes); **specs** 241 (block-demo mirror) + 164 (hydrated goto everywhere, `?name=` migration) | owner: procedure and suite speed first, so every later round goes faster; a suite whose reds mean something |
| 2 Planned next round | **scroller** 213 (Message Scroller API gaps); **markdown** 213 (the optional markdown component, owner decisions E2); **theme** 239 (`all: unset` vs the baselines) + 243 (hover fills) + 124/125 (chart nits); **chat-nav** 148 (Chat category) | 213 was already next; small theme fixes ride along |
| 3 Overlay and interaction primitives | **focus** 240 (Base UI lifts); **drawer** 156 (snap points, non-modal, responsive Dialog/Drawer demo); **toast** 138 (depth parity); **carousel** 237 (free-scrolling switch) | one primitive per lane, disjoint files |
| 4 Virtualization and scrolling | **virtual** 244 (`VirtualGrid`, on the shared engine of 154) then 153 (find-in-page for windowed lists, same files, same lane); **home** 174 (scroll jank causes 1 and 5); **utilities** 186 (scroll-fade/shimmer docs page + install path) | 244 and 153 share `primitives/src/virtual/` |
| 5 Docs and site | **props** 165 (generated props table); **fonts** 162 (self-hosted Geist); **rtl** 137 (RTL by construction); **demos-a** / **demos-b** 188 (richer demos, split by component) | reads the APIs rounds 2-4 settled |
| 6 Quality sweeps | **listeners** 167; **overrides** 170; **isolation** 218 (typography reset + variant-slot gate); **cleanup** 266 (repo and dev-docs cleanup, owner 2026-10-19); **split** 267 (oversized source files into modules, pure moves, owner 2026-10-19); then **suite-prune** 152b (delete, merge and re-tier specs) last | sweeps after the feature rounds; 152 last so it reviews the final suite |
| 7 Held items | 136 (Questionnaire); 157 (InputOtp alphanumeric keypad) | were on hold; owner now wants everything done |

Owner inputs, not lanes: the iPhone re-test (rows 28, 30, plus this round's touch-focus and keyboard-inset checklist)
right after this deploy; decisions before round 1 ends: CI back on (4, 33), upstream engagement and the upstream
filings of rows 130, 131, 98 (6, 14 -- outward-facing, so only with the owner's go), older WebKit (15), components
beyond shadcn (16, then it becomes its own round).
