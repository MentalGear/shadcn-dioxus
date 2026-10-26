---
id: 468
title: "Bug: form reset leaves a controlled text input showing empty while Rust keeps the old value"
status: "APPROVED 2026-10-26 by the owner (\"Ok for the rest\", backlog thread); for the next session (owner…"
size: "S"
area: inputs
updated: 2026-10-26
---

# Row 468: Bug: form reset leaves a controlled text input showing empty while Rust keeps the old value

## Item

On `<form>` reset the controlled text input has no reset path: the DOM shows an empty field while Rust still holds "abc", and the next render of its template writes "abc" back at an arbitrary moment.

**Where:** both renderers (a checkbox's state converges through the legacy reset listener, on desktop with a frame of skew; the text input does not).

**Reproduce first:** a controlled `Input` in a `<form>`; type "abc"; press a reset button; the field is empty; trigger any re-render of the template (change an unrelated signal that the input's component reads) and check the field.

**Source:** [`board-2026-10-26-r5/order-runtime.md`](../research/board-2026-10-26-r5/order-runtime.md) scenario D (finding 3, no owner for `value`).

Related to [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) but not part of it: the plan's construct subsumes this defect later, and the fix does not wait for the plan. Source: the last section of [`board-2026-10-26-r5/SYNTHESIS.md`](../research/board-2026-10-26-r5/SYNTHESIS.md) (live defects the board found in today's code).

## Status / trigger

APPROVED 2026-10-26 by the owner ("Ok for the rest", backlog thread); for the next session (owner 2026-10-26: "We will do it all in a fresh session").

BACKLOG. Found by board round 5; reproduce first. Independent of plan approval.

## Size

S

## Detail

- **Files owned:** To be found by the reproduction.
- **Depends on:** None. The plan's value owner (rows 447 and 458) subsumes it by construction: on `reset` it re-dispatches a real `input` on the control with the predicted post-reset level, so the owner's own rsx `oninput` sees it in order (Dioxus 0.7.9 also delivers `onreset`); a local fix (an `onreset` that sets the signal) is allowed meanwhile.
- **Proof:** A Playwright spec that fails before the fix and passes after; run on the web build and, where the bug is renderer-dependent, the desktop-cadence build of [row 442](./442-desktop-cadence-harness-and-desktop-pass.md).
- **Hours (inferred):** not estimated (size guess S).
