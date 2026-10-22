---
id: 355
title: "Adopt Google Rust style guide items: workspace lint table and conventions"
status: "BACKLOG -- Round 6 sweep, after row 267's file split (owner approved 2026-10-21)"
size: "S-M"
area: process
updated: 2026-10-21
---

# Row 355: Adopt Google Rust style guide items: workspace lint table and conventions

## Item

Adopt the Google Rust style guide items Tom approved on 2026-10-21 ("OK" to all five), from the "Google Rust style guide review" thread. Evaluation, measured on main 1bf5a8d1: https://claude.ai/artifact/PnQoSZWJrE8LMuNbnDYewN

## Status / trigger

**BACKLOG -- Round 6 sweep, after row 267's file split (owner approved 2026-10-21)**

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
