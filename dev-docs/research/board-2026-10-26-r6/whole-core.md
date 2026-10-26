# Board round 6, lens `whole-core` (2026-10-26)

**Verdict: HOLDS WITH CHANGES**

**DESIGN findings: 1, DETAIL findings: 10**

Summary:
- The outer core holds a sixth time: the root rule, the deadline table, listener and command kinds, the closed bridge with one eval sink, A/G/R and sinks plus ratchet. I wrote the strongest alternative before reading the plan (below). On the class guarantee it is no stronger, and on desktop parity it is weaker. Measured against it, the plan's extra weight sits in three places: the executor fork, the witness machinery and the round-5 order additions.
- **The one DESIGN finding is a simplification** (finding 1). The command kind's "page reconciler or Rust transaction, decided per kind by witness" should be one page executor. The transaction branch keeps every page-side part anyway: the boot `pageObserver`, the exit wait and the toggle reports. It moves no decision into Rust, and per kind it puts two executors on one checkbox `<input>`, which breaks the plan's own "one executor per element" rule.
- The round-5 transition port is right in kind but unbuildable as written. The executor's `toggle` report runs after the trigger's `click` (finding 2), and no bridge root renders anything to host the port (finding 3).
- Phase 0 does not yet prove its inner rules. The `prevent` counterfactual is green by construction for any in-app target of a document-bubble listener, so proof port A (`hotkey` for L6) will most likely demote its only effect (finding 4). The trajectory projection cannot see `indeterminate`, `value` or `checked` (finding 5). The focus owner has no counterfactual in the phase-0 harness (finding 6), and `scrollOwner` is first built inside a port (finding 7).
- Hours: phase 0 re-sums to 81. The full port re-sums to about 346-383, inside "345-385". Two stale "250-280" figures and a 4,117/4,127 line count remain (finding 11).
- A new contributor faces about 90 named mechanisms (count below), and a lane that touches one behaviour must learn about 35 of them. Findings 1, 8, 9 and 10 remove or merge four of them without losing the class guarantee.

Evidence labels: **V** = read in the repo or in the pinned crate source, **W** = a web or spec source, **I** = inference. Dioxus paths are relative to `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`. "root" is `l434d4/dev-docs/research/sync-js-root-2026-10-26.md` at 45c7e0a2. "b28" is `/home/user/wt/b28` (batch 29 with the bridge as built). "l30" is the bridge lane.

---

## The alternative, written before reading the plan

From the eval sites (131 `eval(` hits in `primitives/src` and `preview/src`, b28), the bridge (`bridge/{mod,session,transport,behaviours}.rs`, `ts/bridge.ts` at 1,056 lines, one `Fixture` behaviour) and Dioxus 0.7.9:

- V: on web, Dioxus handlers run inside the DOM dispatch (`dioxus-web-0.7.9/src/dom.rs:110-115`), and every event downcasts to its `web_sys` event (`dioxus-web-0.7.9/src/events/*.rs`, `Synthetic<web_sys::Event>`), so on wasm Rust can decide anything synchronously.
- V: on desktop, a vdom event is a synchronous XHR that returns only `{preventDefault, stopPropagation}` (`dioxus-interpreter-js-0.7.9/src/ts/native.ts:352-395, 568-593`).
- V: primitives depend on no web-sys, and `web` is also on for desktop (`preview/Cargo.toml:38-39`).

**The strongest alternative I could build ("thin core").**
1. One sealed transport plus clippy def-path sink bans.
2. A closed set of page TS modules. A module is added only by an owner grant row at the merge base. The row cites its deadline-table row and a characterisation spec that fails when the module's effect is stubbed.
3. Commands as `data-dx-*` levels, applied by one page executor.
4. Everything else in Rust over the bridge, with one inbound sequence and a per-slot `based_on` fence on Rust writes.
5. Nothing else: no per-site typestate, no executor fork, no trajectory probes.

I rejected the other candidates:
- web-sys on wasm with a desktop shim: two implementations per behaviour, rejected three times before.
- A Dioxus fork that adds epochs or timestamps to serialized events: installed users run stock Dioxus.
- A wasm page agent: a second artefact.

