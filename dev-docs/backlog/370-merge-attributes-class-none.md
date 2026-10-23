---
id: 370
title: "merge_attributes: a class: None replaces the base class instead of being ignored"
status: "Open. Found by the biggrid lane in batch 10."
size: "S"
area: primitives
updated: 2026-10-23
---

# Row 370: `merge_attributes` and an absent class

## Item

`merge_attributes` in `primitives/src/lib.rs` joins two `class` texts, but for any other pair it takes the later value (`_ => attr`). A themed component that forwards an `Option<&str>` class as `class: None` therefore wipes the base class: every unaligned `TableCell` and `TableHead` inside `DataTable` lost `dx-table-cell` (UA padding, no border) until the lane passed `""` instead of `None` (`data_table-large.spec.ts` caught it). The DataTable instance is fixed; the class is not.

## Status / trigger

Open. Found by the biggrid lane in batch 10.

## Size

S

## Detail

Fix by construction in `merge_attributes`: a `class` (and `style`) whose value is `None` or empty never replaces an existing one, with unit tests for `Text + None`, `None + Text` and `Text + ""`. Then drop the `""` workaround in `data_table` and check the other themed wrappers that forward an optional class.
