# General backlog -- open rows

One file per row. Everything outstanding across the plan, the harness and the follow-ups found during execution
lives in `dev-docs/backlog/`; **this file is only the generated index of the OPEN rows** and
[`backlog-done.md`](./backlog-done.md) the generated index of the done ones. A citation such as
"`dev-docs/backlog.md` row 116" (about 250 files and 530 comments carry one) resolves by id: look in the table below
for an open row and in `backlog-done.md` for a finished one, or glob `dev-docs/backlog/*116-*` and
`dev-docs/backlog/done/*116-*`.

## Scope

**Owner rule (2026-10-21): the library keeps to primitives and shadcn element parity**, even where an element is more
complex (the chat kit counts as parity). Bigger stateful items that need their own state management, such as a
Markdown or rich-text editor, belong to a higher-level framework or an external library, not to this backlog; a row
proposing one is closed as out of scope (row 337 is the first).

## How it works

- **Open rows:** `dev-docs/backlog/<NNN>-<slug>.md` (`NNN` = the zero-padded row id, never renumbered, ids are never reused).
- **Done rows:** the same file under `dev-docs/backlog/done/` once its status starts with LANDED / FIXED / RESOLVED /
  CLOSED / SUPERSEDED / DONE and names no open remainder. Anything partly done stays in the open set. A done row keeps
  its front matter, the four sections and a short summary with the landing commit or PR; the long narrative lives in
  git history (code cites rows by id, so the file stays).
- **A row file** has front matter (`id`, `title`, `status`, `size`, `area`, `updated`) and four sections, in this
  order: `## Item`, `## Status / trigger`, `## Size`, `## Detail`. `title`, `area` and `updated` are hand-editable
  (`area` is a coarse tag, `updated` the latest date the row mentions); `status` and `size` are derived from their
  sections.
- Sub-ids such as "row 31a / 31b" or "item 21" in citations live inside that row's file (31, 21).
- **To add a row:** pick the next free id (the highest id in the two indexes, plus one), copy any row file, rename it
  `<NNN>-<slug>.md`, fill the front matter and the four sections.
- **To update a row:** edit its sections (append dated notes to Detail, change Status / trigger when it lands). When it
  lands, `git mv` it into `dev-docs/backlog/done/`.
- **Then run `scripts/backlog-index.sh`** (syncs `status`/`size`, rewrites both indexes) and commit the row file and
  the index files together. `scripts/check-backlog-index.sh` (a `run-gates.sh` gate) fails on a stale index, a missing
  front matter field, a duplicate id, a row in both sets, and a broken relative link under `dev-docs/`.
- Never hand-edit between the GENERATED markers; edit this header freely.
- The current order of work is in [`research/backlog-order-2026-10-18.md`](./research/backlog-order-2026-10-18.md) and the
  live state in [`STATUS.md`](./STATUS.md). The original monolithic file (244 rows at `de7059ba`) and its narrative
  sections are in git history before row 266's cleanup (`3c584cc2`).

<!-- BEGIN GENERATED: scripts/backlog-index.sh rewrites everything up to END; edit the row files, not this block -->

## Open rows (71)

