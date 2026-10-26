# Board round 5, lens `admission`: per-effect admission by counterfactual witness

**Verdict: HOLDS WITH CHANGES**
**DESIGN findings: 0, DETAIL findings: 13**

- The construct holds: per effect, one ground (a counterfactual witness read by a library probe), structural detection as inventory, witnesses that run. I could not find an effect for which the ground itself has to be replaced. Every gap below can be closed by defining a parameter of the witness.
- It is not yet a construction in practice, though. The library owns *when* a probe reads. The author still owns *what* the probe reads (the `wrongPaint` predicate, the scenario, the declared target, and the row, which picks between a probe and the owner's verdict). There is also no control arm. So the round-4 class (an author-controlled witness) persists one level down. Finding 1 removes it with a library-defined trajectory criterion and scenarios taken from the vectors.
- The effect vocabulary is not closed:
  - `sameFrameWrite` is a deadline class, not an effect;
  - the built adapter's `attrs` effect is unwitnessed;
  - `after`, `release` and the reconciler's ops have no row;
  - `stop` and `dataTransfer` sit under a probe that cannot go red for them on Chromium.
- Several things page-side still escape admission: the bridge internals (modality writer, exit wait, watchdog, epoch minters) are an allowlisted category under another name. Two more contradictions: a green `focus`/`capture` witness demotes the effect to a verb that `FORBIDDEN_VERBS` bans, and witnesses run only on Chromium, which demotes effects that exist for other engines.
- Phase 0 cannot decide `reconcile` by its own rule: the order ground needs row 442, and the lag ground needs a verdict nobody schedules. The order counterfactual can instead be built page-side in the channel wrapper.
- Cost is small, about 80-100 tests and 4-6 minutes of worker time, provided order witnesses do not need a second (desktop-cadence) build per deploy.

Labels: V = read in code or source, W = web/spec, I = inference.

---

## 1. DETAIL, major (borderline blocker). The probes are library-timed but author-specified, so admission is still author-controlled

**Claim attacked.** Root `sync-js-root-2026-10-26.md:314` and `js-interop.md:101-103`: "a red counts only from a library commit-point probe ... never from ... an assertion the behaviour's author wrote".

**Evidence.**
- (V, root `:303-311`) The probes take an author argument: `wrongPaint(predicate)`, `defaultAction(occurred)` (which default action, on which input), and `outcomeOrder` (which order is the expected one). The author also writes the input scenario: the pointer path for `capture`, the key timing for `focus`, the growth sequence for `stickToEnd`.
- (I) **The delay arm is tautological for any visible write.** Take `wrongPaint(() => !effectApplied)` read one frame after the trigger. Delayed by 3 frames, it is red by construction. That is the "assert the deadline itself" route that round 4 removed from spec assertions, moved into the predicate.
- (V, root `:267-268, 303-311`) **The author picks the procedure by picking the row.** The same effect (`setVar`, `scrollBy`) gets the `wrongPaint` probe under "before paint" and needs the owner's verdict under "next frame". The row is declared by the behaviour's author.
- (V, root `:305-306`) **The author picks the counterfactual by declaring the target.**
  - A `document`/`window` target gets "dropped", and `defaultAction(occurred)` is then red whenever the prevent fires.
  - An element target gets "bubble at the element", a real test of need.
  - Declaring a broader target buys the weaker test. Example: a `keydown` Escape whose target is a focused element inside a Dioxus panel could be decided by an rsx handler on that panel.
- (I) For the "dropped" arm, then, the witness proves the effect is *reachable*, not *needed*. Point 4 of admission (`:315`, "the witness proves that each declared one is needed") overclaims. Whether a window-level cancel is needed at all is a product requirement, which belongs to the `adds:` row and the owner.

**Concrete change** (all are probe and harness parameters):
1. **A library-defined predicate for every write effect: trajectory inclusion.**
   - The harness records a library projection of the host subtree at the end of each frame, in both arms: rects, scroll offsets, computed `opacity`/`visibility`, `document.activeElement`.
   - The witness is **red iff the counterfactual arm paints a projected state that the real arm never paints**. This is the off-trajectory test: an unplaced panel, or a scroller not at the bottom while following.
   - A lagging effect paints a time-shifted copy of the real trajectory and is green. Its row is then **next frame by derivation**, and only a `lag`/`order` verdict admits it.
   - So the author no longer chooses between "before paint" and "next frame": the witness classifies.
2. **Scenarios come from the conformance vectors.** For every vector whose `expected_effects` contains E, the harness replays the event as a trusted CDP input on the construction site's host. Vectors are already required and proven against `decide` (`bridge/behaviours.rs` test `the_vectors_parse_and_cover_every_kind`, V).
3. **The cancel counterfactual is derived from the actual target.** If the replayed event's target has a Dioxus-rendered ancestor that the event bubbles through, the counterfactual is a bubble install there (the rsx placement). It is "dropped" only when the target is outside the app (`body`, `document`, `window`).
4. **Point 4 is reworded:** for a cancel with no other placement, the witness proves the cancel is reachable; the row proves it is required.

**Subsumes:** author predicates, the author's row choice, target-declaration gaming, and the tautological delay arm.

**Why not DESIGN.** The ground (a counterfactual witness read by a library probe) is unchanged. What changes is the probe's definition, a parameter. If the owner reads "the predicate is the author's" as the round-4 class persisting, this is the one borderline item.

## 2. DETAIL, major. No control arm, and the violation type can be forged

**Claim attacked.** Root `:316`: the witness "passes only if the error is the probe's typed `WitnessViolation`".

**Evidence.**
- (V) Playwright 1.60 (`playwright/node_modules/playwright/lib/worker/workerProcessEntry.js:2555`): a test fails when `status !== expectedStatus`. The planned wrapper therefore works against timeouts (`timedOut` is not `failed`, so it is red), and it can turn every non-violation error into an unexpected pass, which is also red. This part of the plan is sound.
- (I) **But nothing requires the probe to stay silent in the real arm.**
  - A predicate that is always true (`wrongPaint(() => true)`), a scenario that is broken in both arms, or a behaviour that does not work at all all produce the violation, and the witness "passes".
  - A spec can also `throw new WitnessViolation()` itself if the class is importable.

**Change.**
- Every witness is a pair: the same scenario with witness mode **off** must not throw (the control), and with the effect moved it must throw.
- `WitnessViolation` can be constructed only inside `playwright/witness/probes/**` (a tsc-API ban, the same machinery as item 0.1).
- The static gate checks both arms exist for each witness.

**Subsumes:** the "trivially red" route that the plan only closed for timeouts.

## 3. DETAIL, major. The unit is per construction site, but the gate, the types and the adapter work per kind

**Claim attacked.** Root `:300` ("at each Rust construction site's parameters ... a `const` check fails") and `:636` ("the gate checks one per effect").

**Evidence.**
- (V) `Behaviour` is a serde enum with runtime fields: `attr: String`, `value: String`, `cancel: bool` (`l30/primitives/src/bridge/behaviours.rs:46-66`). A `const` assertion cannot evaluate a value with heap `String`s built from props, so the "or a `const` check" path is infeasible for every behaviour with string or regex parameters: `hotkey{key}`, `otpFilter{regex}`, `settle{controlSelector}`.
- (V) `behaviours.gen.ts:22`: "What each kind declares at its sample parameters (the real declaration depends on them)". `Decision<E>` is generated per kind, so TypeScript never sees a site's parameters.
- (V) The adapter's only per-install guard is passivity (`bridge.ts:691-699`). That stops `prevent` for `hotkey{prevent:false}`, but nothing stops `focus`, `setValue` or `attrs` from an install whose parameters exclude them.
- (I) The gate counts one witness per (kind, effect), so a **new construction site reuses an old site's witness**. A new `hotkey{prevent:true}` for a shortcut whose default is harmless passes on L6's witness. That is the cheapest gaming route left.

**Change.**
- Effect-bearing parameters become types (typestate constructors, `Hotkey::<Prevent>`); a construction with an empty effect set is unrepresentable rather than checked.
- `use_behaviour` is `#[track_caller]` and sends a site id with the install.
- Rust sends the install's effect set, and the adapter refuses outside it (per install, not per kind).
- The debug coverage hook `__dxBehaviourCoverage` records (site, effect), and the gate requires a witness per **site id** and effect, with stale entries failing.

**Subsumes:** site reuse, runtime-parameter riders, and the infeasible `const` path.

## 4. DETAIL, major. The effect vocabulary is not closed, and two probe pairings cannot go red

**Claim attacked.** Root `:303-311` (the witness table), `:330-345` (registry effects), and `:351` (the closed vocabulary `capture, release, scrollBy, setVar, setValue, focus, after`).

**Evidence.**
- (V) There are three vocabularies that disagree.
  - `sameFrameWrite` appears in the registry and the witness table but is a deadline class, not an effect. `stickToEnd` writes `scrollTop` (no `scrollTo` effect exists), `settle` calls `scrollIntoView`, and `reconcile` calls `show`/`hide`/`showModal`/`close` and sets `indeterminate`/`defaultChecked`. None of these is in the vocabulary.
  - `Decision<E>` "generated from the Rust effect set" cannot be generated from a class name.
- (V) **The built adapter has an `attrs` effect that no table lists.** It writes `data-*`/`aria-*` on the host synchronously (`bridge.ts:700-706`), the fixture uses it (`ts/behaviours/fixture.ts`), and it carries paint-relevant attributes such as `data-swiping`. Today it is an unwitnessed rider in every behaviour.
- (V) `after(frame|ms)` and `release` have no row. Round 4 named `after` as a rider needing its own witness; the folded table gives it no next-best placement.
- (W) **`stop` is paired with `defaultAction(occurred)`.** `stopPropagation()` does not change the canceled flag (DOM, <https://dom.spec.whatwg.org/#dom-event-stoppropagation>), so this witness can never go red and `dialogKeyForward` can never be admitted.
- (I) **`dataTransfer` "dropped" with `defaultAction`:** the drag starts with or without `setData` on Chromium. Firefox needs data set to start a drag of a non-link element, so the need lives on another engine (finding 5). The `dragSource` half is therefore expected green on Chromium, contrary to `:320`.

**Change.** One closed effect list, generated from Rust:
- `prevent`, `stop`, `dataTransfer.{setData, dropEffect}`, `attrs`, `setVar`, `scrollBy`, `scrollTo`, `scrollIntoView`, `setValue`, `focus`, `capture`, `release`, `show`, `hide`, `setProp(indeterminate | defaultChecked)`, `after`;
- `send` and `remember` are explicitly exempt.

Each entry has a counterfactual and a probe:
- `stop`: dropped; probe "a later listener on the path ran" (`outcomeOrder`).
- `dataTransfer`: dropped; probe "the drop target received the data or the drop was refused" (`outcomeOrder`), run only on the engine where it matters, else deferred to row 442.
- `after`: re-entry routed through Rust (a `crate::sleep` plus a command), with the re-entered effects judged by finding 1's criterion.
- `attrs`: delayed, with the trajectory criterion.

**Subsumes:** "is next-best defined for every effect" (the table at the end of this report).

## 5. DETAIL, major. Chromium-only witnesses demote effects that exist for other engines

**Claim attacked.** Root `:316` ("A witness that turns green later (an engine gains native support) is red and demotes the effect to R"), together with `stickToEnd`, the `dragSource` half and the fallback-engine placement.

**Evidence.**
- (V, root `:342`, `:421`) `stickToEnd` exists for the states native anchoring misses: offset 0, fractional DPR, Safari 26 and older.
- (I) On Chromium with `overflow-anchor`, most growth scenarios keep the bottom natively. The delay witness is then green except in the offset-0 case, which finding 1's vectors must include.
- (V, `playwright.config` / `sandbox-chromium.cjs`, CLAUDE.md row 374) The container picks the newest Chromium. A revision that improves anchoring turns the witness green, the deploy suite goes red, and the only fix offered is a port.
- `dragSource` has the same problem (finding 4).

**Change.**
- Witnesses run under fallback emulation: `stripAnchorSupportsBlock`, `overflow-anchor: none !important`, and the A sheet removed (the specs already have these switches, root `:407`, `:421`). The counterfactual is judged against the engine the effect exists for.
- A witness that turns green opens a backlog row and is quarantined; it does not block the deploy.
- Demotion requires green under fallback emulation, or a Firefox/WebKit run (row 442).

**Subsumes:** A-masks-behaviour false greens.

## 6. DETAIL, major. A green `focus` or `capture` witness has no legal landing

**Claim attacked.** Admission point 2 (focus and capture are "delayed", so green means R) against root `:168` ("commit-sensitive effects are never async `call` verbs ... pointer capture and a `focus` meant for the next keystroke exist only as effects of a named behaviour") and `:621` (`FORBIDDEN_VERBS` bans `setPointerCapture`, `releasePointerCapture` and plain `focus`; `focus_prevent_scroll` is legal only from caller TSV rows).

**Evidence (V, the cited lines).** Admission sends a green effect to R, but R has no verb for it. Two parts of the plan contradict each other. Separately, touch pointers are implicitly captured (W, Pointer Events 3, <https://w3c.github.io/pointerevents/#implicit-pointer-capture>), so a `capture` witness driven by touch is green by construction.

**Change.**
- State the landing: a green `focus`/`capture` witness adds a caller-listed verb row (as `focus_prevent_scroll` does) keyed to the witness id, and `FORBIDDEN_VERBS` admits it only from that row.
- Reword `:168` as "never async verbs unless their witness is green".
- The `capture` scenario is mouse/pen only (from the vectors).

## 7. DETAIL, major. Bridge internals are an allowlisted-module category under another name

**Claim attacked.** Executive summary `:46` ("There is no allowlisted-module category"), section 4 `:534`, and `:293`, `:368`.

**Evidence (V).** Several page-side programs with commit-relevant effects are "bridge internals with an inventory row", not admitted:
- the runtime modality writer, a capture `pointerdown`/`keydown` writer of `html[data-dx-modality]` that the ring CSS and `openFocus` read in the same event (`:534`; today `popover_sync.rs:294`, `styles.rs:316`);
- the executor's exit wait (`:293`, "a bridge internal with an inventory row beside the watchdog");
- G's watchdog and the `pageObserver` discovery (`:291`, `:368`);
- the toggle-event suppression (`:294`);
- the epoch minters (finding 8).

The plan's own counterfactuals would judge each of them:
- watchdog: dropped, so the CSS fail-open alone (red with `animation: none !important`);
- exit wait: Rust waits for a `finished` fact (red: an invisible modal holds the page inert);
- modality writer: Rust writes it from a root handler (red: a ring flash after a mouse click).

**Change.** Admission covers **every effect the shipped TypeScript applies outside a verb row**, behaviour or internal. Each internal gets its witness pair in the same harness. The identifier scan of `bridge.ts` (`:633`) maps each effect site to either a verb row or a witnessed internal.

**Subsumes:** "does anything page-side escape" for the bridge.

**Still outside, and honestly so:** class (c) (`PREPAINT_JS`, `index.html`, `theme_seed`), because no counterfactual exists before wasm loads, and the hub's one `addEventListener`, which is mechanism with no effect of its own.

## 8. DETAIL, major. The order contract's page epoch for scroll has no owner

**Claim attacked.** Root `:276-278`: a page transition "mints `current + 1`". For scroll position, the slot owner is "the scroller's behaviour ... or a fenced setter row" (S4, V6, W2, L13, `:469-471`, `:618`).

**Evidence.**
- (W) A programmatic `scrollTop` write fires `scroll` exactly like a user scroll (CSSOM View, the scroll event steps), so a fenced setter cannot learn "the user scrolled since epoch e" from `scroll`.
- (I) Minting needs `wheel`/`touchstart`/`keydown`/`pointerdown` (scrollbar) listeners on each fenced scroller. That is a stateful page program that is neither a verb row (the one-guard cap), nor a reducer, nor an admitted behaviour. Lane 3.0's epochs are per install only (`bridge.ts:666-674`).

**Change.** One bridge internal `scrollEpoch`, installed with the first fenced setter on a scroller, refcounted, witnessed by `outcomeOrder` under the reorder of finding 9. Name the epoch source for every slot kind:
- open state: the executor's toggle report;
- focus: `openFocus`/`roving`;
- value: `otpFilter`.

## 9. DETAIL, major. Phase 0 cannot decide `reconcile` by its own rule, and the decision is per kind anyway

**Claim attacked.** Item 0.4 (`:665`): "first `reconcile`'s witness", where the page reconciler stays only on "a `lag` verdict or a red order witness" (`:290`). Also row 442: "The witness harness (item 0.6) and its order specs run first on this build".

**Evidence.**
- (V) The order ground needs the desktop-cadence build (row 442: about 14 h, outside phase 0 and outside the 250-280 h).
- (V) The lag ground needs an owner verdict, but `reconcile` is not one of the three measured ports (`:756-760`), and 0.5 runs last.
- So at 0.4 neither admitting ground exists, and "a declaration admits nothing" selects the Rust `transaction` by default, which risks re-porting T4-T7, S8, S9 and the modal driver later.
- (W/I) The kinds differ:
  - a delayed popover open paints nothing wrong (`display: none`);
  - a delayed `indeterminate` paints the wrong glyph (off-trajectory, red), yet the registry files it as `sameFrameWrite` and the executor paragraph as lag/order;
  - `defaultChecked` never paints.
- (V, `:293`) `openFocus` "runs inside the open step", but a Rust `transaction` is a list of closed verb rows and has no op that runs a behaviour program.

**Change.**
- (a) Build the order counterfactual in the channel wrapper (`bridge.ts:342-360` is already the seam). In witness mode, hold a fact until the next trusted event of a Dioxus-delegated type has finished dispatch, then release it; across sessions, release in reverse scope-height order. On web the Dioxus handler runs inside dispatch (`dioxus-web-0.7.9/src/dom.rs:88-116`, V), so this reproduces desktop's "event inline, fact later" without a patched build. Row 442 remains the fidelity check.
- (b) Decide the executor per kind.
- (c) Either give the `transaction` an `openFocus` op or keep the open step page-side for focus.
- (d) Fix the wording: the order row reorders a `send`, so "`send` stays synchronous" (`:313`) needs "except under the order counterfactual".

## 10. DETAIL, major. "Deterministic on Chromium and debug builds" is overclaimed

**Claim attacked.** Root `:314`: "The probes key on event-loop order ... so they are deterministic".

**Evidence.**
- (V) `afterFrames(n)` nests rAFs (`bridge.ts:337-340`). Called from a task, the first rAF fires in the *current* frame's rendering step, so `n = 1` lands before the same paint and the counterfactual does nothing for task-originated effects. If the latency session yields 1 frame, every delay witness of such an effect is green by artefact.
- (I) `focusAtNextInput` and `capture` compare a wall-clock CDP input (Playwright `keyboard.press`/`mouse.move`) with a frame-counted delay. Near the boundary (a 33 ms repeat against a 2-3 frame delay at 16.7 ms) the outcome races.
- (I; ResizeObserver spec, broadcast in the document's observer list order, not re-fetched) A probe RO "registered after the behaviour's" stops being after it once a behaviour installs later (any host mounted after page load). The probe then reads before the write: a false red in the witness arm, and a red control (finding 2).

**Change.**
- Count the delay in **completed paints**: rAF, then a post-paint hop, repeated n times.
- Dispatch the probe's inputs at frame counts, not milliseconds (CDP from a page-side frame counter, or `HeadlessExperimental.beginFrame` on the headless shell).
- All behaviour observers go through one adapter-owned RO dispatcher (behaviours are DOM-less, so they cannot create their own), and the probe hooks in after it.

## 11. DETAIL, minor. Witness mode ships, and the DOM-less compile breaks on the built `register`

**Evidence.**
- (V, `:667`) `ts/witness.ts` is "imported by the adapter through one line", and the deploy suite runs witnesses on the release build. A global switch that disables `preventDefault` would then ship to users and be reachable by any page script, and its bytes count against the JS ratchet.
- (V) `ts/lib/register.ts:8` reads `window.__dxBridge`, and every behaviour imports it. A `ts/behaviours/**` program with `lib` excluding `dom` (`:315`) does not compile as built.

**Change.**
- Witness mode lives in a separate bundle behind a non-default cargo feature (the `legacy-impl` pattern), and the deploy suite builds the witness variant.
- Behaviours `export const decide`, and the adapter registers them.

## 12. DETAIL, minor. Single-effect knockouts miss redundant pairs

**Evidence (I).** If two effects cover each other, each single knockout is green and both get demoted, and together they break. Examples:
- L6's document `keydown` Escape prevent and L2's rsx `oncancel` prevent on a modal `<dialog>`: either stops the close;
- `stickToEnd` against A (finding 5).

**Change.** Demote one green effect at a time and re-run the remaining witnesses on the demoted tree. The rule says so, and the gate records the order.

## 13. DETAIL, minor. Suite cost and selection

**Evidence and estimate.**
- (V/I) Construction sites times effects from the registry: hotkey 3, scrollLock 2, otpFilter 2, dropAnywhere 2-3 plus dragSource 3, pan about 6, settle about 4, scrollStep 1, openFocus about 10 overlay families, stickToEnd 1, reconcile about 6 (with order), roving 2. That is about 45-50 witnesses, and about 90-100 tests with the control arms of finding 2.
- (V) At 3.1 s per test (`playwright/durations.json`: 12,213 s for 3,959 tests, debug, 2 workers) plus a fresh page per arm, that is about 5-8 minutes of worker time, about 3-4% of the suite, inside the 4,787 s headroom.
- **The dominant cost is not the witnesses.** It is the plan's dependence of order witnesses on row 442's build: a second SSG build per deploy and a fourth 3.6 GB tree (`:709`), which the lane's one-build-copy disk rule forbids inside `test:changed`. Finding 9(a) removes it.
- (V, `tools/select.mjs:265-275`) A change to `bridge/behaviours.rs`, `conformance/behaviours.json` or `ts/witness.ts` maps to shared plumbing (the broad set) once more than 30 components use the bridge. Witness specs under `playwright/witness/**` need a rule mapping each to its behaviour's files, otherwise only the broad set selects them.

**Change.** Add the select rule, and add the durations entries with the witness lane (item 0.6).

---

## Is "next-best placement" defined for every effect? (answer table)

| effect | plan's next-best | defined? |
|---|---|---|
| `prevent` | dropped / bubble at element | yes; target derivation and the reachability-not-need caveat (finding 1) |
| `stop` | dropped, `defaultAction` | wrong probe, never red (finding 4) |
| `dataTransfer` | dropped, `defaultAction` | wrong probe; the need is on Firefox (findings 4, 5) |
| `setVar`, `scrollBy`, `setValue` | delay, `wrongPaint` | yes, but the row and predicate are author-chosen (finding 1) |
| `sameFrameWrite` | delay | not an effect (finding 4) |
| `focus`, `capture` | delay | yes, but a green result has no legal landing (finding 6); timing (finding 10) |
| `attrs` (built) | none | missing (finding 4) |
| `after`, `release` | none | missing (finding 4) |
| `remember`, `send` | exempt | fine; `send` is reordered by the order row (finding 9d) |
| reconciler ops | `sameFrameWrite` vs lag/order | inconsistent and per kind (finding 9) |
| G's watchdog, exit wait, modality writer, epoch minters | none (bridge internals) | missing (findings 7, 8) |

## Attacks that failed

- **Playwright expected-failure semantics:** `timedOut !== failed`, so a timeout is red (V, `workerProcessEntry.js:2555`). The wrapper can turn other errors into an unexpected pass. The plan's mechanism works.
- **Hiding an effect from the inventory:** `decide` gets picked fields and returns data, and the adapter applies it (`bridge.ts:676-710`). With the DOM-less compile (once finding 11 is fixed) there is no route.
- **The delay-only counterfactual is optimistic** (it decides on fresh inputs, while R decides on stale facts). For absolute and relative writes it produces the same late-write class as a shadow decision fed by delayed facts, and decisions that would change are covered by the order counterfactual. No change needed.
- **The boot probe** is R (Rust nodes plus `transaction`), and **reducers** are capped table rows whose only effect is dropping a `send`. Neither escapes.
- **The per-effect unit against one executor per element:** the kinds that share an element (`<dialog popover>`) share their witness outcome, so per-kind executors (finding 9) never split an element.
- **Unchanged from the admission side:** the root rule, the deadline table, the two kinds, the closed bridge with one eval sink, A/G/R, and sinks plus ratchet. Every finding above is a parameter of the witness, a vocabulary row, a scope line or a sequencing fix.
