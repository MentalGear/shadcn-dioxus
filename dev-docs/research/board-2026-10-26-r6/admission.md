# Board round 6, lens `admission`: admission in its round-5 form

**Verdict: HOLDS WITH CHANGES**
**DESIGN findings: 0, DETAIL findings: 13**

- The construct holds: per (site, effect), a counterfactual witness read by a library probe, or a cited platform rule. Every break below closes inside one of those two grounds. None of them adds, removes or replaces a ground or the unit.
- **The second ground is not yet closed as written (F1, F2, F3).** Three gaps: "no probe in the available engines can observe it" is a judgement; "available engines" is unbounded (only `chromium-1194` is installed); and "owner-approved" means in practice a grant commit by the main loop (root `:678`). Row 453 already uses a platform rule row to admit a preview script ("the allowlisted one"). Separately, three effects sit in a third state that is neither ground: clipboard, `dataTransfer`, and quarantined witnesses. The fix is a closed, cited category table that the gate checks from the effect name and its parameters. Activation-gated effects belong there by class, because a delay counterfactual cannot model provenance on any engine.
- **The round-5 probe fixes are half-sound (F4, F5).** Trajectory inclusion cannot see colour, glyph or pseudo-class state. The plan's one predicted red, `indeterminate`, is set on an `opacity: 0` `aria-hidden` bubble input and paints nothing. In the other direction, the criterion goes red by noise whenever anything else in a document-level host animates. `defaultAction(occurred)` and `outcomeOrder` still take author arguments.
- **The site ids are not implementable (F6, F7).** `#[track_caller]` ids move with every edit above the site, which a lane cannot re-witness itself. They also collapse behind wrappers (L6 goes through `use_global_escape_listener`, which has six callers). A `test.fixme` keyed to the variant activates for the wrong site, and the "scenarios from vectors" cannot set up a page.
- **Page-side escapes remain (F9).** A pure `decide` reaches the DOM through `(globalThis as any)`. `window.__dxBridge` exposes `register`, the live behaviour table and every verb. The class-(c) modality writer outlives boot, so the "admitted" runtime copy never runs on the preview.
- **Witness mode behind a cargo feature contradicts "no second SSG build" (F8).**

Labels: V = read in code or source, W = web/spec (URL given), I = inference.

---

## 1. DETAIL, major. The platform rule ground is decided by a judgement and a main-loop commit, not by a closed rule. It is already admitting a script

**Claim attacked.**
- Root `:327`, `:350`, js-interop `:114-123`: "only where no probe in the available engines can observe the commit point ... an owner-approved grant row at the merge base ... never a lane's own decision".
- Round-5 synthesis: "admission has a second, closed ground".

**Evidence.**
- **(V) "Owner-approved" has no mechanical form.** The grant rule this ground reuses accepts "a grant commit by the main loop on the batch branch" (root `:678`). The TSV row (`effect, site, citation, engine, check line, row`, `:350`) is ordinary repo text, and the main loop writes backlog rows and TSV lines routinely. So whether "no probe can observe it" is checked by nobody; it is decided by the agent that scopes the lane. That is round 3's "declared, not checked", moved from the lane to the main loop.
- **(V) "Available engines" is unbounded.**
  - `/opt/pw-browsers` holds only `chromium-1194` and `chromium_headless_shell-1194`.
  - Row 442 makes WebKit and Firefox "manual" (`442-*.md` item 2: `A/G specs: chromium (CI), webkit, firefox (manual)`).
  - Yet `:350` requires "Where Playwright WebKit can observe it (the clipboard case), the witness must run on that engine".
  - So whether an effect gets ground 1 or ground 2 depends on an engine set that the text never fixes, and that changes when row 442 lands. Nothing re-evaluates existing platform rows when it does.
- **(V) The check line is never executed.**
  - Row 442 item 3 defines `desktop-pass.md`'s checklist (the C5 table, a Tab-cancel smoke check, the gate's first-frame checks, the keyboard inset). Platform rows' check lines are not in it.
  - Nothing ties a row to a stamp, so the "manual check" is a sentence.
