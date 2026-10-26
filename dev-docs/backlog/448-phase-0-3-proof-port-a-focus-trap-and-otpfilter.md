---
id: 448
title: "Phase 0.3: proof port A (the 16 dead not(web) eval sites, focus-trap.ts, and the otpFilter behaviour for S11)"
status: "APPROVED 2026-10-26 (owner: \"okay for the plan\"; scope 2, the recommended default, since no scope…"
size: "L"
area: overlays
js-grant: js_file=200 (inferred ceiling: otp_filter.ts about 30 relocated, the adapter extension in bridge.ts about 150, focus-trap.ts is deleted; the main loop's grant row at the merge base is the number the gate reads)
adds: [otpFilter]
updated: 2026-10-26
---

# Row 448: Phase 0.3: proof port A (the 16 dead not(web) eval sites, focus-trap.ts, and the otpFilter behaviour for S11)

## Item

The first listener port and the first named behaviour: lane 3.DJ (delete the 16 eval sites in `#[cfg(not(feature = "web"))]` arms that compile only for Blitz, where eval is a no-op, plus `ts/focus-trap.ts`, its `js/` output and manifest line, the allow-TSV entry and `focus_trap_script`), then `otpFilter` for S11 (25 lines, an element target, `beforeinput` missing in 0.7.9: its `prevent` and `setValue` exercise `defaultAction` and `wrongPaint`, so its witnesses are red by construction; it also takes `insertFromPaste` and `insertReplacementText`, which fixes the live OTP paste defect of [row 474](./474-bug-input-otp-paste-loses-digits-to-maxlength.md)). It also builds the registry restructure (one Rust file and one vectors file per behaviour, a registry macro that collects the variants), explicit `site!("name")` ids (a `Site` only the macro can construct), the per-install effect set that the adapter refuses beyond (round 7 cut the per-site typestate), the checks over those artefacts that item 0.1 leaves to this row (the `site!` literal scan, the scenario binding, the witness-test presence check, the DOM-less behaviour program with the generated `Decision<E>`, the `decide` registration, the lease check on the generated file) and the adapter for `otpFilter`. The cancel counterfactual here is site-relative and also drops the cancel when Dioxus lacks the event or a field the decision reads, so `otpFilter`'s witnesses (no `onbeforeinput` in 0.7.9) are red by construction and not by accident. `hotkey` is not built here (round 6: L6's witness is expected green by construction for an in-app target and it most likely decomposes into a bridge `listen` plus an rsx `prevent_default`; `hotkey` is built by the first of 3.OV and 3.GS that needs it). L23 is not here (it keeps the primitive's behaviour, row 3.FC). Why: it proves the construct end to end on a real block and deletes dead JavaScript first.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (phase 0).

## Status / trigger

APPROVED 2026-10-26 (owner: "okay for the plan"; scope 2, the recommended default, since no scope was named). Phase 0 of the row 434 step 3 plan; it starts once the rows ordered before the plan have landed (STATUS.md, "Next", owner 2026-10-26).

## Size

L

## Detail

- **Files owned:** `dialog.rs`, `alert_dialog.rs`, `popover.rs`, `dropdown_menu.rs`, `menubar.rs`, `context_menu.rs` (the `not(web)` arms only), `lib.rs` (the focus-trap bundle constants only), `ts/focus-trap.ts`, `primitives/src/js/focus-trap.js` and its manifest line, `input_otp.rs` (S11's script and install only), `ts/behaviours/otp_filter.ts`, `bridge/behaviours/mod.rs` (the registry macro, replacing `bridge/behaviours.rs`), `bridge/behaviours/otp_filter.rs`, `primitives/conformance/behaviours/otp_filter.json`, `bridge/session.rs` (`use_behaviour` only), `ts/bridge.ts` (the adapter's per-install effect-set refusal), `primitives/tsconfig.json`, the site scan and presence check under `scripts/check-bridge-behaviours.*` (after 0.1). Must not touch `top_layer/`, the rest of `lib.rs` (L1, L2 and L6 are 3.OV's), `label.rs`, the rest of `input_otp.rs` (3.FC).
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md), [row 446](./446-phase-0-6-witness-harness-and-the-first-witnesses.md) (the `otpFilter` witness `test.fixme` it turns green), [row 447](./447-phase-0-7-the-order-runtime.md) (`session.rs` order).
- **Proof:** `witness(otpFilter, prevent)` and `witness(otpFilter, setValue)` go from `test.fixme` to red-then-passing expected failures; the input-otp, dialog, alert-dialog and menu specs on a release build; `check-bridge-behaviours`.
- **Hours (inferred):** about 12: 13 in round 6, less about 3 for the typestate and 2 for the coverage hook that round 7 cut, plus about 4 for the checks that item 0.1 hands over (S11's 25 lines stay counted in 3.FC's relocated column).
