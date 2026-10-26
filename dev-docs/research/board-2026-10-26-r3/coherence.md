# Round 3, lens `coherence`: is the plan coherent and in proportion?

VERDICT: HOLDS WITH CHANGES

DESIGN findings: 3, DETAIL findings: 14

Summary:
- The root rule, the closed bridge with one eval sink, the sink-plus-ratchet enforcement and the deadline table's first three rows hold. I could not break them from the documents.
- DESIGN 1: the command kind (`reconcile`) and the gate (G) contradict each other on who calls `show` and what gets reported back.
- DESIGN 2: admission is self-declared (`sameFrameWrite` and `focus` admit any program), and the plan's own table places `settle` and `scrollStep` elsewhere.
- DESIGN 3 (proportion): the construct needs about 35-40 agent-hours, not 235-270. About 72 hours of the plan (27%) remove roughly 2 lines of JS between them.
- The estimate cannot be checked. It is the execution member's judgement. Step 2's measured yield was about 3 JS lines removed per agent-hour; the plan implies 15.
- The owner's measurement plan cannot be run by one person as written. It needs iOS 26 and 27, Windows WebView2, Android, 120 Hz displays, and Safari-side recorders that do not exist.
- Many DETAIL contradictions between the documents (listed below). The lane tables disagree with the prose on file ownership in four places.

Evidence labels: V = read in the repo at /home/user/wt/b28 (branch queue/batch-28 at 02a2b880), W = web source, I = inference. Paths below are relative to `dev-docs/` unless they start with `scripts/`. "root" = `research/sync-js-root-2026-10-26.md`, "residue" = `research/sync-js-residue-2026-10-26.md`, "434" = `backlog/434-javascript-in-typescript-modules-not-rust-strings.md`.

## Numbers used (computed from `research/js-audit-2026-10-25.md` string-extent lines and the root's lane table)

"Removed" means the audit block's JS ceases to exist (Rust, CSS, platform or deleted). "Relocated" means the block's lines go into a `ts/behaviours/*` module or the bridge and keep counting as JS. Classification is I (from root section 3 and 3.1). The hours are the root's own (section 5.4, midpoints).

| lane | agent-h | lines removed | lines relocated to TS | removed per hour |
|---|---:|---:|---:|---:|
| 3.DJ dead JS | 5 | 196 | 0 | 39 |
| 3.PL placement (T4-T12, S1, G) | 28 | 755 | 24 | 27 |
| 3.PV preview glue | 11 | 311 | 0 | 28 |
| 3.SC collection, activity | 4 | 118 | 0 | 30 |
| 3.MS message scroller (W8) | 37.5 | 1,025 (at legacy removal) | 40 added | 27 |
| 3.WB wheel band (C5) | 14 | 477 (0 if Rust loses) | 0 | 34 or 0 |
| 3.OB observers | 7 | 144 | 0 | 21 |
| 3.GS gestures, toast | 13 | 246 | 193 | 19 |
| 3.CA carousel | 28 | 412 (at legacy removal) | 659 | 15 |
| 3.FC form primitives | 10 | 101 | 36 | 10 |
| 3.OV overlay core | 26 | 253 | 124 | 10 |
| 3.VS virtual settle | 10 | 87 | 122 | 9 |
| 3.SL scroll lock | 8 | 2 | 94 | 0.2 |
| 3.HV frame-rate vdom handlers | 10 | 0 | 0 | 0 |
| 3.0b verbs, skeletons, gates | 19 | 0 (adds) | - | 0 |
| 3.FZ freeze and `?dx-impl` | 11 | 0 (moves about 2,700 lines) | - | 0 |
| row 443 kit | 10 | 0 (adds) | - | 0 |
| row 442 (outside the 235-270) | 14 | 0 | - | 0 |
| legacy removal | 2 | (books the 3.MS/3.CA lines) | - | - |
| total | 267.5 | 4,127 | about 1,290 | 15 |

