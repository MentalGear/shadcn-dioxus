# Round 7, lens `coherence-execution` (2026-10-26)

Plan read: `lane/js-plan-docs-r6` @ 640cad14 (`/home/user/wt/l434d4`). Code: `/home/user/wt/b28` (queue/batch-29), `/home/user/wt/l30` (lane 3.0), the analysed commit `d10d34b5`, Dioxus 0.7.9 sources. Read only; nothing built or run. Labels: V (read in code or crate source), W (web), I (inference).

**Verdict: HOLDS WITH CHANGES**

**DESIGN findings: 0, DETAIL findings: 17**

- The core construct is consistent across the root doc, the residue doc, js-interop.md, row 434 and rows 442-474. The scripted check of the 3.3 ledger passes: all 103 audit ids appear exactly once, each with a lane. None of the contradictions I found needs a new construct.
- Phase 0 cannot prove itself as written. Proof port A's `prevent` witness is green by construction under round 6's site-relative counterfactual (finding 1). Proof port B has no red available inside phase 0 (finding 2). Four phase-0 proofs need artefacts from outside phase 0, and the rows carry no grant or `adds:` front matter (findings 7, 8).
- Two round-6 folds contradict other rules: the port's "never inside a form" rule against the popup-boundary rule (finding 3), and the executor keyed by scope id against the `(scope, nonce)` slot key (finding 4).
- One census is wrong. 84 `set_focus(` sites are mostly the Rust-state `FocusState::set_focus`. The real DOM focus writes are about 28, and the roving hub is one line, `collection.rs:520` (finding 5). That shrinks the roving row and moves it onto 3.SC's file.
- Execution: phase 0 is one serial chain of about 85 of its 89 h. Item 0.1 builds gates over artefacts that 0.3 and 0.7 create later. Per-site `#[expect]` cannot attach inside rsx (findings 6, 10).
- Newcomer test: js-interop.md places 2 of 5 new behaviours correctly. It stalls on the other three: a cancel of a hot event at an element, measuring and streaming, and the print or UA-snapshot deadline (finding 16 and part 3).

---

## Part 1 and 2: numbered findings

### 1. DETAIL, major (blocker for item 0.3's stated proof). `otpFilter`'s `prevent` witness is green by construction

**Claim attacked.** Root `:748` and row 448: `otpFilter`'s "`prevent` and `setValue` exercise `defaultAction` and `wrongPaint`, so its witnesses are red by construction".

**Evidence.**
- (V) Root admission point 2, `prevent` row (`:340`): the next-best placement is "installed in the bubble phase at the host element the owning component renders (the site's own host: the rsx placement)".
- (V) The OTP listener is an element-target listener on the input itself: `input_otp.rs:365-381` (`listen(el,'beforeinput', ..., true)`, `e.preventDefault()` at `:378`).
- (W, DOM dispatch) A bubble listener on that same element runs at the target and sets the same canceled flag before dispatch ends. So the counterfactual arm cancels exactly as the real arm does: `defaultAction` cannot diverge, and the witness is green.
- This is the same argument round 6 used to demote L6. It also hits `labelClick` (a `mousedown` on the label itself, `label.rs:67-71`) and the contenteditable `beforeinput` filter.
- (I) The demotion then lands on "the rsx placement", which does not exist: Dioxus 0.7.9 has no `onbeforeinput`, and `MouseData` has no `detail`.
- (I) `setValue` is green too for a typed rejected character or a paste. With the cancel kept, the delayed arm paints the old value and then the new one, a time-shifted copy, so trajectory inclusion is green. It goes red only on the composition path, where `beforeinput` cannot be cancelled.

**Change.**
- Derive the `prevent` next-best placement from the table's own first-row condition. If Dioxus delivers the event and every field the decision reads, it is bubble at the site's host. Otherwise it is R, which for a cancel means **dropped** (an async answer arrives after dispatch).
- Give `otpFilter`'s `setValue` witness an IME composition scenario (CDP `Input.imeSetComposition`).

**Subsumes.** The proof-port-A red, `labelClick`'s witness and the contenteditable filter's witness: the whole "missing event or field" class.

### 2. DETAIL, major. Proof port B has no admitting red inside phase 0, and L5 is described two ways

**Claim attacked.** Root `:62`, `:325`, `:383`, `:640`: the executor is "admitted as a whole by the reds it already has (the exit wait's, and `indeterminate` once the projection can see it)". Phase 0.4 (row 449) "flips" the exit-wait, watchdog and port witnesses.

