# Board lens: block-mapping (row 434 step 3), 2026-10-26

**Verdict: HOLDS WITH CHANGES.** The A/G/R split is right for most blocks, but several mappings do not match the code, and the lane plan has a compile-order break and unowned shared files.

- **Blocker 1:** T9/T10 are text constants spliced into T8, T11 and the `TOP_LAYER` scripts. 3.PL (batch 1) would delete them while T11 (3.CM, batch 2) still splices them.
- **Major:** the "one delegated capture-phase interpreter" collides with `listener-inventory.spec.ts` (no lazy page-lifetime blocking listeners), with bubble-phase `defaultPrevented` checks (label, sidebar), and with L11, which is stateful rather than a fixed predicate.
- **Major:** W8's Rust mode machine fed by Dioxus `onscroll`/`onresize` costs one blocking XHR per event on desktop (V). The plan rejects exactly that cost for C1.
- **Major:** L1/L2 do not need B1. `oncancel` and element `onkeydown` are plain Rust, sync, and the root doc's own rule §2.4 says to use them. L21 is mislabelled B2.
- **Major:** B2/B3/B5 are bespoke programs behind enum names (B2: 5 users, 4 output modes; B3: 1 user; B5: 3 different programs). "Residue = 0" is a relabel of about 1,040 lines.
- **Coverage:** the whole suite is Chromium-only (V), so A on Firefox/WebKit/iOS 26 and Safari 27's `overflow-anchor` cannot be proven by any existing spec. RTL, touch and reduced-motion gaps are listed in §3.
- **Order:** the backlog row already puts W8, C5, C1 last and one at a time, which is consistent with dependencies. Root doc §5.4 and residue §5 still show the old parallel batch 3.

Labels: V = verified in code/source, W = web source, I = inference. Repo paths are under `/home/user/wt/b28` (`primitives/src/...`). Nothing was built or run.

---

## 1. Findings

### F1 (blocker): T9/T10 are not blocks; 3.PL/3.CM ordering breaks the build

- **Attacks:** table rows T9/T10 ("G (Rust)"), lane 3.PL (batch 1) versus 3.CM (batch 2), both owning `top_layer/scroll_clamp.rs`.
- **Evidence (V):**
  - `scroll_clamp.rs:58` is `VIEWPORT_METRICS_JS` and `:261` is `MEASURE_AT_FULL_WIDTH_JS`. Both are `const` text spliced into other scripts.
  - T8 splices them (`anchor_fallback.rs` `{viewport_metrics_js}`, `{measure_js}`).
  - T11 (`use_point_anchor_clamp`, `scroll_clamp.rs:348-428`) splices both, with `viewport_metrics_js = VIEWPORT_METRICS_JS, measure_js = MEASURE_AT_FULL_WIDTH_JS`.
  - `select/components/list.rs:102` carries its own copy of the probe.
  - The same file holds T12, `CLOSE_ON_SCROLL_JS` (`:141`, a (b) block with IntersectionObserver and a quiet window), which no lane maps.
  - A Rust test `viewport_metrics_js_is_the_only_viewport_read` pins the splice.
- **Why it breaks:** 3.PL in batch 1 removes T9/T10 as "dissolves under A", while T11 stays JS until batch 2 and needs both constants. Either the build breaks or 3.PL keeps them as dead-ish residue, which contradicts "allowlist ends empty".
- **Change:** move T11 into 3.PL (same file, same construction: a point-anchor element plus the same gate; about +4 h), and make 3.CM only the `context_menu.rs` call-site edit. Assign T12 (`use_close_on_scroll`) to a lane: it is `listen`/`observe` over the bridge, and it can wait until the T8 hooks are stable.
- **Subsumes:** the cross-lane constant coupling, and the ownership gap for `scroll_clamp.rs` between 3.PL and 3.CM.

### F2 (major): the delegated interpreter must be lazy, per-rule-kind and phase-faithful

- **Attacks:** best-practice fold-in "one delegated capture-phase interpreter"; B1 claim "fixed-arity predicate over local data".
- **Evidence:**
  - (V) `playwright/oracle/tier2-html/listener-inventory.spec.ts` (header, assertions 1 and 2):
    - the multiset of listeners on window/document/html/body after open/close must equal the baseline taken before the first open;
    - no non-passive `wheel`/`touchstart`/`touchmove` listener may remain;
    - rows 149/159 are the incident.
  - (V) `toast.rs:529-540`: the F6 registry is reference-counted so "a page without a toaster holds no listener at all and `listener-inventory.spec.ts` stays at its baseline". That precedent already exists.
  - (V) `lib.rs:331-348`: Escape is a document bubble-phase listener.
  - (V) `label.rs:67`: label mousedown is a bubble-phase listener on the label, and it reads `!event.defaultPrevented`.
  - (V) `sidebar/component.rs:227`: window keydown, bubble, reads `event.defaultPrevented`.
  - (V) `dioxus-web-0.7.9/src/dom.rs:100-117`: Dioxus web never calls native `stopPropagation`, only `preventDefault`, so a Rust `stop_propagation()` never hides an event from a document listener. That check is why I do not claim an Escape-ordering regression.