- Step 2's actual yield (434:100-106): 2B 22 lines in 5.5 h, 2C 26 lines in 7 h, 2A about 69 lines in 10.8 h, 2E about 14 lines in 10.7 h. That is about 130 lines in about 45 h, or 3 lines per hour. The plan implies 15 per hour, five times higher. Step 2's lanes also ran 1.1-2.2 times their estimates (434:64).
- The estimate has grown from 66 to 86 to 155 to 235-270 (434:64). The root says the hours are "the member's judgement, not measurements" (root:528, citing execution F7). Nothing in the repo measures them.
- Lanes with about zero removal: 3.0b, 3.FZ, kit, 442, 3.HV, 3.SL = 72 h, 27% of 267.5. They are infrastructure, relocation, or desktop-perf work that is not "bespoke JS". 3.0b and the kit are part of the construct or its proof; the rest are not.
- The three feel-critical ports (3.MS, 3.WB, C1) with their support (3.FZ, kit, 442, removal) cost 88.5 h and remove 1,567 lines, 38% of all removal. They carry the plan's biggest unknowns: W8's Safari path, C5's possible reversion, and desktop latency. One of the three has a built-in exit that returns 477 lines to TypeScript.

## DESIGN findings

### 1. DESIGN, major. The command kind and the gate contradict each other. One of the two must change shape.

Claim attacked: root 2.2 "Kinds" and 2.3 item 1 ("the clear is the `show` verb's own JavaScript turn ... the reconciler applies opens through the same function").

