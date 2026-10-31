---
id: 474
title: "Bug: Input OTP paste loses digits because maxlength truncates before the filter runs"
status: "APPROVED 2026-10-26 by the owner (\"Ok for the rest\", backlog thread); for the next session (owner…"
size: "S"
area: inputs
updated: 2026-10-31
---

# Row 474: Bug: Input OTP paste loses digits because maxlength truncates before the filter runs

## Item

The Input OTP filter's `beforeinput` test requires `typeof e.data === 'string'` (`input_otp.rs:376-380`), but `data` is null for `insertFromPaste`: the pasted content is in `dataTransfer` (W3C Input Events, overview). The input also carries `maxlength` (`input_otp.rs:423`), and the browser trims a paste to `maxlength` first (inferred engine behaviour). So a pasted code with separators, "123-456", becomes "123-45" before any filter sees it, and filters to "12345": a digit is lost. Not yet reproduced.

**Where:** web and desktop, any engine that trims a paste to `maxlength`; reproduce on Chromium first.

**Reproduce first:** paste "123-456" into a 6-slot Input OTP and read the value; paste "123 456" and "12-34-56" too, and compare with the expected six digits.

**Source:** [`board-2026-10-26-r6/order-slots.md`](../research/board-2026-10-26-r6/order-slots.md) finding 14 (the synthesis cites it as "order-slots 10"; the report numbers it 14; carried by [`board-2026-10-26-r6/SYNTHESIS.md`](../research/board-2026-10-26-r6/SYNTHESIS.md), "Live defects found in today's code").

Related to [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) but not part of it: the plan's `otpFilter` (proof port A, [row 448](./448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md)) subsumes this defect, and a local fix does not wait for the plan.

## Status / trigger

APPROVED 2026-10-26 by the owner ("Ok for the rest", backlog thread); for the next session (owner 2026-10-26: "We will do it all in a fresh session").

FIXED on `lane/otp-paste-474` (2026-10-31), awaiting the merge batch. Reproduced on Chromium first (a build of the base): pasting `123-456` into the six-slot control gave `12345`, `123 456` gave `12345`, `12-34-56` gave `1234`, and `aB3-xY9` into the alphanumeric control gave `aB3xY`: the browser trims a paste to `maxlength` (`maxlength="6"` on the element) when it inserts it, before the `input` filter runs, so separators used up slots. Fix: the `<input>` carries no `maxlength`; the page-side filter in `InputOtp` applies the pattern and the length bound to the text of a `beforeinput` (cancel, insert the accepted part over the selection) and again to the DOM value on `input`, and the Rust `oninput` bounds the signal. A keystroke into a full value stays a no-op. The block stays at 23 JS lines (baseline unchanged). Specs: `playwright/input-otp-pattern.spec.ts` (5 of 12 red on the base build, 12 of 12 green after).

## Size

S

## Detail

- **Files owned:** To be found by the reproduction; expect `primitives/src/input_otp.rs`.
- **Depends on:** None. The plan's `otpFilter` takes `insertFromPaste` and `insertReplacementText` itself (cancel, filter, insert), which fixes it by construction; a local fix is to drop `maxlength` from the element and enforce the length in the filter, or to handle `paste` by reading the clipboard text in the filter.
- **Proof:** A Playwright spec that fails before the fix and passes after (a paste with separators fills all six slots); run on the web build.
- **Hours (inferred):** not estimated (size guess S).
