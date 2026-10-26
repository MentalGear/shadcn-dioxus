---
id: 450
title: "Phase 0.5: __dxBridgeStats, a desktop smoke launch and the one latency session"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "M"
area: tooling
updated: 2026-10-26
---

# Row 450: Phase 0.5: __dxBridgeStats, a desktop smoke launch and the one latency session

## Item

The stats half of the row 443 kit plus the latency session: a desktop smoke launch config for the preview (the desktop build has never been built, row 23), a read-out that works on a release webview (release `dioxus-desktop` turns the inspector off, so a launch config behind `?dx-perf` or a dump written to a file through Rust), a ping workload page at frame rate under a representative render, and a one-page owner script; then one session on one of the owner's devices. Why: the next-frame budget, the desktop-cadence delay and the pass baseline all come from one real desktop latency distribution, and the current bridge carries almost no traffic until ports land.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (phase 0).

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

M

## Detail

- **Files owned:** `primitives/src/ts/perf/**` (the stats half), `bridge/transport.rs` counters only (after 0.1), the preview's desktop launch config and workload page (new files), `dev-docs/latency-session.md` (new).
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) (`transport.rs` order). Feeds [row 443](./443-owner-measurement-kit-perf-recorder-and-impl-switch.md) and [row 442](./442-desktop-cadence-harness-and-desktop-pass.md).
- **Proof:** The owner's session itself (a person's time on a real device); the read-out's own smoke test.
- **Hours (inferred):** about 8 (3 before; +5 for the launch config, the read-out, the workload page and the script, inferred; building the desktop binary is the owner's).