**Evidence.**
- (V) The modal driver is `use_dialog_open_driver` at `lib.rs:652`, and item 0.4 "must not touch the rest of `lib.rs`". So in phase 0 the executor's modal kind has no consumer.
- (V) `indeterminate` and `defaultChecked` are applied in `checkbox.rs` and `default_checked.rs`, which belong to 3.FC. Root `:383` itself says the `indeterminate` witness is "expected green".
- (V, root 2.3 item 5) The watchdog arms only for declared pending states. The per-surface list and the pending CSS are 3.PL's (row 452); O3 and V19 are 3.GS's and 3.PV's. In phase 0 there is no pending state to drop the watchdog against.
- **L5 is not a dialog close step.** The audit (`js-audit-2026-10-25.md:95`) defines L5 as "`use_animated_open`: wait for exit animations before unmount", used by 15 components including Accordion, Drawer and Listbox. The cited `lib.rs:1060` is `fn use_animated_open` (V). Twelve primitives files call it (V: accordion, alert_dialog, context_menu, dialog, dropdown_menu, hover_card, listbox, menu_root, navbar, navigation_menu, popover, tooltip).
- The plan describes L5 two ways:
  - "the executor's exit wait: Rust's `close` command ... the page ... calls `close()`" (3.1 `:517`, ledger `:571`);
  - "methods: ... `getAnimations` (L5)", a bridge verb row (5.2 `:696`).
- Accordion and Listbox have no reconciled kind, so the executor cannot be their placement.

**Change.**
- State which red admits the executor *in phase 0*. Either 0.4 adds a fixture route with a modal on the executor (the exit-wait red), or the port's order red (the `popover_sync.rs:200-225` dismiss-then-`onclick` case) counts. Item 0.4's proof then names that red.
- Split L5:
  - modal close: the executor's exit wait (3.OV);
  - unmount hold for the non-reconciled users: R (`getAnimations` row plus a `finished` fact; a late unmount paints nothing wrong) or A (`@starting-style`/`allow-discrete`).

**Subsumes.** Phase 0.4's unfalsifiable proof line. The L5 placement for Accordion, Drawer and Listbox.

### 3. DETAIL, major. The transition port's "never inside a form" contradicts the popup-boundary rule

**Claim attacked.** Root `:295`: "a host inside a form renders the port as the panel's sibling outside it".

**Evidence.**
- (V) `dioxus-interpreter-js-0.7.9/src/ts/serialize.ts:432-446`: `target.closest("form")`, then the whole `FormData` for `input`, `change`, `submit`, `reset` and `click`.
- (V, CLAUDE.md popup-boundary) A popover or dialog stays a DOM descendant of its trigger.
- So for every in-form popup, any sibling of the panel is inside the form too. In-form popups exist: V, `preview/src/components/combobox/variants/{multiple_form,clear_form}/mod.rs` use `DemoForm`.
- "Each owner renders its own port in its own component" leaves no element outside the form. On Android a big form's port event exceeds the header limit and is dropped silently (root `:169`), which loses the order guarantee.

**Change.** Fire the port with an event type that `serializeEvent` does not extend with form values (anything outside those five types that dioxus-web converts with a plain `web_sys::Event`; `convert_form_data` accepts any `Event`, V `dioxus-web-0.7.9/src/events/mod.rs:67`). Alternatively, state in the slot table that in-form hosts carry the form's cost and add a payload-size check. Drop the "sibling outside it" sentence.

**Subsumes.** Every in-form Select, Combobox and DatePicker transition.

### 4. DETAIL, major. The executor's divergence memory is keyed by scope id, which the slot key forbids

**Claim attacked.** Root `:327`: "the executor records 'the browser diverged at epoch e' keyed by the owning scope's id (`scope`)". The command attribute carries `(scope, state, based_on)` (`:327`, `:444`; js-interop `:58`).

**Evidence.** (V) Root `:291` keys every slot by "the owning scope plus a mount nonce ... never ... by scope id alone: Dioxus ... reuses freed scope ids (`scope_arena.rs:25-26`)". Verified: `vacant_entry()` and `ScopeId(entry.key())`. A dismissed panel's divergence memory is then inherited by the component that reused the id, and its first open command is dropped.

**Change.** The command attribute carries the slot id `(scope, nonce)`, and the executor keys by it. One sentence in 2.2 kinds, 2.3 item 4 and js-interop.

**Subsumes.** The re-key and responsive-swap resurrection case that the nonce was added for.

### 5. DETAIL, major. The `set_focus` census conflates a Rust-state method with the DOM write; the roving row is sized and owned wrongly

