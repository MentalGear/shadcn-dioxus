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

## Update 2026-10-25 (lane `accordion-label-384`): the TODOs answered

- `primitives/src/accordion.rs` "controlled version": built. `Accordion` has `value: ReadSignal<Option<Vec<String>>>`, `default_value: Vec<String>` and `on_value_change: Callback<Vec<String>>` (one list shape for single and multiple mode, as `ToggleGroup` uses one set for both), and `AccordionItem` has an optional `value` (default: its position as a decimal string). The open set moved from a `Vec<usize>` of registration ids to the list of item values; uncontrolled behaviour is unchanged (`default_open` seeds the state without calling `on_value_change`, `collapsible`, single versus multiple, `hidden_until_found` reveals, the item's `on_change`). The themed wrapper forwards all four. Demo `controlled`, docs.md section "Controlled", spec `playwright/accordion-controlled.spec.ts`, two unit tests.
- `primitives/src/accordion.rs` "rewrite this to use collapsible": TODO deleted, no rewrite. `AccordionContent` already shares `use_animated_open`, `use_beforematch`, `hidden_attribute` and `PanelHidden` with `Collapsible`, so the open/close animation, the until-found reveal and the SSR open state are the same code. What stays accordion-only is what Collapsible has no notion of: the group's single/multiple open set, `collapsible`, and the roving focus across triggers. Folding an item into a `Collapsible` would put one more `use_controlled` open state per item beside the group's, with no user-visible defect it fixes.
- `primitives/src/label.rs` "prevents selection on double click": built, in the primitive. Dioxus 0.7.9's `MouseData` / `PointerData` do not expose the click count (`detail`), and a `web_sys` downcast would add a dependency to the primitives crate, so the label owns a scoped `use_js_listeners` script on its own element (`mousedown` with `event.detail > 1` is cancelled unless a handler already did or the target is a button, input, select or textarea, Radix's rule). It is installed when the pointer first reaches the label (`mouseenter`, which always precedes the click that starts a double click) and removed when the label unmounts, so a page full of labels runs no script until the pointer arrives and `listener-inventory` sees nothing on `window`/`document`. The label carries `data-dx-label="<unique id>"` to be found by the script (its `id` belongs to the caller). Spec `playwright/label-double-click.spec.ts`.
- `preview/assets/dx-utilities.css:11` `docs/demo`: the generator line now says "demo page layouts"; CSS regenerated, baseline shrunk by one.
