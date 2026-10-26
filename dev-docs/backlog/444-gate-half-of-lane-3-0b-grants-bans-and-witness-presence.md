---
id: 444
title: "Phase 0.1: the gate half of lane 3.0 (grant rows, sink and setter bans, witness-presence check, FORBIDDEN_VERBS)"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "L"
area: tooling
updated: 2026-10-26
---

# Row 444: Phase 0.1: the gate half of lane 3.0 (grant rows, sink and setter bans, witness-presence check, FORBIDDEN_VERBS)

## Item

The enforcement half of the row 434 construct: the grant rule and the ratchet-base compare, bundle-byte counting, the landed gate's holes, `adds:` front matter, the `Program` seal and the `Program::Frozen{module}` variant, the `tsc`-API scans, the DOM-less `ts/behaviours/**` compile with the generated `Decision<E>` type, the witness-test presence check (a pair per site id and effect), the platform-rule-row TSV with its reader, the `WitnessViolation` constructor ban outside the probes, the lease rule, `FORBIDDEN_VERBS` on both enums, the verb-row and `REDUCERS` caps, budgets held in a TSV, `check-hot-events`, the clippy sink ban with per-site `expect`s, the `DataTransfer` setter ban and the `MountedData::{set_focus, scroll, scroll_to, scroll_to_with_options}` ban, the rsx native-slot gate (`value:`, `checked:`, `selected:`, `open:`, `hidden:` outside `initial_*`), the reasoned allowlist rows (V5, V8, V9, V11 and the two named exceptions), the fate table and the registration in `run-gates.sh` and `gates.md`. Why: without it the plan is a document, not a construct that makes the class unable to recur.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (phase 0).

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

L

## Detail

- **Files owned:** `scripts/check-bridge-behaviours.*`, `scripts/check-js-volume.*`, `scripts/lib/ratchet.py`, `scripts/check-hot-events.sh`, `scripts/build-js-bundle.sh`, `scripts/check-js-bundle.sh`, `scripts/run-gates.sh`, `scripts/check-installed-paths.sh`, `dev-docs/gates.md`, `clippy.toml`, `primitives/src/bridge/transport.rs` (the `Program` variants only), `primitives/tsconfig.json`, `primitives/src/ts/lib/register.ts`, `primitives/src/ts/bridge.ts` (the registration lines only). Must not touch component files, except the `expect` attribute lines.
- **Depends on:** Lane 3.0 merged (it lands with batch 29; tracked in [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) as item 0.0), lane VR (landed).
- **Proof:** Gate self-tests that turn red on their mutations; `check-bridge-behaviours` and `check-js-volume` on the merged tree; `scripts/run-gates.sh` full on the merge batch.
- **Hours (inferred):** about 17 (13 before round 5; +4 for the two new bans, the rule-row TSV and the registration change).
