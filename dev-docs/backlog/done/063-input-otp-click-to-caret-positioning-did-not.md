---
id: 63
title: "Input OTP click-to-caret positioning did not generalize past the first slot — LANDED 2026-09-17…"
status: "Landed 2026-09-17"
size: "S"
area: build
updated: 2026-09-17
---

# Row 63: Input OTP click-to-caret positioning did not generalize past the first slot — LANDED 2026-09-17…

## Item

**Input OTP click-to-caret positioning did not generalize past the first slot — LANDED 2026-09-17, user-reported.** Row 60 fixed the OTP input's hit-target (every slot's box now correctly hands a click through to the real, invisible `<input>` via `pointer-events: none` on the decorative overlay) but left a second, deeper bug the user then caught: clicking slot 0 felt normal, but no other slot did. Root-caused live, before writing any fix: on an **empty** field, clicking slot 0, 3, or 5 all placed the caret at `selectionStart` 0; on a field filled with `"123"`, clicking slot 0 landed at index 2 while slots 1/4/5 all landed at index 3. Cause: the real `<input>` is `opacity: 0` with `font-family: inherit` and no `letter-spacing` tuned to the visual 40px slot grid (`input_otp/style.css`'s `.dx-input-otp-input` sets none) — a native browser places a click's caret by mapping its x-coordinate to where it thinks the *actual rendered glyphs* are, and since those invisible glyphs were never spatially aligned to the visible boxes, that mapping is essentially arbitrary past the trivial "empty field, nothing to click past" case that made slot 0 look uniquely correct. This is exactly why upstream `input-otp` (the library shadcn/ui's own `InputOTP` wraps) never trusts native click-to-glyph placement at all — it computes the clicked slot from geometry and calls `setSelectionRange` explicitly, which is what this fix does too. **Fixed** with a new `onclick` handler on the real input (`primitives/src/input_otp.rs`) that reads the click's client coordinates and spawns a one-shot `document::eval` (`snap_caret_to_slot`, mirroring this same file's own `use_caret_sync` and `virtual_list.rs`'s `sync_container_scroll` for the "one-shot eval, not a persistent listener" shape) that does a plain point-in-rect test against every `[data-slot-index]` element's own `getBoundingClientRect()` — not `elementFromPoint`, which the row-60 `pointer-events: none` fix makes blind to the slots entirely — scoped to this specific `InputOtp` instance's own overlay sibling so more than one on a page stays correct, and needing no special-casing for `InputOtpGroup`/`InputOtpSeparator` grouping since every slot's own rect is tested regardless of which group it's in. **Clamping rule:** `min(clicked_slot_index, value.chars().count())` — not an arbitrary choice, the only one `HTMLInputElement.setSelectionRange` itself can honor, since it clamps its own arguments to `[0, value.length]`. Consequence, confirmed and treated as correct rather than a shortfall: on a genuinely empty value every slot's click still lands at index 0, because no real `<input>` can place a caret past text that doesn't exist yet — verified this doesn't make the UI lie about it (`data-active` stays `true` only on slot 0, `false` on every other clicked-but-necessarily-clamped slot). **Verified by execution, independently re-checked by the orchestrating session:** `cargo check --workspace`/`cargo fmt --check` clean; all three composition/class-prefix/css-literal lint scripts clean; before/after comparison via a genuine `git stash`/unstash of just this file (not hypothetical) reproduced the exact bug numbers above pre-fix and the exact corrected numbers post-fix; 4 new regression tests added to `playwright/input_otp.spec.ts` (empty-field clamping, partially-filled clamping across all 6 slots, fully-filled exact positioning, disabled-state no-op) — full suite **11/11 passed** on Chromium (7 pre-existing + 4 new); independently re-confirmed by the orchestrating session with its own live click test against a running `dx serve` instance, matching the subagent's numbers exactly at every slot for empty/partially-filled/fully-filled values

## Status / trigger

Landed 2026-09-17

## Size

S

## Detail

User-reported, found while re-testing row 60's fix on the live site; delegated to a single Sonnet subagent (files: `primitives/src/input_otp.rs`, `playwright/input_otp.spec.ts` only, disjoint from row 62's concurrent lane), independently re-verified — cargo/lint, the full existing+new suite, and the orchestrating session's own from-scratch click-position measurements, not just the subagent's reported numbers — before landing