- **Problems:**
  1. A page-lifetime non-passive `wheel`/`touchmove` listener, which any "one interpreter at document capture" installs naively, fails assertion 2 and brings back the row-149 compositor wait.
  2. Capture-phase evaluation makes the `defaultPrevented` predicate (listed in B1) vacuous: nothing has run yet. L23 and V22 read it today because they run last.
  3. Rules need an explicit `target`, `phase` and `passive`, or the semantics change.
- **Change:**
  - Rules carry `{target: window|document|element, phase, passive}`.
  - The interpreter attaches one listener per distinct (target, type, phase, passive) while at least one rule needs it, and removes it with the last rule (ref-counted, like `activity.rs`).
  - Non-passive wheel/touch only while an L10/L11 rule is live.
  - `defaultPrevented` is only legal in bubble-phase rules.
  - The existing inventory spec becomes the gate's behavioural half.
  - `scripts/check-bridge-rules.sh` gets a rule: no `addEventListener` of a blocking type in `ts/` outside the lazy attach function.
- **Subsumes:** the 18 `listener-ok:` and 4 `blocking-ok:` markers, and the L23 "arm on mouseenter" workaround (one delegated rule instead of one listener per Label).

### F3 (major): W8's Rust mode machine on Dioxus `onscroll`/`onresize` is a blocking XHR per scroll event on desktop

- **Attacks:** residue doc §2.2 item 6 ("Fed by Dioxus `onscroll` ... and bridge `listen`"), the `onresize` fallback, W1's "or Dioxus `onscroll`/`onresize` (lane 2D)", and O3's `onresize`.
- **Evidence (V):**
  - `dioxus-interpreter-js-0.7.9/src/ts/native.ts:353-395` (`handleEvent` calls `sendSerializedEvent`) and `:568-593` (`xhr.open("POST", endpoint, false)`): every Dioxus event on desktop/mobile is a synchronous XHR that blocks the webview main thread.
  - Root doc §2.4 rejects the C1 trampoline for wheel/touch exactly for this: "Desktop pays one blocking XHR per event ... unacceptable at 60-120 Hz".
  - `message_scroller.js` header: "Hot-path discipline: ... Rust receives one message when the published { start, end } scrollability CHANGES, never per scroll event or token."
  - The same XHR cost applies to lane 2C's already-moved S2/S3 on desktop.
  - W1 already uses async `dioxus.send`, which is cheap.
- **Change:** for W8, W1 and O3 do not feed Rust from Dioxus element events during fast streams. Give the bridge `listen`/`observe` a page-side reducer option (`emit` only when a named derived key changes: the `key === last` dedupe that `bridge.rs:293` and the photo-wall bridge already do). Intent classification (wheel toward end at end) stays a B1-style page-side predicate that emits only transitions. State the rule in 3.0: Dioxus element events only for low-rate events (click, keydown), never scroll, resize-during-stream, wheel or pointermove.
- **Subsumes:** the desktop XHR risk for W8, W1, O3 and S2/S3, and the unmeasured desktop assumption in the W8 lane.

### F4 (major): T8 "A" does not delete the JS; it removes the outcome check that exists because support flags lied; nothing can test the A arm off Chromium

- **Attacks:** table row T8 ("A, plus a G fallback") and its 12 h estimate for T8+T9+T10+S1.
- **Evidence:**
  - (V) `anchor_fallback.rs:21-28`: the design measures the outcome and does not trust `CSS.supports('anchor-name')`, because "on Firefox, `CSS.supports` reports `true` ... yet a popover-promoted element's `anchor()`-based top/left can still fail to resolve". The plan's A replaces `anchor()` per side with `position-area` and trusts it.
  - (V) `playwright/suite.local.config.ts:36`, `ssg.local.config.ts:41`, `oracle.local.config.ts:12`: each has a single `chromium` project. `playwright.config.ts` defines firefox/webkit but no deploy gate runs them. `stripAnchorSupportsBlock` (`top-layer.spec.ts`) only simulates "no anchor engine" on Chromium.
  - (V) What the block does besides placement, and the plan only partly accounts for:
    - publishes `--dx-anchor-width`, `--dx-available-height/-width`, `data-placed-side` and `--dx-anchor-offset-x/-y` at every exit (`anchor_fallback.rs` `publishPlacement`);
    - runs a `ResizeObserver` for content growth, a 30-frame settle loop (the DatePicker grid grows after mount), `animationend` re-place for submenus inside an animating parent, capture scroll, resize and `visualViewport` tracking;
    - the exclusion-zone verdict on the CSS arm (a conforming placement is accepted only if inside the visual viewport and not overlapping the anchor).
  - (V) Consumers: `select/style.css:142,189` and `combobox/style.css:159` read `--dx-available-height`; `tooltip.rs` and `tooltip/style.css` read the placed side and anchor offsets.
