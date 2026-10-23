# dev-docs: the map

Everything here is for the people and agents who work on this repository. Start with **[`STATUS.md`](./STATUS.md)**
(where we are right now), then the rules ([`lane-rules.md`](./lane-rules.md), `../CLAUDE.md`), then pick a row from the
[backlog](./backlog.md). **Living** = kept current as work lands; **reference** = stable, read when the topic comes up;
**historical** = a record of a past round or the research behind it, left as written (check the date before acting on it).

## Status, order, backlog (living)

| Path | What it is |
|---|---|
| [`STATUS.md`](./STATUS.md) | One page: current round, what is merged / deployed, what is running, the next round, owner inputs pending. The main loop keeps it current. |
| [`backlog.md`](./backlog.md) | The **index** of the backlog (generated; "`backlog.md` row N" in code and docs resolves here). Open and done tables, links to the notes. |
| [`backlog/`](./backlog/) | The backlog itself, **one file per row**: `<NNN>-<slug>.md` (open), `done/` (landed, fixed, resolved, superseded), `notes/` (the old file's narrative sections, verbatim). Edit a row file, run `scripts/backlog-index.sh`, commit both; `scripts/check-backlog-index.sh` is the gate. |
| [`research/backlog-order-2026-10-18.md`](./research/backlog-order-2026-10-18.md) | **The execution order** of the open rows (rounds 0-7 and why), owner 2026-10-18. Supersedes the order blocks in the backlog notes. |
| [`research/next-round-plan-2026-10-15.md`](./research/next-round-plan-2026-10-15.md) | The lane plan of the round that ran 2026-10-18 (lanes, owned files, proof protocol, baselines). |
| [`plan.md`](./plan.md) | The original phased plan (phases 0-6, open decisions, definition of done) and its landing log. Authoritative for phases 0-6; the rows after them live in the backlog. |

## Rules for sessions and lanes (living)

| Path | What it is |
|---|---|
| `../CLAUDE.md` | Project instructions: delegation, the gate list and the one gate entry point (`scripts/run-gates.sh`), build rules. |
| [`lane-rules.md`](./lane-rules.md) | The brief every delegated lane gets: the fast loop (`dx serve` + `scripts/dev-wait.sh` to iterate, `pnpm run test:changed` to verify), worktree, the global build lock, mtime touch, merge-then-fast-gates before handback, the gate procedure. |
| [`agent-model-fit.md`](./agent-model-fit.md) | Which agent model (Haiku / Sonnet) fits which lane; a running log, one row per delegated lane. |
| [`dev-loop.md`](./dev-loop.md) | The measured fast dev loop: edit, wait, inspect, one spec, suite. |
| [`dx-serve-hot-reload.md`](./dx-serve-hot-reload.md) | How to trust (and not trust) a `dx serve` result when verifying a UI fix. |
| [`preview-composition.md`](./preview-composition.md) | The one architectural rule of `preview/` (themed wrappers only), enforced by `scripts/check-preview-composition.sh`. |
| [`supply-chain/README.md`](./supply-chain/README.md) | The supply-chain and dependency security playbook, written to be read and copied alone: scope, threat model (what is and is not defended), the controls table (control, threat, enforcing script, config, status enforced / manual / planned, how to adopt), recorded decisions (no vendoring), and the "Adopt this in your project" file list. Drift-gated by `scripts/check-supply-chain-doc.sh`; the root [`../SECURITY.md`](../SECURITY.md) is the reporting policy. |
| [`motion.md`](./motion.md) | How animation works: tokens, reduced motion, compositor-only loops, visibility gating, and the gates behind them. Also on the site's `/docs`. |

## Reference (stable)

| Path | What it is |
|---|---|
| [`conformance-harness.md`](./conformance-harness.md) | The Playwright conformance harness: three rule tiers (APG / HTML / Radix-opinion), sources, calibration, current inventory. |
| [`component-backlog.md`](./component-backlog.md) | The component catalog harvest (upstream PRs, forks, shadcn gaps), re-rated; ranked queue of components. |
| [`capability-gaps.md`](./capability-gaps.md) | Capabilities upstream lacks (form participation, focus restore, scroll lock, collision, typeahead, RTL) and which fork closed each. |
| [`lifting-from-forks.md`](./lifting-from-forks.md), [`recommended-implementations.md`](./recommended-implementations.md) | How to port from forks (licensing, lift shapes, traps) and what to build per gap. |
| [`fork-network-research.md`](./fork-network-research.md) | The former `README.md`: summary of the 81-fork research with its document index and confidence notes. |
| [`mutants-baseline.md`](./mutants-baseline.md), [`baseline.md`](./baseline.md) | One-off baselines (cargo-mutants 2026-08-31; the Phase 0 green baseline 2026-08-29). |

## Issues and upstream drafts

| Path | What it is |
|---|---|
| [`issues/`](./issues/) | Records of defects found while working: duplicate-attribute root cause, findings and guard prototype (backlog row 93), the `@supports` scoping bug, chart stage 2 handoff (resolved). |
| [`issues/drafts/`](./issues/drafts/) | Drafts for upstream Dioxus / dioxus-components filings. Outward-facing: nothing here is filed without the owner's go (backlog rows 6, 14). |

## Research (`research/`; historical unless it says otherwise)

Dated evidence behind rows and decisions; the backlog rows link to the file that has the numbers.

| Path | Topic |
|---|---|
| `backlog-order-2026-10-18.md`, `next-round-plan-2026-10-15.md` | **Living:** see above. `next-round-plan-2026-10-14.md` is the earlier draft of it. |
| `round3-2026-10-05.md` | Round 3 evidence: modal close glitch, stylesheet delivery, listener inventory, VirtualList numbers, fonts. |
| `phase-c-plan-2026-10-13.md`, `theme-2026-09-26/`, `theme-2026-10-03/` (with `tools/`) | Theme migration to shadcn's system (row 111): role-token map, border-box, Nova vs base, generators. |
| `chart-2026-09-19.md`, `chart-forks-2026-09-19.md`, `chart-parity-2026-10-03/` | Chart engine decisions and the shadcn / Recharts parity snapshot (rows 88, 117-119). |
| `carousel-*.md`, `carousel-loop-2026-09-25/`, `carousel-overscroll-bench.html` | Carousel engine, overscroll, loop research (rows 91, 96, 102, 103). |
| `scroll-jank`, `shadcn-catalog`, `unification-audit`, `virtualization-survey` (all `-2026-10-04.md`) | Round-2 research notes feeding rows 136-154. |
| `message-scroller-parity-2026-10-14.md`, `chat-kit-demos-2026-10-14.md` | Chat kit and Message Scroller parity (rows 146, 213). |
| `shadcn-demo-gaps-2026-10-11.md`, `shadcn-demo-parity-audit-2026-10-11.md`, `docs-audit-2026-10-17.md` | Demo and docs parity audits. |
| `dependency-audit-2026-10-14.{md,json}` | Compile-time and supply-chain audit (row 217, markdown decisions). |
| `dependency-evaluation.md`, `dependency-scanners.md` | Row 371 (parked): the dependency rule applied to every user-compiled crate, and the scanner and criteria review behind the planned controls of the [supply-chain playbook](./supply-chain/README.md). |
| `touch-open-focus`, `mobile-keyboard`, `nav-scroll-smooth`, `user-select-inventory` (`-2026-10-17.md`), `baseui-focus-2026-10-18.md` | Touch, keyboard, scroll and focus investigations (rows 226-229, 240). |

## Historical one-offs (left in place so links keep working)

| Path | What it is |
|---|---|
| [`next-steps-2026-10-03.md`](./next-steps-2026-10-03.md), [`next-steps-2026-10-04.md`](./next-steps-2026-10-04.md), [`next-steps-2026-10-05.md`](./next-steps-2026-10-05.md) | Paste-ready thread briefs written after those rounds. Superseded by the backlog order and STATUS. |
| [`phase4-spike-findings.md`](./phase4-spike-findings.md) | The native `<dialog>` spike (Phase 4); the decision is recorded in `plan.md`. |
| [`adopt-fork-fixes.md`](./adopt-fork-fixes.md), [`adopt-fork-fixes-results.md`](./adopt-fork-fixes-results.md) | The fork-fix mining brief and its results (four fixes adopted, ~11 conditional, ~45 rejected). |
