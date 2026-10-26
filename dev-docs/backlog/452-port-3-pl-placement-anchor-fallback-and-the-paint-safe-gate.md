---
id: 452
title: "Port 3.PL: placement (T1, T2, T8-T12, S1), the paint-safe gate and the boot probe"
status: "APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript…"
size: "L"
area: overlays
js-grant: js_file=24 (relocated lines, root doc section 5.4; the main loop's grant row at the merge base is the number the gate reads)
adds: []
updated: 2026-10-26
---

# Row 452: Port 3.PL: placement (T1, T2, T8-T12, S1), the paint-safe gate and the boot probe

## Item

Port the anchor-placement blocks (T8, T9, T10, T11, T12, S1, T1, T2) and build the G pieces that consume the executor: the pending CSS, the fail-open names, the per-surface list with each surface's paint tier and `engine: always|fallback`, the boot probe (Rust-rendered `popover=manual` nodes from first idle, a closed `documentElement.dataset` setter row, `html[data-dx-anchor]`) and the conditional `data-dx-anchored`. It consumes the executor and `popover_sync.rs` (item 0.4) and edits neither. Why: V19 and O3 (G) need its pending CSS, fail-open names and per-surface list.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript removed per hour.

## Size

L

## Detail

- **Files owned:** `top_layer/{anchor_fallback,scroll_clamp,styles,mod,attributes}.rs`, `select/components/list.rs`, `context_menu.rs` (the T11 call site only), the theme/engine CSS for anchoring and the gate, the popover/select/combobox component sheets. Must not touch `lib.rs`, `menu_sub.rs`, `navigation_menu.rs`, `modality.rs`; const names that `lib.rs` splices (`OPEN_FOCUS_JS`) stay stable.
- **Depends on:** [row 449](./449-phase-0-4-proof-port-b-the-executor-and-popover-spreads.md) (the executor), [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md), [row 445](./445-phase-0-2-the-legacy-freeze.md); a grant row.
- **Proof:** The boot-probe spec, a nested-submenu and a reopen x3 spec, the first-frame spec (this row's, not phase 0's), `g-latency`, top-layer and select specs on a release build; Firefox and WebKit via [row 442](./442-desktop-cadence-harness-and-desktop-pass.md).
- **Hours (inferred):** 28, a full-lane figure (inferred; 36-48 h of wall; the plan's sum subtracts the 5 h that went to item 0.4); removes about 755 lines, relocates 24 (27 per hour).
