# Board lens: simpler-alternative (row 434 step 3), 2026-10-26

**Verdict: HOLDS WITH CHANGES.** A, G and R hold: that is where JavaScript actually leaves the codebase. The over-built part is B (the closed `Rule` enum, one interpreter and the parity gate).

Summary:
1. The B layer moves almost no logic into Rust. The recogniser, the settle loop, the scroll-capacity walk and the value filter all stay in TypeScript, and only their parameters come from Rust. The line count of the B interpreter is about the same as the blocks it replaces, plus a DSL, a serialisation format, conformance vectors in two languages and a parity gate.
2. About 16 JS call sites that the plan migrates are dead code. They sit in `#[cfg(not(feature = "web"))]` arms, which in every supported build are Blitz only, where `document::eval` does nothing. They include the whole focus trap (L4) and `ts/focus-trap.ts`, the only file in the TS chain.
3. L23 (Label double-click) is already handled by CSS that ships: `user-select: none` on `.dx-label`.
4. The delegated `data-dx-rule` + `closest()` form only finds rules on the target's ancestor chain. Those are exactly the events a Dioxus element handler already decides synchronously, with full Rust state, on every renderer including Blitz. The cases that really need page-side rules (window, document, "outside") cannot be found that way. For non-passive `wheel`/`touchmove`, a delegated listener that lives as long as the page brings back the rows 149/159 regression.
5. Change: delete the dead code first. Then send every element-scoped decision to an rsx handler, and replace the `Rule` DSL with about five named, typed TS functions, each with a typed config and installed only while active. Merge the 11 verbs into 2 (`subscribe`, `run(ops[])`), and drop the batched `sample` mode, which only C5 needs.
6. web-sys, custom elements, dropping desktop parity, more CSS and upstream APIs were all tested. None is a simpler construction overall (see "attacks that failed").

Evidence labels: **V** = verified in code or crate source; **W** = web; **I** = inference.

---

## Findings

### 1. (major) The plan migrates dead JavaScript into the new interpreter

**Claim attacked:** root doc §1.3 and §3.1 map L4 to "B1 `focus-trap(container)`, already TS; it becomes the B1 effect". Lane 3.B1 owns "`ts/focus-trap.ts`, `dialog.rs`/`alert_dialog.rs`/`popover.rs` (focus-trap glue only)". L6 is listed as used by "Dialog, AlertDialog, Popover, DropdownMenu".

**Evidence (V):**
- `primitives/src/lib.rs:104-118`: the focus-trap bundle's doc says the "Native (Blitz) modal overlays (`dialog.rs`/`alert_dialog.rs`/`popover.rs`'s `#[cfg(not(feature = "web"))]` arms) are its only consumers". `FOCUS_TRAP_JS` and `focus_trap_script` are both `#[cfg(not(feature = "web"))]`.
- `dev-docs/reference/platform-rationale.md` (Caveat 2): on Blitz, `document::eval` is `NoOpDocument.eval`.
- `templates/starter/Cargo.toml.liquid:17-20` and `preview/Cargo.toml:38-41`: web, desktop, mobile and server all switch on `dioxus-primitives/web`. So a `not(web)` arm is compiled only for Blitz, where an eval cannot run.
- A scan of every `#[cfg(not(feature = "web"))]` fn body (my script; output reproduced here):

  | file:line | JS calls that cannot run |
  |---|---|
  | `dialog.rs:478` DialogContentModal | `use_global_escape_listener` :499, `use_outside_dismiss` :500, focus-trap eval :507 |
  | `alert_dialog.rs:235` | escape :244, `use_scroll_lock` :251, focus-trap eval :254 |
  | `popover.rs:488` PopoverModalContent | `use_scroll_lock` :512, escape :517, outside :518, focus-trap eval :522 |
  | `popover.rs:813` PopoverNonModalContent | escape :831, outside :832 |
  | `dropdown_menu.rs:892`, `menubar.rs:974` | `use_outside_dismiss` :935, :988 |
  | `context_menu.rs:895` | outside :963, `CONTEXT_MENU_SCROLL_BLOCK_JS` :978 |

