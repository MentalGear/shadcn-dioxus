---
id: 451
title: "Port 3.SC: shared collection and activity scripts (S7, S15)"
status: "LANDED in PR #180 (2026-11-13) plus the follow-up that adds the js-interop rows: S7 and S15 run on…"
size: "M"
area: primitives
js-grant: js_file_bytes=1200, bytes=450 (measured on lane/port-sc-451 on top of row 454: js/bridge.js +324 and ts/bridge.ts +407 bytes for three closed rows; inferred ceiling: five closed bridge rows for S7 and S15, about 450 bytes of TypeScript plus about 350 in the bundle, with margin; the row that first needs a verb adds it, root doc section 5.2)
adds: []
updated: 2026-11-14
---

# Row 451: Port 3.SC: shared collection and activity scripts (S7, S15)

## Item

Port S7 (`collection.rs`, with the per-row `onmounted` of `collection.rs:738`) and S15 (`activity.rs`) off bespoke eval. Highest yield of the candidates: about 30 JavaScript lines removed per agent-hour.

Part of the [row 434](../434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

LANDED in PR #180 (2026-11-13) plus the follow-up that adds the js-interop rows: S7 and S15 run on the bridge; three closed rows added; the legacy scripts are gone. The per-row `onmounted` handle stays (see "The per-row `onmounted`"); [row 514](../514-the-per-row-mounted-handle-has-no-owner.md) owns it, and `scripts/check-hot-events.allow.tsv` excuses `mounted` in that row's name (owner 2026-11-14). The skipped-subtree paths could not be pinned by a spec (see "Not provable on Chromium"); that is a finding, not an open item. (APPROVED 2026-10-26, scope 2.)

**Grant (set on main by the main loop):** `js_file_bytes=1200`. Measured on top of row 454's own raise: `primitives/src/js/bridge.js` 22234 -> 22558 (+324 bytes) is the only `js_file_bytes` file this row raises; `bridge.d.ts` is unchanged. The TypeScript source `bridge.ts` grew 47097 -> 47504 bytes (+407), which the `check-bridge-behaviours` budget counts in canonical tokens, not `js_file_bytes`. The grant covers the raise (324 of 1200). Neither `check-js-volume` nor `check-bridge-behaviours` is re-baselined here.

## Size

M

## Detail

- **Files owned:** `collection.rs`, `activity.rs`, and the bridge files for the additions below: `primitives/src/ts/bridge.ts`, its bundle `primitives/src/js/bridge.js` (with `bundle.manifest`), `primitives/src/ts/types/bridge.d.ts` where a message type changes, and `primitives/src/bridge/{mod,events,session}.rs`. Must not touch the rest. `collection.rs:520` is the roving hub that the conditional roving-focus row ([row 462](../462-port-roving-focus-78-set-focus-sites.md)) would route through the focus owner, so that row waits for this one.
- **Depends on:** [row 444](../444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) (grant rule), [row 445](./445-phase-0-2-the-legacy-freeze.md) (legacy freeze), and row 454's observer changes in `bridge.ts` and `bridge/**`, which land first because both rows edit those files (the intersection fields sit in the same region); a grant for `js_file_bytes` (see Status).
- **Proof:** The components that use them (specs of `collection`/`activity` consumers on a release build); `check-js-volume` baseline rows lowered.
- **Hours (inferred):** 6 (inferred: 2 for the bridge additions, 2.5 for wiring and deleting both scripts, 1.5 for proof); removes about 118 lines of JavaScript (about 20 per hour).
- **Decision in Rust:** the logic this row moves is a pure Rust function over typed inputs; only reading and writing the page goes through the bridge ([`decisions.md`](../../decisions.md), "Behaviour is decided in Rust").

### The per-row `onmounted` (`collection.rs:738`)

It stores the row's `MountedData` handle (a Rust-side value: `set_focus` goes through it) and calls `request_order_pass`. It reads nothing from the page and needs no bridge verb of its own: its order trigger rides the document-order read below, and the handle stays because the roving hub's focus writes use it (row 462 owns that). On desktop it is one blocking request per row; removing it means replacing the handle with a focus-owner write, which is row 462's, so this row leaves it.

### The construction as built

- **S15, `activity.rs`.** `use_activity` keeps the same hooks (`use_motion`, `use_motion_when`, `use_motion_active_for`, `use_entered_view_when`, `use_document_visible`) and their signals. Per subscriber: one bridge `IntersectionObserver` (`rootMargin` `ROOT_MARGIN` for a loop; the eleven `ENTER_THRESHOLDS` for an entrance), whose entry feeds `decide::loop_seen` / `decide::enter_seen`; the element's skipped state is read with `Prop::CheckVisibility` when it is seen and again after a `contentvisibilityautostatechange`; the output is `decide::loop_active`, `document_active` or `entered`. The page-wide facts are shared: the first mounted subscriber holds the two document listeners (`visibilitychange`, then a `Prop::VisibilityState` read; `contentvisibilityautostatechange`, which moves a counter), through the `LISTENERS` / `TAB_HIDDEN` / `CV_EPOCH` globals; when it unmounts the next subscriber takes over. Deadline: none (R), one hop later than the page-side registry.
- **S7, `collection.rs`.** `request_order_pass` reads `Prop::AttrsOf` of the collection's marker attribute on the document (`marker_read`), turns the answer into slots with `slots_from_markers`, and feeds the unchanged `apply_dom_order`. Deadline: none; the initial focus placement already waits for `order_pending`.
- **The per-row `onmounted`** stays: it stores the `MountedData` handle that the roving hub's focus writes use (row 462) and asks for the order pass; it reads nothing from the page.

### The decisions

- `activity.rs` module `decide`: `loop_active`, `document_active`, `entered`, `loop_seen`, `enter_seen`, `ENTER_THRESHOLDS`; 10 unit tests.
- `collection.rs`: `apply_dom_order` (3 tests), `slots_from_markers` and `marker_read` (2 tests).

### Bridge additions this row made

Of the five additions specified at stage 1, row 454 had already added two: `Prop::VisibilityState` (tab visibility) and `Prop::CheckVisibility` (the skipped-subtree read: `checkVisibility({ contentVisibilityAuto: true })`, null where the browser has none, which the decision reads as not skipped). Those are reused. Three remain, each one member call or read with no guard, closed on both sides, with parity tests (`the_events_table_is_this_enum`, `the_props_table_is_this_enum`'s neighbour test for the parameterised read, the `ts-test` observer and `attrsOf` tests, `public-api.txt`):

| Rust | TypeScript | receiver | fires |
|---|---|---|---|
| `Prop::AttrsOf { attr, selector }`, wire `attrsOf:{attr} {selector}`; `Props::strings` | a `read()` branch like `attr:`: `Array.from(el.querySelectorAll(sel), n => n.getAttribute(attr))` | document or element | once per order pass |
| `EventKind::Contentvisibilityautostatechange` | `EVENTS` row `{ recv: "d" }` | document, listened with capture | when a `content-visibility: auto` subtree starts or stops being skipped |
| `Intersection { intersection_height, root_height }` | two members of the existing observer message (`rootBounds` falls back to `innerHeight`) | the observed element | with every observer callback |

### JavaScript removed and added (measured with `check-js-volume --report`)

Removed: `activity.rs` js_lit 108 -> 0 and install 1 -> 0; `collection.rs` js_lit 6 -> 0, sink 1 -> 0, expect_disallowed 1 -> 0: 114 lines, 1 sink, 1 install, 1 waiver. Added: `bridge.js` +324 bytes (`bridge.ts` +407 bytes of source). Global total `js_lit * 120 + js_file_bytes`: about -13,400.

### Differences from the old registry (findings)

1. An element already skipped when it comes into view is read as skipped (the old script learned it only from a later event).
2. One observer per subscriber, where the old registry shared one per kind. Cost reasoning (inferred, not measured on a desktop webview): a browser `IntersectionObserver` per element is the normal pattern and each observer holds one target, so the per-element work is the same as one shared observer with N targets; the bridge opens one eval per subscriber, as `use_js_listeners` did. What would have grown with N is the document-level work (N `visibilitychange` and N `contentvisibilityautostatechange` listeners, each one IPC message per event, and N skipped-state reads per event); that is why the document facts are shared through one leader, and why a skipped-state read runs only for subscribers currently in view. The cost class of the page-wide signals is therefore unchanged, and no shared observer per kind is needed.
3. The tab's initial visibility is a read after mount (one hop), where the old script read it synchronously at subscribe; until it answers the motion is the optimistic `true`, as before the observer answered.

### Renderer notes (inferred except where a spec ran)

- Web: runs, proven on Chromium by the specs below.
- Desktop and mobile webview: all reads and observers go through `document::eval` via the bridge, with the IPC hop per message; the decisions are the same Rust.
- Blitz: `get`, `listen` and `observe` answer `Unsupported`, so every subscriber keeps its optimistic default (active) and S7 keeps creation order, exactly as a failed eval did before. A native arm would need a visibility signal and a document-order query from the renderer.

### Proof

Built with `lane-ssg.sh` (the wasm contains `attrsOf`), Chromium only (the WebKit and Firefox builds are not installed in this container; the Chromium revision was linked under a scratch `PLAYWRIGHT_BROWSERS_PATH` because `/opt/pw-browsers` has revision 1243 where the pinned Playwright asks for 1223). Specs by name: `oracle/tier2-html/motion-gating`, `carousel-rewind-autoplay`, `chart-entry-animation`, `toast-pause`, `spinner`, `bridge-dom`, `attribute-listeners`, `tabs`, `toolbar`, `radio-group`, `tag_group`, `combobox`. The motion-gating spec's two probes of the old `window.__dxActivity` registry now assert the observable behaviour (the bridge is up before the parked check; the document listener inventory returns to its baseline).

### What `dev-docs/js-interop.md` needs

Done: the bridge tables list `AttrsOf`, `Contentvisibilityautostatechange` and the intersection height fields next to row 454's additions (`write`/`Slot`, `publish`, the viewport targets and fields).

### Not provable on Chromium (finding)

The two paths "an element already skipped at subscribe" and "a skipped-subtree change while in view" need an element for which the observer says "seen" while `checkVisibility({ contentVisibilityAuto: true })` says "skipped". Measured on the debug build (Chromium, a `content-visibility: auto` wrapper moved by a transform, viewport 10 px high, an `IntersectionObserver` with the registry's 200 px `rootMargin` beside the `checkVisibility` read): at wrapper offsets 140 to 160 px both say "in view" (`io` true, `cv` true); from 165 px on `cv` says skipped and the observer says not intersecting, though 200 px of margin would still cover the element up to 210 px. With a 50 px high viewport the observer turns false (at 260 px) before `cv` turns skipped (by 300 px), and with a 100 px wide by 50 px high viewport both flip between 220 and 240 px; no offset had the observer true and `cv` skipped. A skipped subtree has no box, so the observer drops the element in the same step. So `seen && skipped` never holds there; every outcome the registry produces (paused, then running again) comes from the observer alone, and a spec for either path would pass with the skipped logic removed. No such spec was written. The `contentvisibilityautostatechange` listener stays exercised by the listener-inventory case; whether the second signal earns its page-wide listener is a product question for the owner, not a test.
