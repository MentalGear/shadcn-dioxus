# Round 4, lens `coherence`: do the plan documents say one consistent thing?

VERDICT: HOLDS WITH CHANGES

DESIGN findings: 0, DETAIL findings: 25

Summary:
- I found no contradiction that touches the root rule, the deadline-indexed rule, the admission grounds, the closed bridge or enforcement by sinks plus ratchet. Every finding below is a stale line, a lane boundary, a count, a term or a gap in the agent-facing page.
- The plan's prose was folded after round 3; row 434 (lines 61, 63, 67), the residue doc and `js-interop.md` were not folded completely. Row 434:61 still carries round 2's G text (a `data-dx-anchored` gate, "the generation is page-owned"), which the root doc reversed.
- Lane 3.0's branch (`/home/user/wt/l30`, HEAD 033cc695) has moved past the plan's description of it: the gesture verbs and `commit` are already gone, the transport is already a `Program` enum, the lease is already mandatory for a non-passive behaviour. Four documents still describe the earlier state.
- `js-interop.md` states five things as landed that the landed gate does not do (a `dd::eval` fn pointer, `gloo_events`, `.md`/`.svg` scanning, the "what ships" counting, an allowlist with reasons for the pre-wasm class).
- Ownership: 27 class-(b) audit ids have an owner but no recorded placement; L1/L2 have two owners; the six class-(c)/seam ids have none; seven behaviour-building rows edit three files that no row owns; the `reconcile` modal, `indeterminate` and `defaultChecked` kinds and the boot hook have no owner.
- An agent reading only `js-interop.md` and CLAUDE.md lands in the right place for the dialog hotkey, in a planned-but-absent place for the drag (with an ambiguity about the per-move output), and gets contradictory instructions for a new popover open path (section C).
- The one item I could argue is DESIGN is finding 15 (admission is per module, so one detected effect admits undetectable ones such as `setVar`, `scrollBy`, `setValue`). I kept it DETAIL: the two grounds and the closed effect vocabulary already exist; the fix applies them per effect.

Evidence labels: V = read in a file (path:line given), W = web source, I = inference. Paths: "root" = `dev-docs/research/sync-js-root-2026-10-26.md`, "residue" = `.../sync-js-residue-2026-10-26.md`, "434/442/443" = `dev-docs/backlog/NNN-*.md`, "r1/r2/r3" = the three `SYNTHESIS.md`, "b28" = `/home/user/wt/b28`, "l30" = `/home/user/wt/l30` (branch `lane/js-bridge-3-0`). All documents are under `/home/user/wt/l434d3/dev-docs/`.

---

## Findings

### 1. DETAIL, major. Row 434:61 still states round 2's G, which the root doc reversed.

- Claim attacked: row 434 "Step 3 adopted" is "the current plan" (434:43).
- Evidence (V):
  - 434:61: "Gate by `:popover-open:not([data-dx-anchored]):not([data-dx-placed])` (the tier predicate, a page-owned `html[data-dx-anchor]` ...)". The selector shown uses the vdom attribute; the parenthesis says the page attribute is the predicate. root:325 says `data-dx-anchored` "cannot carry an engine tier" and "is no longer the gate's predicate"; the gate is `html:not([data-dx-anchor="a"]) :popover-open:not([data-dx-placed])`.
  - 434:61: "the **generation is page-owned**". root:330 (2.3 item 4): "One generation, minted by Rust with the command"; js-interop:42 says the same ("a Rust-minted generation"). r2:35 is the source of the stale wording ("The generation is page-owned (`data-dx-placed` had two writers)").
  - 434:67: "loading via `asset!` + `document::Script` with a bounded ready handshake (#5919)". root:529 and l30's row 434 text say that route was **not adopted** (the bundle is evaluated through the transport at first use).
  - 434:63: "the built branch has it [the lease] optional". See finding 2(c).
- Change: rewrite 434:61, :63 and :67 from root 2.3 and 5.2, or replace the G and bridge-loading paragraphs by a one-line pointer to root 2.3 and 5.2. 3.PL's brief is built from row 434, so this is the line an agent implements from.
- Subsumes: the r2 text in finding 22.

### 2. DETAIL, major. Four documents describe lane 3.0 as it was before its last commits.

