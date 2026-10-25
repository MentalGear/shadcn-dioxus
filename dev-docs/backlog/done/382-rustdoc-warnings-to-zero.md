---
id: 382
title: "Rustdoc warnings to zero: shrink the check-rustdoc baseline (87 entries) and make the gate strict"
status: "LANDED (lane/row-382-rustdoc-zero, 2026-10-25): every site fixed in the doc comment (private links…"
size: "S"
area: primitives
updated: 2026-10-25
---

# Row 382: rustdoc warnings to zero

## Item

`scripts/check-rustdoc.sh` (row 266) runs `cargo doc` for the default and `web` arms and fails on a new warning. The 87 existing warnings sat in `scripts/check-rustdoc.baseline.tsv`. The row first guessed they were mostly unclosed HTML tags; the inventory said otherwise: 6 were unclosed HTML tags, about 55 were public doc comments linking private items, and the rest unresolved links (`r#loop` and `r#virtual` raw identifiers, `web`-only items, stale names), ambiguous links (`derive`, `window`, `cv_chunks`) and two redundant explicit targets.

## Status / trigger

**LANDED (lane/row-382-rustdoc-zero, 2026-10-25):** every site fixed in the doc comment (private links became code spans, a literal `<tag>` went into a code span, ambiguous links got `derive()` / `mod@`, stale links point at the item that exists), `cargo doc` reports 0 warnings on both arms, `scripts/check-rustdoc.baseline.tsv` is deleted and `scripts/check-rustdoc.sh` fails on any warning (no `--update-baseline` / `--allow-grow` any more). Reopen only if a rustdoc lint needs a reviewed exception.

## Size

S

## Detail

Mechanical, closed file list: fits a Haiku agent with the gate as the verifier.
