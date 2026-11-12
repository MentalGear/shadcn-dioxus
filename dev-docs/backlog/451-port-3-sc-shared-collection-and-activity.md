---
id: 451
title: "Port 3.SC: shared collection and activity scripts (S7, S15)"
status: "Stage 1 built on lane/port-sc-451, awaiting merge: the pure decisions are written; the bridge…"
size: "M"
area: primitives
js-grant: none
adds: []
updated: 2026-11-12
---

# Row 451: Port 3.SC: shared collection and activity scripts (S7, S15)

## Item

Port S7 (`collection.rs`, with the per-row `onmounted` of `collection.rs:738`) and S15 (`activity.rs`) off bespoke eval. Highest yield of the candidates: about 30 JavaScript lines removed per agent-hour.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

Stage 1 built on `lane/port-sc-451`, awaiting merge: the pure decisions are written; the bridge additions wait for row 454. (APPROVED 2026-10-26, scope 2.)

**What the row found.** Neither script ports with the verbs on main: S7 needs a document-order read, S15 needs tab visibility, the `content-visibility` signal and two intersection fields (spec below). Section 5.2 of the root doc makes the row that first needs a verb the row that adds it, so this row owns the bridge additions; `js-grant: none` and the old "Files owned" line were wrong.

**Grant this row needs (inferred):** `js_file_bytes=1200` (the five additions below are about 450 bytes of TypeScript and about 350 bytes in the bundle, about 800 in all, rounded up). The ported `activity.rs` script (110 lines) and `collection.rs` script (8 lines) leave `js_lit`, so the global total `js_lit * 120 + js_file_bytes` falls by about 13,000. The front-matter `js-grant:` line stays as is; the main loop sets the grant on main, because the gate reads it at the merge base.

## Size

M

## Detail

- **Files owned:** `collection.rs`, `activity.rs`, and the bridge files for the additions below: `primitives/src/ts/bridge.ts`, its bundle `primitives/src/js/bridge.js` (with `bundle.manifest`), `primitives/src/ts/types/bridge.d.ts` where a message type changes, and `primitives/src/bridge/{mod,events,session}.rs`. Must not touch the rest. `collection.rs:520` is the roving hub that the conditional roving-focus row ([row 462](./462-port-roving-focus-78-set-focus-sites.md)) would route through the focus owner, so that row waits for this one.
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) (grant rule), [row 445](./445-phase-0-2-the-legacy-freeze.md) (legacy freeze), and row 454's observer changes in `bridge.ts` and `bridge/**`, which land first because both rows edit those files (the intersection fields sit in the same region); a grant for `js_file_bytes` (see Status).
- **Proof:** The components that use them (specs of `collection`/`activity` consumers on a release build); `check-js-volume` baseline rows lowered.
- **Hours (inferred):** 6 (inferred: 2 for the bridge additions, 2.5 for wiring and deleting both scripts, 1.5 for proof); removes about 118 lines of JavaScript (about 20 per hour).
- **Decision in Rust:** the logic this row moves is a pure Rust function over typed inputs; only reading and writing the page goes through the bridge ([`decisions.md`](../decisions.md), "Behaviour is decided in Rust").

### The per-row `onmounted` (`collection.rs:738`)

It stores the row's `MountedData` handle (a Rust-side value: `set_focus` goes through it) and calls `request_order_pass`. It reads nothing from the page and needs no bridge verb of its own: its order trigger rides the document-order read below, and the handle stays because the roving hub's focus writes use it (row 462 owns that). On desktop it is one blocking request per row; removing it means replacing the handle with a focus-owner write, which is row 462's, so this row leaves it.

### The decisions (stage 1, written, not yet wired)

- S15, `activity.rs` private module `decide`: `loop_active(tab_hidden, seen, skipped) -> Option<bool>` (unknown until the observer answers, a hidden tab is an answer), `document_active`, `entered`, `loop_seen`, `enter_seen` (the 0.4 ratio or 0.4 of the root height rule), `ENTER_THRESHOLDS`; `ROOT_MARGIN` and `ENTER_RATIO` stay the module's constants. 10 unit tests.
- S7, `collection.rs`: `apply_dom_order` is already pure Rust with tests (ascending no-op, fresh ascending range, creation order); `slots_from_markers(prefix, markers)` turns the read's attribute values into slots. 1 new unit test.

