---
id: 436
title: "Code-quality skill review: code-review and simplify skills over the whole codebase, after the row 426-435 refactors"
status: "BACKLOG. It was in the plan next to row 365 but never had a row, so 365 ran without it. The owner…"
size: "M for the review. Each accepted finding…"
area: process
updated: 2026-11-06
---

# Row 436: Code-quality skill review

## Item

A subagent pass with the session's code-quality skills over the whole codebase. The owner asked for it before the adversarial board ("load a subagent with a code quality review skill and let it analyse the codebase"). `code-review` finds correctness bugs, each with a failure scenario. `simplify`'s criteria find reuse, simplification, efficiency and altitude cleanups.

## Status / trigger

BACKLOG. It was in the plan next to row 365 but never had a row, so 365 ran without it. The owner said on 2026-10-25: "wait for the code review etc after the refactor above". It runs once rows 426-435 have landed, so it reviews the refactored code, and it comes before row 383.

## Status

REVIEWED 2026-11-06 (one read-only lane, Sonnet, by hand with both skills' criteria): [`../research/code-quality-review-2026-11-06.md`](../research/code-quality-review-2026-11-06.md). 12 findings: 0 high, 3 medium, 9 low; sized S 9, M 3. Scope was the overlay engine, the newest shared code and the row 426-435 files; `preview/src`, `scripts/` and `playwright/` were not reviewed. Nothing is verified in a browser. The accepted findings become rows after the owner reviews the table.

FIXED 2026-11-07, overlay-engine group (lane q436-overlay; specs in `playwright/overlay-engine.spec.ts`, each red on the base build and green after): F1 (`menu_sub.rs` passes `Some(sub.open.into())`; red because the Escape after ArrowLeft left the parent open), F2 (`use_animated_open` takes the close branch only while `show_in_dom` is true; red: 29 close-animation evals at mount of the dropdown-menu page, 0 after), F11 (`DISMISS_JS` ignores `isComposing` and keyCode 229; JS volume unchanged, the `focusin` block's closing `}, true);` joined its last line), F4 (`use_js_listeners_with(deps, setup)` in `js_listener.rs`: the plain inputs are the hook's deps, so a change reinstalls; `use_placement` in `top_layer/anchor_fallback.rs` passes id, anchor, point, side, align and gap through it; red: a second right-click on the open context menu landed at the old transform plus the new offset, (1140, 320) instead of (700, 220)). F4 covers the context menu point, a submenu's `sub_side` and every anchored overlay's side/align/gap. Not covered: `menu_root::use_root_id` (caller `id` written once by a plain `use_effect`; it needs `use_effect(use_reactive!(|caller| ..))`, left to the lane that owns `menu_root.rs`).

## Size

M for the review. Each accepted finding becomes its own row.

## Detail

- Four read-only reviewers in parallel on one detached worktree of the merged tree, each owning one area: `primitives/src` (Sonnet), `preview/src` (Sonnet), `scripts/` (Haiku) and `playwright/` (Haiku).
- Each reviewer invokes the `code-review` skill on its area as a path target (effort high for the Rust areas, medium for the others, findings only, no `--fix`). If the skill refuses a path target, the reviewer applies the same criteria by hand, then applies `simplify`'s criteria by reading.
- Per finding: severity, `file:line` verified, the failure scenario, the fix, and whether it is a class with a fix by construction. The report skips anything already in the backlog or the row 365 note. The output goes in `dev-docs/research/code-quality-<date>/<area>.md` and then to the owner for a decision before anything is built.
