---
id: 450
title: "Phase 0.5: __dxBridgeStats, a desktop smoke launch and the one latency session"
status: "APPROVED 2026-10-26 (owner: \"okay for the plan\"; scope 2, the recommended default, since no scope…"
size: "M"
area: tooling
js-grant: js_file_bytes=1470, bytes=1400 (bytes granted by the owner 2026-11-14 on the measured growth: ts/bridge.ts +1,336; 80 lines at 18.3 bundle bytes per source line, inferred ceiling: the stats half of ts/perf, which never ships; the main loop's grant row at the merge base is the number the gate reads)
adds: []
updated: 2026-11-14
---

# Row 450: Phase 0.5: __dxBridgeStats, a desktop smoke launch and the one latency session

## Item

The stats half of the row 443 kit plus the latency session: a desktop smoke launch config for the preview (the desktop build has never been built, row 23), a read-out that works on a release webview (release `dioxus-desktop` turns the inspector off, so `window.__dxBridgeStats` and `?dx-perf` cannot be read there: **the read-out is an environment variable** that makes Rust dump the stats to a file), the busy-case spin hook (moved here from the kit), a ping workload page at frame rate under a representative render, and a one-page owner script; then one session on one of the owner's devices. Why: the next-frame budget, the desktop-cadence delay and the pass baseline all come from one real desktop latency distribution, and the current bridge carries almost no traffic until ports land.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (phase 0).

## Status / trigger

APPROVED 2026-10-26 (owner: "okay for the plan"; scope 2, the recommended default, since no scope was named). Phase 0 of the row 434 step 3 plan; it starts once the rows ordered before the plan have landed (STATUS.md, "Next", owner 2026-10-26).

**Premise check, 2026-11-13.** This row was written before the decision kernel (row 509) and the witness harness (row 446) existed. [`research/premise-check-447-450-2026-11-13.md`](../research/premise-check-447-450-2026-11-13.md) lists which of its claims about the source are false or moved, where its construction changes, the grant columns it needs and its place in the lane order. A lane reads that section before its first edit.

**Code half LANDED 2026-11-14 with the batch of that date (lane `lane/bridge-stats-450`); the desktop half under "Open" waits for a machine with `webkit2gtk`.** Where the construction landed, per the premise check:
- The counters are in the shipped bridge: `channel()` in `ts/bridge.ts` counts `sent` and `recv` per channel and samples the page's share of a round trip (`turn`) in `window.__dxBridgeStats`; nothing is under `ts/perf/`. A message flagged `stats` is answered with that object, which is how Rust reads it (no new op, no new `Prop`).
- Rust owns the aggregation: `bridge/transport/stats.rs` (counts, bytes, the last 4,096 queue and round-trip samples, nearest-rank percentiles) and the hooks in `transport.rs`. `DX_BRIDGE_STATS=<file>` writes the read-out about once a second through a rename; `DX_BRIDGE_PING=1` adds one ask per frame. Timing and byte counts run only while the read-out is on.
- The busy-case spin is owned by Rust: `DX_BRIDGE_SPIN=<ms>` is read where `DX_BRIDGE_STATS` and `DX_BRIDGE_PING` are read and rides as a `spin` field on every pump message; the page burns that long per message each way from the first message it receives. The page reads no query string; a spec sets `window.__dxBridgeSpin` before load. Session channels (arrays of commands) carry no field and do not spin.
- Proof: Rust unit tests for the read-out file, the percentiles, the environment variable and the stats answer; `bun test primitives/ts-test`; `playwright/bridge-stats.spec.ts` (4 tests, Chromium: three on a blank page, one on the served site).
- Measured growth: `bridge.js` 28,438 to 29,008 bytes, `ts/bridge.ts` 52,382 to 53,718, `transport.rs` js_lit and `session.rs` sink unchanged by this row; no `types/bridge.d.ts` change.
- Cost of one counted message: about 160 ns of Rust for an asking message with the read-out on (three clock reads, one map insert, remove and two ring pushes; measured in a release micro-benchmark on the host, not in a webview) and a few ns with it off; about 85 ns of page time per received plus sent message (measured in bun).

**Open.**
- The workload is the environment's `DX_BRIDGE_PING` (a Rust loop in `stats.rs`): it runs under whatever render the owner opens, which is the representative render, and adds no route.
- The desktop launch config (`preview/desktop-latency.env`) and `dev-docs/latency-session.md` are unbuilt: `webkit2gtk` is absent here and the desktop build has never been built (row 23). The owner's first run confirms the build command, that the file appears and that `page` is not `null`.
- The read-out starts with the first bridge message (the pump), so a page that never uses the bridge writes no file.

## Size

M

## Detail

- **Files owned:** `primitives/src/ts/perf/**` (the stats half; it never ships as a bundle entry), `bridge/transport.rs` counters only (after 0.1), the preview's desktop launch config and workload page (new files), `dev-docs/latency-session.md` (new).
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) (`transport.rs` order). Feeds [row 443](./443-owner-measurement-kit-perf-recorder-and-impl-switch.md) and [row 442](./442-desktop-cadence-harness-and-desktop-pass.md).
- **Proof:** The owner's session itself (a person's time on a real device); the read-out's own smoke test.
- **Hours (inferred):** about 9: 8 in round 6, +1 for the environment-variable read-out and the busy-case spin hook (3 before that; building the desktop binary and running the session are the owner's, and are not provable in the lane container).
