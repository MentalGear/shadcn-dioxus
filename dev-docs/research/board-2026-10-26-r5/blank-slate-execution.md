# Board round 5: blank-slate and execution lens (row 434 step 3)

**Verdict: HOLDS WITH CHANGES**

**DESIGN findings: 1, DETAIL findings: 10**

- My blank-slate design matches the plan on the root, the two kinds, the closed bridge with one eval sink, A/G/R, and sinks plus ratchet. Two differences survive. web-sys on web loses to the plan, as rounds 2-3 found. The admission ground loses to mine at one point.
- DESIGN 1: the one admission ground, a counterfactual witness read by in-page probes, has nothing to read when an effect's commit point is outside the page. Example: a cancelled F6, or Ctrl/Cmd+B (Firefox opens its bookmarks sidebar). It also has nothing to read when the engine that would show the red is not in the container (WebKit's clipboard gesture rule). Every `hotkey` user has a witness that cannot go red. So proof port A demotes itself to `listen`, and on WebKit a clipboard write would ship broken.
- Execution: the 103-id ledger is complete and unique (checked by script). 17 spot-checked ids land on the code; two of them are placed wrongly (L22, L23).
- Phase 0 cannot run as written:
  - item 0.4 writes its own witness, and nobody owns `hotkey`'s witness;
  - `ts/bridge.ts` has no owner after 0.0;
  - 0.4 needs 11 consumer files it does not own.
- Scope-1 batches put 2-3 lanes on `bridge/behaviours.rs` at the same time. The roving-focus row (78 `set_focus` sites in 32 files) is left out of the 250-280 h, although the root says it is included.
- The owner's latency session needs a release desktop build of preview that has never been built. Release dioxus-desktop also turns off the web inspector, so the stats cannot be read.
- Using only `js-interop.md`, a new agent cannot place any of my three test behaviours (copy button, typeahead in Listbox, auto-growing textarea) without stalling or breaking the deadline table.

Labels: V = verified in the repo or the Dioxus 0.7.9 sources; W = web source; I = inference.

---

## Part 1. Blank slate (sketched before reading the plan)

**What I read first:** `js-audit-2026-10-25.md` (103 ids); the eval sites in `b28` (`js_listener.rs`, `label.rs`, `lib.rs:331-400`, `collapsible.rs:75` and others); and the Dioxus 0.7.9 sources. The facts the sketch rests on:
- **web eval** runs the body synchronously at the call (`dioxus-web-0.7.9/src/document.rs:118-220`, `Function::new_with_args(..).call1`).
- **Dioxus handlers** run synchronously inside the DOM event. `prevent_default` is applied right after the handler (audit section 2, V).
- **web-sys reach on web:** `dioxus::web::WebEventExt::as_web_event()` returns the raw `web_sys` event, so `detail`, `target` and `ToggleEvent.newState` are readable. `MountedData` downcasts to `web_sys::Element` (`dioxus-web-0.7.9/src/events/mod.rs:186-206`, `events/mounted.rs:147-151`, `dioxus-0.7.9/src/lib.rs:95-97`). So on web, Rust can reach every API the audit files under (b) and (d), synchronously and in the same tick.
- **Primitives depend on no web-sys** (`primitives/Cargo.toml`). The `web` feature is also on for desktop (`preview/Cargo.toml:39`), so `web` means "has a webview", not "wasm".

My construction, Rust first:
1. **Placement by one question:** "Does Dioxus deliver the event and every field, at human rate?" If yes, an rsx handler, whatever the deadline. Otherwise, a typed Rust capability layer: on wasm, web-sys calls (Rust, synchronous); on a webview renderer, one closed bridge with one eval sink. Page JavaScript is written only where Rust has no thread at the commit point:
   - pre-wasm code;
   - desktop and mobile, where Rust is behind IPC.
2. **Admission for page-side effects:** a closed list of commit points (cancel, capture, activation, focus for the next input, pre-paint write), declared per effect. A witness is required wherever the commit point can be observed in a test. A cited UA or spec default stands in where it cannot.
3. **Commands:** Rust effect plus a bridge call.
4. **Enforcement:** a sink ban outside the layer, plus a shrink-only ratchet with grants.

**Comparison with the plan, one difference at a time:**