- **(V) The grant list is already the allowlisted category under another name.**
  - Row 453 `:14`: "V14 is a platform rule row: ... the preview's own clipboard writer is the allowlisted one".
  - That writer is not a vocabulary effect at a Rust behaviour site. It is an inline handler string: `preview/src/site/code.rs:191` and `preview/src/site/gallery.rs:187` (`"onclick": "navigator.clipboard.writeText(...)"`), counted as a sink by `check-js-volume.sh:26`.
  - So a platform row is being used to admit a script, which contradicts both `:46` ("There is no allowlisted-module category") and the row's own key `effect<TAB>site`.

**Concrete change** (the ground is unchanged; its applicability becomes a rule):
1. **A closed category table, edited only through an owner decision row.** Each category has one W citation, and the gate decides membership from the effect name and the site's parameters:
   - (a) `prevent` of a UA-reserved shortcut: the `hotkey`'s key and modifiers must appear in a cited per-engine shortcut table (F6, Ctrl/Cmd+B). The gate reads `Hotkey` parameters from the site.
   - (b) An activation-gated API effect, by effect name: `clipboardWrite`, and later `requestFullscreen`, `showPicker`, `share`. See F3 for why this is by class.
   - (c) An engine-specific default action from a cited table: Firefox requires drag data to start a drag of a non-link element.
2. **A platform row may only sit on a `site!` (F6) of a registry behaviour and a vocabulary effect.** A preview script goes to the preview-only reasoned allowlist that V5, V8 and V9 already use. That list is ratcheted, and it says plainly that it is not admission.
3. **Each row names `observable_on`**: the engine where a witness could see the commit point, or `none`. The gate turns the row stale (red) once that engine appears in a gate-run Playwright config at the merge base. "Available engines" then means exactly "the projects the gate configs launch".
4. **Each row's check line is a line of `desktop-pass.md`.** The gate reports how many rows have no passing stamp for their engine, and a cap on unverified rows is the owner's number.

**Subsumes:**
- who decides (the table, not the main loop);
- routing an awkward effect (it fits no category);
- the engine bound;
- growth of the grant list (bounded by categories, each a platform fact);
- V14's misuse.

## 2. DETAIL, major. Some effects have no ground or two grounds: a third, undeclared state exists

**Claim attacked.** "Admission ... per effect: the counterfactual witness red, or ... a cited platform rule row" (root `:327`), read as "exactly one of two".

**Evidence (V, all root lines).**
- **Clipboard is assigned both ways.** At `:350` it is "Where Playwright WebKit can observe it (the clipboard case), the witness must run on that engine". At `:359` and `:388` it is "a clipboard write ... is a platform rule row on WebKit's gesture rule".
- **`dataTransfer` has three outcomes.**
  - At `:335`: "run only on the engine where it matters ..., otherwise left to row 442".
  - At `:359`: "a platform rule row or a row-442 witness, expected green on Chromium".
  - So until row 442 exists, the effect is in "left to row 442", which is neither ground.
- **Quarantine and the demotion bar.** At `:350`: "A witness that turns green later ... opens a backlog row and is quarantined, and does not block the deploy; demotion requires green under fallback emulation, or a Firefox or WebKit run".
  - The emulation switches named there are A-only: `stripAnchorSupportsBlock`, `overflow-anchor: none`, and the A sheet removed. No emulation exists for the `dragSource` half or any non-A effect.
  - So a non-A effect that is green on Chromium can never be demoted. It is not admitted either, since it is not red. It ships page-side on neither ground.
  - Who decides that an effect "serves another engine", and so qualifies for this protection, is not stated. In practice that is the author: a third, declared ground.
- **Quarantine does not check its own trigger.** It applies to any witness that "turns green later", whether an engine changed or a lane made the effect redundant. A lane that makes an effect unnecessary therefore passes on quarantine instead of demoting it.

**Change.**
- One rule per (site, effect):
  - if the effect's class is in F1's category table, ground 2 applies;
  - otherwise ground 1 applies.
- A ground-1 witness that is green under Chromium and under every entry of a closed emulation list demotes the effect.
- Quarantine applies only when the Chromium revision recorded with the witness result (the deploy already resolves it through `sandbox-chromium.cjs`) differs from the one of its last red, and it carries an expiry of N batches.
- `dataTransfer` is category (c) until row 442's Firefox project exists, at which point F1 rule 3 turns that row into a witness.

**Subsumes:** the double assignment, the limbo state, and quarantine as a back door.

