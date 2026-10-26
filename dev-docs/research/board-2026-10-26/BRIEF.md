# Adversarial board on row 434 step 3 (2026-10-26)

You are one independent member of an adversarial review board. Your job is to BREAK the plan below, not to confirm it. An agent handed a plan tends to return it confirmed; resist that. A finding counts only with evidence: a file:line in the repo, a line in the Dioxus 0.7.9 sources (~/.cargo/registry/src/*/dioxus-*-0.7.9 and friends), a spec or MDN/WHATWG/CSSWG citation, or a reproducible reasoning chain. Label each claim V (verified in code/source), W (web source, cite URL) or I (inference).

Repo (read only, do NOT edit, commit, build or run cargo/dx/playwright): /home/user/wt/b28 (branch queue/batch-28). The plan:
- /home/user/wt/b28/dev-docs/backlog/434-*.md, sections "Step 3 design" and "Step 3 adopted"
- /mnt/project-files/research/sync-js-root-2026-10-26.md (root cause, constructions A/B/G/R, per-block table, gate, lanes)
- /mnt/project-files/research/sync-js-residue-2026-10-26.md (moving W8 message scroller, C5 wheel band, C1 macros to Rust)
- /mnt/project-files/research/dioxus-best-practice-review-2026-10-26.md (Dioxus idioms; 4 musts folded in)
- /home/user/wt/b28/dev-docs/research/js-audit-2026-10-25.md (the 103 blocks)
The owner's bar: the peak solve-by-construction (a construction that makes the whole class of problem unable to recur), Rust first, JS only where it truly must be.

Output: write your report to /mnt/project-files/research/board-2026-10-26/<your-lens>.md. Start with a verdict line (HOLDS / HOLDS WITH CHANGES / BROKEN) and at most 8 lines of summary. Then numbered findings, each: severity (blocker / major / minor), the claim in the plan it attacks, evidence, the concrete change you propose, and what that change subsumes. End with "attacks that failed" (what you tried that did not break it) so the synthesis knows what was covered. Hand back a 10-line summary.