Evidence (V, l30 HEAD 033cc695 against the plan text):
- (a) Gesture verbs. js-interop:126 says `set_pointer_capture`, `release_pointer_capture`, a plain `focus` and `commit` "exist on the branch and are removed by 3.0b"; root:527 says "Lane 3.0's tables ship `setPointerCapture`, `releasePointerCapture`, `focus` and `commit` as verbs ... 3.0b removes them". In l30 `Method` (`primitives/src/bridge/mod.rs:260-330`) has no such variant, `ts/bridge.ts:16-17` says they "are NOT verbs ... There is no `commit` either", and `scripts/check-bridge-behaviours.sh:34,95,364-376` already enforces it. (The `"focus"` hit at `bridge.ts:132` is an `EVENTS` row, not a method.)
- (b) Transport. root:515 says lane 3.0's `transport::open(body: &str)` and `request(expression: &str)` are `pub(crate)` string seams to be replaced. l30 `transport.rs:124-132` already has `enum Program { Serve, Pump, Legacy(&str) }` and `pub(in crate::bridge)`. What remains is the `Legacy(&'a str)` variant (the quarantined `js_listener` script body, built only in `bridge/legacy.rs`), which js-interop:144 ("a `Program`-typed sink") and root:515 ("variants map to `&'static str` constants") do not mention; root:452 does know it ("the existing `Program::Legacy`"), so root:452 and root:515 contradict each other.
- (c) Lease. root:315 and 434:63: "the landed lane-3.0 branch makes it an optional `lease_ms`, so a `prevent` variant with none passes every gate". l30 `behaviours.rs:103-111` (`lease_for`) gives every non-passive behaviour a lease (`CANCEL_LEASE_MS`) by construction, and l30's row 434 text says so. The gate-side rule ("mandatory for any `prevent` or `stop` variant", a check on the generated file) is still planned, but the premise is stale, and root:683 ("differs from this plan in four places") is now three.
- (d) `focus_prevent_scroll`. js-interop:126 says it "stays as one closed row", js-interop:41 says it is "planned". l30 has no such row (grep of `bridge/*.rs` and `ts/bridge.ts` finds none).
- Change: after lane 3.0 merges, update js-interop:126, root:315/515/527/683 and 434:63 to the built state (one commit); until then add "as of l30 033cc695" banners. The bridge-latency "never" setting is also already built (finding 17).

### 3. DETAIL, major. `js-interop.md` presents five behaviours as landed that the landed `check-js-volume` does not have.

Evidence (V, b28 `scripts/check-js-volume.sh`):
- js-interop:170, "a bare `eval` function pointer ... (sink count, aliases included; landed)". The sink regex (script line 124) is `\bdocument\s*::\s*eval\b|\buse\b[^;]*?\beval\b|\beval\s*\(`. I ran it: `let f = document::eval;` matches; `use dioxus::document as dd; let f = dd::eval;` does **not** (no `use ... eval`, no call parenthesis). 434:63 and r2:38 list exactly this as a hole to close.
- js-interop:175 lists `gloo_events` as gated. The `ffi` regex (line 129) names `gloo_utils`, not `gloo_events`. root:537 lists `gloo_events`/`gloo_file` as to-be-added.
- js-interop:174 lists "raw HTML with a handler in a `.md` or `.svg` that reaches `dangerous_inner_html`" under `check-js-volume`. The universe (script lines 112-118) is `.rs`, `JS_EXT` and `HTML_EXT` only; root:537 lists `.md`/`.svg` scanning as a hole.
- js-interop:194-195, "What counts is what ships (bundle bytes and JS held in Rust), not tests, `.d.ts`, generated files". The landed gate counts every tracked `.ts` outside `docs/`, `playwright/`, `scripts/`, `dev-docs/` and `primitives/src/js/`: l30's baseline diff shows `primitives/src/ts/types/bridge.d.ts 89`, `behaviours.gen.ts 16` and `primitives/ts-test/*.ts` (361, 730, 274) as `js_file` rows. root:541 item 6 plans the change.
- js-interop:51 says the pre-wasm class is "allowlisted with its reason". The landed mechanism is baseline rows without reasons (`scripts/check-js-volume.baseline.tsv:5` `preview/index.html 7467`, `:36` `theme.rs`, `:69` `modality.rs`) plus inline `/*listener-ok: ...*/` markers inside `PREPAINT_JS` (`preview/src/theme.rs:143`). root:538 plans the reasoned TSV and "no inline-marker escape".
- Two pointers repeat the over-claim: `dev-docs/README.md:34` (b28) names `check-bridge-behaviours` as a gate `js-interop.md` points to (no such script in b28), and `CLAUDE.md:51-53` says "browser APIs only through the bridge (`primitives/src/bridge/`)" with no "planned" (that directory does not exist in b28).
- Change: tag each of the five cells "planned" (or remove the claim), and add "(planned; lane 3.0)" to the CLAUDE.md and README sentences. See Appendix A for the full claim check.

### 4. DETAIL, major. The agent-facing page and the root disagree on who adds closed-table rows, and on what "3.0b" is.

- Claim attacked: js-interop:138 "Missing rows (...) are added once by 3.0b, so a lane consumes rows and does not add them".
- Evidence (V): root:520 says the same, then: "(under the default scope of section 5.0 the row that consumes a missing row adds it, with its grant)"; root:488 says the same under option B; root:312 likewise. The default (B) is what js-interop:17-19 itself adopts. "3.0b" appears as two things: the gate half is phase-0 item 0.1 (root:568); the "verb-and-skeleton sweep" half no longer exists under B but is still cited (js-interop:119 "Lane 3.0b (row 434 phase 0) changes it as marked"; js-interop:138; root:312, 454, 520, 536, 543). js-interop:212-214 also says "A verb row is data" without saying that a new row raises `bridge.ts`'s `js_file` and so needs a grant (js-interop:190-193, root:540).
- Change: say in js-interop:138 and :212 "the row that first needs the verb adds it, with its grant row"; define "3.0b" once as "the gate half (0.1)" and delete the sweep references, or reinstate the sweep as an item with hours.
- Subsumes: part of finding 7.

### 5. DETAIL, minor. js-interop duplicates a sentence and singles out one behaviour.

- js-interop:189-190: "A rise fails, and a baseline edited to match is compared with the baseline at the merge base. A raise passes today with a note on the baseline row, compared with the merge base's baseline" says the same thing twice.
- js-interop:107 "`settle` must show its witness" reads as if only `settle` does; root:282 says `settle`, `stickToEnd`, `openFocus` and `reconcile` all pass only by witness (js-interop:98-99 already says `sameFrameWrite` and `focus` cannot be detected).
- Change: one sentence each.