## 3. DETAIL, major. A delay counterfactual models R's time, not R's provenance, so no engine can witness an activation-gated effect

**Claim attacked.** Root `:350`: WebKit requires a clipboard write inside the user gesture, so "where Playwright WebKit can observe it ... the witness must run on that engine before the effect may be demoted".

**Evidence.**
- (W) Transient activation is a time window, not a dispatch scope: <https://html.spec.whatwg.org/multipage/interaction.html#transient-activation-duration> ("implementation-defined ... at most a few seconds").
  - A 3-frame delay inside the page keeps the activation in Chromium (the plan says so itself: "a call 3 frames late still works there").
  - (I) WebKit forwards the user-gesture token through timers for about a second, so a rAF- or timeout-delayed call in Playwright WebKit very likely keeps it too.
- (I, the plan's own words at `:350`) The real failure is provenance: on WKWebView, R's call "arrives by `evaluate_script`, outside the gesture".
  - No in-page delay reproduces a call made from a native task with no activation, on any engine. So the WebKit witness for clipboard is expected green, and would demote an effect that breaks on Safari and WKWebView.
- (V) Round 3 already had this shape as a "structural effect: ... an activation-gated API" ground. It was dropped in round 4, and round 5 brought it back as a judgement.

**Change.**
- Activation-gated effects are ground 2 by class: F1 category (b), decided from the effect name, with no "can a probe see it" judgement.
- Delete the "(the clipboard case)" sentence at `:350` and the row 442 item 6 clause that schedules a WebKit clipboard witness.

**Subsumes:** "is every effect assigned to exactly one ground by a rule" for the provenance class.

## 4. DETAIL, major. Trajectory inclusion is blind to most paint and goes red by noise; the predicted `indeterminate` red is false

**Claim attacked.**
- Root `:345`: "records a library projection of the host subtree ...: rects, scroll offsets, computed `opacity` and `visibility`, and `document.activeElement`. A witness is red iff the counterfactual arm paints a projected state that the real arm never paints".
- Root `:318`, `:353`: "a delayed `indeterminate` paints the wrong glyph (off-trajectory, red)".

**Evidence.**
- **(V) The `indeterminate` claim is false.**
  - S8 sets `input.indeterminate` (`primitives/src/checkbox.rs:298-316`) on the BubbleInput. That input is `opacity: "0"`, `pointer_events: "none"`, `aria_hidden: "true"`, `tabindex: "-1"`, and translated off its box (`checkbox.rs:325-347`). The visible glyph comes from `data-state` through the vdom.
  - (W) The attribute does not affect the form either. <https://html.spec.whatwg.org/multipage/input.html#dom-input-indeterminate>: "It has no effect except for changing the appearance of checkbox controls."
  - So a late `indeterminate` paints nothing and submits nothing, and its witness is green. With all reconciled kinds green (popover open, `indeterminate`, `defaultChecked`), only the order witness can keep the page reconciler.
- **(I) The projection is blind.**
  - Any paint change outside rects, scroll, opacity, visibility and focus is invisible: colour, background, glyph or icon swaps, `:checked`/`:indeterminate`/`:popover-open`-driven styles, text content, `clip-path`, and AX state.
  - An `attrs` effect such as `data-state` or `data-highlighted`, or a `setVar` that drives a colour, is therefore green by construction and demoted without being examined.
- **(I) The projection is over-sensitive.** Red means "a subtree state never painted by the real arm", so the state compared is a product.
  - With any independently animated element in the host subtree, the pairs (phase_j, old effect state) at later frames never occur in the real arm, so every time-shifted copy is off-trajectory and red.
  - Hosts of `hotkey`, `scrollLock` and `dropAnywhere` are `document` or `window`: the projection is the whole preview page, with every demo's infinite animation in it.
  - Two separately run arms also sample interpolated values at different frame times. Unless frame time is fixed, "never painted" is true of almost every animated frame.
  - That makes "admission by noise" the cheap direction for an author: put anything animated on the host.
- **(V) The control arm cannot catch either problem.** The fold's control is "witness mode off must not throw" (`:348`). For a criterion that compares two arms, that is trivially green unless it means two independent real runs.

**Change.**
1. **Per-element trajectories.** Compare each element only against its own real-arm trajectory.
2. **A wider projection.** Add the matched state pseudo-classes (`:checked`, `:indeterminate`, `:popover-open`, `:modal`, `:open`, `:placeholder-shown`), the `data-*`/`aria-*` attributes, and the computed values of every property the host component's sheet declares. The last can be derived from `style.css`, which the gates already parse.
   - Alternative (I): pixels of the element's clip, taken under `HeadlessExperimental.beginFrame` on the headless shell, which also fixes frame time.
3. **Pause unrelated animation.** In the harness, set the existing `[data-dx-motion="paused"]` rule (theme sheet; CLAUDE.md motion-gating) on everything outside the host.
4. **Make the control real-vs-real.** Two independent real runs must be green under the criterion; this is the determinism check.
5. **Correct `:318`, `:353` and the registry note**: `indeterminate` is decided like `defaultChecked`.

**Subsumes:** false greens on non-geometric paint, admission by concurrent animation, the vacuous two-arm control, and phase 0's wrong expectation for `reconcile`.

## 5. DETAIL, major. The cancel and order probes still take author arguments

**Claim attacked.** Root `:344`: "never from ... a predicate the author wrote"; the table at `:333-340`.

**Evidence (V).**
- The fold made only write effects library-defined (trajectory inclusion).
- `prevent` still reads `defaultAction(occurred)` (`:333`): which default, observed how, is the `occurred` argument, and nobody is named to write it.
- Each `outcomeOrder` row states its own expectation:
  - "a later listener on the path ran" (`:334`);
  - "the drop target received the data" (`:335`);
  - "a capture applies at the next pointer event" (`:338`);
  - the order rows (`:340`).
- That is round 5 finding 1's class, persisting in the cancel half.

**Change.**
- **`defaultAction`** becomes an arm comparison over default-action observables defined by the library per event type. Red iff the counterfactual arm's observables leave the real arm's trajectory. The closed table:

  | event | default-action observables |
  |---|---|
  | `keydown` | the native layers' open state, `activeElement`, the scroll chain's offsets, input value and selection, activation |
  | `wheel` / `touchmove` | the scroll chain's offsets |
  | `beforeinput` | value and selection |
  | `mousedown` | `getSelection()`, `activeElement` |
  | `drop` | `location`, insertion |
  | `dragover` | drop acceptance |
  | `click` | activation, navigation, toggles |

- **`outcomeOrder`** means the counterfactual's outcome differs from the real arm's recorded outcome. The harness installs the sentinels itself (a later listener on the path, a drop target, a pointer-event recorder on the host). There is never an expected order.
- Every probe is then one criterion: arm divergence over library-defined observables.
- O1 and V22, where the observables are blind, are F1 category (a).

## 6. DETAIL, major. `#[track_caller]` site ids are not stable, collapse behind wrappers, and are enumerated only where exercised

**Claim attacked.** Root `:328`: "`use_behaviour` is `#[track_caller]` and sends a **site id** ... the gate requires a witness per site id and effect, and stale entries fail, so a new site cannot borrow an old site's witness".

**Evidence.**
- **(V, std) Ids move with every edit.**
  - `Location::caller()` is file, line and column, so any edit above a site re-keys it.
  - Combined with "stale entries fail", "the witness ... must exist at the merge base" and "a lane cannot admit its own behaviour" (`:354`), a lane that only edits `dialog.rs` above line 442 cannot merge in one step.
- **(V) Ids collapse behind wrappers.**
  - L6's construction goes through `use_global_escape_listener` (`primitives/src/lib.rs:333`), then `use_global_keydown_listener` (`lib.rs:359-374`).
  - The wrapper is called from six sites: `dialog.rs:442, 499`, `alert_dialog.rs:244` and `popover.rs:517, 831`.
  - `#[track_caller]` on `use_behaviour` sees the wrapper's line unless every wrapper is also `#[track_caller]`. So a new caller of the wrapper borrows L6's witness, which is exactly the route round 5 finding 3 meant to close.
- **(V) Enumeration is runtime-only.**
  - The coverage hook is debug-only (`__dxBehaviourCoverage`).
  - The deploy runs the release suite, and `test:changed` runs a subset.
  - So no single run enumerates every site, and a site no spec mounts needs no witness.
- **(V) Pre-authored witnesses cannot name the site.** Item 0.6's pre-authored `test.fixme` (`:357`) cannot name a `file:line` that the lane has not written yet.
- **(I) Command behaviours are ambiguous.** For `openFocus` and the reconciled kinds the "construction site" is undefined:
  - the executor is one boot install;
  - the 13 hook spreads in 11 files (`:319`) are the real sites;
  - read the first way, all consumers share one witness.

**Change.**
- **Explicit stable names.** Write `use_behaviour(site!("dialog.escape"), Hotkey::<Prevent>::new(..))`.
  - A `Site` can be constructed only by the macro.
  - Wrappers take a `Site` parameter, so a new caller must name a new site.
  - A static gate scans `site!` literals (unique; each at exactly one location, cross-checked against `track_caller` in debug) and keys witnesses by name.
- For command behaviours, the site is the command attribute's spread site.

**Subsumes:** wrapper borrowing, edit churn, unexercised sites, and pre-authoring.

## 7. DETAIL, major. `test.fixme` keyed to the variant is the wrong key; "scenarios from vectors" cannot set up a page

**Claim attacked.**
- Root `:354`: "a witness for a not-yet-built variant lands first as a `test.fixme` keyed to the variant's existence in `behaviours.gen.ts`, hash-checked at the merge base, which the lane then flips to live".
- Root `:346`: "the harness replays the event as a trusted CDP input on the construction site's host".

**Evidence.**
- **(I) The variant key activates witnesses for the wrong site.**
  - `hotkey` serves L6, O1 and V22. Item 0.3 adds the variant for L6, so a variant-keyed witness for V22 goes live while V22 is still the legacy script. Its counterfactual then moves nothing, and the run fails on an unrelated lane.
  - Flipping by edit changes the hash that is checked at the merge base.
- **(V) Vectors do not describe a page.**
  - Vectors are `{config, snapshot, event, expected_effects}` triples for `decide` (`:391`), and a snapshot is `decide`'s input, not page state.
  - Replaying "on the host" needs a route, a host locator and setup steps. L6's needs "a div layer over a native dialog or popover" (`:359`). No vector encodes these.
  - Vectors are written by the behaviour's own lane (js-interop `:252`), so as stated the lane chooses the scenarios.

**Change.**
- **No edit to flip.** Use a conditional `test.fixme(!sites.includes("dialog.escape"))`, read from the generated site list (F6).
- **Scenario files.** One binding file per site: route, host locator, setup steps, then the vector's event.
  - It is authored with the witness by the non-owning author.
  - The gate checks that every vector whose `expected_effects` contains E has a binding.

## 8. DETAIL, major. Witness mode behind a cargo feature is a second release build, which point 7 says is not needed

**Claim attacked.** Root `:353` ("a second SSG build per deploy (a fourth 3.6 GB tree) is not needed") against `:354` ("a separate bundle behind a non-default cargo feature ... the deploy suite builds the witness variant").

**Evidence.**
- (V) Behaviour JS is embedded in the wasm and evaluated from it (`bridge/behaviours.rs` `module()`: `include_str!("../js/behaviours/fixture.js")`).
- (I) A feature-gated witness bundle is therefore a separate wasm and a separate `dx` SSG build per deploy, inside the build lock.
- (V) The plan already ships a release test knob: `__dxBridgeDelayFrames` is read in release (`ts/bridge.ts:331-335`).

**Change (I).**
- Keep the release bundle witnessable without a rebuild:
  - the adapter applies effects through one frozen apply table and sets a read-only marker `{site, effect}` while it applies them;
  - a Playwright `addInitScript` patches the prototypes (`Event.prototype.preventDefault`/`stopPropagation`, `setPointerCapture`, `focus`, `scrollTo`, `setAttribute`, `showPopover`/`hidePopover`) before page scripts run;
  - the patch drops, delays or rehomes a call only when the marker names the site and effect under test.
- The order counterfactual's hold becomes a mode of the existing delay knob.
- The release bundle then holds no switch and no extra bytes beyond the marker.

## 9. DETAIL, major. Page-side code still escapes admission

**Claim attacked.**
- Root `:352`: "Admission covers every effect the shipped TypeScript applies outside a verb row".
- Round 5's attack that failed: "Hiding an effect from the inventory ... With the DOM-less compile there is no route".

**Evidence.**
- **(V) `decide` purity is a token denylist.**
  - `IMPURE` in `check-bridge-behaviours.sh:99-100` lacks `globalThis`, `self`, `Reflect`, bare `Function(` (`SINKS` catches only `new Function`), `constructor`, `getComputedStyle`, `scrollBy` and `__dxBridge`.
  - (I) The planned DOM-less compile accepts all of these:
    - `(globalThis as any).scrollBy(0, 9)`;
    - `"".constructor.constructor("...")()`, which is typed `Function`;
    - `(globalThis as any).__dxBridge.transaction([...])`, which runs any verb row.
  - Each is an effect outside the declared set that witness mode never moves, so it is never witnessed.
- **(V) The global bridge object is open.**
  - `window.__dxBridge` exports `register` and the live `behaviours` table (`ts/bridge.ts:1040-1053`).
  - `register` overwrites without checking (`behaviours[kind] = impl`), so any bundle or page script can swap a kind's `decide`.
  - `call`, `set` and `transaction` are public, so a class-(c) or legacy script can show a reconciled kind outside the executor: a second show writer.
- **(V) Class (c) outlives boot.**
  - `PREPAINT_JS` (`preview/src/theme.rs:143`) installs the page-lifetime modality writer behind the `__dxModality` guard.
  - `MODALITY_JS` (`primitives/src/modality.rs:70`), the runtime copy that point 6 admits, is byte-identical and returns early on every preview page.
  - So the internal's witness can never go red: the class-(c) copy keeps writing.
  - The preview, where every witness runs, measures a configuration that user apps (which have no `PREPAINT_JS`) do not have.

**Change.**
1. **Purity by construction, not by denylist.** A free-identifier allowlist by tsc scope analysis (`Math`, `Number`, `String`, `Array`, `Object.keys`/`freeze`, `RegExp`); `any` and `as any` banned in `ts/behaviours/**`; the members `constructor`, `__proto__` and `prototype` banned; computed access only on typed records.
2. **Close the global.** After boot, freeze the table and drop `register` from the global; the adapter registers each module's exported `decide`. `callPublic` is restricted to the legacy allowlist and goes with it.
3. **Class (c) is exempt only before boot.** At boot the bridge adopts class-(c) listeners that outlive it: `PREPAINT_JS` records them in `__dxPreboot`, and boot replaces them with the admitted internal. Otherwise the witness harness serves a pre-paint snippet without the modality block.

## 10. DETAIL, minor. The internals' effects are outside the vocabulary table

**Evidence (V).**
- **The transition port.** Its effect is a synthetic event dispatched into the vdom (`:291`), and it has no vocabulary entry.
- **The watchdog** is given "dropped" (`:352`), while the table gives `attrs` "delayed" (`:336`).
- **The modality writer** writes `attrs` on `<html>`, but the adapter's `attrs` writes only the host (`ts/bridge.ts:704-708`).
- **Toggle suppression and the epoch minters** are `send` and `remember`, which are exempt (`:343`). Their "witness pairs" have nothing to move.

**Change.**
- Add `dispatchPort` (counterfactual: carried by the pump; probe: the order row).
- Allow `attrs` on `documentElement` for internals.
- Make "dropped" the table's counterfactual for fail-open effects.
- Remove suppression and the minters from the admitted list as exempt.

## 11. DETAIL, minor. `release` is green by construction and has no landing

**Evidence.**
- (W) Pointer capture is released implicitly after `pointerup` and `pointercancel`: <https://w3c.github.io/pointerevents/#implicit-release-of-pointer-capture>.
- So a delayed `release` on up is green by construction.
- (V) Point 5 lands only `focus` and `capture` (`:351`), and `FORBIDDEN_VERBS` bans `releasePointerCapture`.

**Change.**
- `release`'s counterfactual is "dropped", and its green landing is deletion.
- The same applies to any effect that the platform performs implicitly.

## 12. DETAIL, minor. The order hold has no timeout, its release order is not the scheduler's, and the page lacks the heights

**Evidence.**
- **(V, `:353`) No timeout.** "Held until the next trusted event ... has finished dispatch". If no event follows, the fact is lost, and the arms then differ by loss, not order. Any outcome probe goes red, which admits the page reconciler by artefact.
- **(V) Not the scheduler's order.**
  - The scheduler orders by height ascending, then id (`dioxus-core-0.7.9/src/scheduler.rs:116-118`).
  - "Reverse scope-height order" is not that order, and the page has no heights.
  - `ScopeState::height()` is public (`scopes.rs:117`), so Rust can send it.
- **(I) The pump removes the case.** After the one inbound pump, cross-session reorder models nothing.

**Change.**
- Release at the earlier of two points:
  - the next delegated event's dispatch is done (a task posted from a window capture listener; a microtask would run before Dioxus's handler);
  - k completed paints.