- **Problems:**
  1. For arrows, available-size and exclusion-zone checks, the A arm still needs a Rust measure-and-publish path, not only "after open" but on scroll, resize and growth. That is the same 500 lines moved to Rust with async reads, plus a new CSS arm, so two implementations per engine class instead of one JS plus the existing CSS arm.
  2. The fallback is not a rare engine: Safari ≤ 25 and iOS ≤ 18, Firefox < 147 and WebKitGTK (unverified) all take it, and the plan keeps it.
  3. The 12 h figure does not price the Rust port, the first-frame gate, the slow-channel runs or the new specs.
- **Change:**
  - Keep a Rust "verify the outcome" step on the A arm: after the gate's measure, compare the painted rect to the contract spots and the zone, and fall back to Rust placement on mismatch. That keeps the Firefox lesson.
  - Add a Firefox and WebKit Playwright run for a named subset: `top-layer`, `combobox-placement`, `select-scroll-buttons`, `tooltip-arrow`, `popover`, `date-picker`, `close-on-scroll`. The configs already exist; only the deploy wiring is missing.
  - Re-size 3.PL to about 18-20 h (I) and say so.
- **Subsumes:** the "A works on Firefox/Safari" assumption, which is currently untestable.

### F5 (major): G needs a generation token and a pending state derived from `open`; the pending style must be `animation: none`

- **Attacks:** G mapping for T8, T9, T10, T11, S1, O3, V19, and root doc §2.3 (`animation-play-state: paused`).
- **Evidence:**
  - (V) `anchor_fallback.rs:~560` (last comment of the script): "No cleanup of data-placed-side / --dx-anchor-offset-* ... a superseded run's teardown resolves AFTER its successor's first (synchronous) publish and would erase it". This is the same race class as a late `done` write.
  - (V) `lib.rs:~690`: the dialog driver uses `__dxDialogGen` for the same reason (a stale exit wait wakes up later).
  - (I, CSS Animations): an animation with zero delay, paused at currentTime 0, is in its active phase and applies the 0% keyframe (`scale(.95)`). So under `paused` the S1 read is a transformed rect.
  - (V) `select/components/list.rs:ITEM_ALIGNED_JS` measures with `offsetTop`/`offsetHeight` precisely to avoid the pop-in `scale`.
- **Problems:**
  1. open → close → open in one task with a slow channel (N = 2): the first run's `done` lands on the second open and paints run-1 geometry unplaced.
  2. Unmount before the answer.
- **Change:**
  - Put `data-dx-place="pending"` in the same render as `data-state="open"` (derived, not written later).
  - `done` is written only with a matching generation.
  - The pending selector is `animation: none` (the enter animation starts when it is removed), not `paused`.
  - Add a spec: open/close/open within one task at N = 0 and N = 2.
  - Add a first-frame spec asserting `opacity: 0` while pending (already planned) and that no frame paints `done` with stale geometry.
- **Subsumes:** the whole G class, instead of one fix per block.

### F6 (major): L1 and L2 are not B1 rules; L21 is not B2