**Claim attacked.** Root `:300`, `:319`, `:531`, `:771`, row 462 and js-interop `:53`: "82 `set_focus(` sites in 36 files (78 of them in 32 primitives files)". They join clippy `disallowed_methods` "with a per-site `#[expect]`", and the roving row is about 30 h at "0.6-0.8 h per file".

**Evidence.**
- (V, b28) There are 84 `set_focus(` sites. Most are `FocusState::set_focus(Option<usize>)`, Rust-only state: `fn set_focus` exists at `collection.rs:281`, `radio_group.rs:49`, `toolbar.rs:26` and `drag_and_drop_list.rs:223`. Arguments: `Some(index...)` dozens of times, `None`, `index`.
- (V) `MountedData::set_focus(true|false)` appears 28 times in 18 files.
- (V) The roving DOM write for every collection widget is **one** call, `collection.rs:520` (`control_mount_focus`, `md.set_focus(true)`).
- (V) `tabs.rs:389` (`ctx.focus.set_focus(Some(index()))`) is the state method.
- `collection.rs` is 3.SC's file (row 451), yet the roving row's file list is "the 32 files".
- Ledger `:558` puts S6 and L19 under "done in step 2 / none". The `MountedData` write ban requires every `MountedData` focus write to go through the focus owner, so S6 and L19 have no owner.

**Change.**
- Recount by def path: about 28 `MountedData` focus writes plus 7-9 `scroll_to`. Size the roving row on the hub: `collection.rs` through the focus owner, plus the non-collection widgets (tabs, radio, toggle-group, date segments). Give it `collection.rs` after 3.SC, or merge it into 3.SC.
- Assign S6 and L19 to the row that drains their `#[expect]`.

**Subsumes.** The roving estimate, the `#[expect]` count, and the owner of the non-roving `MountedData` sites.

### 6. DETAIL, major. Item 0.1 builds gates over artefacts that later items create, and two items claim one gate

**Evidence.**
- (V) Item 0.1 (`:744`, row 444) owns "`FORBIDDEN_VERBS` derived from the slot table". Item 0.7 (`:747`, row 447) owns "the slot table ... and its gate check, the verb partition and `FORBIDDEN_VERBS` derived from the slot table". So there are two owners, and 0.1 runs before the slot table exists.
- Item 0.1 owns "the `site!` literal scan", but the `site!` macro is item 0.3's (row 448).
- Item 0.1 owns the generated `Decision<E>`, but 0.3 restructures the registry into a per-file macro with typestate (`bridge/behaviours/mod.rs`).
- The rule "Behaviour lanes ... never edit a gate script" (5.3 item 11) means 0.7's slot-table gate needs `run-gates.sh`, `gates.md` and `check-bridge-behaviours.*`. All three are 0.1's files, and none is in 0.7's list.

**Change.**
- 0.1 builds each of these checks against a synthetic fixture with a self-test, and declares the real-input wiring a one-line hand-off owned by 0.3 and 0.7. Or move the slot-table and `site!` checks into 0.7 and 0.3 with `run-gates.sh` registration lines granted to them.
- Delete the duplicate `FORBIDDEN_VERBS` claim from one item.

**Subsumes.** The phase-0 ownership disjointness claim (`:752`).

### 7. DETAIL, major. Phase-0 rows carry no `js-grant:` or `adds:` front matter, so items after 0.1 are born red

**Evidence.**
- (V) No row 444-467 has a `js-grant`, `adds` or `js-fix` front matter line (grep of the first 9 lines of each).
- 5.3 item 5: once 0.1 lands, a raise is valid only against `js-grant:` at the merge base.
- 5.3 item 7: a new kind needs `adds: [<kind>]` at the merge base.
- What phase 0 then raises after 0.1:
  - 0.6: `bridge.ts` apply table, `ts/witness.ts`;
  - 0.7: `scroll_classifier.ts`, pump, focus owner;
  - 0.3: `otp_filter.ts`, a new kind;
  - 0.4: `executor.ts`;
  - 0.5: `ts/perf/**`.
- Root `:714` says "the main loop lands one grant row per planned behaviour in phase 0". The rows do not do it.

**Change.** Add `js-grant:` (bytes per column) to rows 446-450 and `adds: [otp_filter]` to row 448 before 0.1 merges. Rows 451-467 name "a grant row"; give them the field too.

**Subsumes.** Every phase-0 handback stalling on the ratchet.

### 8. DETAIL, major. Several phase-0 proofs cannot turn red or green in this container, or within phase 0

