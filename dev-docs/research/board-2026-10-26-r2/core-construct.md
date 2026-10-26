# Board round 2, lens `core-construct` (row 434 step 3), 2026-10-26

**Verdict: HOLDS WITH CHANGES**
**DESIGN findings: 1, DETAIL findings: 8**

Summary:
1. The root ("publish before, decide locally"), the closed bridge with one transport seam, A/G/R as layers and enforcement by sinks plus a ratchet all hold up under attack. I found no construct that beats them, and none of them is broken.
2. The one design flaw is in **the three-way placement rule**. Its discriminant is *where* the listener sits (element, window, capture, stream). The root's discriminant is *which commit point* the decision serves and *by when*. Because of that mismatch, the arms overlap (C4, L11, T12, S2/S3), the plan contradicts its own rule (W4 `beforematch`, L2 `close`), and the rule cannot place command programs at all (`settle`, `scrollStep`, `dxOpenFocus`).
3. It also cannot place the deadline class, so the plan settles it by accident of today's code: drawer drag (L30) goes to R while toast swipe (O5), which is the same gesture class, goes to `pan`. Roving focus is async `set_focus` in lane 2B, yet "focus for the next keystroke" is commit-sensitive in section 1.4.
4. Arm 1 ("rsx handler, synchronous on every renderer") holds only when the decision's one commit-time effect is `prevent_default`. A Rust handler's DOM writes, `set_focus` and pointer capture are all asynchronous on desktop (V).
5. The fix amends the rule and keeps the construct: index the rule by commit point, add a command arm and a latency-budget axis for deadline commit points, and derive registry admission from the generated effect sets.
6. The closed verb set has a hole: "verb" is undefined, and a verb needs no backlog row, so a 114-line policy (L16 `dxOpenFocus`) is mapped to "bridge transaction". The fix: a verb row is one member call, checked on the AST.
7. The transport should order bridge operations by construction, after the edits of the render that issued them and over one channel. Today a lane rule with no gate and no desktop CI does that job.
8. The remaining findings are details: G's `beforetoggle` reset is an unregistered behaviour, `js-interop.md` still teaches the dropped `Rule` DSL, behaviour composition through `defaultPrevented` needs an ordering rule, the rule has no migration trigger when Dioxus grows, class (c) has no single source, and the layer order needs better wording.

Labels: **V** = verified in this repo (`/home/user/wt/b28`) or in the pinned Dioxus 0.7.9 sources. **W** = web source (none fetched this round). **I** = inference from V facts.

Note on the brief: it names the layer order "R, then A, then G, then B". The plan's actual order is delete, rsx, A, R, G, B (root doc 5.1, row 434 "Order per block"). I attack the actual order (finding 9).

---

## Findings

### 1. DESIGN, major: the placement rule is indexed by listener location, but the root is indexed by commit point and deadline

**Claim attacked:** the three-way placement rule, root doc section 2.2 (`sync-js-root-2026-10-26.md:206-212`) and row 434 "The three-way placement rule (B)". The rule is said to be "decided per behaviour, and enforced by the gates of section 5.3".

**Evidence.**

a. **The plan contradicts its own rule (V).**
   - Arm 2 names `close` and `beforematch` as "an event Dioxus lacks" and routes them to a named behaviour (`sync-js-root:211`).
   - The same document maps L2's `close` to "one bridge `listen(dialog, 'close')`" (`:312`) and W4's `beforematch` to "bridge `listen(beforematch)`" (`:327`).
   - Arm 2 also names "the capture phase", yet T12 (a capture-phase close-on-scroll) is mapped to bridge `listen` with options (`:326`).
   - L3/L7/L8/L9 are "outside" targets, which arm 2 sends to a behaviour, yet they are mapped to bridge `listen` "no cancel, so async is fine" (`:313`).
   - The mappings follow an unstated axis ("is there a commit-time need?"), not the stated rule. Two agents reading the table would place these four rows differently.