### 6. DETAIL, major. Phase and lane vocabulary is dangling after the scope split.

- root:470 "the five-phase plan" and root:484 "A. Port everything now: the five-phase plan": root 5.4 now has "phase 0", "candidate rows", "the owner-measured ports" and (under option A) "batches 1-4". There are no phases 1-4.
- residue:24 "each its own deploy in phase 4", :93 "runs last and alone (phase 4)", :263 "3.CA runs earlier, in phase 3". Root 5.4:561 puts 3.CA in batch 4 with 3.MS and 3.WB (the "measured group"); "alone" and "phase 3" contradict it.
- residue:5, :24 (via :258), :79, :258 call lane 3.FZ "phase 0" and say "Phase 0 (lane 3.FZ) constructs the bridge seam and the proof ports and freezes the legacy arms"; 443:18 and :26 call it "phase 0 of the row 434 plan" in the same sentence that says "under the default scope B, only when a measured port is scheduled". root:488, :602 and :452 put 3.FZ in the measured group, existing "only when a measured port is scheduled"; phase 0 is items 0.0 to 0.5 and contains no 3.FZ.
- Change: replace "phase N" by "batch N" or "the measured group"; delete "phase 0" next to 3.FZ in residue and 443; define "five-phase" as "phase 0 plus batches 1-4" once.

### 7. DETAIL, major. `popover_sync.rs` and T4-T7 have two owners in the prose.

- root:410: "Lane 3.PL owns `popover_sync.rs`"; root:411: "go with the rest of `popover_sync.rs` (the reconcile proof port of section 5.0, otherwise lane 3.PL)"; root:626: 3.PL "now also holds T4-T7 and the G pieces".
- root:571 (item 0.4), :594 ("`popover_sync.rs` and `ts/behaviours/reconcile.ts` phase 0 (proof B), then read-only for 3.PL and 3.OV"), :617 ("T4-T7 to proof port B") and the 3.PL row (:584, "consumes ... and edits neither") say 0.4 owns it. The proof ports exist under every option (root:466, :561), so "otherwise 3.PL" has no case.
- Change: delete "otherwise lane 3.PL" and the two stale sentences (:410, :626).

### 8. DETAIL, major. The `reconcile` kinds, its boot hook and the boot probe have no owner.