- **Attacks:** §3.1 rows L1, L2 ("B1 + tiny residue") and L21 ("B2").
- **Evidence:**
  - (V) `dioxus-html-0.7.9/src/events/generated.rs:19` has `oncancel => cancel`, and `dioxus-web-0.7.9/src/events/mod.rs:8,57` converts it (`convert_cancel_data`). The audit row L2 already says "`oncancel` exists ... `prevent_default` works on web". The repo uses `oncancel` nowhere.
  - (V) `lib.rs` `DIALOG_CLOSE_FORWARD_JS` installs its keydown listener on the dialog element itself. State lives in five JS expandos shared across two evals and `dxOpenFocus`: `__dxClosing`, `__dxActivate` (set on keydown, consumed by the next open-driver run), `__dxCancelAt` (written by L2's cancel handler), `__dxQuietClose` and `__dxOpener`.
  - (V) The keydown body is a command, not just a gate: finish the dialog, `close()`, `opener.focus()`, `opener.click()`, with a 400 ms deferral.
  - (V) Root doc §2.4: "Use [C1] wherever the listener can be a vdom element listener for a `dioxus-html` event that needs no listener option." Both listeners are element-level on the dialog.
  - (V) `pointer.rs:33`: L21 is `listen(window, 'pointerdown|move|up|cancel')` plus `dioxus.send`. There is no recogniser, no cancel and no capture, so it is a plain `listen` verb (and it must stay installed only while a pointer is held, row 159).
- **Change:**
  - **L2:** `oncancel` on the `<dialog>` with `prevent_default()` and Rust-held `cancelled_at`. Keep one bridge `listen(dialog, 'close')`.
  - **L1 forward:** Rust `onkeydown` on the `<dialog>`, with `closing` and `cancelled_at` in Rust. The opener click and focus go through bridge `call`. No `recent()` predicate, no expandos.
  - **Keep:** the exit wait as a bridge call with a Rust generation (L5).
  - **Risk:** capture phase plus `stopPropagation` becomes bubble through the vdom, so a child's own `onkeydown` runs first. Prove with `modal-quick-reopen` (includes Space), `modal-exit`, `dialog.spec`.
  - **L21:** R, with `use_window_listener` scoped to "while a pointer is tracked".
- **Subsumes:** `recent()`, the stateful B1 residue, 4 expandos and the "tiny residue" exception in the gate allowlist.

### F7 (major): L11 is a stateful program, not a fixed-arity predicate, and its distinguishing behaviour has no spec

- **Attacks:** B1 table ("`can-scroll(axis, delta)`, combined by AND/NOT only") and the L11 spec list ("dialog.spec, modal-*.spec, scroll-main-thread").
- **Evidence (V):**
  - `scroll_lock.rs:SCROLL_BLOCK_JS`: the touch path needs `touchstart` coordinates remembered across events (`dxTouchStartX/Y`), then a delta decided at `touchmove`.
  - The key path needs the key map and a role/form-control skip.
  - The wheel path picks the axis by `|dx| > |dy|`.
  - The walk is RTL-aware (`fromLeft = rtl ? scrollLeft + max : scrollLeft`).
  - Specs read the `window.__dxScrollLocked` expando (`playwright/popover-non-modal.spec.ts:74`, 17 uses overall), so it is a contract.
  - `oracle/tier3-radix/scroll-lock.spec.ts` only asserts that a wheel over the page does not scroll while locked and does afterwards. Nothing covers the exemption (wheel over a scrollable region inside the modal, a horizontal code block, RTL, touchmove, keys in a form control).
- **Change:**
  - Write the characterization specs first (in-dialog scroller allowed, horizontal and RTL, touchmove blocked outside, keys skipped in inputs and role elements) against today's JS, then move.
  - Model it honestly as one named rule `scroll-lock` with internal gesture state, not as a composable predicate.
  - Keep `window.__dxScrollLocked` published by the interpreter, or retire it together with its 17 spec reads in the same change.
- **Subsumes:** the false "no variables" claim and the gap between the table and the specs.

### F8 (major): "no residue" is a relabel; B2/B3/B5 are bespoke programs

- **Attacks:** "Result for the 22 blocks: Residue 0", B vocabulary "closed", gate "residue allowlist ends empty".
- **Evidence:**
  - **B2 (users L30, O5, C4, V17, and L21 wrongly), four output modes:**
    - Drawer: no DOM write, streams `start/move/end` to Rust per rAF, and reads `el.dataset.snap === 'partial'` (`drawer.rs:GESTURE_JS`).
    - Toast: writes `--toast-swipe-x/y`, `data-swiping`, `user-select`, then a fly-out computed from `offsetWidth`; direction from the viewport half (`toast.rs:1433`).
    - Carousel: `scrollBy` then a layout read before and after to discover what the scroller refused (`feedOverdrag`/`contentOffset`, `drag.rs`), rubber at a true end, drag-free coasting (TAU 325, MAX_V 2.5), nearest-slide settle by `align` and RTL, snap suspend and restore on `scrollend` or timeout, a `dx-carousel-rest` event consumed by C3, and click suppression.
    - Photo wall: absolute `scrollLeft/Top` from the press origin, plus a second minimap drag (`photo_wall/mod.rs:BRIDGE_JS`).
  - **B3:** one user (S14); O3's fallback is hypothetical.
  - **B5:** three different programs:
    - W3 (`scroll_to.rs:SCROLL_DOM_JS`): `scrollIntoView` first, stable for 2 frames, 30/20 tries, `cover` as top/start/end, skips pinned cells.
    - W5 (`find.rs:SETTLE_MATCH_JS`): centre only, stable for 4 frames, 60 frames, takeover token, cancel on wheel/touch/pointer/key through a separate listener set.
    - W7 (`virtual_grid.rs:FOCUS_CELL_JS`): no settle loop at all after the cell is found, focus with a nested control selector, nearest only, `cover.header`, `at` pre-scroll for an unmounted cell.
  - So a B5 "settle" would change W7's behaviour.
- **Effect:** the JS that stays is about 1,040 lines plus the interpreters; the gate only moves the boundary.
- **Change:**
  - L30 → R. It already streams to Rust and cancels nothing; it needs only the bridge `listen` verb.
  - O5, C4 and V17 stay B2, but B2's spec states its output strategies (css-var, scroll-by-with-refusal, scroll-absolute, stream) as distinct variants, each with a `bun test` and a spec tag.
  - S14 → a bridge `observe(..., publish: var)` option, not a rule family.
  - B5 becomes a `reveal` verb with explicit parameters: `settle: {stable, max} | none`, `preScroll`, `focus: {controlSelector}`, `cancelOnInput`.
  - Add W4 (`beforematch` bridge with a next-frame re-hide) and W6 (MutationObserver tabindex management), which share files with 3.B5 but appear nowhere in the table.
  - Report the JS that remains as a number in `STATUS.md`.
- **Subsumes:** the "closed vocabulary" overclaim, four hidden creep points, and two unmapped blocks.

### F9 (major): the shared generated bundle makes the lane ownership non-disjoint

- **Attacks:** §5.4 "files it owns / must not touch".
- **Evidence (V):**
  - `scripts/build-js-bundle.sh:25` and `scripts/check-js-bundle.sh:49` glob only top-level `ts/*.ts`, so `ts/rules/*.ts` is not hashed or listed in the manifest.
  - `primitives/src/js/bundle.manifest` is one file; each entry produces one minified `.js` with imports inlined. A single `bridge.ts` entry (the interpreter plus every rule) gives one generated `bridge.js` and one manifest line pair that every TS-touching lane (3.0, 3.B1, 3.B5, 3.GS, 3.SF) rewrites, a certain conflict on a minified one-liner.
  - Shared in batch 1: `bridge/rules.rs` (3.B1 and 3.B5). The interpreter switch file (`ts/rules/index.ts` or equivalent) has no owner.
  - `scripts/check-bridge-rules.sh` is owned by "Lane 3.0" in the backlog row but by 3.B1 in root doc §5.4.
  - `playwright/durations.json` (slow-spec-budget gate) is edited by every lane that adds a spec.
  - `top_layer/attributes.rs` writes `data-dx-anchored`, and the G gate attribute naturally goes there, but 3.PL's file list omits it.
  - L1's open driver (`lib.rs`) and `popover_sync.rs` splice `crate::top_layer::TOP_LAYER_INK_STYLES_INJECT_JS` (from `top_layer/styles.rs`, owned by 3.PL) and `crate::modality::{MODALITY_JS, OPEN_FOCUS_JS}`.
  - `virtual/bridge.rs` is claimed by 3.B5 ("W1 as R if 2D has not already"), while 2D is still in flight (`STATUS.md`).
- **Change:**
  - Lane rule: lanes do not commit `primitives/src/js/*.js` or `bundle.manifest`. The main loop runs `scripts/build-js-bundle.sh` once per merge, and `check-js-bundle` is a main-loop gate.
  - Extend the build and check scripts to `ts/**/*.ts` in 3.0.
  - Split `bridge/rules.rs` into one file per region (`rules/gate.rs`, `pan.rs`, `reveal.rs`, `publish.rs`) with a tiny `mod.rs` owned by 3.0.
  - Name one owner for the gate script and the interpreter switch.
  - Add `top_layer/attributes.rs` to 3.PL; keep the const names stable for lib.rs and popover_sync.rs.
  - Sequence `durations.json` updates in the main loop.
- **Subsumes:** guaranteed merge conflicts across the batch.

### F10 (major/minor): C4 and C5 share one `transform` and a handoff protocol across two lanes

- **Attacks:** lanes 3.CA and 3.WB called disjoint; C5 output as `--dx-wheel-band` plus a CSS `translate` home; C4 spring as CSS `linear()`.
- **Evidence (V):**
  - `wheel.rs:563`: "A mouse/pen press hands the element's transform to `CAROUSEL_DRAG_JS`: drop any wheel band first, so the two never both own it."
  - `wheel.rs:567-573` and the cleanup write `el.style.transform = ''`.
  - `drag.rs:215`: `applyBounce` writes `el.style.transform = translateX|Y(...)`.
  - `wheel.rs` header: "Known open item (pending a real-device check)": Safari/WKWebView rubber-bands natively, so the band could stack on the platform's own, and the ownership test (at rest at an edge at gesture start) is the only guard.
  - C5 still needs a page-side wheel listener with per-event `edgeGaps()` rect reads (`atLimit` is per event). That is a metric absent from B3's list, so C5 is R plus a bespoke sampler.
- **Backlog row vs root doc:** the backlog row already says "Last, one at a time: 3.MS, then 3.WB, then deleting `js_macros.rs`", and 3.CA is in batch 3 before them. Root doc §5.4 and residue §5 still show 3.CA, 3.WB and 3.MS in one parallel batch 3. Treat the backlog as authoritative and fix the two research docs so a lane prompt cannot be built from the stale table.
- **Change:**
  - 3.CA must keep C4's bounce on inline `transform` until 3.WB merges, or 3.CA introduces the shared `--dx-band` channel and the pointerdown zeroing for both.
  - The C5 desktop-lag measurement is not enough. Also assert on macOS WKWebView that the band does not stack on the native bounce.
  - List the children-extent metric as a named `sample` field.
- **Subsumes:** the double-owner window between 3.CA and 3.WB.

### F11 (minor): V11 is the transport itself

- **Evidence (V):** `eager_head.rs` is a `Document` impl whose `eval` delegates to `inner`. The bridge is loaded by `document::Script`, which goes through that same document, so V11 cannot depend on a bridge `head.insert` verb.
- **Change:** class it as the seam implementation (an allowlist entry with a reason, like the (c) blocks). The hand-written JS is only the `data-covers` guard; the insert string comes from `dioxus::document::create_element_in_head`.

### F12 (minor): V19 changes behaviour and needs a DOM change

- **Evidence (V):** `questionnaire/component.rs:HEIGHT_JS` animates any height change of at least 1 px via a ResizeObserver on the form (validation text, answers, a font load), not only item swaps. The G plan ("Rust knows when the item swaps", read the old height, pin, swap) covers swaps only. It reads the new natural height from "an inner wrapper", but the children are caller-laid-out "in any layout", so a wrapper changes the flex/gap layout. The old height must also be cached via `onresize` (desktop XHR per resize).
- **Change:** decide explicitly whether non-swap growth should still glide. If yes, B3 or a bridge `observe` is the construction. If no, say it is a behaviour change, and use a `data-height-pin` on the existing form element plus a forced `get_client_rect` (no wrapper).

### F13 (minor): the unassigned (b) rows and the public surface

- **Evidence:**
  - The audit has 55 (b) blocks. The plan routes 12 and assigns lanes for those; the other 43 (T4-T7, T12, L3, L5, L7-L9, L13, L16, L24, L27-L29, S7-S10, S12, S13, S15-S18, W4, W6, C6, V5, V7-V10, V12-V14, V18, V20, V21, V23) have no lane.
  - Preview styled components (V22 sidebar, V17 demo, and later every `dx components add` consumer) would call rule APIs from `dioxus_primitives`, so `Rule` and the hooks are public API.
- **Change:** add the rule/hook types to the public surface list in 3.0 and to `check-installed-paths`/`cargoDependencies` expectations, or keep V17/V22 on a private `listen` hook. Assign or explicitly defer the 43 (b) rows.

---

## 2. Per-block check

"Table OK" = the block does what the table says. "Covered" = the chosen construction covers its behaviour.

### Class (d)

| id | Table OK | Covered | What breaks first |
|---|---|---|---|
| T8 | Yes (more behaviour than listed: F4) | Partly | Firefox/iOS CSS arm that reports support but fails; arrow/available publish on scroll/resize; submenu inside an animating parent (`animationend` re-place) unproven under A |
| T9 | Const, not a block (F1) | n/a | Build if 3.PL deletes it before T11 moves |
| T10 | Const, not a block (F1) | n/a | same |
| T11 | Yes | Partly | WebKit `visualViewport` offset correction (`context_menu.rs:visual_viewport_offset`), point-anchor corner semantics (I, needs spec), same-file coupling (F1) |
| L30 | Yes, but it is R, not B2 | Yes | Over-engineering: `data-snap="partial"` and nested-drawer owner would become B2 parameters |
| S1 | Yes | Partly | Pending style must be `animation: none` (F5); RTL branch and window-resize re-place untested; `:popover-open` poll replaced by Rust knowledge of open |
| S11 | Yes, but part is a value rewrite, not a gate | Yes | IME composition and paste go through the `input` filter (same as today); needs a caret-restore test; the regex is `gv` with `gu` fallback, which `Regex` predicate must keep |
| S14 | Yes (no sync need) | Yes | One-user B3 (F8); per-scroll writes in engines without scroll-driven animations |
| O3 | Yes | Partly | The row-421 ordering needs two renders with a forced style read between, so Rust cannot own the `li` style (the JS comment says Rust never restyles the `li`); desktop two-batch risk; `heightOf` needs computed padding/border |
| O5 | Yes | Yes | Touch swipe not in spec; RTL toaster swipe untested; `data-swiping`, `user-select` and fly-out output modes (F8) |
| W1 | Yes (R) | Yes | Needs `listen` options: no rAF coalescing, per-key dedupe, RTL clamp; not Dioxus `onscroll` on desktop (F3) |
| W3 | Yes | Partly | B5 differs from W3 on cancel and settle (F8); RTL inline branch has no spec |
| W5 | Yes | Partly | Cancel-on-input is a separate listener set today; `__dxFindSettle` token |
| W7 | Mostly | No | W7 has no settle loop; B5-settle changes behaviour; control-selector focus is outside B5's list |
| W8 | Yes (passive, V) | Partly | Follow-bottom sentinel is plausible (I); mode machine fed by `onscroll` is the problem (F3); `min-height: calc(var + 100%)` needs a definite viewport height; no Firefox/WebKit run |
| C1 | Yes | Yes | Nothing, once C2-C5 are gone |
| C2 | Yes | Yes | `__dxCancelSnapRestore` and `__dxWhenIdle` cross-eval expandos move with C3/C4 as one unit |
| C3 | Not independently verified beyond the audit | n/a | n/a |
| C4 | Table understates it (F8/F10) | Partly | Refusal-by-layout-read, coasting and settle are one coupled state machine split across B2, R and A |
| C5 | Yes (passive, V) | Partly | Per-event geometry sampler stays page-side; native WKWebView bounce stacking; `performance.now()` to `e.timeStamp` changes tuned timing |
| V11 | No (F11) | n/a | Bootstrap order |
| V19 | Partly (F12) | Partly | Wrapper, behaviour change, desktop |

### The (b) rows routed through B1/B2

| id | Verdict |
|---|---|
| L1 | Not B1 (F6): Rust `onkeydown` plus bridge `call`; exit wait stays a bridge call with a generation |
| L2 | `oncancel` (C1) plus one bridge `listen(close)` |
| L4 | Fine: already TS; three identical glue snippets collapse. Keep the focus walk page-side |
| L6 | Fine: Escape is unconditional (V); top-of-stack stays Rust. Needs document bubble-phase semantics (F2) |
| L10 | Fine: unconditional cancel outside the root; non-passive wheel/touchmove only while open (F2) |
| L11 | Stateful named rule (F7), characterization specs first |
| L23 | Fine, but bubble-phase and `defaultPrevented` (F2); `MouseData` has no `detail`, so it needs a page-side field |
| L25 | Fine; lifetime tied to a signal (`native_drag`), so a document-bound rule needs an RAII guard; first `dragover` may beat the install (same as today) |
| O1 | Fine: needs `get(activeElement)` for the F6 cycle and the refcount (spec precedent) |
| V22 | Fine: shortcut is user-configurable, `defaultPrevented` ordering (F2), public API (F13) |
| L21 | R, not B2 (F6) |
| V17 | Partly B2 (pan writes absolute scroll, minimap drag, `JUMP_JS` is a W7-style reveal, `MutationObserver` cell count); demo-only |

---

## 3. Specs: what would prove each move, and what is missing

All names exist in `/home/user/wt/b28/playwright` (checked by `ls`), except that the item-aligned cases live in `select-scroll-buttons.spec.ts`, not `select.spec.ts` as the table says.

| Move | Existing specs | Missing |
|---|---|---|
| T8/T9/T10 | `oracle/tier2-html/top-layer.spec.ts` (rules 5-15), `anchor-landing`, `combobox-placement` (has the `visualViewport` keyboard simulation), `close-on-scroll`, `tooltip-arrow`, `popup-style-isolation`, `date-picker`, `keyboard-inset`, `oracle/tier3-radix/rtl` (Sub), release `main-thread` | Firefox/WebKit runs; first-frame "no unplaced paint"; open/close/open at N = 0 and 2; nested overlay inside an animating ancestor under A; `position-area` size/shift parity numbers |
| T11 | `context-menu.spec.ts` | RTL, flip-left at the right edge, pinch-zoom offset |
| L30 | `drawer`, `drawer-axis-lock` (touch), `drawer-snap`, `drawer-non-modal`, `drawer-responsive` | RTL drag, nested-drawer owner under touch |
| S1 | `select-scroll-buttons.spec.ts` (item-aligned block with `data-placed`) | RTL, window resize while open, keyboard-up viewport |
| S11 | `input_otp`, `input-otp-pattern` | IME composition, paste with caret in the middle |
| S14 | `scroll-fade*`, `scroll-area-always-show` | Engine without scroll-driven animations (Chromium always has it) |
| O3 | `toast-height`, `toast-stack-motion`, `toast-stacking`, `toast-enter-exit`, `toast-update` | Desktop two-batch ordering (needs slow-channel mode) |
| O5 | `toast-swipe`, `toast-swipe-vertical` (mouse only) | Touch pointer, RTL toaster |
| W1 | `virtual-engine`, `virtual_list`, `virtual-grid*`, `table-rows`, `data_table-large`, `oracle/tier2-html/scroll-main-thread` | Desktop event volume (not measurable here) |
| W3/W5/W7 | `virtual-scroll-to`, `virtual-grid-pinning`, `virtual-list-find`, `virtual-grid`, `data_table` | RTL for scroll-to and find (0 `rtl` hits in `virtual-scroll-to` and `virtual-list-find`); W7 one-shot behaviour pin |
| W8 | `message_scroller`, `message-scroller-api`, `message-scroller-timing` (1 test), `chat-*` | `overflow-anchor: none` forced-fallback run, RTL, reduced-motion `smooth`, slow-channel run, Firefox/WebKit |
| C2-C5 | `carousel`, `carousel-paging-inert`, `carousel-edge-overdrag`, `carousel-drag-free`, `carousel-edge-wheel`, `carousel-wheel-replay` (recordings), `carousel-virtual-wheel` | `cargo test` replay does not exist yet (capture script missing, residue §6); macOS native-bounce stacking; shared-channel handoff test |
| V11/V19 | `global-stylesheet`, `stylesheets-present`, `first-paint-styles`; `questionnaire-height` (reduced motion present) | Non-swap growth |
| L1/L2 | `modal-quick-reopen` (Enter and Space), `modal-exit`, `dialog`, `alert-dialog`, `sheet`, `modal-initial-focus` | None for the new `oncancel` path beyond these |
| L11/L10 | `oracle/tier3-radix/scroll-lock`, `listener-inventory`, `scroll-main-thread` | In-dialog scroller, horizontal, RTL, touchmove, keys (F7) |
| L21/L25/L23/O1/V22 | `slider`, `resizable`, `drag_and_drop_list`, `label-double-click`, `toast-f6`, `sidebar` | none significant |

The spec contract on debug expandos must be preserved or retired together: `window.__dxScrollLocked` (17 uses), `__dxCarouselWheel`, `__dxCarouselTrace`, `__dxJsListeners`.

The slow-channel harness (`window.__dxBridgeDelayFrames`) does not exist yet. Until it does, no G-mapped spec exercises desktop ordering.

---

## 4. Lane plan: ownership and order

- **Order against dependencies:** batch 1 → 2 → 3 is consistent except F1 (T11 and T9/T10) and the bundle problem (F9). The backlog row's "3.MS, then 3.WB, then delete `js_macros.rs`, each its own batch and deploy" matches the owner's W8 → C5 → C1 order. Dependencies check out:
  - 3.WB after 3.CA is needed for the true-end function and the transform channel (F10).
  - C1 can only go last, because C2-C5 all splice the macros.
  - 3.MS needs B5 (3.B5, batch 1) and the G pattern (3.PL).
- **To fix in the research docs:** root doc §5.4 batch 3 and residue §5 still show 3.CA, 3.WB and 3.MS in parallel.
- **File disjointness:** not disjoint as written (F9), and F1 shows a real overlap between 3.PL and 3.CM on `scroll_clamp.rs`.
- **Owner's measurement protocol:** W8 and C5 each ship as their own batch and deploy. Since W8's risk is anchoring quality per engine, run its Chromium specs plus a manual Firefox and Safari 27 probe (no CI project for them), and the `overflow-anchor: none !important` fallback run, before the owner measures.

---

## 5. Attacks that failed

- **Escape/preventDefault ordering under a delegated listener.** I expected a Rust `stop_propagation()` to hide Escape from the document listener. It does not: `dom.rs:100-117` never calls the native `stopPropagation`. Only the `defaultPrevented` readers (L23, V22) are affected.
- **W8 "every listener is passive; no class-(1) work".** Holds. `message_scroller.js:937-944` registers scroll/wheel/touchstart/touchmove with `{ passive: true }`, and there is no `preventDefault` in the file. The sync need is scroll anchoring only.
- **C5 "passive, never cancels".** Holds (`wheel.rs` header and `listen(el, 'wheel', onWheel, { passive: true })`).
- **L6 "Escape cancels unconditionally; top-of-stack is Rust".** Holds (`lib.rs:331-348`).
- **W1 "already an async bridge, no DOM write".** Holds (`bridge.rs:293`). Only the implementation option (Dioxus events) is the problem (F3).
- **S11 as a B1 rule.** Fits (`beforeinput` cancel plus an `input` filter); the value-rewrite companion is the only wrinkle.
- **O3 on web.** The microtask ordering for "height write, forced read, flag" is plausible in the Rust render loop; the risk is ownership of the `li` style and desktop.
- **iOS keyboard (row 210) for T8.** A is layout-viewport only, and a Rust correction on `visualViewport` resize has the same latency as today's rAF `reposition()` (V: `anchor_fallback.rs` tracking), so it is no regression.
- **Gate attacks on the viewport probe (T9).** A removes the 390-versus-375 gutter problem natively (fixed containing block); the gate and `spec-viewport-bound` need no change.
- **Not independently verified:** C3's "no DOM write" beyond the audit text and the plan's quote; the "133 `prevent_default()` calls" figure; desktop/mobile behaviour of anything (not in CI here).
