# Board round 2, lens `execution`: can the hardened plan be executed in this repo and environment?

**Verdict: HOLDS WITH CHANGES**

**DESIGN findings: 0, DETAIL findings: 18**

Summary
- No finding changes the core construct. The root rule, the three-way placement rule, the closed bridge with one eval sink, and A/G/R as layers all survive the execution attack. The lane plan, the landed bridge surface and the environment do not yet match them, and every finding below is a lane boundary, a missing table row, an estimate, a gate rollout or a schedule.
- The closest thing to DESIGN is F2. The landed behaviour skeleton fits only `hotkey` unchanged, so about 985 of the roughly 1,000 behaviour JS lines have no place in it. I rate it DETAIL because the construct (named, declared, scoped, epoch, fail-open) survives and only the effect vocabulary and per-install state grow. If the owner reads "pure `decide()` proven by vectors" as the defining property, F2 becomes DESIGN for `pan`, `settle`, `scrollStep` and `scrollLock`.
- Batch 2 as drawn (5 lanes) is lock-bound, not agent-bound. Its spec passes alone are about 251 min per round of the five lanes (F5). The disk allows one lane build copy at a time (F6). "Five lanes" means five editors and one builder.
- Two lane-ownership holes would stall the plan: G needs `popover_sync.rs`, which 3.PL may not touch (F3), and nobody owns freezing the C2-C4 scripts or building the switch (F4).
- Nearly every later lane needs bridge rows 3.0 does not have (F1). Five lanes would edit the same bridge files at once.
- 155 agent-hours is a plausible effort sum. It is not a wall-clock number: step 2 ran 1.1-2.2 times its estimate (F7). Rows 442, 443 and a 3.0b are outside the 155 and gate the end of the plan.
- Proposed schedule: four phases, 4 lanes per phase, one deploy per phase (section "Schedule").

---

## Numbered findings

### F1. DETAIL, major. 3.0's closed tables lack rows that about 16 later (b) rows need, and no lane owns adding them
**Claim attacked.** Root doc 2.2 and 5.2: "a new verb (data, one table line) is not [a behaviour]" and "the 3.0 bridge's surface is enough for later lanes". Row 434's "Step 3: lane 3.0 builds the bridge" lists the verbs.