**Evidence.**
- **0.3 and 0.4 "specs on a release build"** (rows 448, 449). `lane-rules.md` ("Running the suite"): release builds are the deploy's, lanes verify on one debug `lane-ssg` build. A release tree is 3.6 GB (V, `du`: `release-template` 3.6 G, `deploy` 3.6 G) with 9.4 GB free.
- **0.4's "first-frame" spec** is a 3.PL spec (root 2.3 Proof): no gated surface exists before row 452.
- **0.7's proof includes "the desktop-cadence build of row 442 shows the order cases"** (row 447). Row 442 is outside phase 0 and outside the estimate.
- **0.5** cannot compile its desktop launch config here: preview's `desktop` needs gtk/webkit2gtk (CLAUDE.md; row 23 "WAITING ON OWNER"). "A launch config behind `?dx-perf`" (`:750`, row 450) cannot work on desktop: there is no address bar, and the inspector is decided at launch by `disable_context_menu` (V `webview.rs:453-459`, `config.rs:117`). It must be a build-time cfg or an environment variable.
- **The busy case.** The witness delay is "the p99 of the latency session's desktop distribution under its busy case" (`:343`), but `?dx-bridge-spin` is in row 443's remainder (measured group), not 0.5.
- **The jitter table.** 5.6 asks the session for "the wheel-event jitter table", yet no wheel recorder is in 0.5's files.

**Change.**
- 0.3 and 0.4 prove on debug `lane-ssg` plus `test:changed`.
- 0.4's spec list drops first-frame; 0.7 drops row 442 from its proof.
- 0.5's read-out goes behind an environment variable or cfg, and its proof is "the owner's build starts and dumps". If `rustup target add aarch64-apple-darwin` works here, 0.5 adds a `cargo check --target aarch64-apple-darwin -p preview --features desktop` (I: not tried).
- Move the spin hook and a wheel recorder into 0.5 (about +1 h).

### 9. DETAIL, major. Witness determinism: the host subtree's own animations are not paused, and witnesses are frame-sensitive specs planned on debug builds

**Evidence.**
- (V, root `:355`) The projection includes computed `opacity` and every property the host sheet declares, and pausing applies only "outside the host subtree". The popover hosts in phase 0 play the theme's enter and exit animation (`dx-components-theme.css:1775-1818`, root 2.3 item 2).
- (I) Two real runs sample those values at different frame phases. Under set-based trajectory inclusion the control arm ("two independent real runs must agree", `:370`) then diverges, and the witness admits nothing.
- (V, CLAUDE.md, row 116) Debug builds show 64-153 ms long tasks. A frame-counted counterfactual on a janky debug build drops real-arm frames, which gives false reds and red controls.
- Witnesses are said to run "in `test:changed`" (`:387`), which lanes run on debug.

**Change.**
- In witness mode, pause or zero the host subtree's animations through the same motion switch (`animation-duration: 0s` and `transition: none` under the marker), except for an effect whose counterfactual is the animation itself (the exit wait).
- Mark witnesses timing-class: release only, like `main-thread`, or debug with a stated tolerance.

### 10. DETAIL, major. Phase 0 is one serial chain, 0.1 is undersized, and a per-site `#[expect]` cannot sit inside rsx

**Evidence.**
- **The serial chain.** (V, `:752`) `ts/bridge.ts` and `primitives/tsconfig.json` go 0.0, 0.1, 0.6, 0.7, 0.3, 0.4, and only 0.2 runs in parallel. The critical path is 20+14+18+13+12+8 = 85 of 89 h, with one full `run-gates.sh` (15-21 min) per merge between links. At step 2's 1.1-2.2x that is about 94-190 h of wall. CLAUDE.md's "dispatch lanes concurrently" cannot apply.
- **0.1 is undersized.** 0.1's 20 h covers about 35 gate rules, each needing a `--self-test` per `check-gate-kit.sh`. It also covers about 107 eval lines, about 28-91 `MountedData` sites, about 20 hot-event sites and 3 `DataTransfer` sites of per-site `#[expect]`.
- **No per-site `#[expect]` in rsx.** (I, Rust reference) Lint attributes on expressions are unstable (`stmt_expr_attributes`). An rsx event attribute (`onresize: ...`) and a `Script {}` element are not statements, so the narrowest `#[expect]` is the enclosing statement or function. A second site inside that function is then silently covered, which breaks "shrinks by construction" (5.3 item 1) for the hot-events and `Script` sinks.

