# Board round 3, lens `amended-rule`: the deadline table, the kinds, `reconcile`, admission

**Verdict: HOLDS WITH CHANGES**

**DESIGN findings: 1, DETAIL findings: 6**

Summary:
- The root, the deadline index, the listener/command split, the closed bridge and sink enforcement all held. The design flaw is in **admission**. The adopted rule ("a behaviour must contain a commit-sensitive effect, read from its declared effect set") contradicts the table's own next-frame row. That row's fallback is "a named behaviour where the measured lag exceeds the budget", and a next-frame decision has no commit-sensitive effect by definition. So C5's fallback was moved out to an "allowlisted TypeScript module", a page-side category outside the closed registry. Round 2's core-construct member had proposed a second admission ground (a measured over-budget deadline). The fold dropped it.
- `sameFrameWrite` is declared, not checked. Every program that writes the DOM can declare it, and the doc already calls the admission of `settle` and `scrollStep` provisional. So "closed is a construction, not owner governance" is false as written. The fix is a late-arm witness: run the behaviour's own spec with its effects delayed by the budget, and admit it only if the spec goes red.
- `reconcile` saves a desktop frame only for elements whose observer is already attached. 8 of the 11 popover consumers create the panel in the same render that opens it, so for most popups there is no saving.
- "Level-triggered, so it converges" needs a single writer. Popover and dialog state has two writers (the browser closes too), and a controlled veto leaves the two sides apart. The fix is a generation, the same construction G already uses.
- 9 blocks that step 2 kept in JS have no step-3 placement. L20 (focus with `preventScroll`) is a real gap once 3.0b removes the `focus` verb.
- Round trajectory: the B layer's classification key moved each round (form, then index, now admission grounds). This round's change completes round 2's amendment; it does not open a new direction. Round 4 should find only details if it is folded in.

Evidence labels: V = read in the repo at `/home/user/wt/b28` (`queue/batch-28` @ `02a2b880`) or the Dioxus 0.7.9 sources; W = web source (none fetched this round: the proxy refused html.spec.whatwg.org and w3.org); I = inference, including spec text recalled but not re-read here.

---

## 1. DESIGN, major: admission by declared effect type contradicts the next-frame row and does not close the registry

**Claim attacked.** Root doc 2.2 "Admission" (`sync-js-root-2026-10-26.md:261`): "a variant with none of them cannot be added, so 'closed' is a construction, not owner governance". Also the next-frame row (`:247`): "a named behaviour only where the measured lag exceeds that budget".

**Evidence.**
- (V) The next-frame row's fallback cannot pass admission. A next-frame decision has, by its deadline, no effect at default action, task or paint. The residue doc says so in the same paragraph that places C5 in that row. `sync-js-residue-2026-10-26.md:19` reads "It sits in the deadline table's next-frame row ... a named behaviour only if the measured lag exceeds it". Then `:20` reads "If Rust loses, C5 becomes an allowlisted TypeScript module (... not a registry behaviour, because a passive stream heuristic has no commit-sensitive effect to pass admission)". The same move appears at root `:83`, `:362` and `:406`. So the plan has a fourth page-side category outside the registry, which the executive summary says does not exist (`:53`: "every other JavaScript source is closed at the sink").
- (V) Round 2 proposed the missing ground and the fold dropped it. `board-2026-10-26-r2/core-construct.md:86` reads "The gate requires every variant's effects to intersect {...}, **or the variant to carry a measured over-budget deadline row**". The adopted text (`:261`) and the R2 synthesis (`SYNTHESIS.md:30`) keep only the first half.
- (V) Admission already depends on the budget in practice, against its own claim. `:261` admits `settle` and `scrollStep` "on `sameFrameWrite` and `focus`, and re-checked against the owner's budget ... (I: neither cancels, captures or answers a commit directly)". `:592` adds "the admission of `settle` and `scrollStep` is provisional". The reserved `place` behaviour (`:281`) is admitted only "if 3.PL's first-frame and `g-latency` specs show" that late is not acceptable: that is a witness, used ad hoc.
- (I) `sameFrameWrite` is a declaration, not a property. In the adapter's closed effect vocabulary (`scrollBy`, `setVar`, `setValue`, `after(frame)`, root `:287`), every write applies inside the callback that produced it, so every DOM-writing program can honestly declare a "same-frame write". The only programs admission excludes are pure reporters, which were never behaviours (they are `listen`). `scrollStep` shows the leak: Rust sends `scrollBy` with snap suspended "in the frame of the `scrollBy`". That is two operations in one JavaScript turn, which `transaction(ops)` already does (`js-interop.md:97`). It needs no registry member.

