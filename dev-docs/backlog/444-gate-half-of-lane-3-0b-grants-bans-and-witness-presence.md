---
id: 444
title: "Phase 0.1: the gate half of lane 3.0 (grant rows, sink and setter bans, FORBIDDEN_VERBS, gates over existing artefacts only)"
status: "LANDED in PR #167 (2026-11-07), phase 0.1's gates over the artefacts that exist: the grant rule and…"
size: "L"
area: tooling
js-grant: js_file=10 (owner 2026-11-06: lane B2 closes the bridge global and registration; measured +10 lines: register.ts +1, ts-test/bridge.test.ts +9)
adds: []
updated: 2026-10-26
---

# Row 444: Phase 0.1: the gate half of lane 3.0 (grant rows, sink and setter bans, FORBIDDEN_VERBS, gates over existing artefacts only)

## Item

The enforcement half of the row 434 construct: the grant rule and the ratchet-base compare, bundle-byte counting, the landed gate's holes, `adds:` front matter, the `Program` seal and the `Program::Frozen{module}` variant, the `tsc`-API scans, the tsc scope-analysis rules for `ts/behaviours/**` (the DOM-less program, the generated `Decision<E>` type and the `decide` registration are item 0.3's, with the witness-test presence check, the `site!` scan and the scenario-binding check: this item builds a gate only over artefacts that exist when it runs), purity by tsc scope analysis (no `any`, `constructor`, `__proto__` or `prototype`) and the closed `window.__dxBridge` (no `register`, no live table, no verbs after boot), the platform-rule-row TSV (a closed category table, `observable_on` staleness, the `desktop-pass.md` check-line count) with its reader, the `WitnessViolation` constructor ban outside the probes, the lease rule (the check on the generated file lands with item 0.3), `FORBIDDEN_VERBS` (this item is its **one owner**; item 0.7's slot table partitions it, on both enums), the verb-row and `REDUCERS` caps, budgets held in a TSV, `check-hot-events`, the clippy sink ban with per-site `expect`s, the `DataTransfer` setter ban (the `MountedData::{set_focus, scroll, scroll_to, scroll_to_with_options}` ban is conditional on item 0.6's focus-write witness and is not built here), the native-slot sink scan (an rsx and `Attribute` scan for `value`, `checked`, `selected`, `open` and `hidden` on page-writable elements outside the mount-frozen helper, with a shrink-only baseline of the shipped sites; not clippy, because rsx attributes expand to consts), the reasoned allowlist rows (V5, V8, V9, V11 and the two named exceptions), the fate table and the registration in `run-gates.sh` and `gates.md`. Why: without it the plan is a document, not a construct that makes the class unable to recur.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (phase 0).

## Status / trigger

**LANDED in PR #167 (2026-11-07), phase 0.1's gates over the artefacts that exist:** the grant rule and the ratchet base (A1: `scripts/lib/rows.py`, `ratchet.py`, `check-ratchet-grant`; a raise needs an open row at the base with `js-grant`), what `check-js-volume` counts (A2: bytes of JS files, md/svg scanned, sinks split by kind, `expect_disallowed`, the clippy pin, the reasoned allowlist rows V5, V8, V9, V11 and `theme_seed` the owner approved), the TypeScript scanner (B1: `check-ts-scan`, `typescript` pinned exactly at 6.0.3 in `scripts/ts-tools/`), the `check-bridge-behaviours` core (B2: budgets TSV, `FORBIDDEN_VERBS`, `Program::Frozen`, frozen `__dxBridge`, `adds:` rows; its `js_file=10` raise is this row's grant), the clippy ban on the page-script sink and the `DataTransfer` setters with 77 per-site `#[expect]`s (D), `check-hot-events` (E), `check-native-slots` (F), the installed-components sink check (G) and the platform-rule reader (H). Open, because the artefacts they read are built by later items: the `REDUCERS` cap, the `desktop-pass.md` count (row 442), platform-rule rows (owner decisions on a `site!`, item 0.3), the `MountedData` ban (0.6), the `site!`, witness, scenario-binding, DOM-less program and lease checks (0.3), the slot-table partition (0.7); removing `register` from `__dxBridge` waits for 0.3's `decide`.

