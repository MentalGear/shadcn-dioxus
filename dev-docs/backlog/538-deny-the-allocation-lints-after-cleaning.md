---
id: 538
title: "Deny the allocation lints after cleaning: expect_fun_call now, then or_fun_call and redundant_clone; a take-attributes helper"
status: "BACKLOG, filed 2026-11-19. Before the final review (row 383). Not started. The old optional item in…"
size: "M: item 1 is XS, items 2-3 are S each…"
area: tooling
updated: 2026-11-19
---

# Row 538: Deny the allocation lints after cleaning

## Item

Measured with clippy 0.1.97 on the host target, all targets ([`evidence.md`](../research/repo-evaluation-2026-11-19/evidence.md),
section 3): `clippy::perf` fires 0 times; `expect_fun_call` 0 (a deny compiles today); `or_fun_call` 19 (none in a test
module; 6 do not allocate and need a rewrite or an `#[expect]`; 8 are the combobox demo `map_or(value.clone(), ..)`);
`redundant_clone` 88 (11 in tests; primitives 71, preview 16, kernel 1).

1. Deny `clippy::expect_fun_call` in `[workspace.lints.clippy]` now.
2. Clean `or_fun_call` and deny it (`map_or_else` for the allocating ones).
3. Clean `redundant_clone` and deny it.
4. The `native` and `server` clippy arms were not measured (about 0.36 % of the Rust lines sit behind those features) and
   the scripts member is reached by a workspace deny; count them before denying.
5. A `take attributes` construction for the 237 `props.attributes.clone()` sites in 83 files: `props` is owned by the
   component function, so most can move the attributes (`std::mem::take(&mut props.attributes)`, which three sites already
   do: `navbar.rs:1250`, `virtual_grid.rs:281`, `:1154`). One helper instead of 237 spellings.

## Status / trigger

BACKLOG, filed 2026-11-19. Before the final review (row 383). Not started. The old optional item in [row 355](./done/355-adopt-google-rust-style-guide-items.md)
("fix the real hits of `redundant_clone` (~20)") is superseded by the counts here.

## Size

M: item 1 is XS, items 2-3 are S each, item 5 is M (237 sites, mechanical).

## Detail

- **Proof:** the three lints denied and `scripts/run-gates.sh` green on all three clippy arms.
- Severity: cleanliness, not a measured cost; nothing was profiled.
- Source: [`research/repo-evaluation-2026-11-19.md`](../research/repo-evaluation-2026-11-19.md), section 2.