- Release order is the scheduler's (height, id), taken from the session setup.
- Drop cross-session reorder once the pump lands.

## 13. DETAIL, minor. Estimates and parameters

**Evidence and change.**
- **(I) The cost is undercounted.** Round 5 moved the unit to (site, effect), but the cost stayed at "about 45-50 witnesses, 90-100 tests" (`:355`). A count from the registry table gives about 70-110 pairs, so about 150-220 tests and about 10-16 worker-minutes, still inside the 4,787 s headroom:
  - `hotkey` 1 plus 2 rows;
  - `scrollLock` 2;
  - `otpFilter` 2;
  - `dropAnywhere` 2 plus `dragSource` 3;
  - `pan` 7;
  - `settle` about 7;
  - `openFocus` about 13 by spread site;
  - the reconciled kinds about 12;
  - `scrollOwner` 8-12;
  - `stickToEnd`, `labelClick` and `roving` 4;
  - internals 6-8;
  - order instances about 12.
- **(V) The delay's statistic is unnamed.** At `:336`: "provisionally 3 frames, set by the latency session's desktop distribution".
  - A witness green at the median and red at the tail demotes an effect that fails on slow devices.
  - Change: name the p99 under the busy case.
- **(V) The demotion order is unchosen.** At `:351` "the gate records the order", but nothing chooses it.
  - The plan's example pair is not two witnessable effects: L2's `oncancel` is rsx, which witness mode cannot move.
  - Change: demote in layer order (the effect whose next-best placement ranks higher goes first), with ties broken by site name.