**Evidence (V, `/home/user/wt/l30`, branch `lane/js-bridge-3-0`, `primitives/src/ts/bridge.ts`).**
- The `EVENTS` table (`:102-151`) has no `reset` (L26), `beforematch` (L24, W4, V7; the plan's own example of "an event Dioxus lacks"), `scrollend` (C2, C3, `scrollStep`), `dragend` (the `dropAnywhere` behaviour names it) or `invalid` (V20). `grep -c` for each is 0.
- `METHODS` (`:221-257`) has 15 entries. None is `reportValidity` or `requestSubmit` (S18), `dispatchEvent` (V7), or `getAnimations` (L5, which the plan routes to "bridge `call`").
- `PROPS` (`:273+`) has no image `complete`/`naturalWidth` (S10), `checkVisibility` (S16), `validity` (S17), `location.search` or `localStorage` (row 443's `?dx-impl` read), or `querySelectorAll` order (S7).
- `set` only accepts `--dx-*` names (`:460`). S8 (`indeterminate`), S9 (`defaultChecked`) and V10 (`documentElement.dataset.hydrated`) therefore have no verb.
- `ObserveOptions` (`session.rs:76-104`) has no `publish`, yet S14 is planned as "an `observe(..., publish: var)` option" (root 2.2, 3).
- V5 (cookie, BroadcastChannel), V8 (`postMessage`), V9 (`fetch`) and V14 (clipboard) have no verb. 5.2 says clipboard writes are "not verbs", and the registry has no clipboard behaviour. Nobody has said these take an allowlist entry, which the plan calls "the owner's decision".

Affected rows: L5, L24, L26, S7, S8, S9, S10, S14, S16, S17, S18, V5, V7, V8, V9, V10, V14, W4. That is 18 of the 43 unassigned (b) rows plus the 443 plumbing.

**The collision (V).** Each added row edits `ts/bridge.ts` (one file, 1,063 lines), `bridge/events.rs` or `bridge/mod.rs`, the regenerated `js/bridge.js`, the manifest and `behaviours.gen.ts` (its `EventName` union is generated from `EventKind`). 3.OV, 3.FC, 3.VS and 3.OB run concurrently in batch 2, and none lists those files as owned. Each new behaviour variant also edits the one `bridge/behaviours.rs` enum (`info()`, `declared()`, `module()`, `samples()` arms) and the generated file. 3.OV, 3.FC and 3.VS each add one in batch 2.

**Change.** Do a **3.0b verb sweep before batch 2**, from the list above, so lanes consume rows and never add them.
- Pre-declare the seven behaviour variants as skeletons (enum arm, empty TS module, sample vector). A behaviour lane then fills in its TS module and vectors only.
- Anything not on the list goes to a 3.0b queue that the main loop serves at merge time.
- Add the allowlist decision for V5, V9 and V14 (preview-only, reasoned) to the plan now.

**Subsumes.** The 3.0 surface question in the brief; part of F8's file-ownership list; F16.

### F2. DETAIL, major (borderline DESIGN). The landed behaviour adapter cannot express six of the eight behaviours
**Claim attacked.** Root 2.2: "a pure `decide(config, snapshot, event) -> effects` plus a thin DOM adapter that builds the snapshot and applies the effects"; vectors prove the decisions.

**Evidence (V, `ts/bridge.ts:658-737`, `ts/types/bridge.d.ts`, `bridge/behaviours.rs:72-170`).**
- `Declared` carries ONE `event` per install, and `hubAdd(el, d.event, ...)` registers one listener.
- `DxDecision` is `{prevent, stop, attrs (data-*/aria- on the host), send}`.
- The snapshot is the host's attribute values (`reads`).
- Missing for the planned modules:
  - `scrollLock` (L11, 85 lines): four events (wheel, touchmove, touchstart, keydown), `touchstart` coordinates remembered across events, an RTL-aware ancestor walk from `event.target`.
  - `otpFilter`: two events (`beforeinput` and `input`), a value rewrite, `setSelectionRange`.
  - `dropAnywhere`: three events plus `dataTransfer.dropEffect`.
  - `pan` (O5 74 + C4 545 = 619 lines): pointer capture, `scrollBy` followed by a layout read to learn what the scroller refused, CSS-variable writes, a rAF release animation.
  - `settle` (W3/W5/W7, 122 lines) and `scrollStep` (C2, 114 lines): rAF loops, `scrollend` plus a timeout, snap suspension.
- Only `hotkey` (L6 10 + O1 41 + V22 15 audit lines) fits unchanged. By audit lines about 985 of the roughly 1,000 behaviour lines need the extension.
- The gate (5.3 item 4) bans `requestAnimationFrame`, `setTimeout` and `preventDefault` in `ts/` outside allowlisted files. `settle`, `pan` and `scrollStep` will all need entries. Without a typed place for ticks and timers, the vectors would test serde field names and little else, the failure mode 2.2 itself warns about.

**Change (3.0b).** Extend the skeleton without changing the construct:
- A list of `{target, event, capture, passive}` per behaviour, instead of one event.
- A per-install memory object passed to `decide`.
- Snapshot readers selected by closed name, for example `scrollableAncestor(axis, dir)`.
- A closed effect vocabulary: `capture(pointerId)`, `release`, `scrollBy`, `setVar`, `setValue+selection`, `focus`, `after(frame|ms)` (the tick that re-enters `decide`).
- State plainly that vectors prove `decide`, and the DOM effects are proven by specs.

**Subsumes.** Part of the "does the 3.0 bridge surface suffice" question; F1's skeleton step.

### F3. DETAIL, major. Lane 3.PL cannot deliver G without files that 3.OV owns in the same batch
**Claim attacked.** Lane table: 3.PL "must not touch `top_layer/popover_sync.rs`, `lib.rs`" and 3.OV owns them. Root 2.3 items 1, 4, 7 (clear in `beforetoggle`, generation token "from `open`", "show plus measure is one transaction").

**Evidence (V, b28).**
- The open path lives in `top_layer/popover_sync.rs`. `:147` is the `showPopover()` effect, `:297` is the one eval that composes `showPopover` with `dxOpenFocus` and the modality writer, and `:125`/`:270` are the `toggle` listeners.
- The modal twin is `use_dialog_open_driver` in `lib.rs`.
- The `dxOpenFocus` composition lives in `modality.rs`.
- G's three requirements (a synchronous `beforetoggle` clear, a generation token minted at the open call, and a show-plus-measure transaction returning rects) all sit in those call sites.
- `use_anchor_position_fallback` (`anchor_fallback.rs:440`) cannot start the gate. `toggle` is a post-paint task (root 1.4), and at `beforetoggle` the popover is still `display: none`, so nothing is measurable there. The pre-paint trigger has to be the open call itself.

**Change.** Put T4-T7 (`popover_sync.rs`, 310 lines) into 3.PL, and give 3.OV `lib.rs`, `menu_sub.rs`, `navigation_menu.rs` and the rest of `modality.rs`. `OPEN_FOCUS_JS` keeps its name, so T7's composition does not break. The lanes then share no file. Cost: about +4 h on 3.PL, about -3 h on 3.OV. The alternative is to serialise 3.OV before 3.PL.

**Subsumes.** The "disjoint file ownership" check for the two longest lanes.

### F4. DETAIL, major. Nobody owns the legacy freeze or the switch for C2-C4, and 3.WB needs a file it may not touch
**Claim attacked.** Root section 4 ("freeze first, port second"; the freeze covers "message scroller, C5, and the C2/C3/C4 scripts with their macros, since they are spliced") against the lane table and row 443.

**Evidence (V).**
- Row 443: the switch is "for W8 and C5 only (C1 inherits its users' arm)". 443 also says "the per-port legacy freeze is inside 3.MS and 3.WB".
- 3.CA (batch 4, before the "last" lanes) ports C2/C3/C4. Its own file list excludes `js_macros.rs` and `ts/legacy`. 3.WB owns only `wheel.rs` and `wheel_band.rs` and "must not touch the other `carousel/*` files".
- So the C2/C3/C4 scripts (114 + 411 + 545 = 1,070 lines, plus the macros) get frozen by nobody. 3.CA runs first and ports them with no JS arm.
- The owner's third measurement (root 5.6, "carousel macros (C1) and its users": "settle time and spring curve within one frame and 2 px of the JS arm") needs exactly that arm.
- The bounce hand-off needs both sides edited. `wheel.rs:563` says a mouse press "hands the element's transform to `CAROUSEL_DRAG_JS`", and `drag.rs:215` writes `el.style.transform`. The plan wants "one shared `--dx-band` channel" and a handoff test, but 3.WB may not edit `drag.rs`, and 3.CA is already merged.
- "Byte for byte" is not literally possible. The scripts rely on injected globals (`listen`, `cleanup`, `dioxus.recv()`) and a top-level `return`. As `.ts` they must be wrapped in a function.

**Change.** Add one lane **3.FZ (freeze)** in phase 0, before any port. It does the following:
1. Moves `message_scroller.js`, C2-C5 and `js_macros` into `ts/legacy/*.ts`, wrapped and behaviour-neutral, proven by the existing specs.
2. Adds `legacy-impl` and the `?dx-impl` plumbing with the reading verb from F1.
3. Excludes `ts/legacy/**` from `primitives/tsconfig.json`. It currently has `strict` and `include: src/ts/**/*.ts`, so an untyped 1,065-line file would fail `tsc`. Alternatively `// @ts-nocheck`.

Then 3.CA, 3.WB and 3.MS each add only a Rust arm. The carousel is one switch (C2-C5), measured once. 3.WB's list gains the bounce lines of `drag.rs`, or 3.CA publishes `--dx-band` before 3.WB starts.

**Subsumes.** Row 443's plumbing item; the contradictory text about which blocks get a legacy arm; F8's ordering of the owner measurements.

### F5. DETAIL, major. Batch 2 is lock-bound, and the two lowest-risk lanes cost the most Playwright time
**Claim attacked.** Lane table: "3.PL, 3.OV, 3.FC, 3.VS and 3.OB run concurrently (five lanes)" against `CLAUDE.md`'s "4-5 per batch" and lane-rules "While Playwright runs, set LANE_SLOTS=1".

**Evidence.**
- (V) I ran `node playwright/tools/select.mjs --files <the lane's files>` read-only in b28 and summed `playwright/durations.json` (debug worker seconds; wall is about seconds / 2 at 2 workers):

| lane | files fed to `select.mjs` | specs | debug wall per `test:changed` pass |
|---|---|---:|---:|
| 3.PL | `anchor_fallback`, `scroll_clamp`, select `list.rs`, `context_menu.rs` | 97 | 42 min |
| 3.OV | `lib.rs` (35 specs, 33 min); `modality.rs` alone | 139 | 52 min |
| 3.FC | `collection.rs`, `input_otp`, `checkbox`, `avatar`, `questionnaire`, `collapsible`, `drag_and_drop_list` | 150 | 67 min |
| 3.VS | `virtual/*`, `virtual_grid.rs` | 49 | 26 min |
| 3.OB | `activity.rs`, `keyboard_inset`, `scroll_area`, `chart.rs` | 141 | 64 min |
| 3.SL | `scroll_lock.rs`, `context_menu.rs` | 71 | 35 min |
| 3.GS | `drawer.rs`, `pointer.rs`, `toast.rs` | 70 | 31 min |
| 3.CA / 3.WB | carousel files | 19 | 19 min |

- (I, from V figures) One pass of all five batch-2 lanes is about 251 min of exclusive `pw.lock`, and `LANE_SLOTS=1` blocks every cargo build meanwhile. Three passes per lane is 12.6 h, plus about 3.4 h of lane builds (13.5 min each: `lane-ssg` 9 min + the `web` clippy arm 4.5 min), plus gates 0.7 h and the deploy (677 s release build plus the 52 min suite) 1.05 h. Total is about 17-18 h of exclusive lock, against 14 h mean and 24 h longest lane. There is no slack for a red.
- 3.FC and 3.OB carry the smallest blocks (S11 25 lines, L25 11, S14 31, S15 110) but touch `collection.rs` and `activity.rs`, shared primitives that pull in the broad set.

**Change.**
- Lanes run their component's own specs. The broad set is the deploy's release suite, not each lane's. `select.mjs --files` already does this when the lane lists only its own files.
- Move S7 (`collection.rs`) and S15 (`activity.rs`) out of the early batch, into one small lane run once.
- Four lanes per phase, not five.

**Subsumes.** F6's batch-size question; the "what batch size fits" part of the brief.

### F6. DETAIL, major. Disk allows one lane build copy at a time, and the plan assumes five lanes with `dx serve`
**Claim attacked.** `CLAUDE.md` fast loop ("each agent: own warm target copy, own port") and 4-5 lanes per batch.

**Evidence (V, measured in this container).**
- `df` shows 1.2 GB free at 98%.
- `/home/user/tgt` is 22 GB. The permanent trees are `template` 3.8, `release-template` 3.6, `deploy` 3.6 and `check` 1.7, which is 12.7 GB. Two finished lanes' trees are `lane-2a-tgt` 5.1 and `lane-2d-rel` 3.6. Pruning them gives about 10 GB, which matches the brief's 11-13 GB.
- `lane-slot-target.sh` refuses a copy unless free disk is at least size x 1.2 + 2 GB. That is 6.6 GB for a 3.8 GB copy. With about 10 GB free exactly one slot-1 copy fits, and the second slot's copy is refused. `registry-install-smoke` needs about 6 GB more. It cannot run beside a lane copy.
- Row 442's desktop-cadence build patches `dioxus-web`. That is a different dependency graph and so a fourth 3.6 GB-plus release tree (cold release build 37-50 min per lane-rules), unbudgeted in 442.

**Change.**
- Lanes skip `dx serve`. lane-rules permit this "with little free disk".
- Run `lane-target.sh prune` before each phase.
- The main loop runs the install smoke between phases, with no lane copy alive.
- Build 442's harness tree in a window with no lane copy, starting from the `deploy` tree.

**Subsumes.** The environment half of item 3.

### F7. DETAIL, major. 155 agent-hours is an effort sum, not a schedule, and several lanes are low
**Claim attacked.** Row 434: "about 155 agent-hours after 3.0 and R".

**Calibration (V, git history).**
- Step 2 was sized at "about 26 agent-hours in 5 lanes", about 5 h each. The audit finished 2026-10-25 23:06.
- Time to a merge-ready commit: 2B 04:40 (5.5 h), 2C 06:04 (7 h), 2A 09:56 (10.8 h), 2E 09:52 (10.7 h). 2D is still open at 10:20 (11 h or more).
- Wall time ran 1.1 to 2.2 times the estimate, with the lock queue inside it.

**My sizing of five lanes (I).** Lines are audit string extents. Spec passes are from F5. Hours are my judgement, not measurements.

| lane | plan | mine | basis |
|---|---:|---:|---|
| 3.DJ | 3 | 5 | 16 sites in 8 files; plus a new gate that must read `#[cfg]` items (the tokenizer in `comment-scan.py` is not an item parser) with a mutation self-test; a 35-42 min spec pass |
| 3.OV | 17 | 26-32 | 12 blocks in `lib.rs` (2,075 lines, used by every popup); the first real behaviour (`hotkey` with ack and lease); L16's 114 lines under `check-open-focus` and its three copies; the conditional `dialogKeyForward` fallback; 52 min per pass |
| 3.CA | 14 | 24-32 | 1,070 JS lines in three interlocked blocks (C4's coasting physics and rubber band); 14 carousel specs; `ssr_tests.rs` (1,285 lines) tied to the macros; the `pan` variant that F8 shows is unassigned |
| 3.MS | 20 | 30-45 | A 1,065-line controller with a mode machine, 7 new scenarios, the freeze and switch (moved to 3.FZ by F4), a 26 min pass. This is the most uncertain lane; give it a time box with the TS-module exit |
| 3.WB | 12 | 12-16 | The best specified: a pure classifier, an oracle already exists (`playwright/fixtures/carousel-wheel-recordings.json`, `wheel-replay.ts`, `carousel-wheel-replay.spec.ts`), and only the property test and the phase fix are new |

- Outside the 155 yet on the path: 3.0b (about 8-10 h, F1/F2), 3.FZ (about 10-12 h, F4), row 443 (about 10 h; it must precede 3.MS and 3.WB but sits in no batch), and row 442 (about 14 h).
- Critical path in the original plan: 3 + 24 + 13 + 14 + 20 + 12 + 2 = 88 agent-hours. At the observed ratio of about 2 that is 7-8 days of continuous work, plus three owner round trips.

**Change.** Say in row 434 that 155 h is effort, schedule at about 2 times, and add 3.0b, 3.FZ, 443 and 442 as explicit lanes.

**Subsumes.** Item 2 of the brief.

### F8. DETAIL, major. Dependency order: 443 and 442 sit in no batch, 3.VS waits on an open lane, and `pan` is built in one lane and extended in another
**Evidence (V/I).**
- Row 443: "the `?dx-impl` switch plumbing is needed before the first of 3.MS and 3.WB". Row 442's stamp is the stated condition for marking any G or R mapping done. Neither appears in the lane table.
- 3.VS "starts after 2D merges, otherwise W1 stays with 2D". 2D has been open for more than 11 h (branch `lane/js-bridge-2d`, no commit of its own, five modified files), so batch 2's start depends on an unscheduled lane.
- `pan` is "built by 3.GS" with the `css-var` output only. The `scroll-by-with-refusal` variant is code in `ts/behaviours/pan.ts`, which 3.CA implements in batch 4. 3.CA's file list does not include that file.
- The three-way ordering hotkey (3.OV) to consumers (3.GS, 3.PV), `settle` (3.VS) to 3.MS, and `scrollLock` (3.SL) to the context-menu consumers is correct as written. Pairwise file disjointness among 3.FC, 3.VS, 3.OB, 3.SL, 3.GS, 3.PV and 3.QH also holds. I found no shared file.

**Change.** The schedule below places 442 and 443 explicitly. Name 2D's merge as a precondition, or move W1 to 3.GS-style "R" in a lane that is free. Add `ts/behaviours/pan.ts` to 3.CA's list and `ts/behaviours/**` to 3.0b's skeleton step.

### F9. DETAIL, minor. G's selector as written gates every popup and every dialog, which contradicts the two-tier rule
**Evidence (V, root 2.3).** Item 1: `:popover-open:not([data-dx-placed]), dialog[open]:not([data-dx-placed])` is pending, and "every open path is gated by default" because "pending needs no write". Items 5 and 6: only "where Rust owns the position" is gated, and "where A places the panel, only the Rust-dependent part is gated". If the selector matches all popups, then (a) conforming engines still wait for Rust to write `data-dx-placed`, which is the two to four frames item 5 says to avoid, (b) a modal `<dialog>` that is centred by CSS never gets placed, and (c) a stuck channel hides every popup for the fail-open delay.

**Change.** Gate on an attribute that Rust or the engine probe writes at mount: `[data-dx-place=rust]:popover-open:not([data-dx-placed])`. Never match a modal dialog. The first-frame spec in 3.PL must include a modal dialog and an A-placed menu.

### F10. DETAIL, major. The clippy sink ban cannot go live until the last eval site is gone, and the plan forbids the inline escape that would let it
**Evidence (V).**
- There is no `disallowed_methods` in any `clippy.toml` or source today (`/home/user/wt/l30/clippy.toml` holds one line; `check-js-volume.sh:54` says "a cargo gate ... in row 434").
- `grep` finds 107 eval-bearing lines in `primitives/src` and `preview/src`. The three clippy arms run `-D warnings`. Enabling the rule now turns all three red.
- 5.3 item 3 says "no inline-marker escape", but clippy cannot read a TSV allowlist.

**Change.** Use a per-file `#![expect(clippy::disallowed_methods, reason = "row 434 ...")]`. An `expect` fails with `unfulfilled_lint_expectations` as soon as the site is gone. So it shrinks by construction, and `check-js-volume` can count the attributes as an inventory. Let 5.3 distinguish this from a marker.

**Uncertainty (I).** rsx passes the component as a path value (`dioxus_core::fc_to_builder(#spanned)`, `dioxus-rsx-0.7.9/src/component.rs:225`), not a call. Recent clippy also lints uncalled paths, but I did not run it. The 3.0b self-test mutation `Script { "..." }` should confirm it, and the textual sink gate stays as the backstop.

### F11. DETAIL, minor. A calendar-dated allowlist expiry is a time bomb for every lane
**Evidence (V/I).** Root section 4 and row 443: each legacy file is "an allowlist entry with an `expires:` date that fails after it". There is no date-expiry precedent in `scripts/lib/allow.py`. After that date `run-gates.sh` goes red for every branch in the repo, including ones that touch nothing near the carousel, and a late owner measurement would trigger it.

**Change.** Key the expiry to a stamp instead: the gate fails when `research/measure/<port>.pass` exists and the legacy file is still tracked. Removal is then enforced exactly when the measurement is recorded, and the calendar cannot break other lanes.

### F12. DETAIL, minor. The volume ratchet and the registry disagree about new TypeScript
**Evidence (V).** `dev-docs/js-interop.md` (l30 copy, line 117) says a new `.js`/`.ts` file is forbidden and "a raise is the owner's decision, never a lane's". The plan adds about nine behaviour modules and the `ts/legacy` files. Typed `decide()` plus adapter plus generated types will often exceed the removed Rust-string JS lines. Each lane's `--update-baseline` then needs `--allow-raise`, and the freeze creates "a new file with a count" (the moved `include_str!` is a sink).

**Change.** The owner pre-authorises, once and in the plan, a raise of up to each behaviour's budget row, recorded as `# raised: row 434 budget <kind>`. Lanes then cannot stall at handback.

### F13. DETAIL, minor. `legacy-impl` recreates the 2026-10-04 `web` coverage hole
**Evidence (V/I).** `CLAUDE.md` documents that `cargo clippy --workspace` never enabled `web` or `server`, so those arms were unlinted (row 139). The same applies here. The three clippy arms and `cargo test --workspace` are `--workspace`, so preview's request for `dioxus-primitives/legacy-impl` is unified into the one primitives build. The code behind `cfg(not(feature = "legacy-impl"))`, which is what users compile, is then seen only by `check-primitives-features`' `cargo check`, which is neither clippy nor `cargo test`.

**Change.** Make the switch one function with two `cfg` bodies and keep everything else unconditional. Add a `-p dioxus-primitives` clippy and test arm without the feature to the cargo gates, or run `cargo clippy -p dioxus-primitives --no-default-features --features web`. (`tsconfig` and the byte-for-byte point are in F4.)

### F14. DETAIL, minor. Missing specs: which exist, which a lane must write, and who adds the `durations.json` entry
**Exist (V, `/home/user/wt/b28/playwright`).**
- `oracle/tier2-html/listener-inventory.spec.ts` (the baseline-before-first-open multiset).
- `frame-cost.ts` and the `check-timing-relative` convention.
- `wheel-replay.ts`, `carousel-wheel-replay.spec.ts` and `fixtures/carousel-wheel-recordings.json`.
- `oracle/tier3-radix/scroll-lock.spec.ts`: lock, restore, gutter and nested cycles. It has no `touchmove`, key, horizontal or in-dialog-scroller case (0 hits for `touchmove|touchstart|ArrowDown|PageDown|Space`).
- RTL cases in `message-scroller-api` (3 hits), `select` (3) and `toast-swipe` (4, mouse only).
- `modal-quick-reopen`, `touch-open-focus`, `keyboard-inset`, `top-layer`, `combobox-placement`.
- `bridge-dom.spec.ts` (3.0, `/home/user/wt/l30`).
- Firefox and WebKit projects in `playwright.config.ts:53,59`, but not in the three local configs, and only Chromium is installed (`/opt/pw-browsers`).

**A lane has to write.**

| spec | owner lane |
|---|---|
| `g-latency`, first-frame, open/close/open at N = 0 and 2, nested overlay under an animating ancestor, 400% zoom, RTL subtree, keyboard-reduced `visualViewport` | 3.PL |
| stuck-channel fail-open (Escape still closes at `N = never`) | 3.OV |
| `bridge-cost` and the registry-leak spec (200 mounts) | 3.0b |
| `scrollLock` characterisation (in-dialog scroller allowed, horizontal and RTL, touchmove blocked outside, keys skipped in inputs) | 3.SL, first |
| IME composition and paste with the caret in the middle | 3.FC |
| RTL for `scroll_to`, find, S1, context menu and drawer drag | 3.VS, 3.PL, 3.GS |
| touch toast swipe | 3.GS |
| W8's scenarios, including a run that streams through Rust | 3.MS |
| C5's property test (`cargo`), the replay harness's phase fix | 3.WB |
| the shared-channel handoff test | 3.WB or 3.CA |
| the per-component `pan` baselines (toast, carousel, photo wall) | 3.GS, 3.CA |
| carousel, message-scroller and toast subjects in `main-thread.spec.ts`, plus a relation spec per port | 443 |

That is about 18 new files or extensions. The Firefox and WebKit runs cannot execute in the lane container at all.

**`durations.json` ownership.** The plan says "`playwright/durations.json` is updated by the main loop, in sequence". The `check-slow-spec-budget` gate fails on a spec file with no entry. That gate is in the lane's own `run-gates.sh --fast`, and lane-rules says the file merges through a driver. So each lane adds its own entries. Headroom is 17,000 - 12,207 = 4,793 s, which about 18 specs of 60-120 s will largely use up. The release suite then grows from about 52 min to about 65-70 min unless the budget is deliberately raised with a stated reason.

### F15. DETAIL, minor. Eleven lanes append their reports to the same row file
**Evidence (V).** Lanes 2B and 2C already appended "Progress" sections to `434-javascript-in-typescript-modules-not-rust-strings.md` and edited the audit rows. Eleven more lanes would do the same and conflict.

**Change.** One progress file per lane (`dev-docs/backlog/434-lanes/<lane>.md`). The main loop folds them into the row at merge.

### F16. DETAIL, minor. The 3.0 tree keeps verbs the plan says are not verbs
**Evidence (V).** 5.2: "Commit-sensitive effects are not verbs" (`setPointerCapture`, `releasePointerCapture`, a `focus` for the next keystroke) and "`commit()` is dropped". `ts/bridge.ts` `METHODS` has `setPointerCapture`, `releasePointerCapture` and `focus`. `bridge.ts:23` and `bridge/mod.rs` still document `commit`. `mod.rs` warns they are "not usable at `pointerdown`", but nothing enforces it.

**Change.** 3.0b removes them, or `check-bridge-behaviours.sh` gets a deny-list of verb names so a lane cannot call them. The deny-list is the by-construction half.

### F17. DETAIL, minor. "One at a time, the owner measuring after each" serialises builds that do not need it
**Evidence (I).** 3.MS (message scroller), 3.WB (`wheel.rs`) and 3.CA's carousel files are disjoint once F4's freeze exists. The one-at-a-time rule is about the owner's attention. Today it also serialises three lanes and three deploys (about 1.1 h of exclusive lock each) plus three owner round trips.

**Change.** Build all three Rust arms in parallel after 3.FZ, deploy once with both switches, and let the owner measure in his order (W8, C5, carousel). A C5 loss then costs one follow-up lane (the TS module), not a delay in front of everything else.

### F18. DETAIL, minor. "Never marked done on web evidence alone" has no merge rule
**Evidence (V, root 5.5, row 442).** The stamp needs a real webview or a runner with `webkit2gtk`. This container has none (root 5.5: "no gtk/webkit2gtk here"), and CI is frozen (`CLAUDE.md`, row 33), so an agent can never produce it.

**Change.** Say in the plan that a batch merges on "web-verified; desktop inferred", with the stamp as a tracked post-merge debt list per batch. Otherwise the first G or R lane cannot be closed.

---

## Schedule (item 3)

Constraints used (V): 4 cores, 15 GB; one FIFO build lock; `LANE_SLOTS=1` while Playwright runs; about 10 GB usable disk after pruning finished lane trees, so one lane build copy at a time; Chromium only; release deploy about 1.05 h plus the 21 min full gate per merge; spec costs from F5.

| phase | lanes (4 each at most) | why together | lock load, rough (I) |
|---|---|---|---|
| **0** (day 0-1) | merge 3.0 and R; **3.0b** (F1, F2, F16, F10); **3.FZ** (F4); **443 kit** (`__dxPerf`, the delay and spin hooks); prune finished lane targets | `bridge/**`, `carousel/*` plus `ts/legacy`, and `ts/perf` plus `playwright/` are disjoint. This phase holds no popup or dialog change, so the spec load is low | about 5 h |
| **1** (day 1-4) | **3.PL** (with T4-T7, F3, F9), **3.OV** (without T4-T7), **3.DJ** (about 5 h, merged to the batch first), **3.VS** (after 2D) | 3.PL and 3.OV are file-disjoint after F3. 3.DJ merges first because it removes code both would otherwise touch. 3.PL is the long pole (about 36-48 h wall), so it starts first. One merge, one full gate, one deploy at the end | passes 42 + 52 + 42 + 26 = 162 min, 3 passes about 8 h, plus builds about 2.7 h and the deploy: about 12 h |
| **2** (day 4-6) | **3.SL**, **3.GS**, **3.FC** (without S7), **3.OB** (without S15), with **3.PV** and **3.QH** as small preview-only lanes in the same merge | consumers of `hotkey` (3.OV) and the `pan` skeleton exist. The two broad-primitive rows (S7, S15) go to a separate small lane run once | about 10 h |
| **3** (day 6-9) | **3.CA**, **3.WB**, **3.MS** edit in parallel (disjoint after 3.FZ), plus the S7/S15 lane. One deploy with both switches | F17 | passes 19 + 19 + 26 = 64 min, a smaller load. Builds serialise. About 8 h |
| **4** | the owner measures W8, C5, carousel in his order; then, per stamp, the legacy removal (2 h) and a C5 TS-module lane if Rust loses | F11: expiry by stamp | 1 deploy |

- Deploys: five (phase 0, 1, 2, 3, 4), about 1.5 h of exclusive lock each, 7-8 h in all.
- Row 442's desktop-cadence harness (about 14 h, a 3.6 GB tree) is built in a window with no lane copy, between phases 2 and 3, from the `deploy` tree.
- Install smoke runs before each deploy with no lane copy alive (needs about 6 GB).
- Wall time (I): the sum of the phase critical paths is about 4 + 40 + 24 + 30 = about 100 h, about 4-5 days of continuous running, against the original plan's about 7-8 days (F7), plus the owner's turn-around.

---

## Owner's measurement steps (item 4)

Buildable with the gates as they stand, with the following changes:
1. **Ratchet.** The freeze moves lines out of Rust strings into `.ts` (equal or lower, since blank and comment lines drop out), but each new `ts/legacy` file is "a new file with a count" and its `include_str!` is a sink. Each needs a raise note. F12 pre-authorises them.
2. **Cargo gates.** `legacy-impl` is fine for `check-primitives-features` (each feature alone, `web`, all together) and for `check-installed-paths` and `registry-install-smoke`, because no installed component file names it. It is not fine for default-arm lint coverage (F13).
3. **TypeScript gates.** `tsconfig` is `strict` with `include: src/ts/**/*.ts`. `ts/legacy/**` must be excluded or `@ts-nocheck`'d (F4). The legacy files also need `check-js-bundle`'s manifest entries, which `build-js-bundle.sh` mirrors per module, so there is no collision.
4. **Reading the switch.** `?dx-impl` and `localStorage` need two rows that do not exist (`location.search`, `localStorage`; F1). Without them the plan would need `web-sys`, which it bans.
5. **Spin hook.** `?dx-bridge-spin` "per vdom event" needs a page-side busy loop on Dioxus's root listener. That is page JS with no registry entry, so it needs an allowlist entry, a `js_file` raise and a debug-only guard. The perf recorder in `ts/perf` is the same, and it ships in the deployed docs bundle because the owner measures on the hosted release build. The `listener-inventory` baseline stays valid only because the hook installs when the flag is set.
6. **Component-catalog and demo gates.** The `?dx-impl` switch adds no component or variant, so no catalog, `demo-forms` or `demo-remote-assets` change. The scenarios that stream through Rust can reuse the existing `message_scroller` variants (`agent_chat`, `full_chat`). A new variant would pay the catalog, notes, `all-routes` and `hydration-parity` cost.
7. **The owner's third measurement** (carousel macros "within one frame of the JS arm") has no JS arm in the plan as written (F4).
8. **Spec cost.** Running both arms for the affected specs adds to `durations.json` (F14), and the suite's 17,000 s budget needs a reasoned raise.

## Attacks that failed

- **"`bun` or `tsc` is absent in a fresh container, so no TS lane can build."** `/root/.bun/bin/bun` is dated 2026-10-03 (the image) and reports 1.4.2, which matches the manifest's `bun 1.4.2`. `tsc` is at `/opt/node22/bin`. `session-setup.sh` does not install them, so this is an image dependency, not a hazard I could demonstrate.
- **"The 16 `not(web)` sites are not dead."** `dialog.rs:476-530` is the Blitz-only modal arm with the focus-trap eval, exactly as 5.1 item 0 says. I found nothing that contradicts the deletion.
- **"Per-module bundle outputs still collide on one `bridge.js`."** `build-js-bundle.sh` mirrors every entry's path (`ts/behaviours/fixture.ts` to `js/behaviours/fixture.js`) with one manifest line pair each. The remaining collisions are the shared bridge files (F1) and the sorted manifest, which the main loop rebuilds at merge.
- **"3.FC, 3.VS, 3.OB, 3.SL, 3.GS, 3.PV and 3.QH overlap on a file."** Checked pairwise against the audit locations. No shared file.
- **"The `hotkey`, `pan`, `settle` and `scrollLock` build-before-consume order is wrong."** It is correct as drawn (the `pan` ownership gap is F8).
- **"`slow-spec-budget` blocks the new specs."** Headroom is 4,793 s, enough, but only with a stated raise for the full set (F14).
- **"The `legacy-impl` switch breaks `hydration-parity` or SSR."** The arm is chosen after hydration in an effect, and the markup is identical across arms, so hydration and SSR hold (I). The hole is lint coverage (F13).
- **"Rust `Script { }` in rsx slips past `disallowed_methods`."** Possible (`fc_to_builder(Script)` is a path value, not a call), but recent clippy lints uncalled paths and a textual sink gate backs it up. Left as an I-labelled check for 3.0b's self-test (F10).
- **"The release suite alone is too slow to be in every deploy."** About 52 min with a warm tree is acceptable at the proposed five deploys.
