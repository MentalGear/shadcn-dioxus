---
id: 469
title: "Bug: text typed before hydration is wiped by the first post-hydration render on the SSG site"
status: "QUESTION TO OWNER 2026-10-26: the owner asked where typing before hydration happens naturally…"
size: "M"
area: inputs
updated: 2026-10-26
---

# Row 469: Bug: text typed before hydration is wiped by the first post-hydration render on the SSG site

## Item

Text typed into an input before the wasm arrives is wiped by the first post-hydration render of a template that reads a changing memo: hydration rebuilds with `skip_mutations`, so the vdom remembers the server's `value` while the DOM holds the typed text. `data-hydrated` is set by the root's effect, which runs before descendant effects, so specs cannot see this window.

**Where:** web SSG only (desktop has no SSG); the combobox input re-renders on every `open()` or `active_descendant()` change (`combobox/components/input.rs:152-161`).

**Reproduce first:** throttle the wasm (or block it), type into the combobox input and another controlled input on the SSG build, release the wasm, and open the listbox; check the typed text. A spec needs a barrier later than `data-hydrated` (the last descendant effect).

**Source:** [`board-2026-10-26-r5/order-runtime.md`](../research/board-2026-10-26-r5/order-runtime.md) scenario G (finding 3).

Related to [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) but not part of it: the plan's construct subsumes this defect later, and the fix does not wait for the plan. Source: the last section of [`board-2026-10-26-r5/SYNTHESIS.md`](../research/board-2026-10-26-r5/SYNTHESIS.md) (live defects the board found in today's code).

## Status / trigger

QUESTION TO OWNER 2026-10-26: the owner asked where typing before hydration happens naturally; answered in the backlog thread (browser autofill and back/forward form restoration fill inputs before the wasm loads, as does a fast typist on a slow phone). Starts on the owner's word.

BACKLOG. Found by board round 5; reproduce first. Independent of plan approval.

## Size

M

## Detail

- **Files owned:** To be found by the reproduction.
- **Depends on:** None. The plan's install-time level report (rows 447 and 458) subsumes it by construction; a local fix is to seed the signal from the DOM at mount.
- **Proof:** A Playwright spec that fails before the fix and passes after; run on the web build and, where the bug is renderer-dependent, the desktop-cadence build of [row 442](./442-desktop-cadence-harness-and-desktop-pass.md).
- **Hours (inferred):** not estimated (size guess M).
