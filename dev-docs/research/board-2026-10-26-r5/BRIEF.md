# Adversarial board, round 5 (the cap), on row 434 step 3 (2026-10-26)

You are one independent member of the FIFTH and final adversarial review board. Rounds 1-4 each returned "holds with changes". Every DESIGN change so far sat inside the named-behaviour layer: round 1 its form, round 2 its deadline index and the command kind, round 3 its admission grounds, round 4 its admission unit (now per effect, one ground: a counterfactual witness read by library commit-point probes) plus one order contract (facts carry the observed slot epoch; Rust writes to a page-written slot go through that slot's page owner). The root rule, the deadline table, the two kinds, the closed bridge with one eval sink, A/G/R and enforcement by sinks plus ratchet held in all four rounds. Your job is to BREAK the plan as folded after round 4, not to confirm it. An agent handed a plan tends to return it confirmed; resist that. A finding counts only with evidence: a file:line in the repo, a line in the Dioxus 0.7.9 sources (~/.cargo/registry/src/*/dioxus-*-0.7.9 and friends), a spec or MDN/WHATWG/CSSWG citation, or a reproducible reasoning chain. Label each claim V (verified in code/source), W (web source, cite URL) or I (inference).

Read only; do NOT edit, commit, build or run cargo/dx/playwright.
- Code: /home/user/wt/b28 (branch queue/batch-28, what main will be) and /home/user/wt/l30 (branch lane/js-bridge-3-0, the bridge as built, lands next).
- The plan (branch lane/js-plan-docs-r4): /home/user/wt/l434d4/dev-docs/research/sync-js-root-2026-10-26.md (the root; 1.4 timing contract, 2.2 placement, kinds, admission, order contract, 2.3 G, 3.3 the audit-id ledger, 5 lanes and hours), sync-js-residue-2026-10-26.md, dev-docs/js-interop.md (the rule page every agent reads first), dev-docs/backlog/434-*.md, 442-*.md, 443-*.md, scripts/check-js-volume.sh.
- Prior rounds: dev-docs/research/board-2026-10-26*/ (SYNTHESIS.md plus reports). Do not repeat their findings; attack what changed in round 4 and anything they missed.

The owner's bar: the peak solve-by-construction (a construction that makes the whole class unable to recur), Rust first, JS only where it truly must be.

THE OWNER'S QUESTION: is the MAIN DESIGN now solid, so that only details remain? Classify every finding as exactly one of:
- DESIGN: it changes the core construct (root rule; deadline-indexed placement with listener and command kinds; admission per effect by counterfactual witness; the order contract; the closed bridge with one eval sink; A/G/R; enforcement by sinks + ratchet), or replaces it.
- DETAIL: a parameter, a block's mapping, a lane boundary, a probe, a spec, a gate rule, an estimate, wording.
Be strict both ways. Refining an existing construct's scope or parameters is DETAIL; replacing its unit, grounds or mechanism is DESIGN.

Output: /mnt/project-files/research/board-2026-10-26-r5/<your-lens>.md. Verdict line (HOLDS / HOLDS WITH CHANGES / BROKEN), then "DESIGN findings: N, DETAIL findings: M", then at most 8 lines of summary. Numbered findings: DESIGN or DETAIL, severity (blocker / major / minor), claim attacked, evidence, concrete change, what it subsumes. End with "attacks that failed". Hand back a 10-line summary including both counts.