---

## Attacks that failed

- **Does the second ground replace the first?** No. Ground 2 covers commit points outside the page or outside every engine a gate config launches. With F1's table and F3's class rule it is mechanical, and nothing that a probe can see may use it.
- **Is round 4's author-controlled witness back?** For write effects, not once F4 is folded. For cancels, only through `occurred` (F5). The library owns timing, observables and verdict.
- **Expected-failure soundness and the forged violation:** unchanged and sound (round 5's `workerProcessEntry.js:2555` reading). The constructor ban on `WitnessViolation` is implementable with the tsc API that item 0.1 builds.
- **Completed-paint delay:** sound. `afterFrames` nests rAFs (`ts/bridge.ts:337-340`), and a post-paint hop after each rAF counts real paints in headless Chromium (I). Frame-counted input dispatch is implementable with a page-side frame counter.
- **The page-side reorder seam:** sound in principle.
  - On web the Dioxus handler runs inside dispatch (`dioxus-web-0.7.9/src/dom.rs:88-116`, V).
  - The event's render flushes in the microtasks after dispatch, before a task-released fact (I), which reproduces desktop's "event, render, fact".
  - Only its parameters fail (F12).
- **Typestate for the effect set:** sound. The as-built `Behaviour::Fixture` with `cancel: bool` (`bridge/behaviours.rs:46-66`) shows why `const` could not work. A typed constructor with the effect set sent with the install, enforced per install by the adapter, is implementable. Only the site key fails (F6).
- **Hidden effects through `decide`'s return value:** none. The adapter applies only `prevent`, `stop`, `attrs` and `send` (`ts/bridge.ts:691-711`), and `ATTR_OK` limits `attrs` to `data-*`/`aria-*`. The leak is around the return value (F9), not through it.
- **Rsx inline `onclick` strings escaping the ratchet:** no. `check-js-volume.sh:26` counts an rsx `"onclick":` key as a sink, with a red self-test at `:513-514`. They escape admission only when relabelled as platform rows (F1).
- **The core outside admission:** the root rule, the deadline table, the two kinds, the closed bridge with one eval sink, A/G/R, and sinks plus ratchet were untouched by every attack here.
