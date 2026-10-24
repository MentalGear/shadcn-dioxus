---
id: 413
title: "Property tests and Miri on the pure logic"
status: "OPTIONAL: needs owner approval before it is adopted or scheduled (owner 2026-10-24). Not part of…"
size: "M"
area: tests
updated: 2026-10-24
---

# Row 413: Property tests and Miri on the pure logic

## Item

Add `proptest` property tests, and Miri runs, for the logic that has no DOM: sort, filter, paginate and selection math, collection indexing, calendar and date math. Today the repo has unit tests and `cargo-mutants` (`mutants.yml`), but no property tests, fuzzing or Miri.

## Status / trigger

OPTIONAL: needs owner approval before it is adopted or scheduled (owner 2026-10-24). Not part of the adopted order. A lane must not pick it up until the owner says go. Best done after the DataTable model row (draft A of `2026-10-21-data-table-model-rows.md`), which collects that logic in one place.

## Size

M

## Detail

- Candidates: `preview/src/components/data_table/state.rs` (`sort_rows`, `filter_rows`, `paginate`, which are pure generics with 8 tests), the pure parts of `primitives/src/selection.rs` (it imports `Signal`, so split the math from the signal), and calendar and date helpers. The lane picks the final list after reading the code.
- Example properties: the page index is always in bounds, `paginate` covers each row exactly once across pages, sorting is a permutation and stable, and the selection stays a subset of the row ids after a filter.
- Miri runs on the host target over those tests only, and can't execute JS or other foreign calls. With no `unsafe` in the repo's own code, it mostly checks the dependencies under those tests, so it's cheap to add but expected to find little.
- `proptest` is a new dev-dependency, so it goes through the row 371 rule and the C-table first.
- **Done when:** the property tests run in `run-gates` (or `test:changed` for the touched crates) under a time budget the owner sets.

Source: `/mnt/project-files/backlog-drafts/2026-10-24-optional-safety-rows.md` (draft B), background in `/mnt/project-files/goose-eval/2026-10-24-goose-vs-wasm.md`.
