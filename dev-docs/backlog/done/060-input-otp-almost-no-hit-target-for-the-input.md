---
id: 60
title: "Input OTP: almost no hit target for the input — LANDED 2026-09-17, user-reported. Not a geometry/sizing bug…"
status: "Landed 2026-09-17"
size: "S"
area: build
updated: 2026-09-17
---

# Row 60: Input OTP: almost no hit target for the input — LANDED 2026-09-17, user-reported. Not a geometry/sizing bug…

## Item

**Input OTP: almost no hit target for the input — LANDED 2026-09-17, user-reported.** Not a geometry/sizing bug, despite the report's own framing: the real, invisible (`opacity: 0`) `<input>` in `primitives/src/input_otp.rs` was already correctly sized to match its full visual row (confirmed unchanged: `position: absolute; inset: 0; width/height: 100%`). The actual cause was paint order — a purely decorative, `aria-hidden` overlay `div` (the slot/separator visuals, rendered *after* the real input in the same stacking context) covers almost the entire row and, with the browser default `pointer-events: auto`, silently intercepts clicks before they reach the input beneath it. Confirmed directly via `elementFromPoint`: clicking dead-center of a visible slot resolved to that slot's own decorative `<div>`, not `#otp-main`; only the few-pixel gaps *between* slots (touched by no decorative box) ever reached the real input — exactly matching the report's "almost no hit target." **Fixed** with a one-line `pointer_events: "none"` on the overlay wrapper `div`, inherited by every descendant (no child sets its own `pointer-events`), making the whole decorative layer transparent to hit-testing to match how it's already transparent visually. New regression coverage in `playwright/input_otp.spec.ts`: a full-row bounding-box geometry check, and a real hit-test clicking all 6 slots' centers *and* corners, confirming every point now focuses the real input. **Verified by execution, independently re-checked by the orchestrating session:** `cargo check --workspace`/`cargo fmt --check` clean; all three composition/class-prefix/css-literal lint scripts clean; full `input_otp.spec.ts` suite re-run directly against a live `dx serve` instance — **7/7 passed on Chromium** (5 pre-existing unchanged + 2 new), independently confirming the subagent's own claimed pass (not just trusting its report)

## Status / trigger

Landed 2026-09-17

## Size

S

## Detail

User-reported; delegated to a Sonnet subagent alongside row 59's sidebar-cluster lane and a checkbox-padding audit (disjoint files: `primitives/src/input_otp.rs` + `playwright/input_otp.spec.ts` only), independently re-verified — cargo/lint/Playwright, no shortcuts taken on the subagent's claims — before landing
