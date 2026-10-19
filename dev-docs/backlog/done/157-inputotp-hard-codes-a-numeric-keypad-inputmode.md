---
id: 157
title: "InputOtp hard-codes a numeric keypad: inputmode=\"numeric\" and pattern=\"[0-9]*\" are fixed, so alphanumeric…"
status: "LANDED 2026-10-19 (OtpPattern prop: inputmode, pattern attribute and filter derived from one value…"
size: "S-M (the comparison rates it M)"
area: inputs
updated: 2026-10-19
---

# Row 157: InputOtp hard-codes a numeric keypad: inputmode="numeric" and pattern="[0-9]*" are fixed, so alphanumeric…

## Item

`InputOtp` hard-codes a numeric keypad: `inputmode="numeric"` and `pattern="[0-9]*"` are fixed, so alphanumeric codes show a numeric keypad on phones

## Status / trigger

**LANDED 2026-10-19 (OtpPattern prop: inputmode, pattern attribute and filter derived from one value; demo, docs, spec).** Was COLD (owner decision 2026-10-05), approved by the owner for round 5; see "Update 2026-10-19".

## Size

S-M (the comparison rates it M)

## Detail

`primitives/src/input_otp.rs` (~lines 227-228) sets both attributes in the `owned` block on the one real `<input>`, the block `merge_attributes` applies after the caller's attributes, so a caller cannot override them today. There is no character filtering in Rust: the numeric-only shape rests on `inputmode` and `pattern` alone. shadcn's `InputOTP` takes a `pattern` (digits, digits and letters, letters only, or a custom regular expression) and the input mode follows it. **Fix:** a `pattern`/`inputmode` prop pair on the root with digits as the default (so nothing changes for existing users), a demo for the alphanumeric case, and a spec that types letters into an alphanumeric OTP and checks `:invalid` follows `pattern`. Check rows 63 and 64's click-to-caret and per-slot correction paths against a non-digit value before landing.

## Update 2026-10-19

Owner-approved and landed. `InputOtp` takes one typed prop `pattern: OtpPattern` (`Digits` default, `Alphanumeric`, `Letters`, `Custom { class, inputmode }`, mirroring shadcn's REGEXP_ONLY_DIGITS / _DIGITS_AND_CHARS / _CHARS). The `<input>`'s `inputmode`, its `pattern` attribute, the capture-phase JS filter (built from the same ranges: `[^...]`) and the Rust filter on `oninput` are all derived from it, so keyboard, validity and accepted characters cannot disagree; there is no raw-attribute path (the control's own attributes are applied after the caller's, so the typed prop wins; documented). `Custom.class` is a character-class body parsed by a small dependency-free parser (no regex crate). The filter runs synchronously in the input's own `input` event (an async write-back would race fast typing). Demo `alphanumeric`, docs.md "Accepted characters", unit tests `pattern_tests`, spec `playwright/input-otp-pattern.spec.ts` (letters typed, punctuation rejected with caret kept, digits-only rejects letters, paste filtered, patternMismatch follows pattern, click-to-slot correction with letters for rows 63/64, axe). Known limit: the browser applies `maxlength` to a paste before the filter, so a long paste with separators is truncated first.
