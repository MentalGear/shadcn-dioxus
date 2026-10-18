---
id: 47
title: "styled Select listbox rendered full viewport width on the home page gallery — the \"dropdown menu has full…"
status: "Landed"
size: "S–M"
area: tests
updated: 2026-09-04
---

# Row 47: styled Select listbox rendered full viewport width on the home page gallery — the "dropdown menu has full…

## Item

**LANDED 2026-09-04: styled `Select` listbox rendered full viewport width on the home page gallery — the "dropdown menu has full page width" report, misidentified against `DropdownMenu` in the original (not-reproduced) write-up of this row.** Re-investigated against the actual subject: opening either styled `Select` on the home page (`/`, desktop 1280 wide, both the "Select an option" and "Choose a fruit" cards — also reproduces on `/component/?name=select&`) rendered `.dx-select-list-<hash>[popover]` at 1237–1247px wide against a 1280px viewport, clamped only by its own `max-width: calc(100vw - 2rem)`. Root cause: `min-width: 100%` in `preview/src/components/select/style.css` was written pre-migration, when this listbox was `position: absolute` inside `.dx-select`'s `position: relative` box (100% = trigger width); promoting it to the top layer via `popover` made its containing block the viewport instead, so the same percentage silently started meaning 100% of the *viewport*. A `@supports (anchor-name: --a) { .dx-select-list[popover] { min-width: anchor-size(width); } }` rule had already been added to restore trigger-relative sizing, but was dead in the served stylesheet: `#[css_module]`'s class-hashing does not scope selectors written only inside an `@supports` body (`dev-docs/issues/css-module-supports-scoping.md` — see row 32's notes for this being a second instance of the same gap), so the rule targeted the unhashed `.dx-select-list` while the live element carries `.dx-select-list-e8d9eb03`. And `use_anchor_position_fallback`'s JS fallback (the path a non-anchor engine like Firefox/WebKit takes) only ever corrected *position*, never *width* — so no engine had a working fix. Fixed at the engine level (`primitives/src/top_layer.rs`): every anchored content's trigger width is now published as the custom property `--dx-anchor-width`, unconditionally, from `use_anchor_position_fallback`'s own measurement (`reposition()`, every engine, kept fresh by its existing scroll/resize/visualViewport tracking) and, redundantly, from the shared `@supports` stylesheet's marker-class rules (`--dx-anchor-width: anchor-size(width);`, verified by execution to substitute correctly through a custom property on this sandbox's Chromium). `select/style.css` now reads `min-width: var(--dx-anchor-width, 100%)` and its dead `@supports` block/comment were rewritten to explain the contract. Regression oracle: two new `select.spec.ts` cases (component page and home gallery) asserting the listbox's width tracks its trigger and its left edge stays within 8px of the trigger's — RED before the fix (1237.76px / 1241.14px against ~141–156px triggers on a 1280px viewport, matching the report), GREEN after (listbox width equals trigger width exactly: 156.36px / 141.41px). `oracle/tier2-html/top-layer.spec.ts`'s own Rule 13 ("fit-content width") runs on that file's unstyled fixture and cannot see a themed component's CSS, so it never covered this — the new `select.spec.ts` cases are the real oracle for it; see that rule's own header doc, updated to say so

## Status / trigger

Landed

## Size

S–M

## Detail

Every other themed overlay `style.css` (`dropdown_menu`, `menubar`, `navbar`, `context_menu`, `popover`, `tooltip`, `hover_card`, `date_picker`, `color_picker`) was checked for the same class of bug (a percentage `width`/`min-width` on a top-layer content element) and found clean — each uses a fixed pixel `min-width` (200px/220px) or a `max-width` clamp against the viewport that was already viewport-relative by design (`position: fixed`'s containing block is always the viewport, promoted or not). `combobox`'s list is a fixed `width: 200px` matching its input wrapper's own fixed `width: 200px` — no viewport-relative bug there, and no "wide input" case exists today to make matching via `--dx-anchor-width` worth doing; left as-is
