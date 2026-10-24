---
id: 408
title: "SelectMulti's name prop takes part in form submission"
status: "LANDED (merge queue batch 16, lane/selectmulti-name). Trigger for reopening: a form with a…"
size: "S"
area: primitives
updated: 2026-10-24
---

# Row 408: SelectMulti's name prop takes part in form submission

## Item

`SelectMulti` accepted a `name` prop and rendered nothing with it, so a multi-select inside a `<form>` submitted no values, unlike `Select`.

## Status / trigger

LANDED (merge queue batch 16, lane/selectmulti-name). Trigger for reopening: a form with a `SelectMulti` whose submitted or reset values differ from a native `<select multiple>` with the same options.

## Size

S

## Detail

`SelectMulti` renders a hidden `<select multiple name=...>` mirror built like `Select`'s: one selected `<option>` per value, `initial_selected` on the defaults, and a form-reset listener that restores `default_values` (`primitives/src/select/components/select.rs`). No `required` prop yet. The Form page gains a library-vs-native multi-select row, and the Select docs a Forms section. `playwright/oracle/tier2-html/form-participation.spec.ts` adds six tests, each beside a native calibration; `select.spec.ts`'s gallery listbox locator matches only an explicit `role=listbox`, since a native `<select multiple>` is an implicit listbox.
