---
id: 541
title: "use_previous writes signals inside a memo: keep the previous value in a CopyValue or a render-local"
status: "BACKLOG, filed 2026-11-19. Before the final review (row 383). Not started."
size: "S"
area: primitives
updated: 2026-11-19
---

# Row 541: `use_previous` writes signals inside a memo

## Item

`use_previous` (`primitives/src/lib.rs:397-398`) calls `prev.set` and `last_seen.set` inside its `use_memo` closure. It does
not loop (the memo subscribes to `last_seen`, re-runs once, finds the values equal and stops; reasoned from
dioxus-signals 0.7.10, which has no warning for it), but a write inside a derived value is the pattern the repository's
effect gates exist to keep out. Five consumers: `carousel/root.rs:322`, `carousel/virtual_content.rs:360` and `:469`,
`lib.rs:515`, `input_otp.rs:381`. Construction: keep the previous value in a `CopyValue` or a render-local updated in the
component body.

## Status / trigger

BACKLOG, filed 2026-11-19. Before the final review (row 383). Not started.

## Size

S

## Detail

- **Proof:** the five consumers' specs stay green; a unit test pins that the first read returns `None` and a change
  returns the old value.
- Source: [`research/repo-evaluation-2026-11-19.md`](../research/repo-evaluation-2026-11-19.md), "Corrected or refuted".
