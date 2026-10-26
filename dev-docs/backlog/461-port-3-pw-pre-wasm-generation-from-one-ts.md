---
id: 461
title: "Port 3.PW: generate the pre-wasm copies from one TypeScript file (L15, V1, V2)"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "S"
area: build
js-grant: none
adds: []
updated: 2026-10-26
---

# Row 461: Port 3.PW: generate the pre-wasm copies from one TypeScript file (L15, V1, V2)

## Item

Generate the three copies of the modality writer and the two of `PREPAINT_JS` from one `.ts`; the modality writer's runtime copy becomes a bridge-load internal; the identity checks become a generated-file check. The one class that stays JavaScript on purpose (code that must run before wasm).

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

S

## Detail

- **Files owned:** `primitives/src/modality.rs` (the writer, after 3.OV), `primitives/src/ts/modality.ts` (new), `preview/src/theme.rs`, `preview/index.html`, a new `scripts/build-prewasm.sh`, `scripts/check-open-focus.sh`, `scripts/check-theme-mode-writer.sh`. Must not touch `top_layer/`, the rest of `modality.rs`.
- **Depends on:** [row 456](./456-port-3-ov-overlay-core-escape-dismiss-open-focus.md) (`modality.rs` order), [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md).
- **Proof:** `check-open-focus` and `check-theme-mode-writer` on the generated copies; `touch-open-focus.spec.ts`; the theme specs.
- **Hours (inferred):** 5 (inferred); removes 0 lines.
