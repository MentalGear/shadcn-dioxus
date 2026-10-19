---
id: 79
title: "Reactivity guards: check-hooks-in-closures.sh + check-self-subscribing-effects.sh"
status: "Landed 2026-09-18/19 (batch 2)"
size: "M"
area: overlays
updated: 2026-09-18
---

# Row 79: Reactivity guards: check-hooks-in-closures.sh + check-self-subscribing-effects.sh

## Item

Reactivity guards: `check-hooks-in-closures.sh` + `check-self-subscribing-effects.sh`

## Status / trigger

Landed 2026-09-18/19 (batch 2)

## Size

M

## Detail

**2026-09-26 (audit correction): RECONCILED** — `primitives/src/drawer.rs` (~lines 567-578) now explicitly cross-references the `NON_SELF_TERMINATING` allowlist entry ("Read this comment and that entry together"), closing the contradiction this row asked batch 3 to resolve. **Two new source-level guard scripts, joining the four already listed in `CLAUDE.md`** (`f11b98b`): `scripts/check-hooks-in-closures.sh` (no hook may be called from inside another hook's own initializer closure — the runtime-only panic class row 74 found only by execution, since neither `cargo check` nor clippy nor the doc tests see it) and `scripts/check-self-subscribing-effects.sh` (no `use_effect` may take a tracked read of a signal it also writes in the same run — the class row 73 found only by execution, hanging the main thread). Both are proven against this repo's own history, not just the current tree: `check-hooks-in-closures.sh` fails on `bb65cd6` (the pre-fix `navigation_menu.rs`, lines 259-261 — the exact commit row 74's own account traces the panic to) and passes on `c3dda45` (row 74's fix) and current `HEAD`; `check-self-subscribing-effects.sh` fails on `cdb073b` (the pre-fix `drawer.rs`, the `raw_offset` self-subscription row 73's own account traces the hang to) and passes once `b7037cb` (row 73's fix) is applied. Both scripts are now wired into `.github/workflows/main.yml` and `CLAUDE.md`'s pre-commit gate list, alongside the `npm ci`/symlink rule for lane worktrees' Playwright and stylelint `node_modules` — closing row 72's own bullet on that point. **Three pre-existing findings, fixed by construction in `f0244c2`:** `preview/src/components/color_picker/component.rs:266` (`value`, ColorField's hex-sync effect) and `:389` (`current_hue`, ColorSlider's hue-sync effect) both move to `.peek()` reads; the `value` fix also found the effect's `current.is_empty()` branch was an INDEPENDENT trigger (a field emptied by the user), not just a re-entry guard, so that recovery moved into the `oninput` handler itself rather than being silently lost — verified live that clearing the field still restores the external hex, now synchronously. `primitives/src/collection.rs:647` (`placement`, inside `use_deferred_collection_focus`) also moves to `.peek()`, reasoned correct against every call site (`dropdown_menu.rs`, `menubar.rs`, `context_menu.rs`/`menu_sub.rs`, `navbar.rs`) and now empirically confirmed by the full Playwright suite: none of its 11 failures (row 82) touch any menu-family spec. **One allowlist entry, and a stated contradiction left for batch 3:** `primitives/src/drawer.rs`'s `dragging` read stays tracked (it is the drag effect's own wake/sleep switch, not incidental) and was added to the checker's `NON_SELF_TERMINATING` allowlist instead of being peeked — directly CONTRADICTING `drawer.rs`'s own inline comment on that line (added alongside the `active_pointer_id` fix, row 77 / `2bf6489`), which argues against allowlisting it ("not something to allowlist away either, since it is not incidental"). The allowlisting was done anyway, per an explicit integration instruction that itself paraphrased that same reasoning before still directing the allowlist. Left unreconciled here on purpose: a human should decide whether `drawer.rs`'s comment gets a follow-up acknowledging the allowlist now exists, or whether the allowlist should instead be reverted in favor of a permanently-red, explained exception — a pass/fail script cannot represent that third option today. Batch 3 is asked to reconcile the two