**The plan against it.**
- The plan beats the thin core at one point: its probes are library-owned, so an author cannot write a green, while the thin core's hand-written stub spec can be written green. That is round 3's finding, and it still holds.
- The thin core beats the plan at four points:
  - one executor (finding 1);
  - no witness for a mechanism the platform spec already decides (finding 4's class);
  - no owner layer for a slot whose race is unobserved (finding 6);
  - no cross-session reorder machinery once the pump exists (finding 8).

The plan with these four removals is the best construct. Without them it is heavier than the problem: about 10 live behaviours and about 1,000 lines of page TS end up behind about 90 named mechanisms.

**The mechanism count a contributor must learn** (from root sections 2-5):

| family | members |
|---|---|
| layers, placement | 7 layers (delete, rsx, A, R, G, B, C); 5 deadline rows; 4 conditions; 2 kinds plus the one-shot call; 3 scopes |
| admission | 3 grounds in practice (witness, platform rule row, `verdict.json` lag/order: finding 9); 18 effects; 4 probes; the counterfactual table; control arm; `WitnessViolation`; trajectory projection; completed-paint delays; frame-counted inputs; RO dispatcher; coverage hook; typestate; site ids; per-install effect set; `Decision<E>`; DOM-less compile; witness bundle feature; `test.fixme` keys; demotion order; green-landing caller rows |
| order | slot, epoch, `based_on`, transition port, inbound pump, 4 owner classes (executor, focus owner, `scrollOwner`, native slot owners), slot-table TSV, rsx-read vs bridge-only, "Rust-only per property" |
| command and G | `(scope, state, based_on)` attribute, 2 executors, `pageObserver`, watchdog, fail-open animation, 2 paint tiers, engine-tier attribute, boot probe, `engine: always/fallback`, per-surface list, `applied{gen, rects}`, exit wait, toggle suppression, batch by tree |
| behaviour runtime | decide plus adapter, vectors, lease, ref-count, declared target/phase/passive, composition order, `REDUCERS`, closed verb tables, verb-row cap, `FORBIDDEN_VERBS`, `transaction`, listen/observe/sample/matchMedia, stream modes, `Program` x4 |
| enforcement | volume ratchet, legacy freeze, `js-grant`/`js-fix`/`adds:`, about 6 TSVs, 4 clippy bans, per-site `#[expect]`, rsx native-slot gate, hot-events plus payload table, tsc-API scans, budgets, fate table, slot-table gate, lease gate, witness-presence gate, `select.mjs` rule |
| measurement | `__dxBridgeDelayFrames`, witness delays, order counterfactual, desktop-cadence build, `desktop-pass` stamp, `__dxBridgeStats`, `__dxPerf`, `legacy-impl`, `?dx-impl` |

---

## Findings

### 1. DESIGN (simplification), major: the command kind needs one executor, not one chosen per kind by witness

**Claim attacked.** Root 2.2, Kinds: "Executor: the page reconciler or a Rust transaction, decided per kind by `reconcile`'s witnesses." Item 0.6 owns "the prototype reconciler (a page reconciler against the delayed Rust `transaction`)", and item 0.4 has "+4 for ... the per-kind executor decision".

**Evidence.**
- **The transaction branch keeps every page part.**
  - V root: the boot observer stays: "it is that observer, watchdog only, when the Rust-side executor is used" (2.2 Kinds, page reconciler bullet). The watchdog "keeps working when the executor is a Rust transaction" (2.3 item 3).
  - V root: the exit wait stays: "the wait is the one page-side step the Rust-side executor also keeps" (2.2, one executor per element).
  - V root: toggle suppression and browser-initiated reports are needed in both branches (2.2, "The only caller of show and hide").
  - V root: the epoch compare is page-side in both: "the page drops it" (`based_on` on the transaction too).
  - I: the transaction branch therefore saves no page machinery. It only moves the trigger from the MutationObserver checkpoint to an effect, and on desktop that costs one frame per open on every engine (root 1.4: effects run after the acknowledgement).
- **It moves no decision into Rust.** The desired state is the Rust-written attribute in both branches. The executor applies a Rust level verbatim. "Rust first" is about where the decision lives, so the fork does not serve the owner's bar (I).
- **It contradicts the plan's own one-executor rule** (V b28 `checkbox.rs:290-317`).
  - The one checkbox `<input>` carries `use_default_checked` (S9, `defaultChecked`) and an eval writing `input.checked` and `input.indeterminate` (S8) together.
  - The plan's own predictions are: `indeterminate` red, so page; `defaultChecked` never paints, so green, so transaction (root 2.2 admission point 7). That is two executors on one element.
  - Root 2.2 says: "One executor per element, in a fixed order ... the executor owns all of an element's kinds."
- **The witness that decides the fork cannot see the deciding states** (finding 5), so the per-kind decision would rest on a probe that is blind to it.

**Concrete change.**
- One page executor for every reconciled kind. It is a bridge internal, admitted as a whole by the reds it already has (the `indeterminate` mount case once finding 5 is fixed, and the exit wait's red, root admission point 6).
- Delete "decided per kind" and the "`reconcile` leaves the registry if green" conditional.
- Remove the prototype reconciler from item 0.6 and the per-kind decision from item 0.4.
- The native slot kinds join the same executor, with no later re-decision.
- Saves about 4-6 h and removes one mechanism, plus the `openFocus`-inside-a-transaction question (admission point 7).

**What it subsumes.** The `reconcile` row's conditional status in the registry, js-interop.md:20's "whichever its witness selects", the per-kind witnesses for `reconcile` (`indeterminate` keeps its red as the internal's admission), and the cross-executor ordering on a checkbox.

**Why DESIGN.** It removes one of the command kind's two executor mechanisms, which is the core's command kind. It narrows; it does not replace. If the owner reads "the executor's form" as a parameter of the command kind, it is the one borderline item in this report.

### 2. DETAIL, major: the transition port must fire from a synchronous hook; the executor's `toggle` report arrives after the trigger's `click`

**Claim attacked.** Root 2.2, carrier 1: "a page transition that any rsx-handled decision reads reaches Rust synchronously ... Rust's `seen` is then current at every later Dioxus event". The slot table gives open state's epoch source as "the toggle report, stamped from the level".

**Evidence.**
- W: the HTML hide-popover algorithm fires `beforetoggle` synchronously and *queues* the `toggle` event (<https://html.spec.whatwg.org/multipage/popover.html#hide-popover-algorithm>). Light dismiss runs that algorithm from the pointer events (<https://html.spec.whatwg.org/multipage/popover.html#popover-light-dismiss>).
- I: the trigger's `click` follows `pointerup` in the same input processing, so a port fired from the `toggle` report reaches Rust after the `onclick` it exists to precede. That is the motivating case at `popover_sync.rs:200-225` (V b28).
- W: for `<dialog>`, `close` is a queued task on the user interaction task source (<https://html.spec.whatwg.org/multipage/interactive-elements.html#close-the-dialog>).
- V root 2.3 item 1: `beforetoggle` is absent for `<dialog>` on Chrome before 132, Firefox before 133 and Safari up to 18.7.

**Concrete change.** Name the port's hook per slot:
- popover: `beforetoggle` on the element, read `newState`;
- dialog: an attribute MutationObserver on `open` (its record is delivered at the checkpoint after the close steps, before the next input task) plus `cancel`;
- focus: `focusin`.

These are per-element listeners the executor attaches at discovery, so they join the observer inventory. The "Escape then Enter" and "dismiss then `onclick`" specs must run with these hooks and fail with the `toggle` report.

**Subsumes.** Carrier 1's two named instances.

### 3. DETAIL, major: nothing renders the transition port; the bridge has no root component, and a provider was rejected

**Claim attacked.** Root 2.2, carrier 1: "one hidden element that the bridge's root renders once".

**Evidence.**
- V l30: the bridge uses only `provide_root_context` (`bridge/transport.rs:163, 177, 267`). Its only `#[component]`s are test fixtures (`:474, :486`).
- V l30 `transport.rs:47-51`: it rejects a root provider because "it needs every consuming app to mount a provider".
- V `dioxus-interpreter-js-0.7.9/src/ts/native.ts:352-356`, `getTargetId`: Dioxus routes an event only to an element that carries a `data-dioxus-id`, so a port created by page JS is never seen.
- Smaller points:
  - V root 2.4 C1 rejects the same trampoline ("The synthetic event must be a type Dioxus converts ... an abuse"). Low rate answers its reason 3, and its reason 2 does not apply to `input`. The text should say so instead of contradicting itself.
  - V `serialize.ts:76-84, 168-207`: a plain `new Event("input")` on a hidden input outside any form serialises only `value`, so Android's header limit is no issue.

**Concrete change.** Each owner whose slot is rsx-read renders its own hidden port input inside its own component: the popover hooks already return a spread (item 0.4), and they can return a port element too. Or the owner decides to accept a provider. Drop "the bridge's root renders once". Add a hydration-parity line for the extra element.

### 4. DETAIL, major: the `prevent` counterfactual is green by construction for in-app targets of a document or window listener, so proof port A most likely admits nothing

**Claim attacked.**
- Root admission point 2: `prevent` is "installed in the bubble phase at the Dioxus-rendered ancestor the event bubbles through".
- Phase 0's listener proof is `hotkey` for L6 (item 0.3, row 448).

**Evidence.**
- W (DOM dispatch, <https://dom.spec.whatwg.org/#dispatching-events>): the canceled flag is read after dispatch, so a `preventDefault()` in bubble at any ancestor cancels the default exactly as one at document bubble does. For every target inside the app the counterfactual has the same effect, so the witness is green by construction.
- V b28 `lib.rs:351-372`: L6 prevents Escape unconditionally at document.
- V `dialog.rs:439-442`: its one web caller is `DialogContentNonModal`. Its comment says the listener is at document "because it might not be focused".
- I, for the nesting scenario the plan prescribes (a div layer over a native modal dialog): focus is inside the native dialog, so the target is in-app and the witness is green. The demotion target is then an rsx handler on an element that `DialogContentNonModal` does not render.
- I: without nesting, Escape has no in-page default, so the result is green again.
- So phase 0 may end with no live listener behaviour admitted by a red witness. The same holds for every future document-bubble `prevent` (O1 and V22 already fall to platform rule rows).

**Concrete change.**
- Define the counterfactual relative to the site: the bubble install on the element the owning component renders (the host), and "dropped" when the event does not pass through it.
- State L6's expected outcome. It most likely decomposes into a bridge `listen` (R) for Escape anywhere, plus an rsx `prevent_default` on the panel for nesting.
- Make proof port A a listener whose witness is red by construction: `otpFilter` (S11, 25 lines). It has an element target, `beforeinput` is missing in 0.7.9, and its two effect classes (`prevent`, `setValue`) exercise `defaultAction` and `wrongPaint`. The alternative is `scrollLock`.
- About the same hours.

**Subsumes.** "Admission by witness for a listener is proven before ports depend on it", and it gives the per-site witness machinery a meaning: with a target-derived counterfactual the site barely matters.

### 5. DETAIL, major: the trajectory projection cannot see the states that decide the command kinds, and the vector set can choose the result

**Claim attacked.**
- Root admission point 3: the projection is "rects, scroll offsets, computed `opacity` and `visibility`, and `document.activeElement`".
- Point 7: "a delayed `indeterminate` paints the wrong glyph (off-trajectory, red)".

**Evidence.**
- I from V text: `indeterminate`, `checked`, `value`, `selected` and `defaultChecked` change none of the projected properties, so their witnesses are green by construction, against the plan's own prediction for `indeterminate`.
- I: even with the state projected, the result depends on the scenario:
  - A mount after hydration paints "unchecked" for three frames in the delayed arm only, so it is red.
  - A runtime toggle from checked paints "checked" longer, a state the real arm also painted, so it is green.
  - On an SSG page both arms show the server state until install, so it is green.
- Scenarios come from the author's conformance vectors (point 3), so including or omitting a mount vector picks the result. That is the author-controlled green the plan says it closed.

**Concrete change.**
- Add per-kind native state to the library projection (`:checked`, `:indeterminate`, `value`, `:popover-open`, `[open]`).
- Require every command-kind witness to include a client mount after hydration and a runtime toggle as library scenarios, not author vectors.
- Under finding 1 this only has to admit the single executor, not choose between two.

### 6. DETAIL, major: the focus owner, the `MountedData` write ban and the roving row have no counterfactual in the phase-0 harness

**Claim attacked.**
- Root admission point 6 names "the focus owner" among the internals that need a witness pair.
- Item 0.7 builds it (about 2 h), item 0.1 bans `set_focus`/`scroll`/`scroll_to` with 82 + 9 per-site `#[expect]`s, and the roving row is about 28 h.

**Evidence.**
- V `dioxus-web-0.7.9/src/events/mounted.rs:115-141`: on web, `set_focus` is a direct `web_sys` `focus()` from wasm.
- V root admission point 7: item 0.6's order counterfactual holds page-to-Rust facts only, so nothing in the harness delays or reorders a Rust-to-page `MountedData` write. The desktop race the owner exists for (root 1.4, `MountedData` as a channel) is therefore unobservable in phase 0. Under the plan's own rule the focus owner cannot be admitted before row 442.
- V, the docs disagree:
  - b28 `bridge/mod.rs:51` says "Use Dioxus's own API first ... `MountedData` (`get_client_rect`, `scroll_to`, `set_focus`)".
  - js-interop.md:56 and :62 still route agents to `MountedData::set_focus` and "`MountedData` where it suffices", while :93 calls it an owner bypass.

**Concrete change.** Either:
- give witness mode a focus-write delay hook (a patched `HTMLElement.prototype.focus`; wasm-bindgen calls the method on the object, I), with a red scenario (a late roving write after a click into another field); or
- move the focus owner, the `MountedData` ban and the roving row after row 442's desktop-cadence build reproduces the race.

Then make the three docs say one thing.

### 7. DETAIL, major: `scrollOwner`, the only owner with an input classifier, is first built inside a port that five rows depend on

**Claim attacked.** Phase 0 "proves the inner rules first" (root "Where the paper review ends"; row 434).

**Evidence.**
- V root 5.4: `scrollOwner` is built by 3.VS (+4 h). Its consumers are 3.PV (V6), 3.FC (S4), 3.SL (L13), 3.MS and 3.CA.
- Its classifier must tell the user's scroll from anchoring, snap re-snap, find-in-page, `#fragment`, `focus()` and its own writes (root 2.2, focus and scroll owners). That is the order contract's hardest open question, and nothing in phase 0 touches it.
- Item 0.7's proof ("one order spec per instance") cannot run the scroll instances before 3.VS exists.

**Concrete change.** Put a classifier spike into item 0.7, or a minimal `scrollOwner` for W2's setter into phase 0: wheel, touch, keydown and scrollbar `pointerdown` against an anchoring adjustment and a snap. Keep 3.VS's full owner. About +3 h to phase 0, taken from 3.VS.

### 8. DETAIL, minor: the cross-session reorder machinery exists only for the pre-pump architecture

**Evidence.**
- V root admission point 7: "across sessions it is released in reverse scope-height order".
- V row 442 item 1, last bullet, and item 6: "facts of two sessions in one batch".
- V root 2.2: one inbound pump "is the only page-to-Rust bridge path" and dispatches in sequence order on one root task.
- After item 0.7 no two sessions carry facts, and the pump has no page-side effect to admit (admission point 6 does not list it).

**Change.** Drop the cross-session arm from item 0.6's counterfactual and from row 442. Keep "fact after the next vdom event", and add "Rust write after a native transition".

### 9. DETAIL, minor: admission has three grounds, not two

**Evidence.** V root admission point 3: "only a `lag` or `order` verdict admits it". The counterfactual table: "a next-frame output | ... | `verdict.json` with a reason". The registry rows for `wheelBand` and `pan`'s output variants rest on it.

**Change.** State the owner's verdict as the third, closed ground (hardware evidence for a next-frame row) in the executive summary, js-interop.md and row 434, which today say "the witness or a platform rule row".

### 10. DETAIL, minor: reserved registry entries contradict "a row nothing uses is dead surface"

**Evidence.** V root 5.2: "a row nothing uses is dead surface". The registry lists `dialogKeyForward` ("Reserved") and `place` ("Reserved"), each with effects and admission text.

**Change.** Move both to one "exits, not entries" note under the registry. They get a variant only with their `adds:` row. That is two fewer table rows to learn.

### 11. DETAIL, minor: the hours are consistent; three stale figures

**Evidence (V, re-summed).**
- Phase 0: 17 + 4 + 15 + 11 + 13 + 13 + 8 = 81, which matches rows 444-450.
- Full port:
  - phase 0: 81;
  - candidate rows: 149 less 5 for 3.PL = 144;
  - roving: 24-32;
  - witness authoring: 12;
  - measured group: 85-114 (3.FZ 10-12, kit 7, 3.CA 24-32, 3.MS 30-45, 3.WB 12-16, removal 2);
  - total: 346-383, inside "345-385".
- Item 0.4's "13 call sites in 11 files" matches b28's 13 `use_popover_*` calls.
- Stale:
  - root :622, scope 1 "about 250-280 hours";
  - root :747, row 442 "outside the 250-280";
  - row 434 says "about 4,127 lines", the root says 4,117.

**Change.** Scope 1 becomes about 345-385 less what scope 2 defers. Row 442 is "outside the 345-385". Pick one line count.

Also minor, inside the same item 0.1:
- V `dioxus-rsx-0.7.9/src/attribute.rs:184-200`: rsx attributes expand to const tuples (`dioxus_elements::input::value.0`), not method calls, so clippy `disallowed_methods` cannot carry the rsx native-slot gate.
- It is an rsx scan, and it needs a shrink-only baseline of the shipped sites (`combobox/components/input.rs:152`, `input_otp.rs:434`, `radio_group.rs:436`, `checkbox.rs:341`, the `HIDDEN` slots), or it is born red before 3.FC builds the owners.

---

## Attacks that failed

- **web-sys on wasm with a desktop shim** (V `dioxus-web` events downcast to `web_sys`). Two implementations per behaviour, one of them never run here; the rejection by rounds 2, 3 and 5 holds. The seam's reserved web-sys backend (PR #5194) answers CSP, not the class.
- **An upstream Dioxus patch adding epochs or timestamps to serialized events**, to remove carrier 1 (V `serialize.ts` has a fixed field set). Installed users run stock Dioxus 0.7.9. The plan's "moves to rsx when Dioxus delivers it" reason check covers the upgrade path.
- **"The ratchet plus owner grant rows is enough".** It keeps "cannot recur" but lets a hand-written stub spec be written green. Library probes are the plan's answer to round 3, and they hold once findings 4 and 5 fix what they observe.
- **One global pump sequence as the epoch** (drop per-slot epochs). It fails: the port delivers out of pump order. A contiguous watermark then drops the trigger `onclick` decision that carrier 1 exists for, and a max-seen watermark applies writes Rust has not earned.
- **Per-slot epochs replacing the pump.** It fails for decisions that combine several slots (single-mode Accordion `beforematch` across panels, V `collapsible.rs:64-86`).
- **Desktop event serialization breaking the port's payload** (Android header). It holds: `new Event("input")` on a hidden input outside a form serialises only `value` (V `serialize.ts:168-207`).
- **Phase 0's file order** (`ts/bridge.ts` 0.0, 0.1, 0.6, 0.7, 0.3, 0.4; `session.rs` 0.7 then 0.3). It is consistent with rows 444-450 and transitively complete for every candidate row that needs the witness harness.
