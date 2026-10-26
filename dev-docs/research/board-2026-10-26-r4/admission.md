# Board round 4, lens `admission`: the amended admission rule and the closed registry

**Verdict: HOLDS WITH CHANGES**
**DESIGN findings: 1, DETAIL findings: 7**

- The root rule, the deadline table, the two kinds, the one eval sink, A/G/R and enforcement by sinks plus ratchet held again. I attacked each from the admission side and could not break any of them.
- There is one design-level flaw, and it is in the same layer as rounds 1-3. Admission is still defined by a **list of effect types**, applied **per variant** ("one detected effect or one witness record admits the variant"). It is not derived from the placement rule it serves. Four consequences follow:
  - ground 1 detects what a behaviour can do, not what it must do;
  - riders get in: `pan`'s next-frame outputs, `hotkey{prevent:false}`;
  - the witness is controlled by the author, who picks the row and writes the assertion, so it is still declarable;
  - the plan's own ordering condition (2.2 condition 1(b)) produces page-side code that neither ground covers, so `reconcile` sits under the wrong ground.
- The fix keeps "admission by witness" and states it once, as a construction. Every effect of a behaviour must turn its spec red when that effect alone is moved to its **next-best placement**. That means dropped (what R would do to a cancel), moved to the rsx bubble handler, delayed by R's latency, or reordered against the next vdom event. The red must come from a library commit-point probe, never from a latency or patience assertion. Structural detection becomes a type-checked inventory instead of a ground. With this, admission is a function of the placement rule, and a new placement condition brings its own witness.
- Details: detection aims at the wrong layer (the built adapter applies `preventDefault` centrally); a witness record is a committed file, which is a declaration; activation-gated APIs and pointer capture are not "late is wrong by construction"; three page-side programs sit outside the registry (the boot probe, page-side reducers, the runtime modality writer filed as "pre-wasm"); under B or C, legacy files are an open sink; the verdict schema admits on R bugs; `reconcile`'s ground.
- Trajectory: the layer kept moving because it was an enumeration. Once it is derived, round 5 should find only harness and probe details.

---

## 1. DESIGN, major. Admission is an enumeration applied per variant, so the registry is closed in membership but not in content, and the witness is author-controlled

**Claim attacked.**
- Root `sync-js-root-2026-10-26.md:278-282` ("admitted on one of two grounds ... a declaration admits nothing").
- `:542` ("the gate refuses a variant with neither a detected effect nor a witness record").
- `js-interop.md:180` (same rule).
- The round 3 claim that the registry is now "closed by construction".

**Evidence.**
- **(V) The unit is the variant, while placement is per decision.**
  - The gate refuses only "a variant with neither a detected effect nor a witness record" (root `:542`, `js-interop.md:180`), so one ground admits every effect of the variant.
  - The plan's own registry table leaves effects with no ground:
    - `pan`: "`capture`, `prevent` (C4) (detected), `setVar` or `scrollBy` per output" (`:298`);
    - `otpFilter`: "`prevent` (detected), `setValue` with selection".
  - Its closure rule (`:262`, condition 2) says the per-move output "is a next-frame decision placed by that row". By the table, that is bridge plus Rust unless a `fail` verdict exists. Admission lets it ride inside `pan` on the recogniser's `prevent`/`capture`, with no verdict.
  - The `after(frame|ms)` effect (`:312`), which re-enters `decide`, gives every admitted behaviour a page-side scheduler that needs no ground of its own.
  - The `adds:` rule gates a new **kind** (`:542`). A new effect added to an existing kind passes it.
- **(V) Admission is per kind at sample parameters, but effects depend on install parameters.**
  - `behaviours.gen.ts` (lane 3.0 branch, `/home/user/wt/l30/primitives/src/ts/behaviours/behaviours.gen.ts`): "What each kind declares at its sample parameters (the real declaration depends on them)".
  - The fixture's `decide` returns `prevent` only `if behaviour.cancel` (`ts/behaviours/fixture.ts`).
  - So `hotkey{key, mods, skipEditable, prevent}` (`:293`) with `prevent: false` is admitted on `prevent (detected)`. Yet it is a page-side `listen` with a key filter, which the table places in R. This is a behaviour that "picks up a ground incidentally".
