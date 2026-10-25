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

**Update 2026-10-25 (props baseline at zero, gate strict).** Four lanes documented the 347 `#[component] fn` arguments; the last entry, `primitives/src/navbar.rs NavbarItem.mut props`, was a false positive, not undocumented debt: `pub fn NavbarItem(mut props: NavbarItemProps)` is the derive form (the props struct is the argument), but `preview/props_scan.rs` kept the binding in the name (`mut props`), so its derive-form test (`name == "props"`) failed and the argument was listed as a prop; and a `///` on that argument cannot be written at all (rustc: "allow, cfg, cfg_attr, deny, expect, forbid, and warn are the only allowed built-in attributes in function parameters", the `#[component]` macro only consumes docs when it generates the props struct from the arguments). The scanner now strips `mut ` from an argument name, so `mut props: XProps` is the derive form like `props: XProps`. The attempt to split the header over several lines also showed a second, real defect: `scripts/wrapper-props/main.rs` found a primitive's props struct only from a single-line `pub fn X(mut props: ..)` text match, so any multi-line header made `check-wrapper-props.sh` report every prop of the primitive as dropped. Construction: that scanner now reads every fn header from the token stream (`fn_signatures`: comments and string/char literals masked, attributes and generics parsed, arguments split at top-level commas, `mut`/`ref`/attributes/`///` stripped from the name, a `#[cfg(test)] mod` skipped by its attribute rather than by indentation), so multi-line headers, docs or attributes on the argument, and `mut` or not are one case; its output over the whole tree is byte-identical to the line scanner's (118 dropped-prop lines before and after). `check-wrapper-props.sh` now builds and runs the scanner's unit tests (`rustc --test`; every spelling of a header must give the same dropped props, text that only looks like a header must not count; a mutation that stops stripping `mut` turns four of them red). `scripts/check-props-docs.baseline.tsv` is deleted and `check-props-docs.sh` is strict for arguments exactly as for `#[derive(Props)]` fields (no `--update-baseline`); every run first scans a fixture (undocumented argument, undocumented multi-line argument, undocumented field, and `mut props: XProps` in one and several lines, which must not be reported) and is red unless exactly the three bare ones are named. The row stays open: the history-comment and comment-ref baselines and the accordion/label TODOs remain.