**Concrete change (amends the admission rule).** A variant is admitted on one of two grounds, both checked by the gate.
1. **A structural commit effect.** `prevent`, `stop` (capture-at-el only), `dataTransfer`, or an activation-gated API. These are wrong when late by construction, because dispatch has ended.
2. **A late-arm witness.** This is the only ground for `sameFrameWrite`, `focus`, `capture` and any next-frame fallback.
   - The adapter gets a test mode that delays every effect of a behaviour by `after(frame * N)`, with N the owner's budget (provisionally 1 frame until row 442/443 set it).
   - The behaviour's spec tag must go red in that mode.
   - A measurement stamp (`research/measure/<port>.pass` showing R over budget) is the same witness for the next-frame row.
   - If the spec stays green under delay, the decision is R. Drop the "allowlisted TypeScript module" category: C5's fallback becomes a registry member admitted on its measurement stamp.

**Subsumes.**
- The C5 out-of-registry category.
- The provisional admission of `settle`/`scrollStep`: `scrollStep` becomes a `transaction` plus `listen(scrollend)`, unless its witness is red.
- `place`'s ad-hoc reservation.
- The O5 versus L30 split (finding 5).
- Lane 3.OV's "site by site" roving-focus judgment (`:390`).
- It makes "closed is a construction" true.

**Does not subsume.** The size of the budget, which is the owner's number.

**Why DESIGN.** The brief names the admission rule as core. This change alters what admission means: it adds a second ground and removes a category. It does not replace the registry, the kinds or the table.

## 2. DETAIL, major: `reconcile` saves no desktop frame for mount-on-open panels, which are most popups

**Claim attacked.** `:257`: "On desktop a MutationObserver callback ... runs before paint in the frame the edit lands, so an overlay open saves a frame". Round 2 blank-slate counted this as subsuming "one desktop frame on every overlay open (16 driver call sites: ... `use_popover_shown_while_mounted` ...)".

**Evidence.**
- (V) The timing half holds. Edits are applied inside a rAF callback (`dioxus-interpreter-js-0.7.9/src/ts/native.ts:444-458`), and an MO record queued by them is delivered at the microtask checkpoint after that callback, before style, layout and paint.
- (V) The attach half does not. The plan installs "one attribute-filtered MutationObserver per element" (`:255`) through `use_behaviour`, an effect.
  - On desktop, effects run only after the render's acknowledgement (`dioxus-core-0.7.9/src/virtual_dom.rs:511-530`; root 1.4).
  - `use_popover_shown_while_mounted` is used by tooltip, hover card, select list, combobox list, popover, navbar, navigation menu and toast: 8 of 11 consumers. Only `dropdown_menu`, `menubar` and `context_menu` call `use_popover_sync` (grep).
  - Those 8 render the panel only while open (`tooltip.rs:371,388,452`: `use_animated_open`, then `if render()`). So the element and `data-dx-open="true"` arrive in one render, before any observer exists.
  - The observer attaches in a later webview task and reconciles "at attach". The panel is shown in frame N+1, exactly as today's effect-driven `showPopover`.
- (V) Per-element installs also scale with the element count. S8/S9 put `reconcile` on every Checkbox, Switch and RadioGroup input, so a 1,000-row table means 1,000 install messages and 1,000 observers.

**Change.**
- Make the reconciler one page-side member installed with the bridge at boot. It is a document-scoped `MutationObserver` with `subtree`, an `attributeFilter` over the command attributes, and `childList`, so that an inserted node carrying a command converges in the same rAF microtask. Alternatively, render command-bearing panels persistently.
- Add a cost spec on a churning virtual list, since `childList` records every insertion.
- Correct the claim: the saving applies only to attached observers until this lands.

