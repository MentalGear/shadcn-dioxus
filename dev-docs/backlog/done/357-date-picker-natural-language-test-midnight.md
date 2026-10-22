---
id: 357
title: "Date picker natural_language unit test fails when a run crosses midnight UTC"
status: "Already fixed on main: parse_phrase(input, today) in…"
size: "XS: inject the clock in the test."
area: primitives
updated: 2026-10-22
---

# Row 357: Date picker natural_language test crosses midnight

## Item

Reported by the docs-site lane (batch 4): the date-picker `natural_language` test reads "today" twice, once to build the expectation and once inside the parser, so a run that straddles 00:00 UTC compares two different dates and goes red. The parser should take the reference date as an argument (the public entry point passes today's date), and the test should pass a fixed date.

## Status / trigger

Already fixed on main: `parse_phrase(input, today)` in `preview/src/components/date_picker/variants/natural_language/mod.rs` takes the reference date, `Demo` passes `UtcDateTime::now().date()` once, and every unit test uses the fixed `TODAY = 2026-10-14` (or an explicit date). No test reads the clock. The report was against an older tree.

## Size

XS: inject the clock in the test.

## Detail

Found while verifying batch 4; not reproduced on demand since it needs a run across 00:00 UTC.