- Five of the six `use_global_escape_listener` call sites are in these arms. The only live one is `DialogContentNonModal` (`dialog.rs:442`).

**Consequence (I):**
- 3.B1 would build a `focus-trap` B1 effect whose only consumer no renderer runs.
- The gate rule "every variant is named by at least one Playwright spec" (root doc §2.2) cannot be met for it: no web spec reaches a Blitz arm. The gate would go red, or someone would write a fake spec.

**Change:**
- Before 3.B1, delete `ts/focus-trap.ts`, `js/focus-trap.js`, its manifest line, `focus_trap_script`, and the eval and JS-listener calls in the `not(web)` arms.
- If the Blitz floor should behave, use Dioxus element handlers there. `onkeydown` Escape on the panel actually works on Blitz; eval never will.
- Add a gate rule: no `document::eval` or `JsListeners` inside a `#[cfg(not(feature = "web"))]` item. Such a call is always dead, so the rule closes the class by construction.

**Subsumes:** L4 completely (33 eval lines + 112 TS lines), 5 of 6 L6 call sites, 1 of 2 L10 install sites, 4 dead outside-dismiss sites, 2 dead scroll-lock sites, and B1's `focus-trap` variant. Once its only file is gone, the bundle chain holds nothing until 3.0 adds `bridge.ts`.

### 2. (major) L23 is already done by shipped CSS

**Claim attacked:** root doc §1.3 maps L23 (`label.rs:63`) to "B1 field plus target" (`detail > n`), and lane 3.B1 owns `label.rs`.

**Evidence (V):**
- `preview/src/components/label/style.css:50-54` declares `-webkit-user-select: none; user-select: none` on `.dx-label`, with the reason "shadcn's `Label` carries `select-none`… backlog row 294".
- With that rule, neither a double click nor a drag selects label text. So the script in `label.rs:58-75` (a `mousedown`, `detail > 1`, `preventDefault`), which arms itself on hover, changes nothing on the component the library ships.
- The primitive's doc claim that "a drag still does" select (`label.rs:26-28`) is false once the styled sheet applies.

**Change:**
- Delete the script and the `armed` signal. Keep the primitive Radix-pure only if an unstyled consumer is a stated goal.
- Otherwise, on web, the field is available in the Rust handler: `dioxus_web::WebEventExt::as_web_event()` gives the `web_sys::MouseEvent` (`dioxus-web-0.7.9/src/events/mod.rs:186-206`).

**Subsumes:** L23 and B1's `detail > n` predicate, which has no other user.

### 3. (major) The delegated `data-dx-rule` + `closest()` interpreter covers the wrong set

**Claim attacked:** row 434, "Dioxus best-practice review" bullet: "`data-dx-rule*` read by one delegated capture-phase interpreter". The review (§3.4, rec. 7) says this "needs no install or uninstall".

**Evidence:**
- (V) A Dioxus handler on an element runs synchronously inside the DOM event on web (`dioxus-web-0.7.9/src/dom.rs:88-116`). On desktop it runs by blocking XHR (`dioxus-interpreter-js-0.7.9/src/ts/native.ts:353-395`). In both, Rust state is present at event time, and 133 `prevent_default()` calls (42 files, grep) rely on it.
- (I, by construction) `closest('[data-dx-rule]')` from the event target only finds rules on the target's ancestor chain. Any such event also bubbles through a Dioxus element, which can decide it in Rust with no mirroring.
- (V) The sites that really need page-side code have targets outside the rule's element, so `closest()` cannot find the rule:
  - L6, document Escape: `lib.rs:360-372`;
  - O1, document F6;
  - L10, "target outside the menu": `context_menu.rs:37-46`;
  - L11, window scroll lock;
  - V22, window Cmd/Ctrl+B: `preview/src/components/sidebar/component.rs:219-233`;
  - L25, document `dragover`: `drag_and_drop_list.rs:765-775`.
  These need a registry or an install step anyway.
- (V) For non-passive `wheel`/`touchmove`, a capture listener that lives as long as the page is the measured regression of rows 149/159. `scroll_lock.rs:94-104`: listeners "must not outlive the lock… `DOMDebugger.getEventListeners(window)` after one open-and-close must show no `wheel`/`touchmove` left." So L10 and L11 cannot use the delegated form at all.

