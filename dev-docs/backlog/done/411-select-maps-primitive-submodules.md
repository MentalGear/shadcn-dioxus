---
id: 411
title: "test:changed maps a primitive submodule to the components that reach it"
status: "LANDED (merge queue batch 17, lane/select-virtual). Trigger for reopening: a file under a primitive…"
size: "S"
area: tests
updated: 2026-10-24
---

# Row 411: test:changed maps a primitive submodule to the components that reach it

## Item

An edit to `primitives/src/virtual/find.rs` made `pnpm run test:changed` select the broad 47-spec set (about 18 minutes on a debug build), because `playwright/tools/select.mjs` found no component using `primitives/src/virtual/*.rs`.

## Status / trigger

LANDED (merge queue batch 17, lane/select-virtual). Trigger for reopening: a file under a primitive directory that falls back to the broad set although a component reaches it.

## Size

S

## Detail

The engine module is `mod r#virtual` and its users write `crate::r#virtual::...`; the primitive map matched only `crate::virtual`. A raw-identifier path now counts, and a file inside a primitive directory maps to the components that reach what `mod.rs` re-exports from it, following sibling files transitively (`mod.rs` itself, an undeclared file and a glob re-export keep the whole directory's users). `virtual/find.rs`: 47 -> 13 specs; `virtual/mod.rs`: 42 specs, every real engine user. The self-test has three new checks, and dropping the `r#` match or the submodule map each turns it red.