b. **The arms overlap, and the rule gives no precedence (V/I).**
   - L11's `wheel`/`touchmove` is in arm 2 (window, capture, cancel) and in arm 3 ("continuous stream ... never a vdom handler").
   - C4 is in all three arms:
     - an element `pointermove` that Dioxus supports with no listener option (arm 1);
     - a capture-phase click suppression (arm 2);
     - a stream (arm 3).
   - Lane 2C followed arm 1 and put S2 on vdom `onscroll`/`onresize` (`select/components/list.rs`, row 434 "lane 2C"). The plan now re-routes S2/S3 because of arm 3 (`sync-js-root:373`).
   - Dioxus 0.8 will widen arm 1 (it adds `onbeforeinput`; listener options or window targets may follow). The overlap then gets worse. A `passive: false` wheel prop would put `scrollLock` in arm 1, which means one blocking XHR per wheel event on desktop (`native.ts:353-395`), the cost section 1.4 item 6 forbids.

c. **Arm 1's premise holds only for `prevent_default` (V).**
   - Desktop returns only `SynchronousEventResponse { preventDefault }` from a handler (`dioxus-desktop-0.7.9/src/webview.rs:191-195`, as cited at `sync-js-root:71`).
   - Everything else a Rust handler does reaches the page later:
     - `MountedData::set_focus` is a webview query (`dioxus-desktop-0.7.9/src/element.rs:137-160`, `window.interpreter.setFocus` through `query.new_query`). The webview is blocked in the handler's sync XHR, so it runs only after the handler returns.
     - Renders land in the next rAF (`native.ts:444-458`).
     - `dioxus-html` has no pointer-capture API (`dioxus-html-0.7.9/src/events/pointer.rs`: no `capture` symbol).
   - Yet sections 1.4 item 4 and 5.2 say commit-sensitive effects "exist only as effects of a named behaviour **or as rsx handlers**" (`sync-js-root:132, 375`). For focus routing, pointer capture and pre-paint attribute writes, the rsx half of that sentence is false on desktop.
   - Finding 4 shows a direct consequence: G's own `beforetoggle` reset cannot be the `onbeforetoggle` rsx handler that arm 1 would pick. `onbeforetoggle` exists at `dioxus-html-0.7.9/src/events/generated.rs:300`.

d. **The rule says nothing about the deadline class, so today's code decides (V).**
   - `drawer.rs:525-650` (L30) and `toast.rs:1433-1510` (O5) are the same program:
     - a slop or axis lock (`lockPx` / `SLOP`);
     - an interactive-target skip;
     - a velocity window, then one release decision;
     - a finger-following translation;
     - no `preventDefault` in either.
   - The only difference is where the translation is written. The drawer streams per rAF to Rust and Rust renders `transform: translate…` (`drawer.rs:451, 1069`). The toast writes `--toast-swipe-*` page-side and takes pointer capture (`toast.rs:1490, 1502`).
   - The plan keeps that accident: L30 is R ("no recogniser needed", `sync-js-root:286`, false given the axis lock at `drawer.rs:551-571`), while O5 is `pan` (`:291`).
   - C5, which follows wheel input with a per-frame `transform`, is R behind a measured exit. Its sibling C4 is `pan`.
   - Focus routing splits the same way:
     - Lane 2B moved roving focus (S6 chip navigation, L19 submenu-trigger focus from a key handler) to async `set_focus`, and the house precedents do the same (`toggle.rs:119`, `slider.rs:802`, `command.rs:487`; audit rows L17, L19, S6).
     - Section 1.4 still lists "a `focus` meant for the next keystroke" as commit-sensitive, never async.
     - W7's focus is a behaviour because of it.
   - What these cases share is a *deadline* commit point: the next frame for a finger-follow, the next input event for focus routing. Async Rust can meet such a deadline or miss it depending on renderer latency. The rule treats every commit point as instant (end of dispatch) and has no budget.

e. **The rule cannot place command programs (V).**
   - Half of what stays page-side is not a listener: `settle` (W3, W5, W7) and `scrollStep` (C2) are "a command from Rust" (`sync-js-root:224-225`).
   - `dxOpenFocus`/`dxCloseFocus` is 114 lines of policy that branches on modality, editability and author choice (`modality.rs:87-108`, audit L16). It is mapped to "bridge `transaction`" (`sync-js-root:325`), which by definition has no logic.
   - The three-way rule is about "the target and the event", so it has no arm for any of these. The registry admits `settle` and `scrollStep` against its own exclusion: "a stateful heuristic over a stream ... is R, not a behaviour: both are passive or paint-safe, so they can answer late" (`:237`; risk 3, `:426`). W3 and W5 have no cancel, no capture and no focus effect, so they can answer late.

