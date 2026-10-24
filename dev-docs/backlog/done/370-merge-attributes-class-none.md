---
id: 370
title: "merge_attributes: a class: None replaces the base class instead of being ignored"
status: "LANDED in batch 13 (branch claude/exact-sizes-attr-merge, 76990567). merge_attributes ignores a…"
size: "S"
area: primitives
updated: 2026-10-24
---

# Row 370: `merge_attributes` and an absent class

## Item

`merge_attributes` in `primitives/src/lib.rs` joins two `class` texts, but for any other pair it takes the later value (`_ => attr`). A themed component that forwards an `Option<&str>` class as `class: None` therefore wipes the base class: every unaligned `TableCell` and `TableHead` inside `DataTable` lost `dx-table-cell` (UA padding, no border) until the lane passed `""` instead of `None` (`data_table-large.spec.ts` caught it). The DataTable instance is fixed; the class is not.

## Status / trigger

**LANDED in batch 13 (branch `claude/exact-sizes-attr-merge`, 76990567).** `merge_attributes` ignores a later `class: None` or empty class (`fold_class_into`, `primitives/src/lib.rs:1306`), with the unit test `class_merge_ignores_a_later_none_or_empty_and_adopts_a_later_value_over_none` (`lib.rs:1487`); the `""` workaround in `data_table` is gone (no `class: ""` left under `preview/src/components/data_table/`).

## Size

S

## Detail

Fix by construction in `merge_attributes`: a `class` (and `style`) whose value is `None` or empty never replaces an existing one, with unit tests for `Text + None`, `None + Text` and `Text + ""`. Then drop the `""` workaround in `data_table` and check the other themed wrappers that forward an optional class.