- **(I, from the code shape) Ground 1 detects capability, not necessity.**
  - A conditional `prevent` that no caller ever arms, or one on a rare branch (for example `settle`'s `cancelOnInput` growing a `prevent` on a key), makes the detected set equal the declared set. The gate goes green and the witness is skipped.
  - Matching the detected set with the generated set closes the loop between **declaration and code**. It does not close the loop between **code and need**.
- **(V/I) The witness verifies the author's own row choice.**
  - The test mode "delays every effect of one behaviour by the deadline of its row", and "the behaviour's spec tag must go red" (`:280`). The row and the assertion are both chosen by the author.
  - A spec that asserts the deadline itself goes red by construction. Examples:
    - reading `scrollTop` in the same task as the growth;
    - an `expect.poll` with a 10 ms timeout;
    - "the open lands in the edit's frame".
  - So the red proves only the author's own row choice. That is the "declared, not checked" class that round 3 removed from ground 1, moved into ground 2.
  - The converse fails as well. With a spec-authored user-level assertion, one event task of delay (a `setTimeout(0)`) is invisible to Playwright, whose next CDP key arrives milliseconds later. The rule "green, therefore R" (`:280`) then sends `openFocus` or `roving` to R, which the plan itself rejects on desktop (`:264`, condition 4: `set_focus` is an async webview query).
- **(V) There is no ground for the ordering condition.**
  - Condition 1(b) (`:261`) places non-commutative decisions "by the owning behaviour, which sees both events in DOM order". The examples are the carousel step from the settled index and the reconciler's handling of browser-initiated closes. These decisions are page-side because of **order**, not because of a deadline.
  - Neither ground covers order. The plan already has the right witness, "the desktop-cadence build reorders facts against events and gets one spec per instance" (`:261`), but admission does not name it.
  - So `reconcile` is filed under `sameFrameWrite (witness)` (`:302`). Its open is not a before-paint write: an unshown `[popover]` is `display: none` (W, HTML rendering, `[popover]:not(:popover-open):not(dialog[open]) { display: none }`, https://html.spec.whatwg.org/multipage/rendering.html#flow-content-3). A late open therefore paints nothing wrong; it paints the old state one frame longer. That is latency (the next-frame row: the owner's budget) or order (generations), not a wrong paint. See finding 8.
- **(I) This is the fourth time the layer moved for the same reason.**
  - Round 3 found that the next-frame row could not pass admission, which forced the allowlisted module. Round 4 finds that the ordering condition, the closure rule's outputs and the per-install parameters cannot be expressed either.
  - Each placement row or condition the plan adds creates page-side code that an enumerated admission list has to be patched to recognise. That makes it a class, not a set of instances.

**Concrete change (amends the admission rule).** Admission is **derived from the placement rule, per effect, by a counterfactual witness**:
1. **The unit is the effect, at the parameters of each Rust construction site.**
   - The generated effect set is computed per parameterisation; for example, `hotkey{prevent: false}` has an empty set.
   - A construction whose set is empty does not compile: split the Rust type, or use a `const` check. The call site uses `listen`.
2. **One ground: the counterfactual witness.**
   - For each effect, the adapter's witness mode moves **only that effect** to its next-best placement under the table:
     - `prevent`/`stop`/`dataTransfer`: dropped, which is exactly what R does to them;
     - an element-target default-action decision on an event Dioxus delivers: installed in the bubble phase at the element, which is the rsx placement;
     - writes, `focus` and `capture`: delayed by **R's modelled latency**, provisionally 3 frames, set by step 0's desktop distribution, and never less than the row's deadline;
     - order-ground decisions: the fact reordered after the next vdom event, using the desktop-cadence harness.
   - `remember` (memory) and `send` stay synchronous, so a red cannot be an artefact of a corrupted state machine.
3. **A red counts only from a library probe at the commit point:**
   - `wrongPaint(predicate)`: an RO registered after the behaviour's records the state at the end of the RO phase, which is the painted state;
   - `defaultAction(occurred)`;
   - `focusAtNextInput`: the next key is driven at the platform's key-repeat rate;
   - `outcomeOrder`.
   - Latency numbers never count; they are the owner's budget, and the next-frame row's witness stays the owner's verdict, which is the same counterfactual measured on hardware.
4. **Structural detection becomes the inventory, not a ground** (finding 2): types guarantee that the effects in the code are a subset of the declared ones, and the witness proves each declared one is needed.

**What it subsumes.**
- The ground 1 / ground 2 split.
- The next-frame special case.
- `settle`, `scrollStep`, `roving` and `place` "only on a red witness" become one rule.
- The order condition's page-side owners (condition 1(b)).
- Riders (`pan` outputs, `after` loops, `otpFilter`'s `setValue`).
- `hotkey{prevent:false}`.
- Finding 4's misfiled effects.
- The unchecked "capture phase" reason: its counterfactual is the bubble install.

**What it does not subsume:** the fidelity of the desktop counterfactual (the reorder and latency models are inferred until row 442), and the owner's budget.

**Why DESIGN.** It changes what admission means: the unit (variant to effect), the set of grounds (two enumerated grounds become one derived ground), and what counts as red. That is the same yardstick round 3 used for "adds a second ground and removes a category". The root, the table, the kinds, the sink and the ratchet do not move. If the owner reads "counterfactual per effect" as a refinement of the witness rather than a new rule, this becomes a major DETAIL. In either case, without it the registry's **content** is admitted by human review of grant rows, not by construction.

---

## 2. DETAIL, major. Structural detection targets the wrong layer; make it a type and a DOM-less compile

**Claim attacked.** Root `:279` and `:539`: "the gate finds it with the TypeScript compiler API (symbol resolution over the adapter and `decide`)".

**Evidence.**
- (V) `decide` returns data, and the shared adapter applies it: `bridge.ts:693-698` on `lane/js-bridge-3-0` reads `if (decision.prevent) event.preventDefault(); ... if (decision.stop) ... event.stopPropagation()`.
  - Symbol resolution over the adapter finds `preventDefault` for **every** behaviour.
  - Over `decide` it finds it for **none**, because the effect is the object key `prevent`.
  - A syntactic scan for the key is fooled both ways: `prevent: false`, a spread from a helper, or a dead branch.
- (V) The adapter checks only `d.passive` before applying `prevent` (`:693`). It never checks the generated effect set.
- (V) `decide` receives picked field values (`pickAll(event, impl.fields)`), not the event, so it cannot hide a direct call through the event object.
- (I) A behaviour module could still reach the DOM through globals. The planned "module that reads the DOM" ban (`js-interop.md:180`) is a scan, and `bridge.ts` itself uses `window as any`.

**Change.**
- Generate a per-kind `Decision<E>` type from the Rust effect set, so returning an undeclared effect is a `tsc` error.
- Compile `ts/behaviours/**` as its own program with `lib` excluding `dom` and a tsc-API ban on `any`, so a behaviour cannot name the DOM at all.
- The adapter refuses at run time any effect outside the generated set.
- Keep the identifier scan for `bridge.ts` and legacy code only.

**Subsumes:** the "hide" route; the declared-versus-contained direction of the gate.

## 3. DETAIL, major. A committed witness record is a declaration; the deploy suite must re-run each witness

**Claim attacked.** Root `:280` and `:542`: "the gate checks the committed witness record (spec tag, delay, recorded red result)".

**Evidence.**
- (V) The gates run no browser (CLAUDE.md, lane rules). A record of a past red is a hand-writable file, and `test:changed` runs only in a lane, so nobody re-checks it.
- (V) Playwright supports expected failures (`test.fail()`), and `addInitScript` is in use (36 sites under `playwright/`), so a witness mode can be armed before the bridge loads.

**Change.**
- Each witnessed effect is a `witness(kind, effect)` test that wraps `test.fail()`. It passes only if the thrown error is the probe's `WitnessViolation`; a timeout or locator error is a failure of the witness, which removes the "trivially red" route.
- It runs in `test:changed` and in the deploy's full release suite.
- The static gate checks one such test per witnessed effect.
- A witness that turns green on a later deploy (for example, an engine gains native support) is red and demotes the effect to R.
- Records and `verdict.json` must exist at the merge base, under the grant-row rule, so a lane cannot admit its own behaviour.
- The probes are deterministic on Chromium and on debug builds, because they key on event-loop order (end of dispatch, RO phase), not on frame timing.

## 4. DETAIL, minor. Pointer capture and activation-gated APIs are not "wrong when late by construction"

**Claim attacked.** Root `:279`: "These effects are wrong when late by construction, because dispatch has ended".

**Evidence.**
- (W) `setPointerCapture` works at any time while the pointer is in the active buttons state, and it applies at the next pointer event (Pointer Events 3, https://w3c.github.io/pointerevents/#setting-pointer-capture). A late capture loses moves; it is not structurally impossible.
- (W) User activation is transient for an implementation-defined "transient activation duration ... at most a few seconds" (HTML, https://html.spec.whatwg.org/multipage/interaction.html#transient-activation-duration). Chromium uses 5 s (I). An activation-gated call over the bridge within that window works on Chromium and Gecko. WebKit's clipboard is stricter (I).
- (W) `dataTransfer` writes (read/write mode only in `dragstart`; `dropEffect` during dispatch) and `preventDefault`/`stopPropagation` are genuinely structural: HTML DnD, https://html.spec.whatwg.org/multipage/dnd.html#drag-data-store-mode.

**Change.** Move capture and activation-gated calls to the witness ground. Finding 1 subsumes this.

## 5. DETAIL, major. Three page-side programs sit outside the registry with no admission

**Claim attacked.** "What stays page-side is the bridge, the transport seam, class (c) ... and the closed registry" (root section 4, `:440`).

**Evidence.**
- **(a) The boot probe** (root 2.1, `:225`).
  - It is a multi-step page program: edge shift, reopen ×3, nested submenu, scaled parent. It runs "synchronously at every bridge load".
  - It is neither a verb (the single-member-call cap) nor a behaviour (there is no admission row).
  - By the plan's own logic its deadline is **none**, since an absent attribute counts as fallback. So it is R: rsx-rendered probe nodes plus `transaction`/`get`.
- **(b) Page-side reducers and "transition emitters"** (V root `:378` W1, `:429` L22, `:448` W8; residue `:71`: "Intent classification ... is a page-side transition emitter").
  - This is decision logic (derived keys, RTL normalisation, wheel-intent classification) in a bridge option.
  - It is not in the closed tables of 5.2, not built in `bridge.ts` (V: no reducer code on `lane/js-bridge-3-0`), and not in `js-interop.md`.
- **(c) The "runtime modality writer" is not pre-wasm.**
  - (V) `MODALITY_JS` is prefixed into runtime overlay evals at `primitives/src/top_layer/popover_sync.rs:294` and `top_layer/styles.rs:316`.
  - (V) `theme_seed` evals `PREPAINT_JS` after boot (`preview/src/theme.rs:145`).
  - Both are admitted as "named pre-wasm exceptions" (root section 4) on a reason that is false for them. The modality writer computes `openFocus`'s input, and the theme's ring CSS reads it before paint.

**Change.**
- Add a closed `REDUCERS` (derived-key) table under the same one-expression row cap.
- W8's intent classification goes into `stickToEnd` (it already owns the detach) or into Rust on the batched stream.
- The probe becomes R.
- The runtime modality writer becomes part of `openFocus`'s install (closure rule: a decision includes computing its inputs), or a bridge-load internal with an inventory row.
- Class (c) shrinks to `index.html`'s inline scripts plus the preview-only `theme_seed` fallback, with honest reasons.

**Subsumes:** the "registry as a whole" gap for bridge-internal programs.

## 6. DETAIL, major. Under B and C, legacy files are an open sink; the freeze hashes literals, not sink sites

**Claim attacked.** Section 5.0: "every legacy block can sit behind a per-file `expect` and a frozen baseline row until it is touched". Root `:557` (freeze).

**Evidence.**
- (V) `check-js-volume.sh:176-178` counts a literal, or a joined `concat!` group, as JS only if it contains `[;{}]` and a code word.
  - Pieces joined by `[..].concat()` or `format!` are read one by one.
  - The ratchet's own note concedes "JS assembled by `format!` pieces (seen only through the sink count)".
- (V) The sink column is a per-file count (`:200-204`), and the `eval` regex (`:125`) counts a `use ... eval` line, not calls through the alias.
- (I) So in a legacy file under the planned per-file `#![expect(clippy::disallowed_methods)]`, a lane can:
  - (1) delete a frozen literal, which looks like a port and passes the freeze, then re-point the same sink to new JS built from pieces that do not read as JS, with an unchanged sink count; or
  - (2) add a second eval through the alias.
  - No gate goes red in either case. Under C these files stay indefinitely.
- (I) The freeze has no merge-base compare or grant path. A legitimate one-line bug fix in legacy JS is either blocked (C forces a 24-32 h carousel port) or self-served through `--update-baseline`, since a hash change is not a count raise.

**Change.**
- Use a per-site `#[expect]` on the statement or item instead of a per-file `#![expect]`.
- Freeze each legacy **sink site** by the hash of its whole argument expression in the code view, with literal contents included. A site may be deleted, never re-pointed.
- The freeze compares against the merge base. A change needs a `js-fix:` grant row, the owner's path for fixing in place.

**Subsumes:** B's and C's "edited in place without growing" hole.

## 7. DETAIL, minor. The next-frame verdict can admit on an R defect

**Claim attacked.** Root `:449` and residue `:197`: "Fail on lag, parity or integrity → the `fail` verdict admits `wheelBand` (integrity or parity first look for a bridge bug)".

**Evidence (V).** A parity break, or the band stacking on WKWebView's bounce, is a defect in the Rust port, not a deadline. The "first look" is advisory.

**Change.** `verdict.json` carries a `reason`. Only `lag` (over the owner's budget) or `order` (lost or reordered events) admits. `parity` and `integrity` mean the R arm gets fixed, or the legacy arm stays frozen without being promoted. The verdict must exist at the merge base.

## 8. DETAIL, major. `reconcile`'s witness, the proof port for ground 2, is likely green under honest probes

**Claim attacked.** Root `:302` (`reconcile`: `sameFrameWrite (witness)`) and 5.0 (proof port B exercises the command kind).

**Evidence.**
- (W) See finding 1: a late open paints no wrong frame.
- (V) The other benefits the plan claims are not page-only:
  - convergence through volatile attributes and generations (`:271`) works equally for Rust level-triggered calls;
  - show plus measure in one turn is the existing `transaction` verb (5.2).
- (V) What remains is latency: "an open saves a frame" on desktop (`:270`). That is the next-frame row, whose witness is the owner's verdict.

**Change.**
- Run `reconcile`'s witness first in 0.4.
- If it stays green, ship the command kind as a Rust-side level-triggered `transaction` with the generation. The kind, the volatile attributes, the generation and the one-show-writer rule are unchanged; only the executor moves.
- Keep the page reconciler only on a `fail` latency verdict or a red order witness (vetoed close, then reopen, under the desktop-cadence reorder).
- Either way, phase 0 proves the witness ground on its hardest case instead of assuming it.

---

## Answers to the brief's questions, in short

1. **Structural ground.**
   - A tsc scan can detect these effects reliably only as **types** over effect data (finding 2), not as identifier symbols.
   - Hiding is prevented by the pure-`decide` adapter, closed fully by a DOM-less compile.
   - A behaviour can pick up a ground incidentally: through a parameter-gated `prevent`, or a rider effect in an admitted variant (finding 1).
   - Detected = generated closes the loop between declaration and code, not between code and need.
2. **Late-arm witness.**
   - It is implementable in this Chromium-only setup (`addInitScript`, `test.fail()`, `select.mjs` maps `ts/**`).
   - It is deterministic if the probes are library-owned and key on event-loop order. It is flaky or tautological if the spec author writes the assertion.
   - A spec can be written to go red trivially today (finding 1).
   - The owner's verdict for the next-frame row is sound only if the verdict carries a reason and only `lag` or `order` admits (finding 7).
3. **Registry as a whole.**
   - Membership is closed by `adds:` plus admission.
   - Content is not closed: per-variant riders and new effects in an existing kind.
   - Outside the registry:
     - the boot probe, reducers and the runtime modality writer are not correctly accounted for (finding 5);
     - `PREPAINT_JS` in `index.html`, G's watchdog (inside `reconcile`'s open step, witnessed by the stuck-channel spec with `animation: none !important`) and the hub's one `addEventListener` (fate table) are correctly accounted for.
4. **Scope.**
   - A does not weaken the construct.
   - B and C leave the legacy-site hole of finding 6, which the ratchet does not see (it counts lines and literals, not sink identity).
   - B's witness proof rests on `reconcile`, which is the weakest instance (finding 8).
5. **Trajectory.** See below.

## Trajectory: what round 5 would find

- With finding 1 folded in, admission is a function of the placement rule. A new row or condition then brings its own counterfactual, so the layer cannot drift on its own.
- Round 5 would then find details:
  - probe coverage (AT announcements for `role=alert` insertions, IME composition for `otpFilter`, `touch-action` arbitration for `pan`);
  - the fidelity of the latency and reorder models before row 442;
  - the suite-budget cost of the witness tests;
  - the demotion flow when a witness turns green;
  - the reducer table's row cap.
- Without finding 1, round 5 finds this same class again, through `reconcile`'s green witness, `pan`'s outputs or the order condition.

## Attacks that failed

- **Hiding a structural effect from detection:** `decide` receives picked fields and returns data, and the adapter applies effects (V, `bridge.ts:672-705`). The only route is a global DOM reach, which finding 2's DOM-less compile closes. The construct is sound.
- **Witness flakiness as a design flaw:** with library probes at the event-loop commit points (end of dispatch, RO phase before paint, per the HTML "update the rendering" order), the result does not depend on frame timing or debug-build speed. Flakiness is a property of spec-authored assertions, which findings 1 and 3 remove.
- **Forging the next-frame verdict from a lane:** closed by the merge-base rule the plan already uses for grant rows, once it is applied to `verdict.json` (findings 3 and 7). This is not a design hole.
- **The bridge's own listeners escaping the registry:** the hub's single `addEventListener` and its ref counting are bridge internals with a fate-table row and the listener inventory. They are correctly accounted for.
- **G's watchdog escaping:** it lives in the reconciler's open step (root 2.3 item 3), so it is admitted with `reconcile`. Under finding 1 its drop-witness is the existing stuck-channel spec with an `animation: none !important` ancestor.
- **Option A weakening the construct:** the batches follow phase 0, so admission and the gates exist before any behaviour lands.
- **The root rule, the deadline table, the kinds, the one eval sink, A/G/R and sinks plus ratchet:** attacked from the admission side; none needed to change.