### Bridge additions this row makes

Each stays inside the section 5.2 closed surface: one member call or read per verb row, at most one guard; none is a registry program. Sizes are estimates (inferred) in bytes, the unit of the `js_file_bytes` grant column, which counts `.ts` source and the bundle together.

| # | Rust | TypeScript | receiver | fires | bytes (inferred) |
|---|---|---|---|---|---|
| 1. S7 document order | `Prop::AttrsOf { selector, attr }`, wire `attrsOf:{attr}\u0000{selector}` | a `read()` branch like `attr:`: `Array.from(el.querySelectorAll(sel), (n) => n.getAttribute(attr))`, one call, no guard | document (`Target::Document`) | once per order pass: an item mount or a registration effect, human rate | about 150 ts, 120 bundle |
| 2. tab visibility | `Prop::VisibilityState`, wire `visibilityState` | `PROPS` row `on("d", (el) => el.visibilityState)` | document | one read at subscribe, one per `visibilitychange` | about 70 ts, 55 bundle |
| 3. content-visibility signal | `EventKind::Contentvisibilityautostatechange`, wire `contentvisibilityautostatechange`, receivers `d`, not hot, listened with `ListenOptions::CAPTURE` and no fields | an `EVENTS` row `"contentvisibilityautostatechange": { recv: "d" },` | document | when a `content-visibility: auto` subtree starts or stops being skipped: scroll-driven, bursty, but each is one message and Rust coalesces them to one batch of reads per frame | about 55 ts, 50 bundle |
| 4. is the element skipped | `Prop::ContentSkipped`, wire `contentSkipped` | `PROPS` row `on("e", (el) => !el.checkVisibility({ contentVisibilityAuto: true }))`, one call | element (the subscribed one) | read at subscribe, and for every subscribed element after a batch of event 3 | about 85 ts, 70 bundle |
| 5. intersection fields | `Intersection { intersection_height: f64, root_height: f64 }` | two more members in the existing observer message: `intersectionHeight: e.intersectionRect.height`, `rootHeight: e.rootBounds ? e.rootBounds.height : W.innerHeight` (the old script's own fallback) | the observed element | with every observer callback, as today | about 100 ts, 55 bundle |

**How Rust learns that a subscribed element is inside a skipped subtree.** The old script kept a set of skipped roots from the event's `target` and tested containment. The smallest construction that keeps the decision in Rust is a trigger plus a read: the event (3) carries no field and says only "something changed"; Rust then reads `contentSkipped` (4) for each subscribed element and decides with `loop_active` / `entered`. `checkVisibility({ contentVisibilityAuto: true })` is false exactly for an element in a skipped `content-visibility: auto` subtree (or one with no rendered box, which the observer already reports as not intersecting, so the decision is unchanged). A page-side reducer would keep the old containment set on the page and send a bool per element, which is a registry program with state over a stream (the plan's "a reducer that needs state over a stream is Rust's"), so it is not used. The cost is N reads per batch of events for N subscribed elements, in one `get` message each; the batch is coalesced to one per frame in Rust.

Behaviour differences to confirm at stage 2 (findings, not decided here): (a) an element that is already skipped when it subscribes is read as skipped, where the old script learned it only from a later event; (b) each subscriber owns its observer, where the old registry shared one per kind, which changes callback counts, not answers; (c) event 3 on an engine without it never fires, as before (the read in 4 is `true` there).

Row 3.OB's S16 (`checkVisibility`) wants the same member; if it lands first, row 4 reuses its row instead of adding `contentSkipped`.

**Renderer notes for the additions (inferred).** Web and webview: all five go through `document::eval` via the bridge. Blitz: `get`, `listen` and `observe` are `Unsupported` there, so S15 keeps its "active" default (the server's too) and S7 keeps creation order; a Blitz arm would need `onvisible`-style native visibility and a native document-order query, neither of which exists yet.
