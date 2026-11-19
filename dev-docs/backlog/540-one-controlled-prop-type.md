---
id: 540
title: "One Controlled<T> prop type instead of two encodings of controlled-versus-uncontrolled"
status: "BACKLOG, filed 2026-11-19. Before the final review (row 383). Not started. The public API snapshot…"
size: "M"
area: primitives
updated: 2026-11-19
---

# Row 540: One `Controlled<T>` prop type

## Item

Primitives encode "controlled or not" two ways: `ReadSignal<Option<T>>` (16 `pub open: ReadSignal<Option<bool>>` and 12
`value` fields) and `Option<ReadSignal<Option<T>>>` (4 `value` fields), and one props struct mixes them
(`preview/src/components/combobox/component.rs:12`: `value: Option<ReadSignal<Option<T>>>` beside an `open` of the first
form). Construction: one `Controlled<T>` prop type that every component takes and one hook that resolves it.

## Status / trigger

BACKLOG, filed 2026-11-19. Before the final review (row 383). Not started. The public API snapshot gate makes this a deliberate break, to be batched with
other prop changes.

## Size

M

## Detail

- **Proof:** the encodings counted by `rg` fall to one; the controlled-versus-uncontrolled specs stay green;
  `/docs#controlled` still generates from the props.
- Counts: ledger D-23, D-24 (primitives; preview adds 11 `open`, 3 and 3).
- Source: [`research/repo-evaluation-2026-11-19.md`](../research/repo-evaluation-2026-11-19.md), "Smaller confirmed items".
