---
id: 8
title: "Select trigger → APG select-only-combobox role (role=\"combobox\")"
status: "LANDED 2026-09-18. SelectTrigger now renders role=\"combobox\" (was an implicit <button>), keeping…"
size: "M"
area: overlays
updated: 2026-09-18
---

# Row 8: Select trigger → APG select-only-combobox role (role="combobox")

## Item

Select trigger → APG select-only-combobox role (`role="combobox"`)

## Status / trigger

**LANDED 2026-09-18.** `SelectTrigger` now renders `role="combobox"` (was an implicit `<button>`), keeping `aria-haspopup="listbox"`/`aria-expanded`/`aria-controls` and adding `aria-autocomplete="none"` plus `aria-required` (from `Select`'s `required` prop; `SelectMulti` has none, so its context carries a constant `false`) — commit `1e81ad2`, landed together with row 9 (shared `SelectRootConfig`→`SelectContext` plumbing, one indivisible change). Rule source: the APG combobox pattern's own prose licenses `aria-required` on a combobox/listbox ("About This Pattern", pinned commit `7e4034b` of `w3c/aria-practices`); a plain `<button>` cannot carry `aria-required` at all (axe flags it), silently dropping requiredness from AT — Radix's own Select trigger carries the identical role/attribute set. DOM focus model deliberately unchanged (real roving focus into the listbox, not `aria-activedescendant`). Every `role=button` Select locator updated to `role=combobox` across `select.spec.ts`, `keyboard-matrix.spec.ts`'s Select section, and `form-participation.spec.ts` (8 sites total, grepped repo-wide). New `playwright/oracle/tier1-apg/select-only-combobox.spec.ts`: R1 (role/`aria-haspopup`/`aria-expanded`/`aria-controls`, dual-subject against the vendored APG reference and this library), R2 (`required` exposes `aria-required=true`, single-subject — the reference has no required-field case), R3 (axe clean closed+open). **Calibration note:** `aria-autocomplete="none"` is Radix parity, not APG-calibrated — the vendored APG page never sets it — so R1 deliberately excludes it from the dual-subject check. **Defect found integrating (round 1, not by this lane):** the moment `role="combobox"` landed, axe's `button-name` (critical) started failing on every `SelectTrigger` with no caller-supplied `aria_label` — a plain `<button>`'s accessible name computes from its own text content ("name from content"), but that step does not apply to `role="combobox"`, so every caller relying on the trigger's rendered content (its `SelectValue`'s placeholder/selected text, or custom children) for a "free" name lost it silently. Confirmed by execution at three independent, unrelated call sites (the `select.spec.ts` demo, the homepage gallery, the `top_layer` fixture) plus the new R3 spec — two or more occurrences is a class (`CLAUDE.md`), so fixed as a primitive-level default in `trigger.rs` (commit `c9fbfd7`), not per-call-site. **Negative result kept as a code comment:** a self-referencing `aria-labelledby` (pointing the trigger's own `aria-labelledby` at its own `id`) was tried first and confirmed by execution *not* to resolve for `role="combobox"` in Chromium/axe-core — replaced with a concrete `aria-label`: `ctx.selectable.selected_text()` (already used by `SelectValue`), falling back to the placeholder, gated by the existing `has_own_accessible_name` helper (the same fix shape `SelectList` already uses for its own axe gap, row 34) so an already-labelled call site is untouched. Verified live against the running dev server (every `role="combobox"` site-wide gets a non-empty `aria-label`, including the multi-select's selected-text case) and by a 16/16 green Playwright re-run of every previously-failing test. **One locator missed by the initial grep, found and fixed same round:** `oracle-focus-restore.spec.ts`'s "Select returns focus to its trigger on Escape" test still used `getByRole("button")` (commit `2f0cdfe`) — that file wasn't in the original grep sweep's list.

## Size

M

## Detail

Regression test demonstrated RED on pre-fix `trigger.rs`, GREEN restored (integration round 1, R1 of `select-only-combobox.spec.ts`). See row 34 for the prior, analogous `SelectList` axe fix this row's accessible-name construction reuses the gating helper from
