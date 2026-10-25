---
id: 384
title: "Burn down the ratchet baselines and the leftover TODO comments (props docs, history comments, dead refs)"
status: "LANDED (merge queue batch 23): props-docs baseline deleted and the gate strict; history-comments…"
size: "M"
area: process
updated: 2026-10-25
---

# Row 384: Burn down the ratchet baselines

## Item

Shrink-only baselines keep known debt from growing, but nothing drives them to zero. Today: `scripts/check-props-docs.baseline.tsv` (347 component arguments without a `///` doc, so their props-reference rows are empty), `scripts/check-history-comments.baseline.tsv` (553 comment lines in 226 files that retell history), `scripts/check-comment-refs.baseline.tsv` (17 dead references), and four TODO comments: `primitives/src/accordion.rs:11` (controlled version), `:12` (rewrite on Collapsible), `primitives/src/collapsible.rs:8` (more docs), `primitives/src/label.rs:46` (double-click selection, Radix parity question). Rows 170 (cross-component overrides, 137) and 382 (rustdoc, 87) own their own baselines.

## Status / trigger

**LANDED (merge queue batch 23): props-docs baseline deleted and the gate strict; history-comments and comment-refs baselines at zero; the four TODOs answered (controlled Accordion and the Label double-click guard built, two deleted with reasons).**

## Size

M

## Detail

Fits parallel lanes on disjoint file sets once row 267 has landed: props docs per component directory, history comments per file group, each lane ending with `--update-baseline` so the shrink is locked in. At zero, a baseline file and its allowance can be deleted and the gate becomes strict.

**Update 2026-10-25 (props baseline at zero, gate strict).** Four lanes documented the 347 `#[component] fn` arguments; the last entry, `primitives/src/navbar.rs NavbarItem.mut props`, was a false positive, not undocumented debt: `pub fn NavbarItem(mut props: NavbarItemProps)` is the derive form (the props struct is the argument), but `preview/props_scan.rs` kept the binding in the name (`mut props`), so its derive-form test (`name == "props"`) failed and the argument was listed as a prop; and a `///` on that argument cannot be written at all (rustc: "allow, cfg, cfg_attr, deny, expect, forbid, and warn are the only allowed built-in attributes in function parameters", the `#[component]` macro only consumes docs when it generates the props struct from the arguments). The scanner now strips `mut ` from an argument name, so `mut props: XProps` is the derive form like `props: XProps`. The attempt to split the header over several lines also showed a second, real defect: `scripts/wrapper-props/main.rs` found a primitive's props struct only from a single-line `pub fn X(mut props: ..)` text match, so any multi-line header made `check-wrapper-props.sh` report every prop of the primitive as dropped. Construction: that scanner now reads every fn header from the token stream (`fn_signatures`: comments and string/char literals masked, attributes and generics parsed, arguments split at top-level commas, `mut`/`ref`/attributes/`///` stripped from the name, a `#[cfg(test)] mod` skipped by its attribute rather than by indentation), so multi-line headers, docs or attributes on the argument, and `mut` or not are one case; its output over the whole tree is byte-identical to the line scanner's (118 dropped-prop lines before and after). `check-wrapper-props.sh` now builds and runs the scanner's unit tests (`rustc --test`; every spelling of a header must give the same dropped props, text that only looks like a header must not count; a mutation that stops stripping `mut` turns four of them red). `scripts/check-props-docs.baseline.tsv` is deleted and `check-props-docs.sh` is strict for arguments exactly as for `#[derive(Props)]` fields (no `--update-baseline`); every run first scans a fixture (undocumented argument, undocumented multi-line argument, undocumented field, and `mut props: XProps` in one and several lines, which must not be reported) and is red unless exactly the three bare ones are named. The other parts are below.

## Update 2026-10-25 (lane `accordion-label-384`): the TODOs answered

- `primitives/src/accordion.rs` "controlled version": built. `Accordion` has `value: ReadSignal<Option<Vec<String>>>`, `default_value: Vec<String>` and `on_value_change: Callback<Vec<String>>` (one list shape for single and multiple mode, as `ToggleGroup` uses one set for both), and `AccordionItem` has an optional `value` (default: its position as a decimal string). The open set moved from a `Vec<usize>` of registration ids to the list of item values; uncontrolled behaviour is unchanged (`default_open` seeds the state without calling `on_value_change`, `collapsible`, single versus multiple, `hidden_until_found` reveals, the item's `on_change`). The themed wrapper forwards all four. Demo `controlled`, docs.md section "Controlled", spec `playwright/accordion-controlled.spec.ts`, two unit tests.
- `primitives/src/accordion.rs` "rewrite this to use collapsible": TODO deleted, no rewrite. `AccordionContent` already shares `use_animated_open`, `use_beforematch`, `hidden_attribute` and `PanelHidden` with `Collapsible`, so the open/close animation, the until-found reveal and the SSR open state are the same code. What stays accordion-only is what Collapsible has no notion of: the group's single/multiple open set, `collapsible`, and the roving focus across triggers. Folding an item into a `Collapsible` would put one more `use_controlled` open state per item beside the group's, with no user-visible defect it fixes.
- `primitives/src/label.rs` "prevents selection on double click": built, in the primitive. Dioxus 0.7.9's `MouseData` / `PointerData` do not expose the click count (`detail`), and a `web_sys` downcast would add a dependency to the primitives crate, so the label owns a scoped `use_js_listeners` script on its own element (`mousedown` with `event.detail > 1` is cancelled unless a handler already did or the target is a button, input, select or textarea, Radix's rule). It is installed when the pointer first reaches the label (`mouseenter`, which always precedes the click that starts a double click) and removed when the label unmounts, so a page full of labels runs no script until the pointer arrives and `listener-inventory` sees nothing on `window`/`document`. The label carries `data-dx-label="<unique id>"` to be found by the script (its `id` belongs to the caller). Spec `playwright/label-double-click.spec.ts`.
- `preview/assets/dx-utilities.css:11` `docs/demo`: the generator line now says "demo page layouts"; CSS regenerated, baseline shrunk by one.

**Update 2026-10-25 (lane `refs-todos-384`).** Five dead path citations fixed to the current path or reworded
(`hydration-parity.spec.ts` now cites `scripts/build-ssg.sh`, `check-direction-keys.sh` cites
`playwright/oracle/tier3-radix/rtl-toggle.spec.ts`, `activity.rs` cites row 174, `deploy-preview.sh` and `run-gates.sh`
reworded), `dev-docs/gates.md` regenerated, and `collapsible.rs`'s "TODO: more docs" deleted (every public item is
documented). With the generator fix above, `scripts/check-comment-refs.baseline.tsv` is at 0 entries and
`scripts/check-history-comments.baseline.tsv` was already at 0.