**Subsumes.** The latency claim for all 16 driver sites and the per-element install cost of S8/S9.

## 3. DETAIL, major: "level-triggered, so it converges" fails for two-writer state; commands need a generation

**Claim attacked.** `:255` ("a command that is lost, early or reordered converges") and `js-interop.md:53-54`.

**Evidence.**
- (V) The browser writes popover and dialog state too: light dismiss and Escape (`popover_sync.rs:236-250`, row 115). The plan keeps a controlled-open veto in Rust (`:384`, L6: "the controlled `open` needs the veto").
- (I) The failure chain:
  - The browser closes the panel, and the reconciler reports it.
  - A controlled parent ignores `on_open_change(false)`. Rust's attribute stays `"true"`, and Dioxus writes only changed attributes, so no MO record arrives.
  - The page stays closed while Rust says open. Nothing converges.
  - Any later re-attach reopens a panel the user dismissed, with the enter animation and a focus move. Re-attaches happen on an epoch bump, a teardown and reinstall, or under finding 2's boot reconciler on a remount.
- (I) The coalesced `toggle` case is handled by comparing `newState` with the desired state. The veto case is not, because there is no event and no record.

**Change.**
- The attribute carries `(state, gen)`, with `gen` minted by Rust.
- The reconciler records "the browser diverged at gen g" and re-applies only for a newer gen.
- Rust adopts the browser's state, or vetoes explicitly by bumping gen: a deliberate reopen, a visible decision.
- This is G's page-owned generation (2.3 item 4) used for commands. Use one generation mechanism for both.

**Subsumes.** The veto divergence, reopen on re-attach, and G's separate generation wiring.

## 4. DETAIL, major: nine audit blocks have no step-3 placement, and one is a hole the plan creates

**Claim attacked.** That each of the 103 blocks gets a placement. Section 3 covers the 22 (d) blocks, 3.1 the routed (b) rows, and 5.4 groups the other 43 (b) rows.

**Evidence.**
- (V) No mention anywhere in the root doc for blocks that step 2 **kept in JS**, according to their audit rows:
  - T1 and T2 (CSS moved, insertion kept; T2 "must see the ink rule in the same tick" as `showPopover`);
  - L17's Select arm ("Select stays on the id script");
  - L18 (`activeElement.closest`, `date_picker/context.rs:49`);
  - L20 (`carousel/root.rs:472`, `context.rs:183`: `focus({preventScroll: true})`);
  - L22, S4, V6;
  - W2, whose status waits on lane 2D.
  - Most of these are plain R (`get` plus Rust).
- (V) L20 is a gap the plan itself creates:
  - 3.0b removes the `focus` verb (`js-interop.md:89`; root `:456`, `FORBIDDEN_VERBS`).
  - The roving-focus row sends "a focus after human-rate input" to `set_focus` (`:390`). But `set_focus` has no `preventScroll`, and step 2 measured that it scrolls the page by 157 px (audit L20, which breaks `carousel-paging-inert`).
  - No registry member covers a paging focus hand-off.
- (I) T2's same-tick ink belongs inside the reconcile step, or in a static stylesheet.

**Change.**
- Add a "kept in step 2" table that gives each of these blocks its row and lane.
- Restore `focus{preventScroll}` as a verb, legal only from effects. Enforce the next-input deadline with finding 1's witness, not by banning the API.

**Subsumes.** The unplaced blocks, and the conflation of "the focus API" with "focus for the next input".

## 5. DETAIL, minor: the "this event's task" row depends on the implementation, and two of the five rows need numbers

**Claim attacked.** That the deadline of each decision can be determined objectively.

**Evidence.**
- (V) O5 takes pointer capture (`toast.rs` swipe block, `setPointerCapture`), while L30 follows the same gesture with window listeners (`drawer.rs:641-646`).
- (I) For touch, Pointer Events implicitly captures to the `pointerdown` target. A mouse capture set later still works while the button is held. So "capture" has a this-task deadline only because O5 was written that way, and O5 versus L30 is still placed two ways (`pan` versus R, "re-checked later", `:350`).
- (V) Focus routing for the next input needs a judgment on input rate (`:390`), and the next-frame row needs the budget (`:592`).
- Default action and before paint are objective. This task and next frame are not.
- (I) The budget is **not circular**: build the R arm, measure, fall back. But until it exists, L30, O5, C3 and C4 are placed by argument, and only the carousel and W8 have a frozen legacy arm to switch back to (3.FZ).