**Change:** a three-way rule, enforced by the gate.
- (a) If the target is inside a Dioxus element, Dioxus has the event, and no capture or passive option is needed: rsx handler, Rust logic, no JS.
- (b) If the target is outside the element (window, document, outside): a named page-side function, installed only while active through the existing `JsListeners` lifetime (or `subscribe`).
- (c) If the target is inside the element but needs an event Dioxus lacks (`beforeinput`, `close`, `beforematch`) or capture phase: also a named page-side function.

The gate fails a JS `listen(el, type)` whose type Dioxus supports and which uses no option.

**Subsumes (V for the API, I for behaviour):**
- L2's `cancel` half: `oncancel` exists, and `prevent_default` works on web and desktop. The `__dxCancelAt` expando (`lib.rs:849-850`) and the proposed `recent(ms)` predicate both disappear, because L1 can read the Rust timestamp directly.
- L1, if moved to a Dioxus `onkeydown` on the `<dialog>`. Caveat: bubble phase only, so inner handlers see Enter first.
- T4/T6 `toggle`: `ontoggle` exists, and Rust knows which open it requested.
- S2-style moves.
- L21 and the window-level part of the gesture listeners, once `setPointerCapture` has retargeted the moves to the element (only that one `call` stays). Caveat: on desktop this means a sync XHR per move (see "attacks that failed").

### 4. (major) Rules-as-data is a union, not a vocabulary. Named typed TS functions are simpler with the same guarantees

**Claim attacked:**
- The closed `Rule` enum plus one interpreter, with the gate "enum and interpreter switch one to one, no default arm" (row 434).
- The claim that "the logic of every (b) block moves into Rust" (Step 3 design).

**Evidence:**
- (V, root doc §2.2 table) B1's predicate users:
  - `can-scroll`: L11 only.
  - regex and value filter: S11 only.
  - `focus-trap`: L4 only, and dead (finding 1).
  - `detail > n`: L23 only, and redundant (finding 2).
  - `dropEffect`: L25 only.
  - `exists`: O1 only.
  - `recent(ms)`: L1 only.
- (V) B3 has one user (S14, 31 lines).
- (V) B5's `scroll-step` (C2) is a command, not a reveal or settle.
- (V) B2's parameter list is the union of its users' quirks: 3 output modes, 2 direction modes, 2 resistance modes. The root doc already plans for creep: "band physics stay a small residue inside B2's 'resistance' variant" (§3, C4).
- (I) The creep starts at design time. Each family is sized to swallow exactly today's blocks, so the next behaviour needs a new variant, a backlog row, a TS case, a Rust case, conformance vectors and a parity-gate update.
- (V) After findings 1-3, B1 has 8 users. Three of them have the same shape, a global hotkey: L6, O1, V22.
- (I) The guarantees the plan wants can be had without a DSL:
  - a closed set (allowlisted entry points);
  - review of every new kind (a backlog row);
  - no `preventDefault`, rAF or observers elsewhere (the existing token gate);
  - behaviour tests (`bun test` per function).

  The enum/switch parity check is a regex parser over two languages. It exists only because of the DSL, and it adds to the row's own complaint about "regex-over-Rust complications".
- (I, lines) Today's B1 users come to about 190 lines (L6 10, O1 41, L10 9, L11 85, S11 25, L25 11, V22 ~10). A predicate DSL with AND/NOT, effects, serde, vectors and a gate cannot be smaller than that.
- (I, lines) B2 and B5 interpreters are about as long as the blocks they absorb (746 and 236 lines).
- (I) So B removes about 0 JS lines. The real reductions are:
  - A: T8 499, T11 74, W8 1,065;
  - G: S1, O3, V19, T9, T10;
  - R: W1, C3, V11, C1 (and C5, see finding 6).

**God-module:**
- (V) The plan already concedes that `bridge/rules.rs` is shared by four lanes "in separate marked regions… If that proves uncomfortable…" (root doc §5.4).
- (I) The families share no predicates, so nothing justifies one enum.

