# Adversarial board, round 4, on row 434 step 3 (2026-10-26)

You are one independent member of the FOURTH adversarial review board. Rounds 1-3 each returned "holds with changes"; every DESIGN change so far sat inside the named-behaviour layer (round 1 its form, round 2 its deadline index and the command kind, round 3 its admission grounds, now checked by structural effect or late-arm witness). The plan was amended after each round. Your job is to BREAK the hardened plan, not to confirm it. An agent handed a plan tends to return it confirmed; resist that. A finding counts only with evidence: a file:line in the repo, a line in the Dioxus 0.7.9 sources (~/.cargo/registry/src/*/dioxus-*-0.7.9 and friends), a spec or MDN/WHATWG/CSSWG citation, or a reproducible reasoning chain. Label each claim V (verified in code/source), W (web source, cite URL) or I (inference).

Repo (read only, do NOT edit, commit, build or run cargo/dx/playwright): /home/user/wt/b28 (branch queue/batch-28). The hardened plan:
- /home/user/wt/l434d3/dev-docs/backlog/434-*.md ("Step 3 adopted" section)
- /home/user/wt/l434d3/dev-docs/research/sync-js-root-2026-10-26.md (sections 1-5; 2.2 the deadline-indexed placement rule, kinds, admission and named behaviours, 2.3 hardened G, 1.4 desktop timing contract, 5.4 lane plan)
- /home/user/wt/l434d3/dev-docs/research/sync-js-residue-2026-10-26.md
- /home/user/wt/l434d3/dev-docs/research/board-2026-10-26/SYNTHESIS.md, board-2026-10-26-r2/SYNTHESIS.md and board-2026-10-26-r3/SYNTHESIS.md (what rounds 1-3 changed; their reports are beside them — do not just repeat them)
- /home/user/wt/l434d3/dev-docs/js-interop.md and scripts/check-js-volume.sh (the ratchet that already landed)
The owner's bar: the peak solve-by-construction (a construction that makes the whole class of problem unable to recur), Rust first, JS only where it truly must be.

THE OWNER'S QUESTION FOR THIS ROUND: is the MAIN DESIGN now solid, so that only details remain? Classify every finding as exactly one of:
- DESIGN: it changes the core construct (the root rule "publish before, decide locally"; the deadline-indexed placement rule with listener and command kinds; the admission rule by structural effect or late-arm witness; the closed bridge with one eval sink; A/G/R as layers; enforcement by sinks + ratchet), or replaces it with a different one.
- DETAIL: it changes a parameter, a mapping of one block, a lane boundary, a spec, a gate rule, an estimate, wording.
Be strict: do not label a detail as DESIGN to sound important, and do not hide a design flaw as a detail.

Output: write your report to /mnt/project-files/research/board-2026-10-26-r4/<your-lens>.md. Start with a verdict line (HOLDS / HOLDS WITH CHANGES / BROKEN), then a line "DESIGN findings: N, DETAIL findings: M", then at most 8 lines of summary. Then numbered findings, each: DESIGN or DETAIL, severity (blocker / major / minor), the claim it attacks, evidence, the concrete change, what it subsumes. End with "attacks that failed". Hand back a 10-line summary including the two counts.
