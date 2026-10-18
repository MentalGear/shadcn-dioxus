---
id: 64
title: "Input OTP: clicking a slot and typing didn't correct that digit — LANDED 2026-09-17, user-reported. Row 63…"
status: "Landed 2026-09-17"
size: "S"
area: build
updated: 2026-09-17
---

# Row 64: Input OTP: clicking a slot and typing didn't correct that digit — LANDED 2026-09-17, user-reported. Row 63…

## Item

**Input OTP: clicking a slot and typing didn't correct that digit — LANDED 2026-09-17, user-reported.** Row 63 (same day) fixed the OTP input's click-to-caret mapping, but the caret it placed was a *collapsed* one — correct for positioning, wrong for the user's next step: typing over it. Reproduced live before writing any fix: on a full value `"123456"` (at `maxlength`), clicking slot 2 then typing `9` left the value completely unchanged — the keystroke was silently swallowed; on a partial value `"123"`, clicking slot 1 then typing `9` produced `"1923"` (inserted, shifting `"23"` right), not the expected `"193"` (that slot's digit replaced). **Cause:** a collapsed caret makes native typing *insert* at that position; native *replace* semantics only kick in when an actual selection range covers the character about to be typed over — and insert additionally hits a silent `maxlength` block outright once the value is already full, explaining why the full-value case did nothing at all. This is exactly why upstream `input-otp` never trusts a bare caret for this: **fixed** in the same `snap_caret_to_slot` eval (`primitives/src/input_otp.rs`, row 63's own function) by selecting the one character at the clicked slot — `setSelectionRange(clamped, clamped + 1, 'forward')` — whenever `clamped < value_len` (a real character sits there); when `clamped == value_len` (clicked past the last real character, the ordinary "continue typing" case), the collapsed-caret behavior stays, since there's nothing there to select. **Direction chosen deliberately:** `'forward'` (anchor at `clamped`, focus at `clamped + 1`), matching this component's own left-to-right reading order (`ArrowRight` increases the index, `Delete` already acts forward from the caret) and giving Shift+Arrow a sensible extend-direction if this control is ever driven that way — verified to have zero effect on the *unshifted* nav this module already tests, since unshifted `ArrowLeft`/`ArrowRight` on any selection collapse to that selection's edge regardless of stored direction (the pre-existing "Backspace/Delete/ArrowLeft/ArrowRight" test passes unmodified, confirming this). **Verified by execution, independently re-checked by the orchestrating session:** `cargo check --workspace`/`cargo fmt --check` clean; all three composition/class-prefix/css-literal lint scripts clean; a genuine `git stash`/unstash before/after comparison reproduced the exact broken values above pre-fix (`"123456"` unchanged, `"1923"`) and the exact corrected ones post-fix (`"129456"`, `"193"`); the empty-field and last-character-replace cases were also checked and behave correctly (clicking past an empty value's end still collapses to a plain caret, since nothing exists there to select; replacing the last filled character doesn't insert past `maxlength`); 4 new regression tests added to `playwright/input_otp.spec.ts` under a new "Click-then-type replace" block — full suite **15/15 passed** on Chromium (11 pre-existing unchanged + 4 new), independently re-run and re-confirmed by the orchestrating session with its own from-scratch click-then-type measurements at all three value states, matching the subagent's numbers exactly

## Status / trigger

Landed 2026-09-17

## Size

S

## Detail

User-reported, found while re-testing row 63's fix on the live site; delegated to a single Sonnet subagent (files: `primitives/src/input_otp.rs`, `playwright/input_otp.spec.ts` only, same ownership as row 63's lane, run sequentially after it rather than in parallel since it edits the exact same function), independently re-verified — cargo/lint, the full existing+new suite, and the orchestrating session's own from-scratch click-then-type measurements, not just the subagent's reported numbers — before landing
