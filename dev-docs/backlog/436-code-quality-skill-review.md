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

FIXED on `lane/q436-classes` (F5, F6, F8, F9, F10, F12 of the review; each red test first, then one construction per class):
- F5 ids reach CSS and JS: `primitives/src/escape.rs` (`css_ident`, `css_string`, `anchor_selector`, `js_string`); the three anchor-lookup scripts receive the selector as data, the seven `getElementById` sites bind `js_string`, `modality.rs`'s private `json_string` is gone; gate `check-id-interpolation.sh`; spec `playwright/id-and-button-edges.spec.ts` (demo "Ids with punctuation" in the top-layer fixture). The preview form demo's `getElementById('{form_id}')` takes constant ids and is outside the gate's scope (`primitives/src`).
- F6 constant signals: `use_constant_signal` (`primitives/src/constant_signal.rs`) at the eight render-body sites (select, toolbar, navigation menu, carousel indicators, drag and drop list, slider, command's two); gate `check-constant-signal.sh`.
- F8 two id readers: `primitives/src/caller_id.rs` is the one reader (non-namespaced, non-empty; numbers written out); `form_reset`, `menu_root` and the popover trigger (a third reader the review did not list) use it; `check-id-interpolation.sh` fails a `.name == "id"` read elsewhere.
- F9 unawaited focus: `use_listbox_container_focus` (one effect for both arms, awaits); gate `check-unawaited-focus.sh`.
- F10: `MoveInteraction::start_pointer` tests the button before it cancels (slider, resizable and colour picker share it).
- F12: the bridge's `sweep` returns on a batch with no `removedNodes`; `bridge.ts` 42789 -> 42736 bytes, `bridge.js` 19907 -> 19900, no grant needed.

## Size

M for the review. Each accepted finding becomes its own row.

## Detail

- Four read-only reviewers in parallel on one detached worktree of the merged tree, each owning one area: `primitives/src` (Sonnet), `preview/src` (Sonnet), `scripts/` (Haiku) and `playwright/` (Haiku).
- Each reviewer invokes the `code-review` skill on its area as a path target (effort high for the Rust areas, medium for the others, findings only, no `--fix`). If the skill refuses a path target, the reviewer applies the same criteria by hand, then applies `simplify`'s criteria by reading.
- Per finding: severity, `file:line` verified, the failure scenario, the fix, and whether it is a class with a fix by construction. The report skips anything already in the backlog or the row 365 note. The output goes in `dev-docs/research/code-quality-<date>/<area>.md` and then to the owner for a decision before anything is built.
