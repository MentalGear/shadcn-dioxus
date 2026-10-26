---
id: 450
title: "Phase 0.5: __dxBridgeStats, a desktop smoke launch and the one latency session"
status: "APPROVED 2026-10-26 (owner: \"okay for the plan\"; scope 2, the recommended default, since no scope…"
size: "M"
area: tooling
js-grant: js_file=80 (inferred ceiling: the stats half of ts/perf, which never ships; the main loop's grant row at the merge base is the number the gate reads)
adds: []
updated: 2026-10-26
---

# Row 450: Phase 0.5: __dxBridgeStats, a desktop smoke launch and the one latency session

## Item

The stats half of the row 443 kit plus the latency session: a desktop smoke launch config for the preview (the desktop build has never been built, row 23), a read-out that works on a release webview (release `dioxus-desktop` turns the inspector off, so `window.__dxBridgeStats` and `?dx-perf` cannot be read there: **the read-out is an environment variable** that makes Rust dump the stats to a file), the busy-case spin hook (moved here from the kit), a ping workload page at frame rate under a representative render, and a one-page owner script; then one session on one of the owner's devices. Why: the next-frame budget, the desktop-cadence delay and the pass baseline all come from one real desktop latency distribution, and the current bridge carries almost no traffic until ports land.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (phase 0).

## Status / trigger

APPROVED 2026-10-26 (owner: "okay for the plan"; scope 2, the recommended default, since no scope was named). Phase 0 of the row 434 step 3 plan; the next session starts it after the approved bug rows.

## Size

M

## Detail

- **Files owned:** `primitives/src/ts/perf/**` (the stats half; it never ships as a bundle entry), `bridge/transport.rs` counters only (after 0.1), the preview's desktop launch config and workload page (new files), `dev-docs/latency-session.md` (new).
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) (`transport.rs` order). Feeds [row 443](./443-owner-measurement-kit-perf-recorder-and-impl-switch.md) and [row 442](./442-desktop-cadence-harness-and-desktop-pass.md).
- **Proof:** The owner's session itself (a person's time on a real device); the read-out's own smoke test.
- **Hours (inferred):** about 9: 8 in round 6, +1 for the environment-variable read-out and the busy-case spin hook (3 before that; building the desktop binary and running the session are the owner's, and are not provable in the lane container).