**Concrete change** (amend the rule; keep the root, the bridge, A/G/R and the sinks):
- Restate placement as a two-question procedure derived from the root, applied per decision, not per block:
  1. **Which commit point does this decision feed?**
     - none: it is R. Any target, any phase, any event goes through bridge `listen`/`observe`/`sample`, streams coalesced or batched.
     - paint, for an open, insert or swap: G.
     - the end of dispatch or gesture start (cancel, `stop`, `touch-action`, pointer capture, `DataTransfer`): continue to question 2.
     - a deadline (the next frame for a finger-follow, the next input event for focus routing): compare the measured desktop bridge round trip from row 443 (`?dx-bridge-delay`) with one budget the owner sets once, for example "1 frame". Within budget it is R or rsx. Over budget it is a page-side program.
  2. **Who can decide it on time?**
     - An rsx handler, only if the decision's sole commit-time effect is `prevent_default` and the event is not frame-rate (this folds the hot-events rule into arm 1).
     - Otherwise a registry program, of one of two kinds: a **listener behaviour** (`hotkey`, `scrollLock`, `otpFilter`, `dropAnywhere`, `pan`) or a **command behaviour** (`openFocus`, `gateReset` from finding 4, and `settle`/`scrollStep` only if they pass question 1).
- **Registry admission becomes mechanical.** `behaviours.gen.ts` already carries each variant's effect set (`sync-js-root:231-232`). The gate requires every variant's effects to intersect {`prevent`, `stop`, `pointerCapture`, `focusNow`, `dataTransfer`, `prePaintWrite`}, or the variant to carry a measured over-budget deadline row. That turns "closed" from owner governance into a construction: a behaviour with no commit effect cannot exist.
- **Apply the deadline budget once** to L30, O5, C4, C5, V17, W7 and the 2B roving-focus sites, so the same class lands in the same place. Either the drawer moves to `pan` (`stream` output) or the toast moves to R. Today's choice is not a principle.
- **Fix sections 1.4 item 4 and 5.2:** "or as rsx handlers" applies to `preventDefault` only.

**Subsumes:** the arm overlaps (C4, L11, T12, S2/S3), the W4/L2/L3/T12 contradictions, the L30/O5 and C4/C5 splits, the 2B-versus-1.4 focus contradiction, the admission of `settle`/`scrollStep`, the unplaced L16 policy, and the behaviour-to-rsx migration as Dioxus 0.8 grows (finding 7 is its trigger). **Does not subsume:** how large the budget is. That needs row 442/443 numbers, which is why the budget is a single owner parameter.

**Why DESIGN, not DETAIL:** the brief names this rule as part of the core construct. The change alters its discriminant, adds an arm (commands) and an axis (deadline budget), and changes where at least 4 blocks land. It does not replace the rule's three destinations, and most of the existing mappings survive.

### 2. DETAIL, major: "verb" is undefined, so programs can launder into the verb table

**Claim attacked:** "The bridge surface is closed" (`sync-js-root:370`). "A new verb (data, one table line) is not [a backlog row]" (`:237`).

**Evidence (V):**
- L16, a 114-line decision program (`modality.rs:87-108` plus the policy body; audit row L16), is mapped to "bridge `transaction`" (`:325`).
- A transaction is "several operations in one JavaScript turn" with no branching. So L16 can land only as a verb whose table entry is the 114-line program, or as an unregistered blob in the bridge.
- The verb route has *weaker* admission than the behaviour route (no backlog row, no vector requirement, no spec tag).
- Nothing mechanical says a `CALLS` row is one DOM call. The gate checks closed tables, an own-property dispatch and a receiver kind (`:370`), not the body of each row.