**Change.**
- Split `bridge.ts` before phase 0: the adapter, the apply table and channel wrapper, and the executor as separate modules imported by one entry. 0.6, 0.7 and 0.4 can then edit disjoint files.
- Re-size 0.1 (I: 35-45 h) or split it into three lanes: ratchet and grants; tsc and DOM-less; clippy and scans.
- For rsx sinks, count sites per function in the ratchet baseline (`check-js-volume` inventory) rather than relying on `expect` fulfilment.

### 11. DETAIL, minor. Candidate rows miss dependencies that the file order requires

**Evidence.**
- (V) Row 457 (3.VS: `settle`, `scrollOwner`) depends on 447 and 444 only. Row 459 (3.SL: `scrollLock`) depends on 457 and 444. Neither reaches row 448, which builds the registry macro, `site!` and the typestate that every new behaviour needs.
- Row 459 also lacks row 452, although `context_menu.rs` goes "0.3, 0.4, 3.PL, 3.SL, 3.HV" (`:773`).

**Change.** Add 448 to 457 and 459, and 452 to 459.

### 12. DETAIL, minor. Stale terms and mechanisms that the folds left behind

| Where | Stale text | Current plan |
|---|---|---|
| Root `:861` (section 6) and row 434 `:61` (G bullet) | "Rust mints it with the command" / "one epoch, minted by Rust with the command" | `:292` and `:327` say Rust mints nothing; it carries `based_on` |
| Row 434 `:64` | "a desktop-cadence web build (which also reorders facts across sessions the way the scheduler does)" | Root `:821`, row 442 item 1 and row 434 `:65` itself: cross-session reorder is not modelled |
| Row 442 item 6 | asks the build to carry "facts of two sessions in one batch" | same as above: not modelled |
| js-interop `:139` and row 434 `:57` | "Witnesses run on Playwright WebKit where it can observe the commit point" | Root `:61`, `:380`: Chromium plus a closed emulation list; WebKit runs are manual (row 442) and turn a platform row stale |
| js-interop `:51` | gestures go "all through one bridge session (the drawer's shape)" | The same cell, and root `:488`: start, end and cancel go on the port |
| Residue section 1 | a W8 boot probe and "the fallback is detected by `@supports (overflow-anchor: auto)`" | Root `:413`, `:498`: `stickToEnd` runs on every engine and "the boot probe selects nothing for W8" |
| Residue summary | "the latency budget the owner sets from the row 442/443 measurements" | the latency session, item 0.5 (row 450) |
| Row 434 `:41` | "after five adversarial boards" | six boards |
| Row 442 status | "The witness harness (item 0.6) and its order specs run first on this build" | Phase 0 has no such build |

### 13. DETAIL, minor. A one-shot focus write is described three ways

**Evidence.**
- (V) `focus_prevent_scroll` is described as three different things:
  - a closed verb row, "one member call `el.focus({preventScroll: true})`", legal from a caller TSV: 5.2 `:701`, 3.2 `:544`, js-interop `:57`, row 434 `:55`;
  - "called through the focus owner": 3.2 `:550`;
  - forbidden as a verb, since focus is a slot and "a verb that writes a slot ... exists only as that slot's owner operation": 2.2 `:399`.
- Admission point 5 (`:381`) lands a green `focus` witness as "a caller-listed verb row" and in the same sentence "it routes to the focus owner".
- L17 is placed two ways: the ledger `:573` says "rsx `set_focus` through `TriggerHandle`", while 3.2 `:542` says "a one-shot call of the focus owner (the ... `MountedData::set_focus` ... is a disallowed method)".

**Change.** Make it one rule: a one-shot focus is a focus-owner operation `focus_once{preventScroll}` with `based_on`. Delete the verb-row wording and fix L17's ledger cell.

### 14. DETAIL, minor. Counts that do not reconcile

| Figure | One place says | Another place says |
|---|---|---|
| witness tests | 5.4 `:791`: 90-100 tests, 5-8 worker-minutes | admission point 9 `:388`: 150-220 tests, 10-16 worker-minutes |
| full-port hours after round 5 | row 434 `:66`: 345-385 | root `:798`: 346-383 |
| `prevent_default()` calls | "133" (`:52`, `:462`) | 119 calls; 133 only with comment hits (V, `git grep` at d10d34b5) |
| executor witness pairs | admitted "as a whole, no per-kind decision" | admission point 9 still counts "the reconciled kinds about 12" pairs and "`openFocus` about 13 by spread site" |
| owner of S11 | ledger: 0.3 | 3.FC's blocks and relocated column |
| L23's relocated lines | still counted in phase 0's relocated note (`:752`) | L23 left phase 0 in round 6 |