APPROVED 2026-10-26 (owner: "okay for the plan"; scope 2, the recommended default, since no scope was named). Phase 0 of the row 434 step 3 plan; it starts once the rows ordered before the plan have landed (STATUS.md, "Next", owner 2026-10-26).

Clippy sink-ban probe done 2026-11-06: `disallowed-methods` fires on `document::eval` and the `DataTransfer` setters at every site kind, a per-site `#[expect]` works on statements, closures and rsx attribute values, 69 eval sites and 14 setter calls today; see [`clippy-sink-ban-probe-2026-11-06.md`](../research/clippy-sink-ban-probe-2026-11-06.md).

`check-bridge-behaviours` core done 2026-11-06 (lane B2): budgets are a `Ratchet` over `scripts/check-bridge-behaviours.budgets.tsv` (bytes; the dict is gone); `Program::Frozen{module}` over an empty `FrozenModule` with the three fixed `document::eval` argument shapes checked and the callers table `check-bridge-behaviours.callers.tsv` (one row, `js_listener.rs`); `FORBIDDEN_VERBS` (every `Method` variant, named by no Rust file outside `bridge/`, `METHODS` private to `bridge.ts`); a frozen, non-configurable `window.__dxBridge` (source and bundle) and a `KINDS` check in `lib/register.ts` (the bridge's own `register` stays until item 0.3); the Rust half of the lease rule; the `WitnessViolation` constructor ban outside `playwright/witness/probes/`; the `adds:` rule against the merge base (`scripts/lib/rows.py`); the platform-rule reader wired in; a missing TypeScript compiler is red under `RATCHET_REQUIRE_BASE=1`. The `tsc`-API scans stay in `check-ts-scan`. The three `document::eval` sites of `transport.rs` carry `#[expect(clippy::disallowed_methods)]`, which is unfulfilled (a warning, red under `-D warnings`) until lane D's `clippy.toml` ban is merged with it.

## Size

L

## Detail

- **Files owned:** `scripts/check-bridge-behaviours.*`, `scripts/check-js-volume.*`, `scripts/lib/ratchet.py`, `scripts/check-hot-events.sh`, `scripts/build-js-bundle.sh`, `scripts/check-js-bundle.sh`, `scripts/run-gates.sh`, `scripts/check-installed-paths.sh`, `dev-docs/gates.md`, `clippy.toml`, `primitives/src/bridge/transport.rs` (the `Program` variants only), `primitives/tsconfig.json`, `primitives/src/ts/lib/register.ts`, `primitives/src/ts/bridge.ts` (the registration lines only). Must not touch component files, except the `expect` attribute lines.
- **Depends on:** Lane 3.0 merged (it lands with batch 29; tracked in [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) as item 0.0), lane VR (landed).
- **Proof:** Gate self-tests that turn red on their mutations; `check-bridge-behaviours` and `check-js-volume` on the merged tree; `scripts/run-gates.sh` full on the merge batch.
- **Hours (inferred):** about 36 (30-45), by artefact: grant rule and ratchet rewrite 6, bundle-byte counting 3, the `tsc`-API scans 5, purity and the closed global 4, the platform-rule TSV and reader 3, the clippy sink ban with per-site `expect`s 4, the native-slot scan with its baseline 3, the hot-events baseline 2, `FORBIDDEN_VERBS` 2, the `Program` seal, registrations and `gates.md` 4. It was 20 in round 6, which counted the gates at about an hour each and included checks over artefacts that items 0.3 and 0.7 create. **Phase 0 is a serial chain through `ts/bridge.ts`, so a slip here moves every later item.**
