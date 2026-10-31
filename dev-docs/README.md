# dev-docs: the map

Everything here is for the people and agents who work on this repository. Start with **[`STATUS.md`](./STATUS.md)** (where we
are right now), then the rules (`../CLAUDE.md`, [`lane-rules.md`](./lane-rules.md)), then pick a row from the
[open backlog](./backlog.md). A living doc keeps only its current rules and recipes; dated landing logs, incident stories and
finished research live in [`archive/`](./archive/) and in git (`git log -- <path>`). Code comments cite docs by heading, row number
or phase number, so those headings and ids stay where they are.

## Status, order, backlog

| Path | What it is |
|---|---|
| [`STATUS.md`](./STATUS.md) | One page: where a fresh session picks up, what is merged and deployed, the ordered next rows, owner inputs pending. The main loop keeps it current. |
| [`backlog.md`](./backlog.md) | Index of the **open** rows (generated; "`backlog.md` row N" in code and docs resolves to the row file). |
| [`backlog-done.md`](./backlog-done.md) | Index of the **done** rows (landed, fixed, resolved, superseded), generated alongside. |
| [`backlog/`](./backlog/) | The backlog itself, **one file per row**: `<NNN>-<slug>.md` (open) and `done/` (finished, trimmed to a summary plus the landing commit). Edit a row file, run `scripts/backlog-index.sh`, commit both; `scripts/check-backlog-index.sh` is the gate. |
| [`research/backlog-order-2026-10-18.md`](./research/backlog-order-2026-10-18.md) | The 2026-10-18 execution order, superseded: the current order is [`STATUS.md`](./STATUS.md), "Next" (owner 2026-10-26: rows that can land before the JavaScript plan first). |
| [`research/sync-js-root-2026-10-26.md`](./research/sync-js-root-2026-10-26.md) | The decided JavaScript-to-Rust plan (row 434, approved 2026-10-26, scope 2), backlogged as rows 444-467; [`js-interop.md`](./js-interop.md) is its working form. |
| [`decisions.md`](./decisions.md) | The current owner decisions, one line each with its date and row. |
| [`plan.md`](./plan.md) | The original phased plan: phases 0-6 (finished phases are one line plus the PR), open decisions, definition of done. |

## Rules and recipes for sessions and lanes

| Path | What it is |
|---|---|
| `../CLAUDE.md` | Project instructions: delegation, the commit convention, the one gate entry point. |
| [`gates.md`](./gates.md) | Every gate, generated from the header of each `scripts/check-*.sh`; run them all with `scripts/run-gates.sh`. |
| [`lane-rules.md`](./lane-rules.md) | The brief every delegated lane gets: worktree, the global build lock, the fast loop, the gate procedure, build environment. |
| [`dev-loop.md`](./dev-loop.md) | The measured fast dev loop: edit, wait, inspect, one spec, suite. |
| [`dx-serve-hot-reload.md`](./dx-serve-hot-reload.md) | How to trust (and not trust) a `dx serve` result when verifying a UI fix. |
| [`agent-model-fit.md`](./agent-model-fit.md) | Which agent model fits which lane. |
| [`state-attributes.md`](./state-attributes.md) | The `data-*` state vocabulary: the one helper (`primitives/src/state_attrs.rs`) that emits Radix's `data-state` and Base UI's presence attributes together, the legacy valued booleans, and the gate (`check-state-attributes`). Read before adding a state attribute to a component. |
| [`history-in-comments.md`](./history-in-comments.md) | The rule that comments describe the current state and link history by backlog row, and the ratchets that hold it. |
| [`preview-composition.md`](./preview-composition.md) | The one architectural rule of `preview/` (themed wrappers only), enforced by `scripts/check-preview-composition.sh`. |
| [`motion.md`](./motion.md) | How animation works: tokens, reduced motion, compositor-only loops, visibility gating. Also on the site's `/docs`. |
| [`js-interop.md`](./js-interop.md) | How Rust talks to the page: Rust first, the bridge in `primitives/src/bridge/` second, no JS in Rust strings; an "I want to ... use this" table, the deadline table that says where a decision goes (rsx handler, named behaviour, A, G, R), the bridge verbs, the desktop caveats, what is forbidden and which gate says so (`check-js-volume` and `check-bridge-behaviours` have landed; its landed vs planned table names the rest). Read before writing any interactive behaviour. |
| [`guides/rust-conventions.md`](./guides/rust-conventions.md) | The short list of Rust conventions for primitives and components. |
| [`guides/lifting-code.md`](./guides/lifting-code.md) | Licensing and provenance rules for porting code from forks and upstream libraries. |
| [`supply-chain/README.md`](./supply-chain/README.md) | The supply-chain and dependency security playbook, written to be read and copied alone (threat model, controls table, adoption list); drift-gated by `scripts/check-supply-chain-doc.sh`. The root [`../SECURITY.md`](../SECURITY.md) is the reporting policy. |