- root:302 gives `reconcile{kind}` four kinds (popover, modal, `indeterminate`, `defaultChecked`) in one behaviour. Item 0.4 builds it "for T4 to T7" (popover only; root:571, 6 h). 3.OV is to consume `reconcile{modal}` (root:588) and 3.FC `reconcile{indeterminate, defaultChecked}` (root:587), but root:594 makes `reconcile.ts` "read-only for 3.PL and 3.OV", and 3.FC owns only `otp_filter.ts` and `drop_anywhere.ts`. Adding a kind needs an edit in `reconcile.ts`, its arm in `bridge/behaviours.rs` and its vectors.
- The reconciler is "one observer installed at boot with the bridge" (root:270, :302). l30's row 434 text says `bridge.ts` "leaves a comment seam at `pageObserver` and builds nothing". Neither 0.0 (done) nor 0.4 lists `ts/bridge.ts`.
- The boot probe (root:225: "runs synchronously at every bridge load", writes `html[data-dx-anchor]`) is assigned to 3.PL (root:584), whose files are Rust and CSS and which "edits neither" TS file. r3:37 flagged "3.PL's G pieces without a TS owner"; the fold left the probe's page code, the page-side watchdog (root:329) and the stale-`placed` removal (root:330) unowned or implied.
- Change: have 0.4 build all four kinds, the boot install and the stale-`placed` removal (re-size its 6 h), and give the probe an owner (a `ts/behaviours/probe.ts` row in 3.PL's files, or 0.4).

### 9. DETAIL, major. Seven behaviour-building rows edit three files that no row owns, and the built gate contradicts "behaviour lanes never edit a gate script".

- js-interop:215-219 and root:546 item 11 require, per behaviour: the variant in `bridge/behaviours.rs` (four no-wildcard matches), a module, vectors in `conformance/behaviours.json`, regenerated `behaviours.gen.ts`, a budget row. The lane table lists `bridge/behaviours.rs` only for 0.3 (`hotkey` arm) and 0.4 (`reconcile` arm). The consumers 3.GS (`pan`), 3.FC (`otpFilter`, `dropAnywhere`), 3.OV (`openFocus`), 3.VS (`settle`), 3.SL (`scrollLock`), 3.MS (`stickToEnd`) and 3.CA (the `pan` variant) each need an arm and vectors but list neither file (root:586-590, :605-606). root:611 covers `behaviours.gen.ts` (regenerated at merge) and the bundle manifest, not these two. The 3.OV row also names `behaviours/open_focus.rs` (root:588), a path that does not exist in the design (one enum file; l30 `bridge/behaviours.rs`).
- In l30 a behaviour's budget is a dict literal inside the gate script (`scripts/check-bridge-behaviours.sh:105` `BEHAVIOUR_BUDGETS = {"fixture": 26}`, error at :280 "no entry in BEHAVIOUR_BUDGETS of scripts/..."): adding a behaviour edits a gate script. root:546, js-interop:219 and root:568 ("budgets as a ratchet", item 0.1) make this a 0.1 deliverable; until it lands the sentence is false.
- The effect set and witness record that root:279-280 and js-interop:94-102 compare are not fields of l30's `Declared`/`Info` (`behaviours.rs:96-101`: target, event, capture, passive, lease_ms). No item owns adding them: 0.1's list (root:568) is scripts and `transport.rs`, and 0.0 "must not touch the behaviour rows' files".
- Change: add `bridge/behaviours.rs` (a per-variant arm), `conformance/behaviours.json` and the generated-file test to each consumer row's owned list with a merge order, move budgets into a TSV in 0.1, and put the effect-set and witness fields in 0.1 with `behaviours.rs` named.

### 10. DETAIL, minor. File ownership inside phase 0 and between 0.x and the gate registry.

- 0.1 and 0.2 both own `scripts/check-js-volume.*` (root:568-569; "0.1 and 0.2 (scripts)" run together, root:574). The freeze (0.2) edits the same Python the grant and counting changes (0.1) edit. Order them.
- 0.3 deletes `ts/focus-trap.ts`, `js/focus-trap.js`, its `bundle.manifest` line and, on l30, the `scripts/check-bridge-behaviours.allow.tsv` entry (root:570, :492), none in its owned list (0.0 owns `ts/**`; the manifest and the TSV are 0.1's).
- 0.1 does not own `scripts/run-gates.sh`, the generated `dev-docs/gates.md` (`scripts/gates-doc.py --write`, `check-gates-doc`) or `scripts/check-installed-paths.sh` (root:545 item 10 extends it, no owner).
- root:546 says each new gate is "registered in `run-gates.sh` and in CLAUDE.md's gate sentence". b28 `CLAUDE.md:3-4` says "a new gate never needs an edit to this file" and `:56-57` says "register it in `run-gates.sh`, run `scripts/gates-doc.py --write`; no other doc is edited". 442:37 likewise sends a rule to CLAUDE.md.
- Change: reword root:546 and 442:37; add the three files to 0.1.

### 11. DETAIL, minor. A lane condition that has resolved.

root:378 "Lane 2D owns this file while it is in flight" and the 3.VS header root:589 "(after lane 2D merges; otherwise W1 stays with 2D, which takes its keep-JS exit)". Lane 2D merged (b28 `git log`: d06cebc2 "Merge ... lane/js-bridge-2d", and 434:114 records the keep-JS outcome with no code change). Delete both conditions.

### 12. DETAIL, major. Not every audit id has exactly one placement and one owner.

Full table in Appendix B. Short form (V, from `js-audit-2026-10-25.md` and a scripted search of root sections 3, 3.1, 3.2 and 5.4):
- 27 class-(b) ids have an owner (root:617) but no placement row anywhere in sections 3 to 3.2: L13, L24, L26, L28, L29, S7, S10, S12, S13, S15, S16, S17, S18, C6, V5, V7, V8, V9, V10, V12, V13, V14, V18, V20, V21, V23, P0. Their placement ("R, a closed row") is inferable and never stated, so "every block is placed by the deadline table" (root:364) is not checkable for them.
- L1 and L2 have two owners: 0.3 ("L1/L2 as rsx handlers", root:570) and 3.OV ("the `reconcile` consumers for the modal driver (L1/L2's `showModal`/`close`)", root:588), both in `lib.rs`. Ordered (root:594), but two owners and three placements per id.
- L15, V1, V2, V3, V4 (class c) have no id-level placement, and V2/V3 are not named at all; the work "generate the three copies of the modality writer and two of `PREPAINT_JS` from one `.ts`" (root:442) has no lane and no hours (3.OV owns `modality.rs` "only if `check-open-focus` forces it", 3.PV owns `theme.rs`, nobody owns `preview/index.html`). V11 has a placement (the seam) and no owner.
- 13 ids moved in step 2 (L12, L14, L17 except Select, L19, S2, S3, S5, S6, S19, O2, O4, V15, V16) are absent from root by id except S2/S3 (3.HV). Fine, but the plan has no ledger of what is done.
- P0's owner is "the legacy removal" (root:608), which exists only if the owner schedules the measured ports; under B/C the last `JsListeners::install` can never go.
- The counts do not reconcile: "the 43 (b) rows" (root:393, :617) and "the board listed 41 ids" against 54 class-(b) audit rows plus L26 (55), of which 27 lack a placement row and 50 are not in the 22-block table.
- Change: one machine-checkable table (id, placement, lane) generated from the audit, with "done in step 2" and "class c, stays" as values; give the pre-wasm generation a lane; reword the P0 owner.

### 13. DETAIL, minor. L1's opener focus names the banned verb.

root:397 (L1): "the opener's click and focus go through bridge `call`". root:527 and :428 ban a plain `focus` verb (only `focus_prevent_scroll` from a reasoned caller list, whose first rows are the two L20 sites) and say a focus that a keystroke waits on is `openFocus`. L1 forwards Enter/Space to the opener, so its focus is keystroke-waiting: it belongs to `openFocus`'s close arm, as L18 does (root:427).

### 14. DETAIL, minor. Two definitions of the verb-row cap.

root:285 and js-interop:122/180: "one member call or one member read"; root:514: "one expression plus at most one guard `if`" (because the landed rows hold guards). A gate can enforce one. Pick one and say that the landed `showPopover` guard row is the exception or is rewritten.

### 15. DETAIL, major. Admission is granted per module, while the registry rows contain effects neither ground covers; `pan`'s outputs also contradict the closure rule.

- Claim attacked: root:278-282 ("a declaration admits nothing") together with the closure rule (root:262) and the registry (root:295, :298).
- Evidence (V, reasoning I):
  - Ground 1 (root:279) admits a behaviour whose code contains any one of `preventDefault`, `stopPropagation`, a `dataTransfer` write, pointer capture, an activation-gated call. The registry then lists effects with no ground of their own: `pan`: "`capture`, `prevent` (C4) (detected), `setVar` or `scrollBy` per output" (root:298); `otpFilter`: "`prevent` (detected), `setValue` with selection" (root:295); the closed effect vocabulary (root:312) also has `focus`, `after`, `release`. `setVar`, `scrollBy`, `setValue` are not detectable commit effects (root:280 says so of `sameFrameWrite` and `focus`), and nothing requires a witness for them when they ride in a module admitted by `capture`.
  - The closure rule (root:262, condition 2) says a gesture's per-move `scrollBy`, coasting, rubber band and `css-var` output "are next-frame decisions placed by that row", i.e. bridge stream plus Rust, a named behaviour only on a `fail` verdict. root:298 puts those outputs inside `pan` as variants (`css-var`, `scroll-by-with-refusal` 545 lines, `scroll-absolute`); root:372 and :386 say the output "shrinks" or "stays inside `pan`" depending on a measurement no gate reads.
  - Net effect: the 880 lines of "named behaviour" JavaScript (root:49) are bounded by the per-module budget, not by construction. A new gesture needing different output is an enum parameter of an existing kind: no new `adds:` row (the rule fires on "a new kind", root:542), no grant beyond the budget row, no witness.
- Why DETAIL: the plan already has the two grounds and the closed effect vocabulary; the fix applies them at the unit of the effect. It changes one gate rule and the mapping of O5, C4 and V17. If the owner reads "closed registry" as "closed content", this is the one finding that could be argued up.
- Change: (a) give each effect of the closed vocabulary its own admission ground (`capture`, `prevent`, `stop`, `dataTransfer`, activation-gated: detected; `setVar`, `scrollBy`, `setValue`, `focus`, `after`: a witness or verdict per use); (b) make `pan` outputs separate Rust-side decisions by default (css-var via `set`, scroll via `transaction`), admitted into the module only against a `fail` verdict; (c) say that a new `pan`/`settle` parameter value is a new variant with its own row.
- Subsumes: root:372's "if R passes, `pan`'s `css-var` output shrinks" (made mechanical) and the single-user-variant creep of risk 3 (root:627).

### 16. DETAIL, minor. Two "never" settings.

root:315 and :640 say the stuck-channel spec runs at `__dxBridgeDelayFrames = "never"` (bridge-latency harness, which l30 already implements: `bridge.ts`, `transport.rs`, "`window.__dxBridgeDelayFrames = \"never\"` for fail-open specs" in l30's row 434 text). 442:33 says the desktop-cadence build's "`N = never` ... replaces the planned stuck-channel mode and drives the gate fail-open and WCAG 2.1.2 specs". The lease and fail-open specs then depend on either a built harness or one that does not exist. Pick one and say which spec runs where.

### 17. DETAIL, minor. Residue doc: two stale admission/fallback statements.

residue:62 "It is admitted by its effect (`sameFrameWrite`)", against root:280, :303 and js-interop:98 (`sameFrameWrite` is admitted only by a witness; the round-3 DESIGN change). residue:253 "C5 (measured, TypeScript fallback)" against root:51, residue:20 and :179 (a named behaviour admitted by the `fail` verdict; "no allowlisted-module category").

### 18. DETAIL, minor. "hotkey for L1 and L2".

434:66 "3.DJ with `hotkey` for L1 and L2" and r3:42 "3.DJ with `hotkey` for L1/L2". root:570 and :466: `hotkey` is for L6; L1 and L2 are rsx handlers.

### 19. DETAIL, minor. Counts that no longer reconcile.

- root:477 and r3:42: "lanes that remove almost no JavaScript ... about 72 hours (27%)". Summing the current tables for the same lanes: 0.1 (10) + 0.2 (3) + 0.5 (3) + 3.FZ (10-12) + kit remainder (7) + 3.HV (10) + 3.SL (8) = 51-53 h, 66 h with row 442's 14 h. The 72 is r3's figure with the older 19 h "3.0b"; and 442 sits in the numerator but outside the 235-270 (root:472, :619) in the denominator.
- root:61 reads the 35-40 h construct and the 235-270 h port as additive ("separates the construct ... from the port"); root:619 says the 235-270 includes the construct's 39.
- root:612: "17,000 - 12,207 = 4,793 s". b28 `playwright/durations.json` meta `total_seconds` is 12,213 (l30: 12,217).

### 20. DETAIL, minor. Terms used two ways.

- **A, B, C**: layers (root:166-174: A platform, B registry, C Rust-in-page, plus G and R) and options (root:483-487 and 434:66: A port all, B construct first, C construct only). "What B changes" (root:488) and "B is the default" sit in the same document as "B closed registry".
- **R**: the layer "Rust over the bridge" and "lane R (the JS-volume ratchet)" (root:561).
- **tier**: engine tier set by the boot probe (root:225, :325, :340 "Apple-desktop fallback tier") and the "two tiers" of paint cost (root:331, placement-critical or decoration).
- **command**: the level-triggered kind (root:269, js-interop:82) and "one-shot commands (focus a cell ...) stay bridge calls" (root:276, js-interop:84) inside the paragraph that defines the kind; l30 also calls its `{id, ops}` messages commands.
- **epoch and generation**: root:263 and js-interop:74 ("accepted by epoch") against root:271 and js-interop:42 (`gen`, "the reconciler applies the same rule"); l30 has a typed `Epoch`. Say whether `gen` is the epoch.
- **legacy**: legacy scripts in the baseline (js-interop:199), frozen arms in `ts/legacy` behind `legacy-impl` (root:452), `Program::Legacy` (l30).
- **zero**: "step 0" (the latency distribution, root:638, 443:51, against row 434's steps 1-3), "phase 0", "lane 3.0", item "0.0". root:638 says "Step 0, before anything else" while item 0.5 is last in phase 0 (root:574).
- **owner**: the repo owner, the owning Rust scope ("Lifetime is the owner's", root:316), the owning element (`owner.isConnected`, same paragraph) and a file's lane owner.
- **undefined shorthand**: "class (1)" (root:108, :126, :502; the current docs define only audit classes a-d) and "B2, B3 and B5" (root:442; defined only in round-1 reports).
- Change: rename the options "scope 1/2/3" (or "port-all / construct / on-touch"), the layer R "Rust-over-bridge", the two tiers "engine tier" and "paint tier", one-shot commands "one-shot calls".

### 21. DETAIL, minor. A referenced file that does not exist, and round summaries that carry reversed decisions.

- 442:35 "`dev-docs/desktop-pass.md` holds the checklist" (and root:296, :634): the file does not exist in b28 or l30; 442:37 and the README map entry are "to do". Say "will hold".
- r1 and r2 SYNTHESIS carry text the plan later reversed, with no superseded marker (only r3 states what changed from r2): r1:22 and :27 (C5 becomes an "allowlisted TS module", "an expiring allowlist entry", "deleted at the expiry deploy", against root:456 "no date-keyed expiry" and "no allowlisted-module category"), r1:20 (gate cleared in `beforetoggle`, `animation: none`), r2:35 (generation page-owned, `:not([data-dx-anchored])`, clear in "the `show` verb's own JS turn"), r2:41 (js-interop rewrite, done). Add a one-line banner to each: "Superseded in part; the root doc 2.2 and 2.3 decide."

### 22. DETAIL, major. js-interop.md gives no path for work that needs a mechanism that is not in the tree.

- Evidence (V): js-interop:9-10 says a planned mechanism "does not exist in the tree yet; the rule it states is the design, and until the mechanism lands the review (and the baseline) is the only thing enforcing it". Item 4 says to report what is missing. The page never says what to do today for the three common needs: use the existing hook (`use_popover_sync`-style hooks in `top_layer/popover_sync.rs`, `use_global_escape_listener` in `lib.rs`, `use_anchor_position_fallback`) rather than copy its script; whether an existing hook counts as "porting" (it does not change the baseline); or which edits the legacy-literal freeze will turn red (planned, root:557). The page's own last paragraph (js-interop:235) says the neighbour you copy teaches the way; here the neighbour is a Rust-string eval, which `check-js-volume` will fail only after the work is done.
- Change: add a "today" column to the "I want to ..." table (existing hook to call, or "stop and report: the behaviour is not built"), kept in step with the Status table.

### 23. DETAIL, major. A new popover open path: the page contradicts itself.

- js-interop:42: "Do not call `showPopover`/`showModal` from an effect" (publish a volatile `data-dx-*` attribute; the reconciler applies it). js-interop:46: "call `showPopover` once ... `ElementExt` (`el.show_popover()`, ...)". js-interop:178: a plain `showPopover` or `showModal` verb "on a reconciled kind" is forbidden. "Reconciled kind" is never defined for the agent; row 46 is stated unconditionally. Agent A reads :42 and waits for a reconciler that is not built; agent B reads :46 and writes `el.show_popover()` from an effect, which root:273 forbids.
- Walk-through in section C.
- Change: delete "showPopover" from row 46 (keep `scroll_into_view`, `set_selection_range`, `set_var`), define "reconciled kind" (popover, modal, `indeterminate`, `defaultChecked`) in the command paragraph, and say that a one-off `show_popover()` is legal only for an element that is not one of those.

### 24. DETAIL, minor. A new hotkey on a dialog: two traps the page does not mention.

- Dioxus's `stop_propagation()` never reaches the DOM on any renderer (root:83: it affects vdom bubbling only), so a dialog `onkeydown` that calls it cannot stop the document-level `hotkey` (Escape, F6) or another behaviour from also seeing the key. js-interop:86 says an rsx handler takes exactly one commit-time effect, `prevent_default`, but never says `stop_propagation` is a no-op toward page listeners.
- Nested dialogs: root:293 "the top-of-stack decision for Escape stays in Rust" and L6's top-layer check. js-interop says nothing about a hotkey on one of several stacked layers.
- Change: two sentences in js-interop's "rsx handler" paragraph.

### 25. DETAIL, major. A new drag gesture: the page sends the agent to a behaviour that does not exist, with unstated ordering and CSS rules.

- js-interop:39 (capture → `pan`, "never an async verb") and :45 (stream → `use_sample`, "never a vdom handler") both apply to one drag; the closure rule (js-interop:71-73) says the recogniser stays on the page and the outputs are separate decisions, while root:298 places the outputs in `pan` (finding 15). The page does not say which wins.
- Start, move and end must share one channel: root:518 and :138 say a streamed `pointermove` can be processed after the `pointerup` that followed it on desktop, "mixing a bridge move with a vdom `onpointerup` is not [ordered]". js-interop:67-70 states only the general commutativity condition; an agent can write rsx `onpointerdown`/`onpointerup` plus bridge moves.
- `touch-action` (CSS, root:372 "touch-action stays CSS", root:298 `pan` publishes it) is absent from js-interop.
- Change: one "drag" row in the "I want to ..." table: rsx `onpointerdown` only if the whole gesture is on the bridge (L30's shape), `touch-action` in CSS, capture or cancel needed means `pan` (planned, report), outputs in Rust over `set` or `transaction` unless a `fail` verdict says otherwise.

---

## Appendix A. The four landed/planned claims of js-interop.md's Status table

| claim (js-interop.md:12-15) | b28 (`/home/user/wt/b28`) | l30 (`/home/user/wt/l30`) | verdict |
|---|---|---|---|
| Gates landed: `check-js-volume` (shrink-only, merge-base compare, baseline note for a raise), `check-js-bundle`, `check-js-listeners`, `check-uncleared-intervals`, `check-blocking-scroll-listeners`, `check-theme-mode-writer`, `check-open-focus` | All seven scripts exist and are registered (`scripts/run-gates.sh:99-106,153`). The merge-base compare and `# raised:` note are in `scripts/lib/ratchet.py:23-27,201-218,241` and print under `RATCHET RAISES`. | same, plus `check-bridge-behaviours` registered (`run-gates.sh` +1) | TRUE as listed. The over-claims are in the "forbidden" table, not in this cell (finding 3). |
| Gates planned: grant rule, `adds:` rule, `check-hot-events`, `check-bridge-behaviours` (effect detection, witness record, single-member-call verb rows, lease rule), clippy `disallowed_methods` ban, `Program`-typed transport | None exists: `clippy.toml` has only `allow-unwrap-in-tests`; no `check-hot-events`, no `check-bridge-behaviours`, no `js-grant`/`adds:` parsing; `Program` absent. | `check-bridge-behaviours.sh` (551 lines, 53 mutations) exists with parity, purity, budgets, one place for cancel and frames, closed surface, eval-sink-in-`bridge/`, no gesture verbs. It has NO effect detection, witness record, lease rule, single-member-call row check or `FORBIDDEN_VERBS`. `Program` enum exists (`transport.rs:124`). No clippy ban, no `check-hot-events`, no grant rule. | TRUE for b28; UNDERSTATED for l30: the cell should say "partly built on lane 3.0's branch" and list what is missing there (finding 2, 9). |
| Runtime landed: `PREPAINT_JS` and the modality writer | `preview/src/theme.rs:143`, `primitives/src/modality.rs:70` | same | TRUE. js-interop:51 ("allowlisted with its reason") is not (finding 3). |
| Runtime planned: the bridge (`primitives/src/bridge/`, "not in this tree"), the closed registry, the document-scoped reconciler, `stickToEnd` | `primitives/src/bridge/` absent | bridge, registry with one test fixture (`Behaviour::Fixture`), leases, epochs present; reconciler is a comment seam (`pageObserver`); no `stickToEnd` | TRUE. js-interop:103 "today none exists but a test fixture" is true only on l30. |

## Appendix B. Placement and owner per audit id (summary)

Source: `research/js-audit-2026-10-25.md` (103 ids: T 11, L 30, S 19, O 5, W 8, C 6, V 23, P0 1; there is no T3).

| set | ids | placement in root | owner in root 5.4 | result |
|---|---|---|---|---|
| class (d), 22 | T8-T11, L30, S1, S11, S14, O3, O5, W1, W3, W5, W7, W8, C1-C5, V11, V19 | root section 3, one row each | one lane each; V11 none (seam) | OK except V11 |
| step-2 leftovers, 9 | T1, T2, L17 (Select), L18, L20, L22, S4, V6, W2 | root 3.2 | 3.PL, 3.OV, 3.CA, 3.FC, 3.PV, 3.VS | OK |
| routed (b), 27 | L1-L11, L16, L21, L23, L25, L27, S8, S9, O1, W4, W6, T4-T7, T12, V17, V22 | root 3.1 or registry rows | one lane each, except L1/L2 (0.3 and 3.OV) | L1/L2 have two owners (finding 12) |
| (b) with owner, no placement, 27 | L13, L24, L26, L28, L29, S7, S10, S12, S13, S15-S18, C6, V5, V7-V10, V12-V14, V18, V20, V21, V23, P0 | none by id (root:393, :617 say "assigned") | root:617 | placement missing (finding 12) |
| class (c), 5 | L15, V1, V2, V3, V4 | "class (c) stays" (root:440, no ids for V2/V3) | none | no owner; generation from one `.ts` has no lane |
| moved in step 2, 13 | L12, L14, L17 (not Select), L19, S2, S3, S5, S6, S19, O2, O4, V15, V16 | S2/S3 only (3.HV) | n/a | fine; no ledger |

File-ownership disjointness (root:566-608, parsed): the table repeats a file only for `scripts/check-js-volume.*` (0.1, 0.2), `bridge/behaviours.rs` (0.3, 0.4, ordered) and `ts/perf/**` (0.5, kit; same lane, split by half). The prose "File sharing" paragraph (root:594) serialises `context_menu.rs`, `lib.rs`, `select/components/list.rs`, `chart.rs`, `drag_and_drop_list.rs`, `virtual_*`, `popover_sync.rs`, `transport.rs`, `pan.ts`, `carousel/drag.rs`. It omits 3.FZ before 3.CA, 3.MS and 3.WB on `carousel/{scroll,drag,wheel,mod}.rs` and `message_scroller.rs` (root:452 and :598 say "before any of them"), and the unowned files of finding 9.

## Appendix C. Would an agent reading only `js-interop.md` and CLAUDE.md place new behaviour correctly?

CLAUDE.md (b28:51-53) sends the agent to js-interop.md and states "browser APIs only through the bridge (`primitives/src/bridge/`)" with no "planned".

1. **A new hotkey on a dialog (for example Mod+Enter submits its form).**
   - "I want to" row 1 (js-interop:37): "react to a ... key ... event on an element I render: the Dioxus `on*` prop". The deadline table (js-interop:60): default action, Dioxus delivers `KeyboardData`'s key and modifiers, the event is small, so an rsx `onkeydown` on the `<dialog>`. Row 2 (js-interop:38, `hotkey`) applies only to a window, document or capture target. Correct placement, no JS, nothing to report.
   - If the key must work with focus outside the dialog (a global shortcut), row 2 sends the agent to `hotkey`, which js-interop:14-15 and :103 mark planned, so item 4 (js-interop:30-31) says report.
   - Gaps: the `stop_propagation` no-op toward page listeners and the nested-layer top-of-stack rule (finding 24); today's neighbour is `use_global_escape_listener`'s eval, which the page does not name as legacy (finding 22).
   - Verdict: right place.
2. **A new drag gesture (for example a splitter or reorder handle).**
   - Rows 3 and 6 (js-interop:39, :45) both match. If it needs capture or a cancel, the page says `pan`, a behaviour that needs a backlog row and does not exist (planned). If it needs neither, `use_sample` plus rsx `onpointerdown`, which js-interop:67-70 does not forbid and root:518 says is unordered on desktop when `pointerup` is a vdom event. `touch-action` is absent.
   - The per-move output (`set_var`, `scrollBy`) is placed by the closure rule (condition 2, js-interop:71-73) in Rust, and by the registry (root:298) inside `pan`. A new output is a parameter of an existing kind: no new row, no witness (finding 15).
   - Verdict: lands in the right layer (`pan` or R), but with no interim path, an ordering trap and an ambiguous output placement (findings 15, 22, 25).
3. **A new popover open path (a new component with a panel).**
   - Row 4 (js-interop:42): publish a volatile `data-dx-*` attribute, "do not call `showPopover` from an effect"; the reconciler is planned, absent. Row 5 (js-interop:46): "call `showPopover` once ... `el.show_popover()`". js-interop:178 forbids a plain `showPopover` "on a reconciled kind", undefined. Placement: the page's own rows disagree; with the reconciler absent, the working path is the existing `use_popover_sync`-style hook, which the page never names.
   - Placement of the panel (js-interop:50): "CSS anchor positioning first, behind engine floors and the boot probe; Rust placement behind a paint-safe gate". Neither the probe nor the gate exists; the working path is the existing `use_anchor_position_fallback`, also unnamed.
   - Verdict: wrong or stalled without outside knowledge (findings 22, 23).

## Attacks that failed

- "The numbers do not add up." The 22 class-(d) rows reconcile to the audit exactly (A 1,638, named behaviours 880, R 1,153, G 172, spliced 39, seam 12 = 3,894; V). Phase 0 sums to 39 h (10 + 3 + 17 + 6 + 3), as stated at root:574. The 13 `preventDefault` sites count 13 in root:108-124 and in :502.
- "The registries disagree." js-interop:103-107, root:34, root:289-306 and 434:57 list the same nine behaviours plus the same conditional four and the reserved `dialogKeyForward`; r3's nits (`reconcile` missing from two lists) are fixed.
- "The file:line citations are stale." 31 spot-checks of root section 1.3 and 3 citations against b28 (`lib.rs:367,781,848`, `drawer.rs:525`, `virtual/bridge.rs:293`, `top_layer/popover_sync.rs:125,146,270,297`, `carousel/*`, `input_otp.rs:367`, `modality.rs:87`, ...) all land on the block the table names.
- "There are dead links." A scripted check of every relative markdown link in the nine documents found none; every `research/` and `backlog/` target exists. (`desktop-pass.md` is a prose reference, finding 21.)
- "Round 3's coherence leftovers persist." `3.QH`, `use_resize`, the legacy-removal stamp key (`<port>.pass`), the `focus` verb versus L20, the row 443 kit counted twice, and the allowlisted-module category are all gone or consistent across root:456, residue:272, 443:71 and js-interop:203.
- "The root rule or the deadline rows contradict a document." None does; the three layers I could test against the code (the verb removal, the lease, the transport) are stale in the prose, not in the design.
