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

FIXED on branch `lane/form-reset-468` (2026-11-06), awaiting merge. Found by board round 5; independent of plan approval.

**Repro** (`playwright/form-reset-controlled.spec.ts`, demo `input/variants/reset`): a controlled `Input`, a controlled `Textarea` and an uncontrolled `Input value="preset"` in a form; type, press the reset button. Red on the base build: the DOM empties, the state line stays `name=abc bio=some bio`.

**Root cause.** Every other form control (`Checkbox`, `Switch`, `RadioGroup`, `Select`, `Slider`, `InputOtp`, `Combobox`) already calls `use_form_reset_listener` and converges. Only the themed `Input` and `Textarea` wrappers had no reset path: the browser restores the DOM value and tells no Rust state, and the owner's next render writes its old signal back.

**Fix by construction** (`primitives/src/form_reset.rs`): `use_form_reset_value` is the one reset path for a text control the owner's signal holds. It ensures an id, listens through the existing legacy `use_form_reset_listener` (no new JS), reads the DOM value after the browser restored it through `bridge::get(Prop::Value)`, and the wrappers call `oninput` then `onchange` with a Rust-built `FormEvent` carrying it. Two constructions were needed beyond the hook, both found by running it:
- The `reset` event fires before the browser restores the values, and the microtasks carrying the message run in between, so the read waits one task (`sleep(0)`).
- Reading per field raced: the first field's report re-rendered the owner, which wrote the old text of the not-yet-read fields back (measured: textarea read `""`, then the controlled input read `"abc"`). The fields that heard the reset now queue, one task reads all of them in one bridge transaction, and the handlers run back to back before the owner renders once.
- `bridge::get` awaited from a task (no render of its own) panicked in the flush effect (`runtime.rs:223 unwrap on None`): queued effects run with an empty scope stack, and `flush` reaches `document::eval` and context lookups. `after_render` now runs its closure inside the root scope (`primitives/src/bridge/transport.rs`). The bridge's own callers all ask from render effects, which is why no earlier caller hit it.

**Proof.** The spec is red on the base build (`reset468base`, `name=abc bio=some bio` after reset) and green on the fixed build (`reset468`, 1 passed, 2.2 s on a debug SSG).

**The reset rule (owner question).** A reset is a value change to the control's default and is reported as an edit: `oninput`, then `onchange`, with the restored value; controlled owners follow, and reading the DOM after the reset means a reset the page cancelled with `preventDefault` reports the unchanged value, a no-op. This deliberately differs from React, which fires no `onChange` on a form reset (a controlled React input keeps its state and the DOM is rewritten from it). OPEN QUESTION for the owner: keep this rule, or make the wrappers silent on reset and give owners a separate `onreset`? The plan's value owner (rows 447 and 458) would re-dispatch a real `input` with the predicted level, which is this rule.

**Desktop** (reasoned from code, not run): both the listener and the read go through `document::eval`; the read is one IPC round trip later than on web, and the batching above holds the handlers until all values arrived, so it converges with a frame of lag, the same as the checkbox.

Known limit: a field unmounted between the reset and the read has its handler called on a dropped signal; the window is one task.

## Size

S

## Detail

- **Files owned:** To be found by the reproduction.
- **Depends on:** None. The plan's value owner (rows 447 and 458) subsumes it by construction: on `reset` it re-dispatches a real `input` on the control with the predicted post-reset level, so the owner's own rsx `oninput` sees it in order (Dioxus 0.7.9 also delivers `onreset`); a local fix (an `onreset` that sets the signal) is allowed meanwhile.
- **Proof:** A Playwright spec that fails before the fix and passes after; run on the web build and, where the bug is renderer-dependent, the desktop-cadence build of [row 442](./442-desktop-cadence-harness-and-desktop-pass.md).
- **Hours (inferred):** not estimated (size guess S).