| difference | mine | plan | which is better at the core |
|---|---|---|---|
| web-sys on web | Rust (web-sys) decides on web; the bridge only on webviews | one TypeScript bridge on every renderer; web-sys is a sink outside `transport.rs` (root 5.3 item 2) | **The plan.** Two implementations per behaviour drift by construction (round 2 blank-slate, round 3). Not a finding: it was already rejected for a stated reason that still holds. |
| admission ground | declared commit point, plus a witness wherever it can be observed | one ground: the counterfactual witness, read by library probes | **Mine at one point, the plan everywhere else.** The witness stops self-admission where it can be observed. It cannot admit effects whose commit point is outside the page or engine-specific. See D1. |
| "none" deadline on an event Dioxus delivers | rsx | R: bridge `listen` (root 2.2 table, last row) | **Mine.** The plan routes L22 to a page listener that its own order contract forbids. DETAIL 7. |
| commands | effect plus call | level-triggered attribute with an epoch and one executor | **The plan** (lost and early commands converge). |
| enforcement | sinks plus ratchet | the same, more complete | converge |

---

## Findings

### D1. DESIGN, major. The single witness ground cannot admit effects whose commit point is not observable in the page or in the container's engine

**Claim attacked.** Root 2.2, admission:
- "one ground, the counterfactual witness"
- "A red counts only from a library commit-point probe"
- "`hotkey` (`prevent`) ... expected to be red under their counterfactuals"
- 1.4 item 4: activation-gated calls "take the witness ground of admission like any other effect"

