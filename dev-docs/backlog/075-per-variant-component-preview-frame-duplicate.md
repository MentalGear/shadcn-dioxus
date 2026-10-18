---
id: 75
title: "Per-variant component-preview-frame duplicate id + unscoped Table caption locator (two strict-mode…"
status: "Found in integration round 2 (2026-09-18), fixed round 3 same day"
size: "S"
area: a11y
updated: 2026-09-18
---

# Row 75: Per-variant component-preview-frame duplicate id + unscoped Table caption locator (two strict-mode…

## Item

Per-variant `component-preview-frame` duplicate id + unscoped Table caption locator (two strict-mode violations, one class each)

## Status / trigger

Found in integration round 2 (2026-09-18), fixed round 3 same day

## Size

S

## Detail

Round 2's live integration run found two independent Playwright strict-mode violations. (a) `popover.spec.ts`'s basic test (`page.locator("#component-preview-frame")`, no `.first()`) resolved 2 elements once the `non_modal` demo variant existed alongside `main`, because every "Normal" component page renders one `ComponentVariantHighlight` per variant (main plus each "Variants" entry) and each copy hard-coded the literal id `component-preview-frame` on its own "Demo" `TabContent` — pre-existing for every multi-variant component (calendar's 5 extra variants, sidebar's `floating`/`inset` block variants), just never queried by that bare id before. Fixed by construction, not per-instance (commit `80af124`, cherry-picked from `af6a8d6`), in both `ComponentVariantHighlight` (Normal-kind) and its Block-kind sibling `BlockComponentVariantHighlight` (same defect, same shape, confirmed by reading): `main` keeps `component-preview-frame`; every other variant gets `component-preview-frame-<name>`. Audited all 20 existing `playwright/` call sites (10 spec files) — calendar's 5 already used `.first()` wanting the main demo, a no-op change now correct by construction instead of DOM-order luck; popover's own basic test is the one this fixes. Verified live on the fixing lane's own server: the basic test, previously reproduced failing with the reported strict-mode violation, passes warmed up (1.2s). (b) `table.spec.ts`'s basic test used an unscoped `page.getByText("A list of your recent invoices.")`, which also appears as literal source text in the page's always-mounted "Code" tab (`ComponentVariantHighlight` renders both "Demo" and "Code" `TabContent`s as siblings regardless of which is active) and in the "Manual installation" `<details>` section — so the unscoped lookup resolved 2 elements. **The same class, not a one-off**: any unscoped `page.getByText()`/`getByRole(..., {name})` whose string also appears in the demo's own rendered source is ambiguous on every component page in this app; the lane audited `table.spec.ts`, `data_table.spec.ts` and `sortable-table.spec.ts` for the same shape and found every other lookup already scoped under an attribute-selected container or constrained by a role the syntax-highlighted code spans never carry. Fixed (commit `fdb88bb`, cherry-picked from `5023dea`): scoped to the real `<caption>` inside the `data-slot="table"`-selected table, asserting both visibility and exact text. Not verified live by the fixing lane (cannot run Playwright); confirmed by integration round 3
