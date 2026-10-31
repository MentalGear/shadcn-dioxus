---
id: 452
title: "Port 3.PL: placement (T1, T2, T8-T12, S1), the paint-safe gate and the boot probe"
status: "APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript…"
size: "L"
area: overlays
js-grant: js_file=24 (relocated lines, root doc section 5.4; the main loop's grant row at the merge base is the number the gate reads)
adds: []
updated: 2026-10-31
---

# Row 452: Port 3.PL: placement (T1, T2, T8-T12, S1), the paint-safe gate and the boot probe

## Item

Port the anchor-placement blocks (T8, T9, T10, T11, T12, S1, T1, T2) and build the G pieces that consume the executor: the pending CSS, the fail-open names, the per-surface list with each surface's paint tier and `engine: always|fallback`, the boot probe (Rust-rendered `popover=manual` nodes from first idle, a closed `documentElement.dataset` setter row, `html[data-dx-anchor]`) and the conditional `data-dx-anchored`. It consumes the executor and `popover_sync.rs` (item 0.4) and edits neither. Why: V19 and O3 (G) need its pending CSS, fail-open names and per-surface list.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript removed per hour (yield to be re-measured when taken; row 477 removed or merged most of the original ledger).

## Size

L

## Detail

Re-scoped 2026-10-31 against the tree after [row 477](./477-anchored-overlays-placed-in-document-coordinates.md) (PR #156) and [row 479](./479-scroll-lock-method-and-the-touch-carve-out.md). The ledger items in the item text above are stale; what is left to port:

- **T8** is one `PLACE_JS` (`primitives/src/top_layer/anchor_fallback.rs:39`, in a 236-line file): rule P, document coordinates, transform-only, one `requestAnimationFrame` per frame, CSS anchoring (`anchor.css`) kept as the first placement. **T9** (`VIEWPORT_METRICS_JS`) and **T10** (`MEASURE_AT_FULL_WIDTH_JS`) no longer exist. **T11** (the ContextMenu point) is `Anchor::Point` on the same `PLACE_JS` (`use_point_anchor_placement`, `scroll_clamp.rs:179`), not a separate block. **T12** close-on-scroll is Tooltip-only (`ScrollClose::AnyScroll`, `scroll_clamp.rs:96`; its one user is `tooltip.rs:497`). **S1** is `ITEM_ALIGNED_JS` (`select/components/list.rs:114`), which uses the shared placement helpers (`PLACEMENT_HELPERS_JS`, `scroll_clamp.rs:45`). T1 and T2 (the static stylesheets) are unchanged.
- **Still valid:** the boot probe (Rust-rendered `popover=manual` nodes from first idle, a closed `documentElement.dataset` setter row, `html[data-dx-anchor]`), the paint-safe gate and its pending CSS and fail-open names, the per-surface list with `engine: always|fallback`, the conditional `data-dx-anchored`, and the move of the remaining JS strings (`PLACE_JS`, `ITEM_ALIGNED_JS`, the close-on-scroll script) into TS modules or Rust. The placement gate `scripts/check-anchored-placement.sh` (rule P) already exists and the port keeps it green.
- **The old figure is withdrawn:** "removes about 755 lines, relocates 24 (27 per hour)" counted blocks that 477 deleted or merged. Re-measure the removed and relocated lines, and the yield, against the files when the row is taken.
- **Files owned:** `top_layer/{anchor_fallback,scroll_clamp,styles,mod,attributes}.rs`, `select/components/list.rs`, `context_menu.rs` (the point-anchor call site only), the theme/engine CSS for anchoring and the gate, the popover/select/combobox component sheets. Must not touch `lib.rs`, `menu_sub.rs`, `navigation_menu.rs`, `modality.rs`; const names that `lib.rs` splices (`OPEN_FOCUS_JS`) stay stable.
- **Depends on:** [row 449](./449-phase-0-4-proof-port-b-the-executor-and-popover-spreads.md) (the executor), [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md), [row 445](./445-phase-0-2-the-legacy-freeze.md); a grant row.
- **Proof:** The boot-probe spec, a nested-submenu and a reopen x3 spec, the first-frame spec (this row's, not phase 0's), `g-latency`, top-layer and select specs on a release build, `playwright/anchored-placement.spec.ts`; Firefox and WebKit via [row 442](./442-desktop-cadence-harness-and-desktop-pass.md).
- **Hours:** the plan's 28 h (inferred, a full-lane figure) predates 477; re-estimate when taken.
