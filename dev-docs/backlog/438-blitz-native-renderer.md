---
id: 438
title: "dioxus-native (Blitz) as a supported renderer: parked, research-gated, after row 434"
status: "PARKED. Start when all of these hold: 1. Dioxus 0.8 is stable and the primitives build on it. 2.…"
size: "Estimates from the research: tier 0 M…"
area: primitives
updated: 2026-11-12
---

# Row 438: dioxus-native (Blitz) as a supported renderer

## Item

The owner asked on 2026-10-25 whether the library could target Blitz, the native renderer that has no JavaScript. The research is in [`../research/blitz-feasibility-2026-10-25.md`](../research/blitz-feasibility-2026-10-25.md).

- `document::eval` on dioxus-native returns `Err(EvalError::Unsupported)`, so every eval path needs a Rust arm. Row 434 (Rust first) is the prerequisite.
- Hard blockers in Blitz 0.3.0-beta.4 / dioxus-native 0.8.0-alpha.3 (2026-10-16):
  - no popover or top layer and no `showModal` (Blitz issue #196);
  - `:focus-visible` and `:has()` hard-coded off;
  - no `contenteditable`, so no DatePicker segments;
  - no pointer capture;
  - no resize, toggle or animation events.
- What works: `position: fixed`, transitions and keyframes, `:hover`/`:focus`, the hover and colour-scheme media queries, and `MountedData` rects, scroll and focus, but only on the Dioxus 0.8 line. Inline SVG is static.
- The repo already has a native arm behind `#[cfg(feature = "native")]` (the primitives' opt-in Blitz feature, which row [511](./511-primitives-web-feature-contract.md) introduced) that renders overlays inert.
- **A named target (owner 2026-11-11).** Blitz is where native goes after the webview (decisions.md, "Renderer
  targets"). The start conditions below are unchanged: this row still waits for Dioxus 0.8, for Blitz's top layer,
  `:focus-visible` and `:has()`, and for a test strategy.
- **What row 434 leaves ready for this row.** Every named behaviour's decision is a pure Rust function that
  `dioxus-primitives` can call directly, with no bridge (row [509](./509-rust-decision-contract.md), item 1), and
  `check-native-slots` and `check-hot-events` allow an rsx slot write and an rsx scroll or pointer handler under
  the Blitz arm's cfg (item 5), spelled `#[cfg(feature = "native")]`. Row [511](./511-primitives-web-feature-contract.md)
  has landed, so this row inherits its single opt-in, `dioxus-primitives/native` next to `dioxus/native` (the
  primitives' feature forwards `dioxus/native`, and a preview or starter feature that enables `dioxus/native` must also
  enable `dioxus-primitives/native`, which `check-cfg-axis` asserts), the compile-time assert in `primitives/src/lib.rs`
  that holds the two equal, `NATIVE_ARM` for styled components to branch on in expression position, and the
  `clippy-native` gate that lints the arm against the real dioxus-native renderer.
- **What this row still builds, per behaviour.** A Blitz adapter: deliver the event from an rsx handler, call the
  behaviour's `decide`, apply the effects it returns through rsx and `MountedData`. Also a Rust arm for each kind the
  page executor reconciles (row [449](./449-phase-0-4-proof-port-b-the-executor-and-popover-spreads.md)), and a
  renderer-native answer for each bridge verb a component reads through (`get`, `call`, `listen`, `observe`); until
  one exists the transport answers "unsupported" and the feature degrades.
- **The `feature = "native"` eval sites.** Row [448](./448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md) deleted the
  three focus-trap evals in the native arms of `dialog.rs`, `alert_dialog.rs` and `popover.rs`; they did nothing on Blitz,
  where `document::eval` is unsupported. The Blitz modal overlays are a `div` with `aria-modal` and no focus trap, so a real
  Blitz focus trap is built here.

## Status / trigger

PARKED. Start when all of these hold:
1. Dioxus 0.8 is stable and the primitives build on it.
2. Blitz ships `:focus-visible`, `:has()` and either popover or a documented top-layer replacement.
3. A written test strategy exists for a renderer with no DOM oracle, because the Playwright suite and the gates are browser-only.

## Size

Estimates from the research: tier 0 M, static components M-L, interactive non-overlay L, overlays XL, and charts and DatePicker blocked.

## Detail

Per-tier "claim support" gates are in section 4.4 of the research note. Re-check Blitz issues #196 and #887 when the trigger is evaluated; the sandbox could not read them on 2026-10-25.
