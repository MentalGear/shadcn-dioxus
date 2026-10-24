---
id: 382
title: "Rustdoc warnings to zero: shrink the check-rustdoc baseline (87 entries, mostly unclosed HTML tags)"
status: "OPEN. Fix the doc comments (backticks around the generic or tag), then scripts/check-rustdoc.sh…"
size: "S"
area: primitives
updated: 2026-10-24
---

# Row 382: rustdoc warnings to zero

## Item

`scripts/check-rustdoc.sh` (row 266) runs `cargo doc` for the default and `web` arms and fails on a new warning. The 87 existing warnings, almost all "unclosed HTML tag" from a bare `<T>` or `<div>` in a doc comment, sit in `scripts/check-rustdoc.baseline.tsv`.

## Status / trigger

OPEN. Fix the doc comments (backticks around the generic or tag), then `scripts/check-rustdoc.sh --update-baseline` until the baseline is empty; at zero the gate can become plain `-D warnings`.

## Size

S

## Detail

Mechanical, closed file list: fits a Haiku agent with the gate as the verifier.