**Change:** replace the `Rule` enum and interpreter with these named page-side functions, each with a typed Rust config struct and a `bun test`:
- `hotkey(key, mods, skipEditable, prevent)` for L6, O1 and V22. O1's "exists" becomes "installed while a region is mounted", which Rust knows.
- `scrollLock({allowInside?})` for L11, with L10 as its `allowInside: rootId` option: one listener set, not two.
- `otpFilter(regex)` for S11. With Dioxus 0.8, `onbeforeinput` makes its cancel half Rust.
- `dropAnywhere(effect)` for L25.
- `dialogKeyForward` for L1, unless finding 3 moves it.
- One `pan.ts` with a typed config for L30, O5, C4 and V17.
- One `settle.ts` for W3, W5 and W7.
- C2 stays its own small `scrollStep`.
- S14 stays a plain function, not a family.

The gate stays: no `preventDefault`/`stopPropagation`/rAF/observers in `ts/` outside the allowlisted files, plus a backlog row per new file. Per-family Rust config types live in their own files, so no file is shared between lanes.

**Subsumes:**
- the `Rule` enum and switch;
- `check-bridge-rules.sh`'s parity half;
- the JSON vectors run by two runners (they become `bun test` only);
- `rules.install/remove` and the attribute-delegation machinery;
- the lane merge risk on `bridge/rules.rs`.

It is the same construction as the plan's "Step 3 design" bullet, at a smaller scale.

### 5. (minor) Eleven bridge verbs collapse to two

**Claim attacked:** lane 3.0 builds 11 verbs: listen, call, get, observe, matchMedia, transaction, commit, rules.install/remove, sample, set.

**Evidence (V for the platform, I for the design):**
- `MediaQueryList` is an `EventTarget` with `change`, so `matchMedia` is `listen` on a target descriptor.
- `observe` is a subscription whose callbacks are events. Dioxus itself turns RO/IO entries into dispatched `CustomEvent`s (`dioxus-interpreter-js-0.7.9/src/ts/core.ts:51-107`).
- `sample` is `listen` with `{per: 'frame', batch}`.
- `set` is a `call` of `style.setProperty`. `get` is a `call` that returns values.
- `transaction` is a message that carries an op list, and `commit` is one op in it.
- The review's rec. 10 (one ordered channel per webview) already makes ordering a property of the channel, not of a verb.

**Change:** keep two generic entry points.
- `subscribe(target, kind, options) -> stream`, with removal tied to the owner.
- `run(ops[]) -> results`, applied in one JS turn.

The typed Rust hooks keep the ecosystem names.