- The 3.1 s per test assumes one navigation (I). A pair is two tests with two arms each, and a debug navigation plus hydration is about 1.8 s (lane-rules).

**Change.** One count with its derivation, and quote it everywhere.

### 15. DETAIL, minor. Citation drift (spot-check: about 85 file:line citations)

**Correct (V).**
- In b28: `popover_sync.rs:109-113`, `:239-243`, `:200-225`, `:294`, `:300-305`; `checkbox.rs:298`, `:341`; `collapsible.rs:41-47`, `:122-124`; `tabs.rs:389`, `:499`; `input_otp.rs:367`, `:379` (the cancel is at `:378`), `:434`, `:603-648`; `radio_group.rs:436`; `combobox/components/input.rs:152-161`; `segment.rs:283-299`, `:536-543`; `collection.rs:513-523`, `:738`; `scroll_lock.rs:267-298`; `move_interaction.rs:119-146`; `pointer.rs:33-48`; `lib.rs:330-375`, `:367`, `:781`, `:848`, `:544`, `:571-588`, `:590-606`; `dialog.rs:439-442`; `label.rs:69`; `drag_and_drop_list.rs:767`, `:773`, `:860-865`; `toast.rs:558`, `:529-540`, `:886`; `virtual_list.rs:386-400`, `:486`, `:487-493`; `modality.rs:87-108`; `attributes.rs:142-159`; `anchor_fallback.rs:21-28`, `:493`; all 13 spread call sites (`tooltip.rs:452` ... `combobox/components/list.rs:128`).
- In l30: `bridge/mod.rs:51`, `:151-159`, `:259-301`; `ts/bridge.ts:253-260`, `:337-340`, `:342-360`, `:405-418`, `:691-699`, `:700-708`; `session.rs:424-426`; `behaviours.rs:46-66`, `:124-200` (four no-wildcard functions); `register.ts:8`; `check-bridge-behaviours.sh:361-374`.
- In Dioxus 0.7.9: `dom.rs:88-116`; `events.rs:519-529`; `generational-box lib.rs:107-108`; `scope_arena.rs:25-26`; `elements.rs:1488`, `:1536`, `:1571`, `:1594`; `set_attribute.ts:24-35`; `data_transfer.rs:116-146`; `serialize.ts:279-313`, `:432-462`; `mounted.rs:115-141`; `webview.rs:453-459`; `config.rs:117`, `:127`; `ssr renderer.rs:146-158`; `pointer.rs:99`; `dioxus-0.7.9/Cargo.toml:122-125`.

**Wrong or drifted.**
- `scroll_lock.rs:422,432,450`: right at d10d34b5, but at **413, 423, 441** in b28, the tree that lands next.
- `toast.rs:1517-1535` (`role="alert"`): now about `:1477`.
- `date_picker/segment.rs:380` (`_ => ()`): it is `:358`, wrong at both commits.
- `lib.rs:1060` "L5" is `fn use_animated_open`; see finding 2.

**Change.** Re-read the drifted lines against batch 29 after it lands, or cite symbols, as the ledger already does for `lib.rs`.

### 16. DETAIL, major. js-interop.md leaves a newcomer without the rules that decide three common cases (details in part 3)

