---
id: 384
title: "Burn down the ratchet baselines and the leftover TODO comments (props docs, history comments, dead refs)"
status: "BACKLOG. The history-comment part overlaps row 267's extension (rewrite history comments while…"
size: "M"
area: process
updated: 2026-10-25
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

**Update 2026-10-25 (props baseline at zero, gate strict).** Four lanes documented the 347 `#[component] fn` arguments; the last entry, `primitives/src/navbar.rs NavbarItem.mut props`, could not be documented as a plain argument because splitting `pub fn NavbarItem(mut props: ..)` over several lines turned `check-wrapper-props.sh` red: `scripts/wrapper-props/main.rs` found a primitive's props struct only from a single-line `pub fn X(mut props: ..)` text match, so it reported every NavbarItem prop as dropped. Construction: the scanner now reads every fn header from the token stream (`fn_signatures`: comments and string/char literals masked, attributes and generics parsed, argument split at top-level commas, `mut`/`ref`/attributes/`///` stripped from the name, a `#[cfg(test)] mod` skipped by its attribute rather than by indentation), so multi-line headers, docs or attributes on the argument, and `mut` or not are one case; its output over the whole tree is byte-identical to the line scanner's (118 dropped-prop lines before and after). `check-wrapper-props.sh` now builds and runs the scanner's unit tests (`rustc --test`; every spelling of a header must give the same dropped props, text that only looks like a header must not count; a mutation that stops stripping `mut` turns four of them red). `NavbarItem`'s argument is now on its own lines with a `///` doc. `scripts/check-props-docs.baseline.tsv` is deleted and `check-props-docs.sh` is strict for arguments exactly as for `#[derive(Props)]` fields (no `--update-baseline`; a fixture with an undocumented argument, a multi-line one and a field is scanned on every run and must be reported, otherwise the gate is red). Known leftover, outside this lane's files: `preview/props_scan.rs` still reads `mut props: XProps` as an argument named `mut props` (the derive-form check compares the name with `props`), so NavbarItem's docs page table carries a `mut props` row; strip a leading `mut ` there. The row stays open: the history-comment and comment-ref baselines and the accordion/label TODOs remain.
