---
id: 536
title: "Generic components check their value type at run time: a mismatched option compiles and panics"
status: "BACKLOG, filed 2026-11-19. Read, not run: the claim is reasoned from the definitions (ledger D-4)…"
size: "M: the context type, the five…"
area: primitives
updated: 2026-11-19
---

# Row 536: Generic components check their value type at run time

## Item

`Combobox`, `Select`, `TagGroup` and the `selectable` hook store option values type-erased (`RcPartialEqValue`) and
recover them with `as_ref::<T>()`, panicking when the type differs: `unwrap_or_else(|| panic!("combobox and option value
types must match"))` at `primitives/src/combobox/components/combobox.rs:188`, `select/components/select.rs:202` and `:525`,
`selectable.rs:269`, `tag_group.rs:504`, `:513`, `:585`, `:628`. An option's `T` is a separate generic from the root's
(`SelectOptionProps<T>` in `select/components/option.rs:20` holds `value: ReadSignal<T>`; `Select<T>` is its own
component), so a root of one type with an option of another type-checks and fails only when the option is chosen. The
panic is reachable from type-checked user code, and on `wasm32-unknown-unknown` a panic aborts the page.

Construction to evaluate: a context generic over `T` (the root provides `Context<T>`, the option consumes
`Context<T>`), so a mismatch is a missing context or a compile error instead of a downcast. The type erasure exists so
one context type serves every `T`; the cost to weigh is where that context is stored and whether the option can name `T`
without the caller repeating it.

## Status / trigger

BACKLOG, filed 2026-11-19. Read, not run: the claim is reasoned from the definitions (ledger D-4), no mismatched pair
was built. Not started.

## Size

M: the context type, the five components and their tests; the public API snapshot gate sees any signature change.

## Detail

- **Proof:** a compile-fail or a test that builds a root and an option of different types and does not panic (or does not
  compile); the existing component specs stay green.
- **Residue:** a context keyed by `T` needs the option to be inside a root of the same `T`; the failure for a missing
  root stays a run-time error in the context lookup.
- Source: [`research/repo-evaluation-2026-11-19.md`](../research/repo-evaluation-2026-11-19.md), section 4;
  [`evidence.md`](../research/repo-evaluation-2026-11-19/evidence.md), section 11.
