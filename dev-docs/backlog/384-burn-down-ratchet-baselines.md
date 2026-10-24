---
id: 384
title: "Burn down the ratchet baselines and the leftover TODO comments (props docs, history comments, dead refs)"
status: "BACKLOG. The history-comment part overlaps row 267's extension (rewrite history comments while…"
size: "M"
area: process
updated: 2026-10-24
---

# Row 384: Burn down the ratchet baselines

## Item

Shrink-only baselines keep known debt from growing, but nothing drives them to zero. Today: `scripts/check-props-docs.baseline.tsv` (347 component arguments without a `///` doc, so their props-reference rows are empty), `scripts/check-history-comments.baseline.tsv` (553 comment lines in 226 files that retell history), `scripts/check-comment-refs.baseline.tsv` (17 dead references), and four TODO comments: `primitives/src/accordion.rs:11` (controlled version), `:12` (rewrite on Collapsible), `primitives/src/collapsible.rs:8` (more docs), `primitives/src/label.rs:46` (double-click selection, Radix parity question). Rows 170 (cross-component overrides, 137) and 382 (rustdoc, 87) own their own baselines.

## Status / trigger

BACKLOG. The history-comment part overlaps row 267's extension (rewrite history comments while splitting the six large files): do this row after 267, on what remains. The accordion TODOs are feature questions: answer them (build, or delete the TODO with the reason in the row) rather than keep them.

## Size

M

## Detail

Fits parallel lanes on disjoint file sets once row 267 has landed: props docs per component directory, history comments per file group, each lane ending with `--update-baseline` so the shrink is locked in. At zero, a baseline file and its allowance can be deleted and the gate becomes strict.