Evidence (V):
- root:256: `reconcile` "is the **only** caller of show/hide for its element ... and reports **only browser-initiated changes** (light dismiss, Escape) back to Rust".
- root:386, 2.3 item 7 (root:312), `js-interop.md:31,97`: a Rust-initiated measured open is "the `show` verb plus the rects in one transaction" (`show_popover_measured`, `Transaction`). That is a second caller of `showPopover`, and it reports rects to Rust.
- root:305, 2.3 item 4: "The page mints the generation in the `show` verb's turn and reports it with the open". So an open Rust initiated is reported back, which is not "only browser-initiated".
- `js-interop.md:26` tells agents "Do not call `showPopover`/`showModal` from an effect". `js-interop.md:30,97` offers `el.show_popover()` and `show_popover_measured` anyway.
- A gated open needs three things at once: a level-triggered attribute (`reconcile`), a rects-returning transaction (G), and a page-minted generation reported with the open. The documents never say how they compose. If `reconcile` applies the open on a MutationObserver callback, the measure result has no declared channel to Rust, and a watchdog and a generation must be armed inside `reconcile`. If the `show` verb applies the open, `reconcile` is not the only caller and the lost-command class (#5919) returns for every gated surface.

Concrete change: write one contract for a popup open that is gated. Option (a): `reconcile` is the only caller of `show`, and it reports an `applied{gen, rects}` acknowledgement for every Rust-initiated transition on a gated surface (so "only browser-initiated" becomes "browser-initiated, plus an ack where a gate is armed"); `show_popover_measured` is removed from the public API. Option (b): drop the command kind for gated surfaces and keep an imperative show-and-measure verb there. (a) keeps the construct. Either way, change `js-interop.md:26/30/31/97` and root:256/386 to say the same thing, and give 3.PL and 3.OV a single interface to build against.

Subsumes: DETAIL 8 (3.PL's G pieces have no TS owner in its file list).

### 2. DESIGN, major. Admission is declared, not derived, and the plan's own table disagrees with two admitted behaviours.

Claim attacked: root 2.2 "Admission" (root:260-264) and "closed is a construction, not owner governance" (r2 SYNTHESIS).

Evidence (V):
- The gate reads "each variant's effect set from the generated `behaviours.gen.ts`" (root:261). The effect set is written by the variant's author. `sameFrameWrite` and `focus` are broad enough that any program that writes a scroll position or calls `focus` passes.
- `settle` (W3, W5, W7) and `scrollStep` (C2) are admitted on `sameFrameWrite` and `focus`, with the note "(I: neither cancels, captures or answers a commit directly)" (root:261).
- The placement rule does not place them there:
  - W3's settle loop is "stable for 2 frames, 30/20 tries" and W5's is "60 frames" (root:354-355). A loop that runs over many frames has no same-frame deadline. By the table (root:244-248) its row is "next frame" or "none", which is R.
  - root:258 says "One-shot commands (focus a cell, scroll an item into view, measure now) stay bridge calls". `scrollStep` is a Rust-issued one-shot `scrollBy` (root:277).
  - root:294 "What a behaviour is not": "A stateful heuristic over a stream ... is R, not a behaviour". `settle` is a stateful program over frames with a takeover token and cancel-on-input.
- Single-user behaviours: counting users in root:268-281 and section 3, six of ten behaviours have one use-site (`otpFilter` S11, `dropAnywhere` L25, `scrollStep` C2, `stickToEnd` W8, `openFocus` L16 as a policy, plus the `pan` variants `css-var` O5, `scroll-by-with-refusal` C4, `scroll-absolute` V17, each with exactly one user). Round 1 broke the first design on that point ("one user each ... a language, a serialisation format and a parity gate", root:230). The residue doc itself calls round 1's B2/B3/B5 "bespoke programs behind enum names" (root:399). The three `pan` variants and the three `settle` parameter sets have the same shape.
- The plan also has a second page-side category outside the registry: "an allowlisted TypeScript module" for C5's exit (root:331, 406; residue exec summary). But root 5.3 item 4 says "A new `ts/` file ... must be reachable from the registry index", so an allowlisted module is both required to be in the registry and "not a registry behaviour".

Concrete change: make admission checkable from something the author cannot declare away. Admit a behaviour only if its decision runs inside a commit-time context: an event dispatch whose effect is `prevent`/`stop`/`capture`/`dataTransfer`/`focus`, or a ResizeObserver/rAF callback whose effect is a layout write that must land before the paint (the `stickToEnd` shape). The adapter knows which context calls `decide`, so the gate can check it mechanically. Then:
- `settle` and `scrollStep` are not behaviours. They go to R, or to the owner-allowlisted module class that C5's exit already uses, with a budget row and an owner-written reason.
- Have one second class, not two vague ones: "allowlisted TS module", with a TSV line, a budget, and the same fail-open and lifetime helper as behaviours.
- Single-user behaviours may stay in the registry or move to that class; the difference is the generated-vector and adapter machinery, which is where lane 3.0b's 8-10 bridge hours go (root:499).

Subsumes: the `settle`/`scrollStep` provisional admission note (root:592), the registry-versus-allowlist ambiguity (DETAIL 5).

### 3. DESIGN (proportion), major. The construct needs the sinks, the ratchet, the placement rule, admission and two proof behaviours. It does not need phases 1-4.

Claim attacked: the five-phase plan as the way to make the class unable to recur (root:489-516, 434:64).

Evidence (V and I):
- What stops block 104? The one eval sink (typed `Program`, clippy `expect` that shrinks by construction, root:465), the grant rule (root:469), the file inventory and the hot-events gate. None of these needs a single port. Every legacy block can sit behind a per-file `#![expect]` and a frozen baseline row until it is touched. The plan already says legacy gates "do not retire when their allowlists reach zero" (root:475).
- Porting every block is a different goal: removing existing instances. It is bounded by yield (table above): 4,127 lines for about 267 hours.
- The 4 lowest-yield groups are the ones whose cost does not buy the construct: relocation lanes (3.SL 94 lines moved, 3.VS 122, 3.GS 193, 3.CA 659), `3.HV`, and the measurement-and-switch apparatus (3.FZ, kit, 442, legacy removal).
- Several heavy lanes cannot be verified in the lane container. 3.PL (28 h, 755 lines, visible on every popup open) verifies A on Chromium only; the Firefox/WebKit/Safari 26.0/26.2/27 runs, the forced-failing-engine boot probe and the real desktop pass are "cannot execute in the lane container at all" (root:545, 584). The lane's 28 hours exclude the proof that decides it.
- `3.HV` (10 h, 0 lines removed, about 20 sites) converts working Rust vdom handlers into bridge streams. Lane 2C landed S2/S3 as Rust `onscroll`/`onresize` on 2026-10-26 (434:100); 3.HV reverses that (root:511). The only reason is the blocking-XHR-per-event cost on desktop, which is I and unmeasured (root:150, 582). `check-hot-events` with a shrink-only baseline in 3.0b already stops growth.
- `legacy-impl` plus `?dx-impl` plus `ts/legacy` (3.FZ, 11 h; the 2,700-line move; a new cargo feature axis with its own clippy and test arm; a preview-only gate; a stamp-keyed removal) exists to run two arms from one binary. A pinned older deploy compared with a new one on the owner's devices needs none of it (I; `scripts/deploy-preview.sh` already builds release previews, and the plan only needs "same commit serves both arms" to cancel build noise, which interleaved runs and `calibrationMs` already cover, root:570).

Concrete change, minimal viable construct (I, hours are agent-hours at the plan's own scale):
1. Phase 0, trimmed: lane 3.0 (bridge, typed transport), lane R (landed), the gate half of 3.0b only (grant rows, `Program` seal, hot-events baseline, `adds:` rule), about 10 h. Not the ten behaviour skeletons or the wider adapter: build the adapter for the first two consumers, not for ten.
2. A "freeze" gate that is cheap and by construction: each legacy literal's hash is recorded per file; a diff that edits a baseline-listed script without deleting or porting it is red (about 3 h). That is "port on touch" enforced, not remembered. Today the ratchet counts per-file lines, so editing a script's text while keeping its line count passes.
3. Two proof ports that exercise both kinds: 3.DJ (5 h, 196 lines), `hotkey` (L6, O1, V22) plus L1/L2 rsx from 3.OV (about 12 h), and `reconcile` for T4-T7 only (about 6 h; after DESIGN 1 is fixed).
4. `__dxBridgeStats` only from the kit (about 3 h), and one real desktop pass of the current build to get the latency distribution the "next frame" row needs.
5. Everything else is "port on touch" under the freeze gate, or a later batch keyed to row 442's engines.
Total about 37-40 agent-hours, 14-15% of 267, removing about 450 lines.

Honest cost of this choice: the combined JS total does not fall. Lane 3.0 already raises it (bridge about 874 lines, budget 1,100; 434:110 "a `#! total-raised:` line until the ports land"), and ten skeletons would add more. See DETAIL 12 for the ratchet split that keeps the gate meaningful.

Subsumes: DETAIL 10, 11, 12, 13.

## DETAIL findings

### 4. DETAIL, major. `js-interop.md` states planned gates as landed (stale).

- `js-interop.md:149-153`: "A raise passes only against a grant row that exists at the merge base (a backlog row, open, whose front matter says `js-grant: ...`) ... a lane cannot grant itself." The landed gate is the self-service `--update-baseline --allow-raise "row N reason"` (`scripts/check-js-volume.sh:48-52`, `scripts/lib/ratchet.py:32,262`). Round 2's enforcement member found a lane self-raised about 2.7k lines. The grant rule is plan, not code.
- `js-interop.md:137-139,177-179` list `check-hot-events` and `check-bridge-behaviours` as gates. `scripts/` has neither (V: `ls scripts | grep -E "hot-events|bridge-behav"` is empty). Only line 137 says "planned".
- `js-interop.md:131` "clippy `disallowed_methods` once it lands" is honest; `clippy.toml` exists but the sink ban is not in it yet.
- The bridge itself is not on this branch (`primitives/src/bridge/` does not exist), though `js-interop.md:89-106` describes verbs as present. Line 82 says so once, then lines 87-106 read as current.
Change: mark every unlanded gate and the grant rule "planned (lane 3.0b)" inline, or agents will think a raise needs a row they cannot make and a verb exists that does not.

### 5. DETAIL, major. A page-side module exists in three categories with no single definition.

Registry behaviour (generated, admitted, vectors), "allowlisted TypeScript module" (C5 exit, `ts/legacy/*`), and verb-row. root:331/406 call the C5 exit "not a registry behaviour". root 5.3 item 4 requires every `ts/` file to be "reachable from the registry index". 434:60 says "an allowlisted TypeScript module" and 434:55 says "closed registry". Define the second class once (TSV line, budget, row, owner-written reason, fail-open helper) and say that the registry index lists it.

### 6. DETAIL, major. The legacy-removal gate contradicts C5's exit.

root:413 and 443:71: "the gate fails when `research/measure/<port>.pass` exists and the legacy file is still tracked". If Rust loses the C5 measurement, C5 "becomes an allowlisted TypeScript module" (root:406), which is the legacy file kept. A stamp for a failed measurement would force the fallback to be deleted. Use `<port>.pass` and `<port>.fail`, and make the gate act only on `.pass`. Also unspecified: who writes the stamp and what it contains (root:413 names `research/measure/<port>.pass`; 443:70 and ux-a11y-perf:231 name `research/measure/<port>/<arm>-<device>.json.gz`; a file and a directory with the same stem).

### 7. DETAIL, major. 3.FZ's file ownership contradicts what it must do.

root:500 lists 3.FZ "must not touch `primitives/src/bridge/*`" and says each frozen script is wrapped TS "behaviour-neutral". The wrapped legacy code needs to run, and root:444 says all Rust-to-JS goes through `Program` variants private to `bridge/`. 3.FZ therefore needs a `Program` variant or an allowlisted `Script { src: asset!() }` plus an eval, in a file it may not touch. Root:465 permits exactly one eval site. Specify the call path (for example one generic `Program::Legacy{module}` entry added by 3.0b) before 3.FZ starts. Also `transport.rs` is edited by 3.0b (typed `Program`, `pub(in crate::bridge)`), 3.0 (delay hook, 442:status) and the row 443 kit lane ("`?dx-bridge-delay` ... must not touch `bridge/*` beyond the delay hook", root:499-501), all in phase 0.

### 8. DETAIL, major. The G pieces have no TS owner in 3.PL's file list; four other lane-file conflicts.

- root:503: 3.PL owns "the G pieces (the `show` verb's clear, the page-owned generation, the fail-open names, ...)" but its file list has `top_layer/*`, `select/.../list.rs`, `context_menu.rs` (T11 call site), CSS. The `show` verb lives in `bridge/*` (3.0b) and the generation's only writer is `reconcile` (root:279), built by 3.OV (`ts/behaviours/reconcile.ts`, root:504). Two concurrent phase-1 lanes, neither owning the JS that G needs. (Linked to DESIGN 1.)
- `lib.rs`: root:519 "lib.rs has one owner (3.OV)", but 3.DJ also owns "`lib.rs` (the focus-trap bundle constants only)" (root:502), concurrent in phase 1.
- `context_menu.rs`: root:519 "three owners"; 3.HV's table (root:511) lists `context_menu.rs:531` as a fourth.
- `select/components/list.rs`: 3.PL (S1) in phase 1 and 3.HV (S2/S3) in phase 3 are sequenced, but lane 2C already changed the same lines in step 2 (434:100).
- 3.CA and 3.WB both edit `carousel/drag.rs` (3.WB takes "the bounce lines only", root:515) and `pan.ts` is built by 3.GS and extended by 3.CA; sequenced, but the "must not touch" columns do not say so.

### 9. DETAIL, major. Some step-2 keep-JS blocks have no placement and no lane.

Ids present in the audit and absent from the root: T1, T2 (insertion scripts kept by 2A), L18, L20 (kept by 2B), L22, S4, V6 (kept by 2C), W2 (14 lines), plus V1-V4 and L15 (class c, covered generically). root:526 says the 43 (b) rows without a lane are all assigned, and "L26 and P0 complete the 43", but L18, L20, L22, S4, V6, T1, T2 are step-2 leftovers outside that list. Under the single-sink rule each one is red the day the last `expect` goes.
Worse, L20 needs `focus({preventScroll})` (lane 2B measured a 157 px page move, 434:104) while 3.0b removes `focus` and puts it on a `FORBIDDEN_VERBS` list for both enums (root:456). A non-commit focus that must not scroll has no home. Split the ban: forbid the commit-sensitive uses, keep a `focus_prevent_scroll` verb that is explicitly not for the next keystroke, or put L20 in a behaviour with a stated reason.

### 10. DETAIL, minor. `3.QH` and other stale names.

root:512 "3.PV preview glue, with V19 (3.QH)" and root:540 "3.PV with 3.QH | 8 + 3" keep a lane name that 434:64 and the lane list merged into 3.PV. `board-2026-10-26/SYNTHESIS.md:31` says "Lane 434-R (in flight)" but 434:108-110 records it landed. 434:22 states "Step 3: about 66 agent-hours in 8 lanes over two batches" without a "superseded" marker (only the heading at 434:26 carries one).

### 11. DETAIL, minor. The estimate is counted two ways.

root:545: "Rows 442 and 443 and the lock sit outside the lane hours", but the lane table includes the row 443 kit ("about 10", root:501, 532). 434:64: "About 235 to 270 agent-hours after 3.0 and R ... plus rows 442 and 443". My sum of the table is 235-272 including the kit once. Row 442's 14 h is outside the sum; 3.0 itself and R are outside ("not re-estimated", root:498), so the honest total to compare with the earlier 64/86/155 is higher than 235-270 by 3.0's hours (unstated) plus 14.

### 12. DETAIL, minor. The ratchet's combined total conflicts with building the bridge first.

`check-js-volume.sh:49-52` and 434:110 require `js_lit + js_file` never to rise, and say lane 3.0 needs a `#! total-raised:` line "until the ports land". With the minimal path (DESIGN 3) the total would stay raised for good (bridge about 900-1,100 lines plus behaviour modules). Split the column into `legacy` (JS in Rust strings and files outside `bridge/` and `ts/behaviours/`, shrink-only, no raise) and `sanctioned` (per-file budgets for `bridge.ts` and behaviour modules, root:468). Then the legacy ratchet stays meaningful at every step.

### 13. DETAIL, minor. `check-hot-events` is described two ways.

root:448: "the gate counts **event names across the app** (a per-event-name allowlist, not per file)". root:472: clippy `disallowed_methods` on the event functions with "`#[expect(..., reason)]` at the site as the reasoned allowlist", which is per site. The cost class is per name (one `onpointermove` makes the event blocking app-wide, root:139), so five sites with five `expect`s hide one cost five times. Pick one: count distinct names, and require a reason per name.

### 14. DETAIL, minor. "No inline-marker escape" versus the `expect` attributes.

root:467 "No inline-marker escape ... never a lane's". root:465 puts `#![expect(clippy::disallowed_methods, reason = ...)]` in each file with eval, and root:472 `#[expect(...)]` at hot-event sites. A lane can add one in its own file. The volume gate counts expects as inventory (root:465), so a new one is a ratchet raise, and the claim holds, but the text should say the expect is counted and needs a grant, not "no inline marker".

### 15. DETAIL, minor. Names and orderings that differ between documents.

- `use_resize` (root:459, the leptos-use name list) versus `use_resize_observer` (`js-interop.md:93`).
- Precedence: root:240 and `js-interop.md:40` "read top to bottom; the first row whose deadline the decision has is its placement", versus root:428 "The order is a preference among decisions, not a first match per block". Say which one applies when a block has several decisions (it does: T8, W8).
- `js-interop.md:69` lists nine behaviours plus `reconcile`; root:266-281 lists twelve rows including the two reserved (`dialogKeyForward`, `place`); root's executive summary (root:33) lists nine and omits `reconcile`. 434:55 gets it right. Fix the other two.
- root:81 says "stopPropagation is read by the JS but never sent by Rust (the struct only has `prevent_default`) Web never calls..." (missing full stop; the sentence runs two claims together).

### 16. DETAIL, major. The "next frame" row's budget does not exist when three lanes need it.

root:247, 592: the next-frame row's latency budget "the owner sets once from the row 442/443 measurements". The measurements happen per port in phase 4 (last). L30, O5, C3, C4 and C5 are built in phases 2-3 "by the argument in section 3" (root:592), and `pan` is built in phase 2 (3.GS) before the budget exists. If the budget comes out tight, 3.GS and 3.CA rebuilt for nothing. The budget does not depend on any port: it needs a real desktop latency distribution of the current build (`__dxBridgeStats` round-trip histogram plus the C5 jitter table) taken once. Do that first, in phase 0, with the kit's stats half and one real desktop session. It also gives the desktop-cadence harness (442) its N and jitter "from one measured distribution, not by fiat" (442:33).

### 17. DETAIL, minor. 3.0b's skeleton sweep is registry creep ahead of need.

root:499: skeletons for all ten behaviours (enum arm, empty TS module, sample vector), a grant row per planned behaviour (root:469), the wider adapter. round 2 found that about 985 of 1,000 behaviour lines need the wider adapter (root:287), so the adapter's real shape is unknown until a second consumer exists; building it for ten unwritten behaviours risks the wrong abstraction. Pre-granting all ten means the "needs a row at the merge base" check fires only for an eleventh. Build the adapter for `hotkey` plus `reconcile` (two kinds), then widen per consumer.

## The owner's measurement plan (rows 443 and 5.6): can one person run it?

Not as written. Evidence:
- Devices and OS versions needed (root:574-576, ux-a11y-perf:241-258): iPhone Safari on iOS 26 and on iOS 27, Mac Safari 27, Chrome and Firefox 156 on a trackpad, a macOS WKWebView desktop build, a Windows WebView2 desktop build with a precision touchpad, 60 and 120 Hz, Android touch swipe with TalkBack, VoiceOver, a 240 fps slow-motion video. One phone cannot run iOS 26 and 27 (I; the plan itself forces `overflow-anchor: none !important` for the fallback in scripts, so the manual iOS 26 pass is redundant with a forced run). A Mac owner has no Windows machine or Android SDK in the plan.
- The recorder does not work on the engine the plan worries about. `__dxPerf` is built on `longtask`, `long-animation-frame`, `event` timing and `layout-shift` observers (443:67). MDN lists `PerformanceLongTaskTiming` as "Limited availability" and says it does not work in some of the most widely used browsers (W, https://developer.mozilla.org/en-US/docs/Web/API/PerformanceLongTaskTiming; the page's compatibility table was not captured); the plan itself marks long-animation-frame "Chromium". I believe Safari ships none of the three (I). On iPhone and Mac Safari the kit would record only frame deltas and bridge stats, and the CDP trace helper (443:70) is Chrome-only. Getting a dump off an iPhone needs a tethered Mac Web Inspector session. Port 1's pass criteria (`layout-shift`, "fight frames", long tasks, task time) cannot be evaluated on WKWebView, which is exactly the Safari 26 `stickToEnd` path.
- Workload: port 1 has 7 scripted scenarios times 2 arms times 2 delay settings, plus 8 manual checks on 4-5 browsers; port 2 has 10 flicks per arm per device and 10 manual checks; port 3 adds touch on two phones. That is several person-days per port and three ports in sequence ("one at a time", root:492), plus one real desktop pass per batch (442:35, four batches). The plan counts none of it ("The figure carries no desktop measurement time: that is the owner's", root:545).
- The emulation the owner would use to feel desktop cost on a Mac (`?dx-bridge-spin`, `?dx-bridge-delay`) is, by the plan's own text, "a bridge-latency emulation, not a desktop model" (443:69, 442). The feel verdict would rest on something disowned in the same row.
- Pass criteria are numeric (p95 at most 1 frame, 1 px, 1.1 times task time) against a budget that does not exist yet (DETAIL 16) and requires a JS baseline arm that is the frozen 2,700-line legacy path (DESIGN 3, 3.FZ).
Change: (a) cut manual coverage to what one person owns: one Mac (Chrome, Safari, Firefox trackpad), one iPhone, one Android phone if present; (b) say which numbers each device can produce (frame deltas and bridge stats on Safari; the full set on Chromium only) and make iOS/macOS Safari verdicts a short eyeball checklist plus the forced-fallback script run; (c) make the Windows and macOS desktop-build passes a single owner session scheduled once, not per batch; (d) front-load the one latency distribution (DETAIL 16).

## Attacks that failed

- "The plan's totals are wrong." The 22 class-(d) block sums reconcile exactly with the audit (A 1,638 = T8 499 + T11 74 + W8 1,065; behaviours 880; R 1,153; G 172; spliced 39; seam 12; 3,894 total, V). The lane table sums to 235-272 as stated.
- "The root rule or the first three rows of the deadline table conflict with the code." Nothing in the five documents contradicts them. The Dioxus-source claims they rest on were not mine to re-verify in this lens and I did not.
- "The one-eval-sink construct is incoherent because the legacy scripts need an eval." It holds for shipped code (the `Program` constants, a per-file shrinking `expect`), but the legacy call path is unspecified (DETAIL 7); that is a gap, not a contradiction of the construct.
- "The behaviour names or counts differ across the root and the lane table." They match apart from the three nits in DETAIL 15. The registry in the table, the lane assignments and the "ten behaviours" skeleton sweep agree.
- "The ratchet cannot enforce shrink-only." It does as landed, per file and against the merge base; the hole is the self-service raise (DETAIL 4), which the plan already identifies and schedules.
- "The 235-270 can be shown wrong from the repo." It cannot be shown right or wrong: no measure exists for the plan's hours. The only measured yield (step 2, about 3 lines per hour) says the plan's 15 is optimistic by about five times, but step 2's blocks were the small ones, so it is an indication, not a proof.