## Reference (stable)

| Path | What it is |
|---|---|
| [`conformance-harness.md`](./conformance-harness.md) | The Playwright conformance harness: the three rule tiers (APG, HTML, Radix opinion), rule sources, calibration, how to add a rule. |
| [`conformance-harness-inventory.md`](./conformance-harness-inventory.md) | The inventory table: each spec, its tier and its rule. |
| [`reference/platform-rationale.md`](./reference/platform-rationale.md) | Why the primitives are built the way they are: the platform caveats the code cites (`<dialog>` and the cfg axis, scroll lock, RTL, main-thread cost, the keyboard open contract). |
| [`reference/mutants-baseline.md`](./reference/mutants-baseline.md) | The cargo-mutants baseline (2026-08-31). |

## Issues and upstream drafts

| Path | What it is |
|---|---|
| [`issues/`](./issues/) | Defect records that code or rows still cite: the duplicate-attribute root cause and findings (row 93), the `@supports` scoping bug, the raw-text hydration markers. |
| [`issues/drafts/`](./issues/drafts/) | Drafts for upstream Dioxus and dioxus-components filings, plus the running list of Dioxus friction. Outward-facing: nothing here is filed without the owner's go (rows 6, 14). |

## Research (`research/`: only notes that back an open row)

| Path | Backs |
|---|---|
| `backlog-order-2026-10-18.md`, `next-round-plan-2026-10-14.md`, `next-round-plan-2026-10-15.md` | The execution order and the round lane plans (rows 155 and the open rounds). |
| `round3-2026-10-05.md` | Round 3 evidence (listener inventory, VirtualList numbers, fonts, stylesheet delivery), cited by row 170 and `motion.md`. |
| `collection-model-2026-10-22.md` | The shared collection model (rows 360-364). |
| `shadcn-demo-parity-audit-2026-10-11.md` | Row 188, the owner's demo-parity audit. |
| `code-graph-explorer-2026-10-31.md`, `code-graph-explorer/` | Row 486: the code graph explorer research (tools compared, measured SCIP and cargo-modules runs) and the prototype sources. |
| `radix-vs-base-ui-2026-10-28.md` | The Radix-versus-Base-UI scan of every component, the iPhone overlay root cause and the placement rule (rows 477-482, 296). |
| `dependency-audit-2026-10-14.md`, `dependency-evaluation.md`, `dependency-scanners.md` | Rows 217 and 371 (what users compile and trust), behind the [supply-chain playbook](./supply-chain/README.md). |

## Archive (`archive/`: finished, historical; check the date before acting on anything here)

| Path | What it holds |
|---|---|
| [`archive/rounds/`](./archive/rounds/) | Round briefs and logs: the plan's dated landing log, the conformance-harness narrative, `next-steps-*`, the Phase 0 baseline, the component catalog harvest. |
| [`archive/research/`](./archive/research/) | Finished research behind done rows: carousel, chart, theme migration (with its tools), scroll jank, the unification audit, the virtualization survey, touch and keyboard investigations, `phase4-spike-findings.md` (the native `<dialog>` spike), and the per-gap "what to build" half of `recommended-implementations.md`. |
| [`archive/fork-research/`](./archive/fork-research/) | The 81-fork network research, the capability-gap matrix, the fork-fix mining results, `lifting-from-forks.md` (the full original). |
| [`archive/issues/`](./archive/issues/) | The duplicate-attribute guard prototype. |
| [`archive/scripts/`](./archive/scripts/) | One-off scripts (the backlog split). |