**Subsumes:** 9 TS entry points and their tests. It also shrinks the transport-seam surface (must #3).

### 6. (minor) C5's move to Rust adds a verb mode and an exit artefact and removes nothing

**Claim attacked:** residue doc §3, 3.WB (10 h): a batched per-frame `sample`, a `cargo test` replay, and C2 page-side wasm as the exit.

**Evidence:**
- (V, root doc §5.2) The batched mode of `sample` exists for C5 alone.
- (V, root doc §3, C5 row) C5 is passive and never cancels.
- (I) The port moves 477 lines of JS heuristics to about 477 lines of Rust. It adds per-frame IPC on desktop, a desktop measurement nobody can run here, and a possible second build artefact (C2).
- (I) The replay proof works equally well as `bun test` against the TS.

**Change:** keep C5 as a named TS module on the allowlist, with its reason ("passive stream heuristic, Rust only for policy"). Drop `sample`'s batched mode. Revisit only if a desktop C5 lag complaint ever arrives. This is a judgement call against the owner's "Rust first". It is flagged as **simpler**, not as wrong.

**Subsumes:** batched `sample`, the C2 exit, and most of 3.WB's 10 h.

### 7. (minor) web-sys earns its keep only inside the transport seam, and only for CSP

**Claim attacked:** the plan says nothing of a web transport other than eval ("Web eval uses `new Function`, so strict-CSP apps need `unsafe-eval`").

**Evidence:**
- (V) `dioxus-web` already depends on web-sys 0.3.98 (`Cargo.lock:9296-9297`).
- (V) The repo's own cfg rule allows `target_family = "wasm"` "for genuinely wasm-only internals nested inside a feature-gated hook" (`platform-rationale.md`, "The cfg axis").

**Change (could):** behind the must-#3 seam, the wasm32 backend calls the same `bridge.ts` exports through `wasm_bindgen` extern imports instead of `new Function`. The semantics stay async, so behaviour is the same on every renderer.

**Subsumes:** the `unsafe-eval` CSP caveat on web. It is **not** a reason for logic to fork per renderer (see "attacks that failed").

---

## Attacks that failed (coverage for the synthesis)

- **A web-sys logic path on wasm32, eval only for desktop.**
  - Wins: every decision synchronous in Rust on web, and no B at all on web.
  - Fails: desktop and mobile still need the page-side code, so the webview family gets two logic paths. That is the drift class the repo pays identity gates for (`check-open-focus.sh`, `check-theme-mode-writer.sh`).
  - On web the bridge is already microtask-fast (root doc §1.2), so the gain there is small.
  - Only CSP survives (finding 7).
- **Dioxus element handlers for gestures (B2), via pointer capture and Rust recognisers.**
  - Wins: logic in Rust on every renderer, Blitz included.
  - Fails on mobile, where drawer and toast swipes matter most. Each `pointermove` becomes a blocking XHR, and on Android it is base64-encoded into a header (`native.ts:568-593`). The style write then lands a frame late.
  - A page-side `pan.ts` stays justified, as a typed module (finding 4).
- **Dioxus `onwheel`/`ontouchmove` for L10, C3 or C5.**
  - Fails: root listeners are added with no options (`core.ts:109-130`) on the app root, which is a non-passive target. While any such handler is mounted, every wheel event in the app waits on the main thread. That is rows 149/159 again.
- **CSS for the scroll lock (L11)** with `overflow: hidden` or a frozen body.
  - Fails, as measured and pinned in `scroll_lock.rs:34-80` and `tier3-radix/scroll-lock.spec.ts`: the Chromium root `scrollbar-gutter` is lost.
  - `overscroll-behavior` in its new semantics is not in Safari (root doc §2.1).
- **Other CSS: `interpolate-size`, `popover=hint`, `closedby`, `overlay`.**
  - Fail: not Baseline, or Chromium only (root doc §2.1).
  - View transitions for V19 would cross-fade a snapshot, which looks different.
  - `@starting-style` is already in the plan.
  - L5's exit wait still needs `overlay` for top-layer exits outside Chromium.
- **Custom elements as the rule host.**
  - Real wins (I):
    - lifetime tied to the DOM through `connected`/`disconnectedCallback` (avoids #4064 and the missing unmount hook #5117);
    - config travels in the same edit stream as the render, so it is ordered with it on desktop;
    - a late definition only delays behaviour (elements upgrade when defined), so there is no #5919 race.
  - Fails as a replacement: dioxus-html 0.7.9 has no custom-event support (grep: no hits), so everything going to Rust still needs the eval channel. Marker children also disturb `>`/`:nth-child` selectors, which the variant-scope gate relies on.
  - Keep as a "could" for the install-while-active functions in finding 4.
- **Upstreaming to Dioxus** (window events #2405, listener options #1011, `detail`/`target` on events).
  - Would be ideal: all renderers, Rust.
  - Fails as a plan: nothing in 0.8 alpha 0-3 delivers it (audit §3, review §2). Worth filing in parallel, not something to wait for.
- **Dropping desktop parity in general.**
  - Fails: the plan already degrades desktop to "correct, not frame-exact" under G. Going further would only save the slow-channel harness, which the repo needs anyway, because Chromium CI hides desktop latency.
- **Merging L10 into L11.**
  - Partly holds: the semantics differ (react-remove-scroll "only inside the menu" against "only page-consumable"). It works as an option of one function (finding 4), not as a deletion.
- **L25 as an element `ondragover`.**
  - `DragData::data_transfer().set_drop_effect` exists (`dioxus-html-0.7.9/src/data_transfer.rs:56`). The serialized desktop version ignores it (`:140`).
  - Fails as drawn: it narrows "drop anywhere" to the list box, which is a behaviour change.
