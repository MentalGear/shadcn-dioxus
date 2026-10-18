---
id: 157
title: "InputOtp hard-codes a numeric keypad: inputmode=\"numeric\" and pattern=\"[0-9]*\" are fixed, so alphanumeric…"
status: "COLD (owner decision 2026-10-05): not scheduled, NEEDS OWNER APPROVAL before anyone picks it up.…"
size: "S-M (the comparison rates it M)"
area: inputs
updated: 2026-10-05
---

# Row 157: InputOtp hard-codes a numeric keypad: inputmode="numeric" and pattern="[0-9]*" are fixed, so alphanumeric…

## Item

`InputOtp` hard-codes a numeric keypad: `inputmode="numeric"` and `pattern="[0-9]*"` are fixed, so alphanumeric codes show a numeric keypad on phones

## Status / trigger

**COLD (owner decision 2026-10-05): not scheduled, NEEDS OWNER APPROVAL before anyone picks it up.** Was: BACKLOG (Add later) -- recorded 2026-10-05 from the shadcn comparison; small, listed as a filler; now removed from every pick-up list

## Size

S-M (the comparison rates it M)

## Detail

`primitives/src/input_otp.rs` (~lines 227-228) sets both attributes in the `owned` block on the one real `<input>`, the block `merge_attributes` applies after the caller's attributes, so a caller cannot override them today. There is no character filtering in Rust: the numeric-only shape rests on `inputmode` and `pattern` alone. shadcn's `InputOTP` takes a `pattern` (digits, digits and letters, letters only, or a custom regular expression) and the input mode follows it. **Fix:** a `pattern`/`inputmode` prop pair on the root with digits as the default (so nothing changes for existing users), a demo for the alphanumeric case, and a spec that types letters into an alphanumeric OTP and checks `:invalid` follows `pattern`. Check rows 63 and 64's click-to-caret and per-slot correction paths against a non-digit value before landing.