**Change.** Subsumed by finding 1's witness. Until then, apply the provisional N = 1 frame to O5 and L30 together so the class lands in one place.

## 6. DETAIL, minor: decisions with two deadlines need a closure rule stated in the table

**Claim attacked.** The question whether the table splits two-deadline gestures cleanly.

**Evidence.**
- (V) C4 cancels the committed `pointermove` (`carousel/drag.rs:388`). "Committed" is computed from the move stream (the slop threshold), so the recogniser must be page-side: the root invariant requires the inputs to be present at the commit.
- (V) C4's per-move `scrollBy` output, coasting and rubber band are next-frame decisions. They sit in `pan` by convenience ("one coupled state machine split across `pan`, R and A" by judgment, `:353`).
- (V) O5's `css-var` output is next-frame too (the plan concedes it "shrinks to capture plus a stream" if R passes).

**Change.** Add one line to the table: a decision includes computing every input it reads, which takes its deadline; its outputs are separate decisions placed by their own deadline. Witness them per finding 1.

## 7. DETAIL, minor: several command kinds on one element need one reconciler with an order

**Evidence.**
- (V) A `<dialog popover>` is the non-modal Popover used by DatePicker and ColorPicker (`popover_sync.rs:300-305`). The Popover's modal arm uses `showModal` on a dialog.
- (I) `showModal()` throws `InvalidStateError` on an element whose popover is showing (HTML spec, recalled, not re-read).
- `reconcile{kind}` installs per kind, so two observers fire in creation order, not in a semantic order. `openFocus` already has to run "in the same step".

**Change.** One reconciler per element owns all of its kinds, in a fixed order: closes before opens, `hidePopover` before `showModal`, focus last.

---

## Attacks that failed

- **MO timing on desktop.** The callback runs at the microtask checkpoint after the rAF callback that applies edits (`native.ts:444-458`), before style, layout and paint. For an attached observer, the timing claim is right (finding 2 is about attaching).
- **Ordering against Dioxus's own attribute writes.** MO delivers one batch after the whole edit list, so the reconciler sees the complete render. Dioxus diffs attribute values and writes only changes, so there are no spurious records.
- **Echo loops.** The reconciler never writes the command attribute. Rust's follow-up flip yields desired == actual, a no-op.
- **Coalesced `toggle`.** Comparing `newState` with the desired state handles it.
- **Hydration.** A `[popover]` element and a `<dialog>` without `open` are hidden until shown, so a server-rendered `data-dx-open="true"` paints nothing wrong before the bridge loads, and the reconcile at attach converges. The plan never binds the native `open`.
- **Install ordered after edits on desktop.** Effects wait for the render acknowledgement, so an effect-issued install finds its element.
- **First-match order of the table over the (d) blocks.** No (d) block lands in two rows for the same decision. Multi-row blocks (T8, W8) are split per decision, as stated.
- **The root rule, the one eval sink, and A/G/R as layers.** Re-attacked from this lens and unaffected.

## Question 4: refinement or instability?

| round | what moved | where it sat |
|---|---|---|
| round 1 | the form of B (interpreter to named behaviours) | B |
| round 2 | B's index (listener location to deadline, plus commands) | B |
| round 3 (this lens) | the grounds of admission | B |

Everything outside the B layer has held for three rounds. Round 3's flaw is an omission in folding round 2's own proposal, not a new direction, which points to a classification that is converging. A round 4 would most likely find:
- reconciler protocol details (findings 2, 3 and 7);
- block re-mappings (finding 4, and `settle` and `scrollStep` once witnessed);
- desktop timing claims that are still unverified.

All of these are DETAIL, provided admission gains the witness ground. If it does not, round 4 finds this same contradiction again through C5, `settle` or `place`.