**Evidence.** (V, js-interop)
- **No rule for a frame-rate event at an element.** The deadline table (`:76`) sends an element cancel to rsx. The "follow a stream" row (`:62`) and the forbidden table (`:227`) ban `onwheel` and `onscroll`. The root's tie-breaker, "frame-rate events never take a vdom handler whatever their deadline" (`:286`), is absent from the page. So a cancelable wheel at an element has two contradicting placements.
- **The hooks it names are not callable from preview.** The "today" hooks are `pub(crate)` or private (`use_popover_sync`, `use_popover_shown_while_mounted`, `use_scroll_lock`, `use_global_keydown_listener`, V). A preview or demo author cannot call them.
- **No "use the existing primitive" step.** The page never says "use the existing component first" (HoverCard, Collapsible's `open`, DragAndDropList).
- **The "today" column endorses a banned write.** It says "`MountedData` where it suffices" for scroll writes (`:63`), which the planned ban will flag.

**Change.**
- Add the tie-breaker sentence plus a "cancel on a hot event at an element" row: a named behaviour row, which today means stop and report.
- Add a first row: "a primitive or component already does it: use its props".
- Mark which hooks are crate-private.
- Change "today" for scroll writes to "existing sites only; a new one is a report".

### 17. DETAIL, minor. `ts/witness.ts` and `ts/perf/**` would ship as bundle entries

**Evidence.** (V, l30 `scripts/build-js-bundle.sh:16-18`) Every `.ts` under `ts/` that is not `.d.ts`, `.gen.ts`, `lib/` or `types/` is an ENTRY with its own committed bundle. Items 0.6 and 0.5 put `primitives/src/ts/witness.ts` and `ts/perf/**` there. That contradicts "the release bundle holds no switch for it" (`:387`), and the bytes count against the ratchet as shipped.

**Change.** Put the harness under `playwright/witness/` (it is an init script anyway) and `perf` under an excluded dir (`ts/debug/`) that the bundle script skips, with a reasoned allowlist.

---

## Part 2 summary: can phase 0 be executed here as written?

| item | owner and files | dependencies | proof turns red and green here? | hours |
|---|---|---|---|---|
| 0.1 (444) | Disjoint, but claims `FORBIDDEN_VERBS` and the slot-table gate with 0.7 (finding 6) | Builds over `site!`, the slot table and the per-file registry before they exist | Self-tests yes, on synthetic fixtures only | 20 is low (finding 10) |
| 0.2 (445) | Disjoint (after 0.1) | ok | Yes (mutation self-test) | 4 ok |
| 0.6 (446) | `witness.ts` misplaced (finding 17); owns `playwright/witness/**`, which 0.7's spike also writes (serialized) | ok | Probes on `Fixture`: yes. Determinism issue (finding 9). `otpFilter` and executor witnesses stay `fixme` | 14 plausible |
| 0.7 (447) | Needs 0.1's gate files (finding 6) | ok | Order specs under the page-side hold: yes. Row 442 part: no (finding 8). The classifier spike's scrollbar case needs `ignoreDefaultArgs: ["--hide-scrollbars"]` (V Playwright adds the flag headless; precedent at `scroll-fade-scrollbar.spec.ts:37`) | 18 plausible |
| 0.3 (448) | Disjoint (`not(web)` arms) | Needs a grant and `adds` (finding 7) | `prevent` witness green by construction (finding 1); release-build specs not lane-runnable (finding 8) | 13 ok |
| 0.4 (449) | "One-line spreads" are at least 3 edits per site: bind the return value, spread it in the panel rsx, render the port, and the `not(web)` arm (V `tooltip.rs:452` vs the `#[cfg(not(feature = "web"))]` arm at `:521`). It also needs a `mod` line in 0.7's `bridge/mod.rs` | ok | The executor's admitting reds are unavailable (finding 2); first-frame is 3.PL's | 12 is low by about 3-4 (I) |
| 0.5 (450) | Disjoint | ok | Compile proof impossible (no gtk); `?dx-perf` impossible on desktop (finding 8) | 8 ok, plus the spin hook |

**Shared resources.**
- About 9.4 GB is free (V `df`), so one lane build copy fits at a time. The witness specs add `pw.lock` time.
- The chain is serial (finding 10), so "one build lock" is not the binding constraint: the file order is.

**The owner's latency session: one person with a Mac and an iPhone.**
- **The Mac session is feasible once three changes land:**
  - the read-out goes behind an environment variable or cfg instead of `?dx-perf`;
  - the spin hook is moved in, so the busy case exists;
  - the script says the first desktop build may fail in ways no agent here can reproduce. Row 23 is "verified at feature-resolution level only", so budget an owner debugging loop.
- **The iPhone adds nothing to the latency session.** The session is a desktop distribution. A Dioxus iOS build (`dx serve --platform ios`, Xcode and signing) is not in the script. The iPhone serves only the per-port Safari rows: one iOS version, so "iOS 26 and 27" is one of the two, and the plan already tiers that.
- So the budget and the witness delay rest on one macOS WKWebView distribution. WebView2, WebKitGTK and Android stay inferred.

---

## Part 3: newcomer test (reading only js-interop.md and CLAUDE.md)

| new behaviour | where the page leads | right? | where it stalls |
|---|---|---|---|
| **Sortable list with keyboard reorder** (Alt+Arrow moves the item) | "move focus along a list": `aria-activedescendant` on the container, rsx `onkeydown` with `prevent_default`, reorder in Rust. Correct and buildable today | **Yes**, for the key path | Scrolling the moved item into view: the design says `scrollOwner` (not built); "today" says `MountedData`, which the ban will flag. Pointer drag reorder: `pan` is not built and the `DataTransfer` setters are banned (`dropAnywhere` not built), so stop and report. The page never says DragAndDropList already exists |
| **Rich tooltip that stays open while hovering its content** | "open or close a popover", the hook `use_popover_shown_while_mounted`; rsx `onpointerenter`/`onpointerleave` (human rate) on trigger and content; `sleep` for the delays; `use_anchor_position_fallback` for placement | **Yes**, inside `primitives` (HoverCard's shape) | Hooks are `pub(crate)`, so a preview author cannot use them. A pointer "grace area" between trigger and content needs `pointermove` (a hot event: "never a vdom handler"; `use_sample` not on main), so stop and report |
| **Number input with wheel-to-increment** (cancel the page scroll) | The deadline table says rsx `onwheel` with `prevent_default`. The forbidden table bans `onwheel`, and "follow a stream" says never a vdom handler | **No**: two contradicting placements. The root's answer is "frame-rate events never take a vdom handler whatever their deadline", hence a named behaviour, and none fits (`scrollLock` is window and lock-scoped) | Stalls. The correct path (a new behaviour with an `adds:` row and a witness) is not stated for an element target |
| **Sticky table header that measures its columns** | The rule 1 "CSS first" list does not name `position: sticky` or subgrid, so a newcomer goes to "read size: `MountedData`", "follow a stream (resize): `use_resize_observer`" (not on main; `onresize` is banned) and "write a `--dx-*` property: `ElementExt`" (not on main) | **Partial.** The right answer is A (one scroller with `position: sticky`, or a shared grid or subgrid), which the page's A list omits. The "before paint" row would send a careful agent there | Stalls on measuring (no resize hook on main) and on syncing horizontal scroll (a stream). The page never says Rust can write inline style through rsx |
| **Print-friendly "expand all" toggle** | The button is rsx `onclick` on Rust state. Expanding means writing `hidden=until-found`, a reconciled kind, whose "today" lists only popover and modal hooks. `beforeprint` falls under "a window event when a late answer is harmless", `use_window_listener` (not on main), so report | **Partial.** The right answer is A (`@media print` revealing `[hidden]`/until-found panels) plus Collapsible's `open` prop. A late `beforeprint` answer is not harmless on desktop: the snapshot follows dispatch | No deadline row for a UA snapshot (print). No "use the component's props" step. The reconciled-kinds row's "today" omits Collapsible and Accordion |

**Verdict.** The page leads to the right placement for 2 of 5 today (keyboard reorder, hover tooltip in `primitives`), and only for their core decision. It stalls or contradicts itself on the three others: a hot event at an element, measuring and streaming, and the print or UA-snapshot deadline. The plan's design would place them; the page does not say how.

---

## Attacks that failed

- **The ledger (scripted).** All 103 audit ids appear exactly once in the 3.3 ledger's first column, none is missing, none is extra, and every id has a lane. Script: `/tmp/claude-0/-home-claude/de0bcb28-b15b-5de9-9a2c-16b2a68801c1/scratchpad/ledger.py`. Class counts match the audit: T 11, L 30, S 19, O 5, W 8, C 6, V 23, P0 1.
- **Hour sums reconcile.** Phase 0 is 89 (20+4+14+18+13+12+8). The candidate table is 149. The measured group is 85-114. The total is 356-393.
- **All 13 popover spread call sites** exist at the cited lines in b28.
- **The port is not dropped on web.** dioxus-web's `event_type_matches` for `input` checks against `web_sys::Event` (`convert_form_data`), so a synthetic plain `Event('input')` is not dropped.
- **The root-level native `stopPropagation`** cannot starve Dioxus's own root listener, which sits on the same node (`stopPropagation`, not `stopImmediatePropagation`).
- **Bundle entries are auto-discovered** (`build-js-bundle.sh:33-39`), so no phase-0 item must edit 0.1's bundle script to add a `.ts` entry. Only the shared manifest needs a rebuild at merge, as the plan says.
- **Headless Chromium hides scrollbars**, which looked as if it would kill the classifier spike's scrollbar case. The repo already relaunches with `ignoreDefaultArgs` (`scroll-fade-scrollbar.spec.ts:37`, `listener-inventory.spec.ts:394`).
- **The spec budget headroom** (17,000 - 12,217 s) matches the plan's figure within 4 s.
- **The disk figure** (about 10 GB usable) matches `df` (9.4 GB free).
- **Executor admission as a DESIGN break.** I tried to show that admitting the executor "as a whole" breaks the per-effect admission unit. It reduces to naming which red admits it, and in which item (finding 2): DETAIL.
- **The transition port as a carrier.** I tried to show that form serialisation breaks the carrier. A different event type or placement fixes it, and the carrier stays (finding 3): DETAIL.