**Change:**
- Define a verb in the gate. Each `CALLS`/`GETS`/`SETS` row in the canonical `bun build --minify-syntax` output must be a single arrow function whose body is one member call or one member read on its declared receiver kind, with arguments drawn only from typed fields.
- Anything else is a registry program (finding 1's command kind) with a row and vectors.
- L16 becomes `openFocus` (command), its 114 lines budgeted.

**Subsumes:** registry bypass through the cheaper verb route, and "component logic in the generic bridge".

### 3. DETAIL, major: bridge ordering should be a transport property, not a lane rule

**Claim attacked:** "An op that depends on a render's DOM is issued from an effect or `onmounted`" (`sync-js-root:131, 374`), and ref-counted installation (`:211`).

**Evidence (V/I):**
- On desktop, eval and queries are "a webview task on arrival, unordered with pending edits" (`:125`, `element.rs:30-80`).
- An eval from a handler runs before that handler's render (`:131`).
- The repo already records "separate evals have no guaranteed order" (`lib.rs:590-602`, as cited at `:325`).
- The best-practice review asked for "one ordered Rust-to-JS channel per webview" (`dioxus-best-practice-review-2026-10-26.md:96, 172`, rec. 10). The hardened plan specifies "one eval call site" (`:377`), which is not one ordered channel.
- Ref-counted install and uninstall of a non-passive listener are two bridge messages. If they reorder, the count drifts and a blocking wheel listener leaks (rows 149/159, `listener-inventory.spec.ts`).
- The lane rule has no gate and no desktop CI, so it cannot be verified where it matters.

**Change** (two properties of `bridge/transport.rs`):
1. All Rust-to-JS traffic goes over one long-lived eval (`await dioxus.recv()` loop), so messages are FIFO.
2. Element operations issued while a handler runs are queued and flushed with `dioxus_core::queue_effect`, after that render's edits. Dioxus-web uses the same mechanism to order its head insertions: `dioxus-web-0.7.9/src/document.rs:1, 125-161`.
- Commit-sensitive work never needed "issue now", because it lives in behaviours (finding 1).
- Add one stuck-render spec at `?dx-bridge-delay`.

**Subsumes:** the lane rule, the T5/T7 "same eval" ordering need (a transaction is still useful for atomic results), refcount drift, and a desktop-only class that Chromium CI cannot see.

### 4. DETAIL, major: G's synchronous `beforetoggle` reset is an unregistered page-side behaviour

**Claim attacked:** "The bridge removes `data-dx-placed` synchronously in `beforetoggle`" (`sync-js-root:243`), next to "generic verbs only and no component logic" (row 434, step 3 design).

**Evidence (V/I):**
- Native `popovertarget` and light-dismiss opens never pass through Rust, so a page-side listener must clear the attribute before paint.
- An rsx `onbeforetoggle` (which exists, `generated.rs:300`) writes the DOM only in the next render, which lands in a rAF on desktop (`native.ts:444-458`). Whether that lands before the first open paint is a race, so a stale `placed` paints. This is finding 1c in practice.
- The reset is therefore a listener with a pre-paint effect on every gated element, and it appears nowhere in the registry table (`:216-225`).

**Change:**
- Register `gateReset` (a listener on the popover/dialog, `beforetoggle`, effect `prePaintWrite`), with vectors and the generation token from `:246`.
- For Rust-initiated opens, do the reset inside the open transaction.

**Subsumes:** G's hidden JavaScript, and the "verbs only" claim for the bridge.

### 5. DETAIL, major: `dev-docs/js-interop.md` teaches the design the board dropped

**Claim attacked:** lane R's "the rule and the 'I want to ...' table are `js-interop.md`" (row 434, Lane R). CLAUDE.md:51-53 says to read it "before writing any interactive behaviour".

**Evidence (V):** `js-interop.md` still says:
- "rules as data (a closed Rust `Rule` enum applied by one interpreter)" (`:12-13`);
- lists `commit` as a verb (`:12`), which the plan dropped (`sync-js-root:374`);
- routes conditional cancels to `Rule::Gate` (`:36`), drags to `Rule::Pan` (`:37`), and layout-following values to `Rule::Publish` (`:38`);
- "The only new JS is a new bridge verb or `Rule` variant, each with a backlog row" (`:18`), against the plan's "a new verb ... is not [a row]" (`:237`);
- describes the bridge's purpose as "the interpreter" (`:55-57`).

The doc's own thesis is that "the neighbour you copy teaches the way" (`:52-56`), so this is the highest-leverage text in the plan, and it is wrong.

**Change:**
- Rewrite the rule and the table to match the amended placement procedure (finding 1), the verb definition (finding 2) and the registry kinds.
- Add a gate line: every `Rule::`/`commit` token in `js-interop.md` fails.

**Subsumes:** agents rebuilding the predicate interpreter from the doc.

### 6. DETAIL, minor: behaviours compose through `defaultPrevented`, which needs an ordering rule

**Claim attacked:** "ref-counted per (target, type, phase, passive)" is sound under conflicting needs.

**Evidence (I, from DOM dispatch semantics; V for the plan rules at `sync-js-root:232`):**
- Most of the attack fails (see "attacks that failed"): `prevent` is monotone, so behaviours that only prevent commute in any order.
- The hidden channel is a read. `decide(config, snapshot, event)` takes `defaultPrevented` in its snapshot (V22 reads it, `:218`), and `defaultPrevented` depends on which other behaviours ran earlier.
- Within one (target, phase), the order of listeners is registration order. With lazy, ref-counted attach and detach, that is mount history.
- So a bubble-phase reader and a bubble-phase preventer on the same target are order-dependent. Across phases the order is fixed.
- A future `hotkey` on Space, arrows or Home is also silently disabled while `scrollLock` (window capture, cancels scroll keys) is held. No vector can show this, because vectors are per behaviour.

**Change:**
- Gate rule: a behaviour that reads `defaultPrevented` is installed only at a (target, phase) strictly after every behaviour that may `prevent` the same type. In practice readers sit at window-bubble and preventers at capture, element or document-bubble.
- Add one cross-behaviour vector file (`scrollLock` held + `hotkey`).

**Subsumes:** order-dependent composition.

### 7. DETAIL, minor: no trigger moves a behaviour to rsx when Dioxus grows

**Claim attacked:** that the rule "stays correct as Dioxus evolves". The plan notes 0.8's `onbeforeinput` will make `otpFilter`'s cancel rsx (`sync-js-root:220, 269`) but nothing enforces the move.

**Evidence (V):**
- The pinned 0.7.10 sources are byte-identical to 0.7.9 in `dioxus-html`, `dioxus-web` and `dioxus-interpreter-js` (`diff -rq` found no differences). Nothing has moved yet.
- Every behaviour exists because of one arm-2 reason (window target, missing event, capture, listener option).

**Change:**
- Each `Behaviour` variant declares its reason as data.
- A `cargo test` reads `dioxus-html`'s `generated.rs` event list from the locked version and fails when a variant's "missing event" appears there ("`onbeforeinput` exists: move `otpFilter`'s cancel to rsx").
- Do the same for listener options and window targets once Dioxus exposes them.

**Subsumes:** registry entries kept alive by inertia after an upgrade.

### 8. DETAIL, minor: "one eval sink" is true only for repo call sites; class (c) still has no single source

**Claim attacked:** a single eval sink while class (c) (pre-wasm `PREPAINT_JS`, the modality writer) and `document::Script` exist.

**Evidence (V):**
- Runtime JS also enters through Dioxus itself: desktop inserts `Script`/`Link`/`Meta` through `self.eval(create_element_in_head(..))` (`dioxus-desktop-0.7.9/src/document.rs:46-78`; the trait default does the same at `dioxus-document-0.7.9/src/document.rs:74-82`). So the bridge's own `Script { src: asset!() }` load is an eval on desktop.
- In repo code, at least three call sites need the transport or an exception:
  - `preview/src/eager_head.rs:86` (V11, a `Document` impl delegating `eval`);
  - `preview/src/theme.rs:145` `theme_seed` (`document::eval` of `PREPAINT_JS` for hosts without `index.html`);
  - the modality writer, which apps without preview's `index.html` install at runtime today as a prefix of the overlay scripts (`modality.rs:70`, `theme.rs:860`).
- The plan calls class (c) "out of scope". Row 434's own Detail still claims to subsume the three-copy and two-copy identity checks. The audit had the construction for that (`js-audit-2026-10-25.md:250`: generate `PREPAINT_JS` and `MODALITY_JS` from one `.ts` and embed it), and the hardened plan dropped it.

**Change:**
- State the sink count honestly: one `eval` in `transport.rs`, plus V11's delegation, plus allowlisted `transport::run_static(&CONST)` for class (c) runtime installs.
- Restore the audit's one-source construction for class (c) through the bundle chain. This retires `check-open-focus.sh:34,115` and `check-theme-mode-writer.sh:132` identity checks.
- Say where the modality writer lives once the overlay scripts are gone: in the bridge bundle at load.

**Subsumes:** the two identity gates, and an unlisted runtime eval site.

### 9. DETAIL, minor: the layer order is a preference among decisions, not a per-block first match

**Claim attacked:** "each block gets the first that works" (`sync-js-root:140`), in the order delete, rsx, A, R, G, B (`:351-356`).

**Evidence (V):**
- The plan's own mappings combine layers: T8 is A + Rust verification + G, and W8 is R + A + G + `settle` (`:282, 296`). So "first match" describes no block.
- R and G are disjoint by definition (no paint need versus a paint need), so their relative order is vacuous.
- "rsx before A" let lane 2C put a frame-rate event on the vdom (S2), and left L23 as a script that shipped CSS had already made dead.
- Where the order does bite (R before B for the deadline class), finding 1's budget decides, not the order.

**Change:**
- Restate it as: per decision, delete > A (style-only or platform-owned) > rsx (prevent-only, human-rate) > Rust over the bridge (R, or G when a paint is at stake) > registry program.
- Note that a block may use several layers for different decisions.

**Subsumes:** wording disputes in lane reviews. **No block gets a worse result** under the plan's order once finding 1 is applied. I checked T8, S1, O3, V19, C5 and W8.

---

## Attacks that failed

- **Ref-counting by (target, type, phase, passive) is unsound (I):**
  - `passive` is not part of a DOM listener's identity (type, callback, capture). The plan keys physical listeners with `passive` included, so a passive and a non-passive listener coexist rather than flipping in place, and nothing is re-added out of order.
  - `prevent` effects are monotone and commute.
  - Conflicting phases are separate listeners with a fixed capture-before-bubble order.
  - The only leak is the `defaultPrevented` read channel (finding 6).
- **"The registry degrades into bespoke scripts with a stamp":** partly true by count. About 11 distinct programs serve about 14 blocks (`pan` has 3 output variants, `settle` 3 parameter sets). But it is honest about it ("no residue" was withdrawn), and the generation parity, the effect declarations, the lifetimes and the budgets are real gains. Creep is the risk. Finding 1's effect-set admission turns that risk from governance into a construction.
- **One eval sink is unachievable:** it is achievable for repo-authored JS, with the exceptions named in finding 8. Clippy `disallowed_methods` plus the `Script { src }` allowlist closes the authored routes. Dioxus's own head-insertion evals are not authored strings.
- **A closed verb set makes people route around it:** adding a verb is cheap (one row). The real hole is the opposite one, verbs too cheap (finding 2). Read-then-write needs are covered by G or a command behaviour, so there is no incentive to bypass.
- **Arm 1 breaks if Dioxus drops synchronous desktop events:** that would break 133 existing `prevent_default()` calls across the ecosystem, not this plan. Rows 442/443 detect it. Dioxus 0.7.10 is byte-identical on these paths (V).
- **Liveview breaks "synchronous on every renderer":** liveview events are async (`dioxus-liveview-0.7.9`), but liveview is not a supported target (`templates/starter/Cargo.toml.liquid` features are web, desktop and mobile only).
- **The root is wrong:** no counterexample. Every page-side residue I examined decides from event fields, the DOM or published state.
- **Reorder the layers (A first, or B before G):** no block improves. A never stands alone (it always has a Rust backstop), and B before G would give up Rust-first for S1/O3 for at most 1-2 desktop frames per open, against the owner's bar.
