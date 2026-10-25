---
id: 355
title: "Adopt Google Rust style guide items: workspace lint table and conventions"
status: "IN QUEUE -- items 1-4 on lane/row-355-google-rust-lints for the merge queue; item 5 is…"
size: "S-M"
area: process
updated: 2026-10-25
---

# Row 355: Adopt Google Rust style guide items: workspace lint table and conventions

## Item

Adopt the Google Rust style guide items Tom approved on 2026-10-21 ("OK" to all five), from the "Google Rust style guide review" thread. Evaluation, measured on main 1bf5a8d1: https://claude.ai/artifact/PnQoSZWJrE8LMuNbnDYewN

## Status / trigger

**IN QUEUE -- items 1-4 on lane/row-355-google-rust-lints for the merge queue; item 5 is dev-docs/guides/rust-conventions.md**

Done on the lane (2026-10-25), measured with clippy 0.1.97, `-D warnings`, all three arms clean:

- Item 1: `[workspace.lints]` in the root `Cargo.toml`, `lints.workspace = true` in all six members. A table cannot vary
  per crate, so the per-crate lints stay crate attributes (`missing_docs`, `unwrap_used`, `return_self_not_must_use` in
  `primitives/src/lib.rs`, `unwrap_used` in `cli/src/main.rs`); the gates keep `-D warnings`. `unsafe_code = "deny"`, one
  `#[expect(unsafe_code, reason)]` on the wasm `stderr` static.
- Item 2: 9 library `unwrap()`s in primitives (the estimate was 2) became `expect("<invariant>")`; one was a real panic:
  `CalendarSelectMonth` called `replace_month` on a view date on the 31st, which fails for a 30-day month, so the day is
  now clamped as `nth_month_next` does. cli had none. `clippy.toml`: `allow-unwrap-in-tests = true`.
- Item 3: 30 `#[must_use]` builder methods (toast 12, questionnaire 11, drag-and-drop 2, chart 2, drawer, virtual mode).
- Item 4: `allow_attributes_without_reason`; 83 attributes (67 `#[allow]` + 16 `cfg_attr` allows) all carry a reason.
  `#[expect]` where the lint fires in every build; `#[allow]` kept where it fires in only some (cfg arms, the 24 registry
  `component.rs` re-exports an app uses differently, the 4 derived `PartialEq` on fn-pointer props, which do not see
  `#[expect]`). 4 stale allows were removed (2 `too_many_arguments` on `#[component]` fns, slider `dead_code`, a virtual
  re-export). primitives `rust-version` 1.80.0 -> 1.81.0.

## Size

S-M

## Detail

Order: after row 267 (it touches the same primitives files); item 5 goes with row 266's docs work. Items 1 to 4 are one lane with full gates.

1. One `[workspace.lints]` table in the root `Cargo.toml` with `lints.workspace = true` in every crate, moving the policy out of CI flags and the one crate attribute; `unsafe_code = "deny"` with a single documented allow on `preview/src/wasm_libc_shim.rs` (`#[unsafe(no_mangle)]`). Keep `missing_docs` for primitives.
2. Deny `clippy::unwrap_used` in primitives and cli, `allow-unwrap-in-tests = true` in a `clippy.toml` (about 2 sites in primitives, 0 in cli; preview out of scope).
3. `clippy::return_self_not_must_use` for primitives (about 14 builder sites).
4. `clippy::allow_attributes_without_reason`: convert the 46 `#[allow]`s to `#[expect(lint, reason = "...")]` so stale ones fail; needs the primitives `rust-version` raised from 1.80.0 to 1.81.
5. A ~20-line Rust conventions section in the docs layout row 266 produces (not CLAUDE.md): assert messages say why, an enum over a bare bool parameter, `foo()`/`set_foo()`/`foo_mut()` accessors, an `Ext` suffix for extension traits, no trait only to organize code, a type annotation over a turbofish.

Optional one-off, not a gate: fix the real hits of `redundant_clone` (~20) and `needless_pass_by_value` (~10) by hand.