| # | Item | Status / trigger | Size | Area |
|---|---|---|---|---|
| [4](backlog/004-first-3-browser-ci-run-treat-as-calibration-not.md) | First 3-browser CI run — treat as calibration. Not done — see 2026-09-04 Notes below for what's now known and… | WAITING ON OWNER: CI reactivation (off until further notice, 2026-08-30). The GitHub-hosted… | S–M | ci |
| [23](backlog/023-preview-s-desktop-feature-lacks-dioxus.md) | preview's desktop feature lacks dioxus-primitives/web — a webview-backed desktop build would render the… | WAITING ON OWNER: a desktop build (needs gtk / webkit2gtk, absent in the sandbox). The fix is in… | S | overlays |
| [28](backlog/028-verify-the-2026-09-02-ios-keyboard-anchor.md) | Verify the 2026-09-02 iOS-keyboard anchor-tracking fix (use_anchor_position_fallback… | WAITING ON OWNER: re-test on a real iPhone after deploy. The fix is in; the sandbox has no WebKit… | S to verify | overlays |
| [30](backlog/030-verify-the-2026-09-02-touch-text-entry-font.md) | Verify the 2026-09-02 touch text-entry font-size floor… | WAITING ON OWNER: re-test on a real iPhone after deploy. The fix is in; Playwright's iPhone 13… | S to verify | tests |
| [33](backlog/033-3-browser-playwright-on-every-push-pr-sharded.md) | 3-browser Playwright on every push/PR, sharded — dignifiedquire/dx-components runs chromium+firefox+webkit… | WAITING ON OWNER: CI reactivation (off until further notice, 2026-08-30); then it is a… | S | ci |
| [152](backlog/152-test-suite-review-revisit-the-playwright-suite.md) | Test-suite review: revisit the Playwright suite (~1,786 tests, ~21 min on a release build with 4 workers) for… | IN PROGRESS 2026-10-24. Done: 152a (below); the shared-page fixture, through rows 327, 358 and 334… | M | tests |
| [153](backlog/153-native-find-in-page-for-windowed-virtual-lists.md) | Native find-in-page for WINDOWED virtual lists: keep per-item text in the DOM as hidden="until-found"… | Left: a Firefox 139+ and Safari 26.2 run of the find bar, and a release-build mount timing for 100k… | M | virtual |
| [170](backlog/170-cross-component-css-overrides-190-sites-in-12.md) | Cross-component CSS overrides: 190 sites in 12 sheets declare, in one component's stylesheet, a property that… | BURNED DOWN 2026-10-24: baseline is 25 sites in 1 sheet (sidebar dropdown panel placement, a stated… | M (the decision and the prototype are… | theme |
| [266](backlog/266-repo-cleanup-archive-historical-dev-docs.md) | Repo cleanup: archive historical dev-docs, trim CLAUDE.md and the big dev docs, split backlog.md | Left: deleting the stale remote branches (the owner said go on 2026-10-26, but this environment's… | M-L | process |
| [268](backlog/268-select-scroll-buttons-and-item-aligned-position.md) | Select scroll buttons and position=item-aligned | Only the native (Blitz) arm is left: position=item-aligned emits data-position there and nothing… | S (native arm only) | select |
| [295](backlog/295-press-feedback-on-touch-taps-already-on-by.md) | Press feedback on touch taps: already on by default, so check why taps don't show it | WAITING ON OWNER: iPhone re-test of press feedback on /component/tabs/, /component/menubar/… | S (or none) | theme |
| [296](backlog/296-combobox-list-drifts-from-its-input-on-ios-after.md) | Combobox list drifts from its input on iOS after long scrolling | ROOT CAUSE FOUND 2026-10-28 (owner's iPhone runs of the device lab, iOS 18.7 Safari); fixed by row… | M + device check | overlays |
| [305](backlog/305-side-drawer-scrolling-its-content-also-drags-the.md) | Side drawer: scrolling its content also drags the drawer sideways (iOS) | WAITING ON OWNER: iPhone re-test of the side drawer (scroll its content, the drawer must not drag… | S–M | primitives |
| [371](backlog/371-dependency-evaluation-keep-lift-drop.md) | Dependency evaluation: keep, feature-gate, replace, lift or drop (rule of 2026-10-23) | BACKLOG: unparked by the owner 2026-10-24, stage 4 of the adopted order (after the structure rows… | S each outcome | build |
| [372](backlog/372-scroll-fade-overlay-scrollbar-hybrid.md) | Scroll fade: transition the edge depths (landed) and paint the fade as gradients where the scrollbar is an overlay (deferred) | WAITING ON OWNER: part 1, the overlay-scrollbar hybrid, is deferred ("leave mask for now"); reopen… | M | primitives |
| [383](backlog/383-adversarial-review-board.md) | Adversarial review board: independent expert reviewers attack the finished library, last item of the backlog | BACKLOG. Runs last: after row 365's proposals are answered and every other engineering row has… | M | process |
| [387](backlog/387-docs-clarity-review.md) | Standing docs-clarity review: a technical-writer pass over every component's docs and props for implicit behaviour | STANDING. Run after every round that adds a component or changes a public prop, and before row… | M | docs |
| [390](backlog/390-base-accent-preset-split.md) | Base and accent presets own disjoint variables; charts follow the accent | APPROVED 2026-10-26 by the owner ("Ok for the rest", backlog thread); for the next session (owner… | M | theme |
| [392](backlog/392-docs-snippets-compile.md) | Compile-check the docs.md code samples; samples use the uncontrolled props | BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in… | M | docs |
| [394](backlog/394-coming-from-shadcn-radix-tanstack.md) | A 'Coming from shadcn / Radix / TanStack' table on every component page | BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in… | M | docs |
| [399](backlog/399-docs-review-demos.md) | Demo batch from the docs review | BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in… | S | docs |
| [412](backlog/412-csp-meta-tag.md) | Docs site: a Content-Security-Policy `&lt;meta&gt;` tag | OPTIONAL: needs owner approval before it is adopted or scheduled (owner 2026-10-24). Not part of… | S | build |
| [413](backlog/413-property-tests-and-miri.md) | Property tests and Miri on the pure logic | OPTIONAL: needs owner approval before it is adopted or scheduled (owner 2026-10-24). Not part of… | M | tests |
| [414](backlog/414-formal-proof-spike.md) | Spike: formal proofs of one headless model (Kani, optionally Aeneas to Lean) | OPTIONAL: needs owner approval before it is adopted or scheduled (owner 2026-10-24). Not part of… | M | research |
| [415](backlog/415-pin-actions-to-shas.md) | Pin GitHub Actions to commit SHAs | OPTIONAL: needs owner approval before it is adopted or scheduled (owner 2026-10-24). Not part of… | XS | build |
| [422](backlog/422-docs-page-shared-section-group.md) | Component pages: group the sections every page shares under one header, component-specific sections under another | BACKLOG, outline approved by the owner 2026-10-25 (backlog thread): build it as below. Docs stage… | M | docs |
| [434](backlog/434-javascript-in-typescript-modules-not-rust-strings.md) | Cut JavaScript to what Rust cannot do: audit every eval block, move the rest into Rust, TypeScript only for the remainder | BACKLOG. Umbrella for rows 444-467 (the step 3 plan; the paper review is closed after seven rounds… | L | primitives |
| [435](backlog/435-one-menu-family-behind-three-hosts.md) | One menu-family implementation behind three hosts (DropdownMenu, ContextMenu, Menubar) | BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P10). Ordered before the JavaScript plan (owner… | L | primitives |
| [436](backlog/436-code-quality-skill-review.md) | Code-quality skill review: code-review and simplify skills over the whole codebase, after the row 426-435 refactors | BACKLOG. It was in the plan next to row 365 but never had a row, so 365 ran without it. The owner… | M for the review. Each accepted finding… | process |
| [438](backlog/438-blitz-native-renderer.md) | dioxus-native (Blitz) as a supported renderer: parked, research-gated, after row 434 | PARKED. Start when all of these hold: 1. Dioxus 0.8 is stable and the primitives build on it. 2.… | Estimates from the research: tier 0 M… | primitives |
| [440](backlog/440-run-gates-per-run-log-dir.md) | run-gates.sh writes logs to a shared directory: two concurrent runs on one target overwrite each other's logs | BACKLOG. Found by the row 433 lane (2026-10-26). | S | tooling |
| [442](backlog/442-desktop-cadence-harness-and-desktop-pass.md) | Desktop proof for the JS plan: a desktop-cadence web build, Playwright WebKit and Firefox, and a recorded desktop-pass stamp | BACKLOG. Filed 2026-10-26 from the row 434 step 3 adversarial board. The bridge-latency hook… | M | tooling |
| [443](backlog/443-owner-measurement-kit-perf-recorder-and-impl-switch.md) | The owner's measurement kit for the row 434 ports: __dxPerf, bridge delay and spin, and the ?dx-impl legacy switch | BACKLOG. Filed 2026-10-26 from the row 434 step 3 adversarial board. The bridge-delay hook is… | M | tooling |
| [444](backlog/444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) | Phase 0.1: the gate half of lane 3.0 (grant rows, sink and setter bans, FORBIDDEN_VERBS, gates over existing artefacts only) | APPROVED 2026-10-26 (owner: "okay for the plan"; scope 2, the recommended default, since no scope… | L | tooling |
| [445](backlog/445-phase-0-2-the-legacy-freeze.md) | Phase 0.2: freeze the legacy scripts and sinks (script hashes, per-site sink freeze, js-fix grant) | APPROVED 2026-10-26 (owner: "okay for the plan"; scope 2, the recommended default, since no scope… | S | tooling |
| [446](backlog/446-phase-0-6-witness-harness-and-the-first-witnesses.md) | Phase 0.6: the witness harness and the first witnesses | APPROVED 2026-10-26 (owner: "okay for the plan"; scope 2, the recommended default, since no scope… | L | tests |
| [447](backlog/447-phase-0-7-the-order-runtime.md) | Phase 0.7: the order runtime (inbound pump, per-slot epochs, vdom carrier, slot table, focus owner, one-shot scroll classifier) | APPROVED 2026-10-26 (owner: "okay for the plan"; scope 2, the recommended default, since no scope… | M | primitives |
| [448](backlog/448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md) | Phase 0.3: proof port A (the 16 dead not(web) eval sites, focus-trap.ts, and the otpFilter behaviour for S11) | APPROVED 2026-10-26 (owner: "okay for the plan"; scope 2, the recommended default, since no scope… | L | overlays |
| [449](backlog/449-phase-0-4-proof-port-b-the-executor-and-popover-spreads.md) | Phase 0.4: proof port B (the reconciled-kinds executor, the popover hooks that return the attribute to spread, the owners' onbeforetoggle handlers) | APPROVED 2026-10-26 (owner: "okay for the plan"; scope 2, the recommended default, since no scope… | L | overlays |
| [450](backlog/450-phase-0-5-dxbridgestats-and-the-latency-session.md) | Phase 0.5: __dxBridgeStats, a desktop smoke launch and the one latency session | APPROVED 2026-10-26 (owner: "okay for the plan"; scope 2, the recommended default, since no scope… | M | tooling |
| [451](backlog/451-port-3-sc-shared-collection-and-activity.md) | Port 3.SC: shared collection and activity scripts (S7, S15) | APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript… | S | primitives |
| [452](backlog/452-port-3-pl-placement-anchor-fallback-and-the-paint-safe-gate.md) | Port 3.PL: placement (T1, T2, T8-T12, S1), the paint-safe gate and the boot probe | APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript… | L | overlays |
| [453](backlog/453-port-3-pv-preview-glue-and-platform-rule-rows.md) | Port 3.PV: preview glue (V3-V23) with the V14 allowlist row and the Ctrl/Cmd+B platform rule row | APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript… | M | preview |
| [454](backlog/454-port-3-ob-observers-and-publish-option.md) | Port 3.OB: observers and the publish option (L27, S16, S14) | APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript… | S | primitives |
| [455](backlog/455-port-3-gs-gestures-slider-resizable-drawer-toast-pan.md) | Port 3.GS: gestures and toast (L21, L28-L30, O1, O3, O5 with the pan behaviour, V17) | APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript… | L | primitives |
| [456](backlog/456-port-3-ov-overlay-core-escape-dismiss-open-focus.md) | Port 3.OV: overlay core (L1-L3, L5-L9, L16-L17, L22, L26) with openFocus | APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript… | L | overlays |
| [457](backlog/457-port-3-vs-virtual-settle-and-the-scrollowner-behaviour.md) | Port 3.VS: virtual settle (W1-W7) and the scrollOwner one-shot kind | APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript… | M | virtual |
| [458](backlog/458-port-3-fc-form-and-input-primitives-and-the-native-slot-kinds.md) | Port 3.FC: form and input primitives, label click, and the native slot owners | APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript… | L | inputs |
| [459](backlog/459-port-3-sl-scroll-lock-l10-l11-l13.md) | Port 3.SL: scroll lock (L10, L11, L13) | APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript… | M | overlays |
| [460](backlog/460-port-3-hv-frame-rate-vdom-handlers.md) | Port 3.HV: frame-rate vdom handlers (the check-hot-events baseline) | BACKLOG. Deferred and conditional (round 7): the row 434 step 3 plan is approved (2026-10-26, scope… | M | primitives |
| [461](backlog/461-port-3-pw-pre-wasm-generation-from-one-ts.md) | Port 3.PW: generate the pre-wasm copies from one TypeScript file (L15, V1, V2) | APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript… | S | build |
| [462](backlog/462-port-roving-focus-78-set-focus-sites.md) | Port: roving focus (about 28 real DOM focus writes in 18 files through the focus owner), conditional on the focus-write witness | BACKLOG. Conditional (round 7): exists only if the phase-0 focus-write witness is red; the row 434… | L | primitives |
| [463](backlog/463-port-3-fz-freeze-legacy-scripts-behind-legacy-impl.md) | Measured group support: 3.FZ freeze (legacy message scroller and carousel scripts into ts/legacy behind legacy-impl) | APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript… | M | primitives |
| [464](backlog/464-port-3-ca-carousel-rust-arms-next-to-the-frozen-legacy.md) | Measured port 1: 3.CA carousel (C1-C4, C6, L20) as Rust arms next to the frozen legacy arm | APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript… | L | components |
| [465](backlog/465-port-3-ms-message-scroller-sticktoend-and-native-anchoring.md) | Measured port 2: 3.MS message scroller (the Rust invariant, native anchoring as accelerator, stickToEnd on every engine) | APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript… | L | components |
| [466](backlog/466-port-3-wb-wheel-band-pure-rust-classifier.md) | Measured port 3: 3.WB wheel band (C5 to a pure Rust classifier) | APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript… | M | components |
| [467](backlog/467-port-legacy-removal-keyed-to-the-verdict.md) | Measured group: legacy removal, keyed to each port's verdict.json | APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript… | S | primitives |
| [468](backlog/468-bug-form-reset-leaves-controlled-input-out-of-sync.md) | Bug: form reset leaves a controlled text input showing empty while Rust keeps the old value | APPROVED 2026-10-26 by the owner ("Ok for the rest", backlog thread); for the next session (owner… | S | inputs |
| [469](backlog/469-bug-text-typed-before-hydration-is-wiped-on-the-ssg-site.md) | Bug: text typed before hydration is wiped by the first post-hydration render on the SSG site | QUESTION TO OWNER 2026-10-26: the owner asked where typing before hydration happens naturally… | M | inputs |
| [471](backlog/471-bug-datatransfer-setters-are-no-ops-on-desktop.md) | Bug: DataTransfer setters in rsx drag handlers are silent no-ops on desktop | APPROVED 2026-10-26 by the owner ("Ok for the rest", backlog thread); for the next session (owner… | S | primitives |
| [472](backlog/472-chart-tooltip-fade.md) | Chart tooltips fade in and out with the theme's floating motion tokens | BACKLOG. Owner request 2026-10-26 (backlog thread): "chart tooltips should have a fade in/out… | S | components |
| [473](backlog/473-bug-date-picker-segments-let-the-browser-edit-the-vdom-text-node.md) | Bug: date picker segments let the browser edit the text node the vdom also writes | APPROVED 2026-10-26 by the owner ("Ok for the rest", backlog thread); for the next session (owner… | M | components |
| [474](backlog/474-bug-input-otp-paste-loses-digits-to-maxlength.md) | Bug: Input OTP paste loses digits because maxlength truncates before the filter runs | APPROVED 2026-10-26 by the owner ("Ok for the rest", backlog thread); for the next session (owner… | S | inputs |
| [475](backlog/475-bug-resizable-and-slider-drag-frames-leak-into-the-next-drag-on-desktop.md) | Bug: Resizable and Slider on desktop: a previous drag's queued per-move frames can land in the next drag | APPROVED 2026-10-26 by the owner ("Ok for the rest", backlog thread); for the next session (owner… | S | components |
| [476](backlog/476-ios-checkbox-and-radio-taps-lag-on-the-home-page.md) | iOS: checkbox and radio taps lag about a second on the home page (Filter results demo, Radio Group card) | BACKLOG. Owner report 2026-10-26 (iPhone); the next session reproduces with touch emulation and CPU… | M | components |
| [477](backlog/477-anchored-overlays-placed-in-document-coordinates.md) | Anchored overlays are placed in document coordinates (absolute top-layer panels, transform-only), CSS anchoring kept where supported | BACKLOG. Measured on the owner's iPhone (iOS 18.7 Safari) on 2026-10-28; the owner's verdict on the… | L | overlays |
| [478](backlog/478-overlay-defaults-to-base-ui.md) | Overlay and control defaults follow Base UI (menu check items, Combobox keys, Tooltip, NavigationMenu timing, Select, carousel RTL) | PARTIAL. Owner decided 2026-10-28: Select defaults to item-aligned, as both shadcn variants do.… | S each | overlays |
| [479](backlog/479-scroll-lock-method-and-the-touch-carve-out.md) | Scroll-lock method: Base UI's overflow:hidden with scrollbar-gutter, and no lock for touch-opened anchored popups | BACKLOG. Owner decided 2026-10-28: Menubar is not modal (no scroll lock), and anchored overlays… | M | overlays |
| [480](backlog/480-re-diff-keyboard-inset-against-base-ui-1-9-0.md) | Re-diff primitives/src/keyboard_inset.rs against Base UI 1.9.0 | PARTLY LANDED 2026-10-28 (lane keyboard-inset-480; device check pending). Two 1.6.0 -&gt; 1.9.0… | M | overlays |
| [481](backlog/481-state-attributes-one-helper-emits-data-state-and-presence.md) | State attributes: one helper emits data-state and Base UI presence attributes | BACKLOG. From the scan of 2026-10-28. | M | primitives |
| [482](backlog/482-control-gaps-slider-form-participation-read-only-and-small-attributes.md) | Control gaps: Slider form participation, read_only, Toggle data-pressed, Tabs activate_on_focus | IMPLEMENTED on lane/control-gaps-482, awaiting the merge batch (2026-10-28). Items 1 to 4 are… | S-M | components |

<!-- END GENERATED -->