**Evidence.**
- **The three `hotkey` users.**
  - V: L6 has exactly one caller on the web arm, `dialog.rs:442` (`DialogContentNonModal`). That element is a plain `div role="dialog"` (`dialog.rs:452-470`). The other five calls are in `#[cfg(not(feature = "web"))]` arms that 3.DJ deletes (`dialog.rs:476,499`, `alert_dialog.rs:233,244`, `popover.rs:44/60,517,811/831`).
  - Escape on a div has no in-page default action, so `defaultAction(occurred)` cannot go red. The only exception would be a nesting scenario the witness does not define.
  - V: O1 cancels F6 (`toast.rs:558`). V22 cancels Ctrl/Cmd+B outside editables (`preview/src/components/sidebar/component.rs:225-229`).
  - W: in Firefox, Ctrl/Cmd+B opens the Bookmarks sidebar and F6 focuses the address bar (<https://support.mozilla.org/kb/Keyboard+shortcuts>). Both are browser-chrome actions. An in-page probe cannot observe them, and headless Chromium has no such chrome.
  - So `witness(hotkey, prevent)` is green at all three sites.
  - By admission point 1 the effect set becomes empty, and "a construction whose effect set is empty does not compile; that call site uses `listen`". The cancel is dropped, which is a user-visible regression in Firefox: the sidebar toggle also opens the bookmarks panel. Proof port A ("`hotkey` for L6", item 0.3) proves no listener behaviour at all.
- **Activation-gated calls.**
  - W: WebKit requires `clipboard.write`/`writeText` to be "triggered during a user gesture" (<https://webkit.org/?p=10855>). Safari 18.x rejects even a synchronous call from a `select` `input` event (<https://developer.apple.com/forums/thread/772275>).
  - The witness runs on Chromium only (root 5.4: "Firefox and WebKit runs cannot execute in the lane container at all"). Chromium keeps transient activation for about 5 s (I, round 4 admission finding 4).
  - So a clipboard write delayed by 3 frames is green on Chromium, demotes to an R `call`, and fails on Safari. It also fails on WKWebView, where the call arrives by `evaluate_script` outside the gesture (I).

**Change.** Admission gets a second, closed ground for these commit points: "UA default or engine rule outside the probe's reach". It is one reasoned row per effect, carrying:
- a W citation;
- the engine;
- a `desktop-pass`/manual check line;
- an owner-approved grant row at the merge base.

The witness stays the ground wherever a probe can see the commit point. Where Playwright WebKit can see it (the clipboard case), the witness must run on that engine before the effect may be demoted.

**Subsumes:** the three `hotkey` sites, V14 and any copy button, and any future chrome-level shortcut cancel.

**Why DESIGN:** it changes admission from one ground to two. The other options are worse: drop the cancels (a regression), or make "`defaultPrevented` observed" a probe, which restores self-admission.

### 1. DETAIL, blocker. Witness authorship contradicts the merge-base rule, and `hotkey`'s witness has no owner

- **V (root 2.2 admission point 5):** "The witness test and `verdict.json` must exist at the merge base under the grant-row rule, so a lane cannot admit its own behaviour."
- **Item 0.4 writes its own witness:** "first `reconcile`'s witness (a prototype page reconciler against the delayed Rust `transaction` ...), then the executor it selects". The witness, the prototype and the behaviour are all in the same lane.
- **Item 0.6 does not list `witness(hotkey, prevent)`.** Item 0.3 cannot write it, by the rule above.
- **A witness cannot pass before its behaviour exists:** "a timeout or a locator error fails the witness".
- **Change:** item 0.6 owns the prototype reconciler and the two first witnesses. A witness for a not-yet-built variant lands on the queue branch as `test.fixme` keyed to the variant's existence in `behaviours.gen.ts`. The gate checks its hash at the merge base, and the lane flips it to live.

### 2. DETAIL, major. `ts/bridge.ts` has no owner in phase 0 after 0.0

- **V:** the behaviour adapter is `installBehaviour` at `l30:primitives/src/ts/bridge.ts:648`, and the `pageObserver` seam is in the same file.
- **Who needs it:**
  - 0.3 ("the adapter built for `hotkey`");
  - 0.4 (the executor's page half, the watchdog, the exit wait as "a bridge internal");
  - 0.6 ("the adapter's witness mode", "imported by the adapter through one line").
- **Who lists it:** none of these items lists `ts/bridge.ts`. 0.1's DOM-less `ts/behaviours/**` program also needs `primitives/tsconfig.json`, which no item owns.
- **Change:** name `bridge.ts` and `tsconfig.json` in each item's file list, in the serial order 0.1 → 0.6 → 0.3 → 0.4.

### 3. DETAIL, major. Item 0.4 cannot publish the command attribute from the files it owns

- **V:** `use_popover_sync` and `use_popover_shown_while_mounted` return `()` (`top_layer/popover_sync.rs:109-113, 239-243`). Their 13 call sites are in 11 files: `tooltip.rs:452`, `dropdown_menu.rs:765,1927`, `select/components/list.rs:378`, `hover_card.rs:600`, `navbar.rs:913`, `popover.rs:745`, `toast.rs:886`, `menubar.rs:853`, `context_menu.rs:728,1953`, `navigation_menu.rs:1013`, `combobox/components/list.rs:128`.
- **The conflict:** the command kind needs the element to "arrive already carrying its attribute" (root 2.2), which is a vdom attribute in each consumer's rsx. Item 0.4 owns only `popover_sync.rs` and the executor. The 11 files belong to 0.3, 3.PL, 3.OV and 3.GS.
- **Change:** the hooks return the attribute to spread. 0.4 owns the one-line spreads, sequenced after 0.3, and its hours rise.

### 4. DETAIL, major. Scope-1 batches are not disjoint

- **`bridge/behaviours.rs` and `primitives/conformance/behaviours.json`:**
  - Owned at the same time by 3.OV and 3.VS (batch 1), and by 3.SL, 3.GS and 3.FC (batch 2).
  - The file-sharing paragraph says they go "in the order they are opened". `behaviours.rs`'s `info()`, `declared()`, `module()` and `samples()` are no-wildcard matches (V, l30 `bridge/behaviours.rs:124-200`), so every parallel variant conflicts in four places.
- **Batch 3:** 3.PV owns `preview/src/components/virtual_list/variants/*` (V18). 3.HV owns the `ondragover` at `virtual_list/variants/reorder/mod.rs:163` (V).
- **3.FC's `dragSource` half** edits `preview/src/components/data_table/reorder.rs:363-368` and `virtual_list/variants/reorder/mod.rs:144-149` (root 1.4 item 4). Its file list has neither.
- **Change:** one Rust file and one vectors file per behaviour (a registry macro collects the variants), and corrected ownership lists.

### 5. DETAIL, major. The roving-focus row is outside every estimate and every file order

- **V:** 78 `set_focus(` sites in 32 primitives files (grep, b28).
- **The contradiction:**
  - Root 2.2 condition 3: "The 32 files are in the lane plan and the estimate".
  - Root 5.4: "not sized by any board ... the 250-280 total excludes it".
- **Other effects:**
  - Scope 1 ("port everything now") leaves out the rule's largest application.
  - Its 32 files overlap 0.3, 3.PL, 3.OV and 3.FC, and they are missing from the file-sharing order.
  - The typeahead of `typeahead.rs` (menus, `collection.set_focus`) falls under it too.
- **Change:** size it before the owner chooses a scope, and give it a slot in the order.

### 6. DETAIL, major. One person with a Mac cannot run the latency session as written

**Claim attacked.** Root 5.6 and 443: "one real session of the **current build** on one of the owner's devices (a Mac with WKWebView ...)"; method "a release build".

**Evidence.**
- **V:** preview's desktop build has never been built. Row 23 says "WAITING ON OWNER ... no desktop build exists in the repo and none is exercised".
- **V:** release dioxus-desktop turns the inspector off. `disable_context_menu` defaults to `!cfg!(debug_assertions)`, and `with_devtools(true)` is set only when it is false (`dioxus-desktop-0.7.9/src/webview.rs:453-459`, `config.rs:117`). So `window.__dxBridgeStats`/`dump()` cannot be read from a release WKWebView.
- **Why debug is no way out:** a debug build is invalid for timing by the repo's own rule (CLAUDE.md, row 116).
- **Missing pieces:**
  - no synthetic workload: the current build's bridge carries almost no traffic until ports land;
  - `desktop-pass.md` does not exist.

**Change.** Item 0.5 adds:
- a desktop smoke build (closes row 23 first);
- a preview desktop launch config with `with_disable_context_menu(false)` behind `?dx-perf`, or a dump written to a file through Rust;
- a ping workload page at frame rate under a representative render;
- a one-page owner script (install dx 0.7.9, the build command, Safari's Develop menu, where the JSON goes).

The iPhone rows (tethered Web Inspector, 240 fps video) are feasible as tiered.

### 7. DETAIL, major. Two ledger placements are wrong, and the "none" row lacks rsx

**L23 ("deleted: `.dx-label` ships `user-select: none`").**
- **V:** `playwright/label-double-click.spec.ts:1-9,19-48` lifts that declaration to test "the part an unstyled `Label` user gets" (row 384). It also asserts that a drag still selects, which `user-select: none` forbids.
- Deleting L23 in item 0.3 turns two tests red and drops a public primitive's contract. Round 1 made the deletion conditional on unstyled use not being a goal; the spec says it is one.
- Under the plan's table this is a default-action decision on a field Dioxus lacks (`MouseData` has no `detail`), so it is a behaviour. Otherwise the owner drops the contract and the spec.

**L22 ("R: bridge window listener with a reducer").**
- **V:** the audit marks L22 as a* (`onpointerover`/`onpointerdown` with `PointerData::pointer_type()`, `dioxus-html events/pointer.rs:99`). Human-rate events: `pointerover` is not in the plan's hot list.
- The bridge fact (pointer type) is consumed by the vdom `onpointerenter` decision. On desktop a fact and a vdom event are unordered (root 1.4 item 8), so a touch can open a hover menu.
- That breaks the plan's own order contract ("no decision may depend on the arrival order of two inputs"). rsx keeps both events on one ordered path, in pure Rust.

**Change.**
- L23: becomes a behaviour, or the owner decides to drop the contract.
- L22: becomes rsx on the host elements, in 3.OV.
- The "none" row of the deadline table reads: "an rsx handler if Dioxus delivers the event and its fields at human rate; otherwise R".

### 8. DETAIL, major. `js-interop.md` alone does not place three new behaviours

None of these was used in round 4; round 4 used a dialog hotkey, a splitter and a popover.

**A copy-to-clipboard button in an installable component.**
- No "I want to" row, and nothing in the forbidden table.
- The only mention is a "preview-only allowlist" (`js-interop.md:144-145`), while root 5.3 item 10 bans sinks in installed components.
- The root contradicts itself: 5.2 says clipboard writes are "effects of a named behaviour"; 1.4 item 4 says activation-gated calls take the witness ground (green on Chromium, so a verb).
- An agent stalls, or adds a `writeText` verb row that breaks on WebKit (D1).

**Typeahead in the existing `Listbox`.**
- `listbox.rs:74` moves real focus (`collection.set_focus`). The menus' `typeahead.rs` does the same from rsx.
- The page says "a new composite does not copy that". It is silent on extending an existing roving widget.
- The deadline table puts focus for the next key in the task row ("never rsx `set_focus` on desktop"), so the answer is `roving` (does not exist; owner call) or the unsized roving row.
- Wrong placement or a stall.

**Auto-growing textarea (a same-frame height write).**
- The before-paint row says "A ... behind per-feature engine floors and a boot probe". `field-sizing` is Baseline-low (root 2.1), but floors and the probe exist only for T8's anchoring set, and nothing says who adds a feature's floor or probe.
- G is excluded (continuous), no behaviour exists, and R is a wrong paint.
- A stall, with no stated degraded-fallback rule.

**Change:** add rows for "activation-gated call", "extend an existing roving widget" and "add a new A feature (floor row, probe owner, acceptable fallback)". Each row says "today: stop and report" where nothing is built.

### 9. DETAIL, minor. The scope-2 yield order ignores dependencies

- **Order:** 3.PV (25/h) is listed before 3.PL (27/h), so the table is not sorted by its own column.
- **Dependencies:** V19 (G, in 3.PV) and O3 (G, in 3.GS) need 3.PL's "pending CSS, fail-open names, per-surface list".
- **Change:** order by yield within the dependency DAG (3.PL before any G consumer).

### 10. DETAIL, minor. Stale figures

- **Bridge size:** the plan cites `bridge.ts` "874 lines against a budget of 1,100" and a `BEHAVIOUR_BUDGET = 150` constant (root 5.0, 5.3 item 4). On l30 it is 1,056 lines against 1,080, plus a per-module dict `{"fixture": 26}` (`l30:scripts/check-bridge-behaviours.sh:101-105`).
- **Item 0.2:** 3 h (5.3 item 12) against 4 h (5.4 table).
- **Audit line numbers in `lib.rs`:** drifted by 80-200 lines (L1 :571→:651, L5 :972→:1060, L17 :438→:544, L26 :355→:571). The root's own citations (`lib.rs:367,781,848`) are correct.
- **Phase 0 hours:** sum correctly to 47.
- **Full total:** sums to about 258-287 h against the stated 250-280.

## Execution checks that passed (V)

- **The 3.3 ledger:** 103 ids expected, 103 found, no duplicates, no extras (scripted). Lanes: 3.PV 14, 3.OV 12, 3.FC 11, none 10, 3.PL 8, 3.GS 8, 3.VS 7, 3.CA 6, 0.1/0.4 4, others 1-3.
- **Spot-check of 17 ids against b28:** L4, L6, L10, L13, L22, L23, L24, L26, L28, S7, S10, S16, V10, V12, W6, T12, C6. All locate (L13 and L26 have drifted) and all fit their placement except L22 and L23 (finding 7). L4's "dead" claim holds: `focus_trap_script` is `rsx! {}` on web (`lib.rs:143-147`).
- **Phase 0:** order and serial file use (`transport.rs` 0.0→0.1→0.5; `behaviours.rs` 0.3→0.4) are consistent, apart from findings 1-3.
- **Scope options:** they are coherent as decisions. Scope 3's 47 h buys the construct, but under finding D1 its listener proof port is empty.

## Attacks that failed

- **"web-sys is the Rust-first answer on web."** It works on web (V, `WebEventExt`), but it means two implementations per behaviour, one per renderer family. The plan keeps a web-sys backend behind the transport seam. Held for the third time.
- **"The root is false on web (Rust memory is on the page thread)."** True on web, and the root says so (1.2): it designs for desktop, where the premise holds.
- **"The ledger is incomplete or has two owners per id."** Scripted check: complete and unique.
- **"The root's file:line citations are stale."** `lib.rs:367,781,848`, `toast.rs:558`, `label.rs:69` and the `popover_sync`/`not(web)` sites all land.
- **"Phase 0's hours do not add up."** 13+4+11+9+3+7 = 47.
- **"The order contract cannot be executed."** Its instances are specified and the desktop-cadence harness can reorder across sessions (row 442 item 1). The one placement that breaks it is a ledger error (L22), not a contract gap.
