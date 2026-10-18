# General backlog -- index

One file per row. Everything outstanding across the plan, the harness and the follow-ups found during execution
lives in `dev-docs/backlog/`; **this file is only the generated index** over those files. A citation such as
"`dev-docs/backlog.md` row 116" (about 250 files and 530 comments carry one) resolves here: find the id below, or
glob `dev-docs/backlog/*116-*` and `dev-docs/backlog/done/*116-*`.

## How it works

- **Open rows:** `dev-docs/backlog/<NNN>-<slug>.md` (`NNN` = the zero-padded row id, never renumbered, ids are never reused).
- **Done rows:** the same file under `dev-docs/backlog/done/` once its status starts with LANDED / FIXED / RESOLVED /
  CLOSED / SUPERSEDED / DONE and names no open remainder. Anything partly done stays in the open set.
- **Narrative sections** of the old monolithic file (the "Fresh-session order" blocks, the 2026-09-18 queue
  evaluation, the "What a fresh session should pick up next" lists, the row 116 timing table) are kept verbatim in
  `dev-docs/backlog/notes/`. They are history; the current order is in
  [`research/backlog-order-2026-10-18.md`](./research/backlog-order-2026-10-18.md) and the live state in
  [`STATUS.md`](./STATUS.md).
- **A row file** has front matter (`id`, `title`, `status`, `size`, `area`, `updated`) and four sections, in this
  order: `## Item`, `## Status / trigger`, `## Size`, `## Detail` (the old table's Item, Trigger, Effort, Notes
  cells, verbatim). `title`, `area` and `updated` are hand-editable (`area` is a coarse tag, `updated` the latest
  date the row mentions); `status` and `size` are derived from their sections.
- Sub-ids such as "row 31a / 31b" or "item 21" in citations live inside that row's file (31, 21).
- **To add a row:** pick the next free id (the highest id in the tables below, plus one), copy any row file, rename it
  `<NNN>-<slug>.md`, fill the front matter and the four sections.
- **To update a row:** edit its sections (append dated notes to Detail, change Status / trigger when it lands). When it
  lands, `git mv` it into `dev-docs/backlog/done/`.
- **Then run `scripts/backlog-index.sh`** (syncs `status`/`size`, rewrites the block below) and commit the row file and
  this file together. `scripts/check-backlog-index.sh` (a `run-gates.sh` gate) fails on a stale index, a missing front
  matter field, a duplicate id, a row in both sets, and a broken relative link under `dev-docs/`.
- Never hand-edit between the GENERATED markers; edit this header freely.
- Provenance: split on 2026-10-18 from `dev-docs/backlog.md` at `de7059ba` (244 rows) by
  `scripts/backlog-split-once.py`; `scripts/backlog-split-verify.py` proves every cell and narrative line survived.
  Relative links inside moved text were rebased (`](./x)` became `](../x)` / `](../../x)`); nothing else was changed.

<!-- BEGIN GENERATED: scripts/backlog-index.sh rewrites everything up to END; edit the row files, not this block -->

## Open rows (88)

| # | Item | Status / trigger | Size | Area |
|---|---|---|---|---|
| [4](backlog/004-first-3-browser-ci-run-treat-as-calibration-not.md) | First 3-browser CI run — treat as calibration. Not done — see 2026-09-04 Notes below for what's now known and… | A manual dispatch of playwright.yml on a GitHub-hosted runner: all workflows are… | S–M | ci |
| [5](backlog/005-tier-2-harness-completion-rules-5-checkvalidity.md) | Tier-2 harness completion — rules 5 (checkValidity/reportValidity/willValidate), 7 (label association), 8… | Rule 7 landed 2026-09-19 (batch 3); rule 5 closed as subsumed; rule 8 re-scoped, still open | S (rule 8 remaining) | tests |
| [6](backlog/006-upstream-engagement-where-fixes-land-open.md) | Upstream engagement — where fixes land (open decision #1) + talking to upstream/fork authors (decision #4) | User decision | S to start | upstream |
| [10](backlog/010-phase-5-collision-detection.md) | Phase 5 — collision detection | 5.1 REFRAMED by the 2026-09-01 bits-ui/Radix source comparison, FLIP sub-problem landed same day… | S (flip, done) + S (JS-fallback shift… | theme |
| [14](backlog/014-nudge-dioxus-framework-issues-2962-onclose-4319.md) | Nudge Dioxus framework issues: #2962 (onclose), #4319 (dialog methods), ancestor-traversal gap (complaints.md) | User decision (outward-facing) | S | overlays |
| [15](backlog/015-scrollbar-gutter-on-older-webkit-ios-momentum.md) | scrollbar-gutter on older WebKit + iOS momentum scroll | A product decision that those targets matter | M | misc |
| [16](backlog/016-component-candidates-new-components-from.md) | Component candidates — new components from upstream PRs, the fork network, and catalog gaps | A decision to expand the catalog beyond shadcn (2026-10-05: the in-catalog part is done, see the… | S–XL per item | upstream |
| [22](backlog/022-ssg-lane-in-ci-run-the-playwright-suite-at.md) | SSG lane in CI — run the Playwright suite (at minimum oracle/hydration-parity.spec.ts) against the… | Executed for the first time 2026-09-19 (batch 3); CI job added, workflow_dispatch-only per the… | S | ci |
| [23](backlog/023-preview-s-desktop-feature-lacks-dioxus.md) | preview's desktop feature lacks dioxus-primitives/web — a webview-backed desktop build would render the… | FIXED 2026-09-26 (3633b20), verified at feature-resolution level only; the one remaining check is a… | S | overlays |
| [28](backlog/028-verify-the-2026-09-02-ios-keyboard-anchor.md) | Verify the 2026-09-02 iOS-keyboard anchor-tracking fix (use_anchor_position_fallback… | User re-test after deploy — the sandbox has no WebKit, and page.setViewportSize() cannot reproduce… | S to verify | overlays |
| [30](backlog/030-verify-the-2026-09-02-touch-text-entry-font.md) | Verify the 2026-09-02 touch text-entry font-size floor… | User re-test after deploy — the sandbox has no WebKit, and while Playwright's iPhone 13 emulation… | S to verify | tests |
| [31](backlog/031-design-tokens-option-a-split-in-two-user.md) | Design tokens (option A) — split in two (user decision 2026-09-03). 31a — LANDED 2026-09-04. Token set added… | 31a: landed 2026-09-04. 31b: landed 2026-09-04 | 31a S–M (done); 31b M (done) | tests |
| [33](backlog/033-3-browser-playwright-on-every-push-pr-sharded.md) | 3-browser Playwright on every push/PR, sharded — dignifiedquire/dx-components runs chromium+firefox+webkit… | CI reactivation (user: off until further notice, 2026-08-30) — then it is a workflow-trigger… | S | ci |
| [39](backlog/039-axe-color-contrast-landed-2026-09-03-row-34.md) | axe color-contrast — landed 2026-09-03 (row 34 follow-up), one residue filed. Measuring across 49 routes… | Residue FIXED 2026-09-26 (1e85dfa, a comment-token override in main.css) -- was: a decision to swap… | S (done) | tests |
| [40](backlog/040-auto-kind-popovers-exposure-to-the-opening.md) | Auto-kind popovers' exposure to the opening-gesture light-dismiss false positive — audited 2026-09-03; Navbar… | Navbar half FIXED 2026-10-04 (d9bcfda, verified on the release build of b9e9580); the rest CLOSED… | M (done) | tests |
| [41](backlog/041-navbarcontent-migrated-onto-the-top-layer.md) | NavbarContent migrated onto the top-layer engine (2026-09-03, finding C) without also picking up two… | 41a: landed 2026-09-04. 41b: CLOSED 2026-10-03 as a documented non-claim (no device needed; see the… | S | tests |
| [42](backlog/042-ios-18-safari-device-review-item-1-double-tap.md) | iOS 18 Safari device review, item 1 — double-tap-to-zoom on rapid button taps — landed 2026-09-03. Every… | — (landed) | S–M | tests |
| [43](backlog/043-ios-18-safari-device-review-item-2-top-layer.md) | iOS 18 Safari device review, item 2 — top_layer oracle fixture removed from the / home-page gallery — landed… | — (landed) | S | tests |
| [67](backlog/067-gates-found-red-on-unmodified-main-captured.md) | Gates found red on unmodified main, captured 2026-09-18 before any lane started | Found 2026-09-18; all four red gates FIXED (stylelint beb7cae, clippy fe066c6, rustdoc b01bcd5, the… | S–M | process |
| [68](backlog/068-cross-worktree-cargo-build-directory-hazard.md) | Cross-worktree cargo build-directory hazard | Found 2026-09-18, four independent occurrences | — | process |
| [69](backlog/069-axe-scan-readiness-race-playwright-axe-ts.md) | Axe scan readiness race (playwright/axe.ts), baseline group B | Found + fixed 2026-09-18 (construction); live verification confirmed round 2 | S | tests |
| [70](backlog/070-top-layer-oracle-fixture-probes-intercepted-by.md) | top_layer oracle fixture probes intercepted by the docs Sidebar, baseline group C | Found + fixed 2026-09-18 (construction); live verification confirmed round 2 | S | tests |
| [71](backlog/071-rule-11-self-overlap-regression-baseline-group-d.md) | Rule 11 self-overlap regression, baseline group D | Found + fixed 2026-09-18; live re-run against the integration server confirmed round 2 | S | tests |
| [72](backlog/072-follow-ups-surfaced-by-the-2026-09-18-lanes-not.md) | Follow-ups surfaced by the 2026-09-18 lanes, not built this round | Surfaced 2026-09-18 across lanes C, E, B and integration round 1 | — | process |
| [73](backlog/073-drawer-drag-hang-self-subscribing-use-effect.md) | Drawer drag hang — self-subscribing use_effect wedged the main thread on drag | Found in integration round 2 (2026-09-18), fixed round 3 same day | S | overlays |
| [74](backlog/074-navigation-menu-first-render-panic-hook-called.md) | Navigation Menu first-render panic (hook called inside another hook's closure) + demo/site-chrome… | Found in integration round 2 (2026-09-18), fixed round 3 same day | S | menus |
| [75](backlog/075-per-variant-component-preview-frame-duplicate.md) | Per-variant component-preview-frame duplicate id + unscoped Table caption locator (two strict-mode… | Found in integration round 2 (2026-09-18), fixed round 3 same day | S | a11y |
| [83](backlog/083-daterangepicker-page-hang-dateelement-s-sync.md) | DateRangePicker page hang — DateElement's sync-down/sync-up effects form an unconditional two-way reactive… | Found and fixed 2026-09-19 (batch 3, date-picker + range-hang lanes) | S | inputs |
| [92](backlog/092-new-component-landings-can-leave-preview-s-out.md) | New-component landings can leave preview's OUT_DIR stale — cargo clippy/test/doc fail until build.rs is… | Found 2026-09-20 (round 4 integration, Carousel landing onto a pre-existing target dir); recurs for… | S | build |
| [93](backlog/093-literal-attribute-beside-an-unmerged-props.md) | Literal attribute beside an unmerged props spread with no typed field claiming it — the duplicate-attribute… | Found 2026-09-20 (round 4, f2be1d7's own ..props.attributes/..attributes sweep of primitives/src/… | S–M for the check script itself… | overlays |
| [94](backlog/094-demo-wrappers-that-set-max-width-margin-0-auto.md) | Demo wrappers that set max-width + margin: 0 auto never reach that width — they shrink-wrap instead, so every… | Found 2026-09-20 while bringing Carousel to shadcn visual parity (round 5). Surfaced by a parity… | S per wrapper (add width: 100%); all… | charts |
| [95](backlog/095-a-debug-ssg-build-serves-fine-but-traps-at.md) | A debug SSG build serves fine but traps at runtime, so interaction specs run against it fail like real… | Hit 2026-09-20 (round 5) running carousel.spec.ts against a local static serve of dx build --ssg… | DONE — warning added directly to the… | build |
| [97](backlog/097-a-second-un-fragmented-rsx-sibling-renders-on.md) | A second, un-fragmented rsx sibling renders on the first pass and is silently dropped by the next one | Found 2026-09-20 (round 5) adding a scoped &lt;style&gt; tag beside CarouselContent's track | DONE for this instance (explicit… | misc |
| [98](backlog/098-dx-build-ssg-fails-with-no-such-file-or.md) | dx build --ssg fails with No such file or directory (os error 2) when CARGO_TARGET_DIR is a relative path | Hit 2026-09-20 (round 5) by a lane following this repo's own CARGO_TARGET_DIR=target-lane-&lt;name&gt;… | Workaround is one word: make the path… | process |
| [99](backlog/099-anchor-name-style-set-as-a-bare-literal-beside.md) | anchor_name_style set as a bare literal beside a spread — a duplicate-style class across the anchored-trigger… | Found 2026-09-21 by Rule 4c on its first run (NavigationMenuTrigger), then widened to the whole… | DONE — all 9 sites now route through… | overlays |
| [100](backlog/100-a-no-base-path-build-made-in-the-shared-repo.md) | A no---base-path build made in the SHARED repo-root target/ still emits /dioxus-components/ prefixed markup —… | Hit 2026-09-22 (round 7): all 51 carousel.spec.ts tests reached went red on code that was green an… | Three rebuilds wasted on a stale-output… | misc |
| [101](backlog/101-oracle-tier1-apg-carousel-spec-ts-224-axe-scan.md) | oracle/tier1-apg/carousel.spec.ts:224 axe scan fails on the W3C REFERENCE fixture, not on our component | Seen 2026-09-22 (round 7) in a combined 68-test run; carousel.spec.ts alone is 57/57 green. FIXED… | Unknown; not yet triaged | tests |
| [103](backlog/103-native-sub-scroller-rubber-band-is-real-but.md) | Native sub-scroller rubber-band is real but composited, so it is invisible to… | Measured 2026-09-23 with a script-free scroll-snap control added to the bench; the owner corrected… | N/A — a measurement limit to know, not… | inputs |
| [104](backlog/104-instrumentation-that-keys-off-the-wrong-event.md) | Instrumentation that keys off the wrong event, or records a threshold after it is reset, lies exactly where… | Two telemetry defects found 2026-09-22 (bench v22) while chasing a B2 glitch that the data appeared… | Done for the bench; worth remembering… | misc |
| [105](backlog/105-playwright-baseline-local-config-ts-and-xvfb.md) | playwright/baseline.local.config.ts (and xvfb.local.config.ts/oracle.local.config.ts/ssg.local.config.ts)… | Found 2026-09-25 (round 8) in every full local run this session; reproduced identically on f103bbd… | DONE 2026-09-25 (round 8) | ci |
| [106](backlog/106-this-container-s-chromium-is-1194-no-1223.md) | This container's Chromium is 1194 (no 1223, unlike some earlier sandboxes this repo's docs mention) — 15… | Found 2026-09-25 (round 8) running the full gate suite | Environment/engine class, not a product… | tests |
| [108](backlog/108-tests-that-locate-rendered-content-by-slicing.md) | Tests that locate rendered content by slicing an HTML string from a specific attribute-string match break… | Found 2026-09-25 (round 8) while landing row 93; two occurrences in one round makes it a class per… | BUILT 2026-09-26 (dab751c); was… | tests |
| [109](backlog/109-ssg-interaction-tests-race-hydration-a-click.md) | SSG interaction tests race hydration — a click can land before wasm attaches Dioxus's delegated listeners; a… | Found 2026-09-25 (round 8b) in the final integrated verification at b557dfa: an SSG run of… | S — one client-side effect, one shared… | build |
| [110](backlog/110-playwright-s-own-actionability-pre-scroll-hover.md) | Playwright's own actionability pre-scroll (.hover()/.focus()/.click()'s auto-scroll) animates under the… | Instance 2, 2026-09-25 (round 8b): carousel.spec.ts's "hovering the carousel stops rotation…"… | Diagnosis M (two hypotheses tried and… | tests |
| [111](backlog/111-migrate-the-theme-to-shadcn-s-system-semantic.md) | Migrate the theme to shadcn's system: semantic role tokens, then the Nova style | PHASE C LANDED 2026-10-14; decision 2 APPROVED 2026-10-14 (shadcn white 10%/15%; row 203, not yet… | M (phase A) + S/M (phase B foundations)… | theme |
| [112](backlog/112-carousel-tests-that-sample-a-live-animation.md) | Carousel tests that sample a live animation across an async gap -- both fixed; plus the B3 wheel release… | Found 2026-09-26 while verifying the carousel shadcn-parity merge (22d4c95). Instance 1, FIXED… | done (instance 2 fixed d44a12d; the… | theme |
| [115](backlog/115-tooltip-spec-ts-6-tabbing-out-of-the-trigger.md) | tooltip.spec.ts:6 "tabbing out of the trigger element should hide the tooltip" failed once in a full-suite… | Seen 2026-09-28 in the mode-D integration run (--workers=4, SSG release build): the tooltip was… | S (diagnosis, when it recurs) | tests |
| [124](backlog/124-charts-a-small-pre-hydration-layout-jump-on-the.md) | Charts: a small pre-hydration layout jump on the legend cards, and on pie-interactive's Select (SSR renders… | OPEN -- measured 2026-10-03 in dev-docs/research/chart-parity-2026-10-03/visual-parity.md (SSR… | S-M | charts |
| [125](backlog/125-charts-visual-nits-left-after-the-parity-round.md) | Charts visual nits left after the parity round: radar legend ~12px lower than shadcn; pie-interactive Select… | OPEN -- cosmetic, listed in dev-docs/research/chart-parity-2026-10-03/visual-parity.md; not covered… | S | charts |
| [126](backlog/126-link-s-aria-current-can-emit-a-duplicated.md) | Link's aria_current can emit a duplicated attribute under a base path; the charts gallery tab row uses plain… | LANDED 2026-10-11 (root-caused and fixed by construction; one library residue below) -- the… | S (diagnosis) | charts |
| [130](backlog/130-arborium-2-17-0-s-wasm-libc-shim-lacks-the-data.md) | arborium 2.17.0's wasm libc shim lacks the data symbol stderr that tree-sitter's stack.c… | OPEN (upstream) -- local fix 7ad8421 (a stderr definition in preview); an upstream issue is worth… | S | upstream |
| [131](backlog/131-dioxus-cli-0-7-9-ssg-leaves-every-pre-rendered.md) | dioxus-cli 0.7.9 --ssg leaves every pre-rendered route except index.html stale when public/ already exists | OPEN (upstream) -- locally closed by construction (150e97b, scripts/build-ssg.sh); upstream issue… | S | build |
| [136](backlog/136-questionnaire-shadcn-s-multi-step-question-flow.md) | Questionnaire: shadcn's multi-step question flow (single/multiple choice, freeform "other", skippable… | ON HOLD (owner 2026-10-14: not started, no schedule). BACKLOG -- recorded 2026-10-04 from the… | L | inputs |
| [137](backlog/137-rtl-by-construction-one-row-for-the-three-rtl.md) | RTL by construction -- one row for the three RTL threads: row 13's open follow-ups (optional RTL for… | BACKLOG, MERGED 2026-10-05 (owner decision); the owner then CONFIRMED the solve-by-construction… | M (the docs page alone is S; parts a… | docs |
| [138](backlog/138-toast-depth-parity-loading-type-action-button.md) | Toast depth parity: loading type, action button, promise helper, update/close by id, stacking and… | BACKLOG -- recorded 2026-10-04 from the catalog audit (§6.4); depth, not a catalog gap (toast is… | M-L (was M; the folded-in items add an… | overlays |
| [146](backlog/146-message-scroller-stage-2-visibility-hook-per.md) | message_scroller stage 2: visibility hook, per-call scroll_margin, RTL pass, release-build timing spec | FOLDED INTO ROW 213 (2026-10-14) -- stage 1 built 2026-10-04 (own lane, component-backlog.md)… | M | tests |
| [147](backlog/147-safari-18-25-cannot-find-in-page-text-inside.md) | Safari 18-25 cannot find-in-page text inside content-visibility: auto-skipped content (WebKit bug 283846) | NOTE -- no behaviour change proposed; recorded 2026-10-04 from the virtualization survey (section 5) | S | virtual |
| [148](backlog/148-a-chat-sidebar-category-for-the-chat-kit-and.md) | A "Chat" sidebar category for the chat kit and message_scroller | OPEN -- recorded 2026-10-04 | S | virtual |
| [150](backlog/150-home-page-blocks-registered-their-hooks-on-the.md) | Home-page blocks registered their hooks on the parent scope: MasonryCard called component.call(()), so… | FIXED 2026-10-04, merged (b83640a, PR #84 / 001a5f0, SCROLL-FIX lane); the candidate .call(()) gate… | M | docs |
| [152](backlog/152-test-suite-review-revisit-the-playwright-suite.md) | Test-suite review: revisit the Playwright suite (~1,786 tests, ~21 min on a release build with 4 workers) for… | OPEN -- owner note 2026-10-05; nothing was run for this row, the evidence below is read from the… | M | tests |
| [153](backlog/153-native-find-in-page-for-windowed-virtual-lists.md) | Native find-in-page for WINDOWED virtual lists: keep per-item text in the DOM as hidden="until-found"… | OPEN (research) -- owner idea 2026-10-05; nothing prototyped | M | virtual |
| [154](backlog/154-shared-virtualization-engine-one-engine-in.md) | Shared virtualization engine: ONE engine in primitives/src/virtual/ (core, strategy, DOM bridge) that every… | DESIGN NOTE -- first note recorded 2026-10-05 from the owner question whether the virtualized… | S (note); the build is not estimated | virtual |
| [155](backlog/155-shadcn-parity-wave-group-1-the-invalid-ring.md) | shadcn parity wave (group 1, the invalid ring, LANDED 2026-10-13 with row 145's state recipes): the 27 "Add… | LANDED: group 1 2026-10-13 (with row 145's state recipes), group 7's Sheet side 2026-10-14 (rows… | M-L in total (23 of the 27 are S; four… | theme |
| [156](backlog/156-drawer-has-fallen-behind-shadcn-it-moved-from.md) | Drawer has fallen behind shadcn: it moved from Vaul to Base UI Drawer (snap points, nested drawers, non-modal… | 2026-10-15: batch 2 did NOT take the rest; still open: snapPoints, the non-modal mode and the… | L | overlays |
| [157](backlog/157-inputotp-hard-codes-a-numeric-keypad-inputmode.md) | InputOtp hard-codes a numeric keypad: inputmode="numeric" and pattern="[0-9]*" are fixed, so alphanumeric… | COLD (owner decision 2026-10-05): not scheduled, NEEDS OWNER APPROVAL before anyone picks it up.… | S-M (the comparison rates it M) | inputs |
| [158](backlog/158-theme-follow-ups-accept-shadcn-s-dark-class-as.md) | Theme follow-ups: accept shadcn's .dark class as an alias of data-theme="dark", and suppress transitions… | PART (a) LANDED 2026-10-15 (by construction, lane x158dark 7d16044 + 3a3a9fa; owner approved… | S + S | theme |
| [161](backlog/161-rename-the-website-to-shadcn-dioxus-and-give-it.md) | Rename the website to "shadcn-dioxus" and give it a real favicon (an SVG that follows light/dark, a PNG… | IN PROGRESS (owner decision 2026-10-10) -- the site brand and favicon are being changed now (BRAND… | S-M | docs |
| [162](backlog/162-self-host-the-docs-site-s-fonts-no-runtime.md) | Self-host the docs site's fonts (no runtime requests to Google) and give the library --dx-font-sans /… | OWNER DECIDED 2026-10-05: option (a), self-host Geist + Geist Mono (~52.5 KB). Not started, except… | S-M | docs |
| [164](backlog/164-a-fixtures-goto-that-waits-for-hydration-49.md) | A fixtures goto that waits for hydration (49 spec files lack the wait), and migrate the 362 legacy ?name=… | OPEN -- recorded 2026-10-05 (audit); scheduled in the next-session order (step 4, just before row… | M (the fixture is S; the migration is… | tests |
| [165](backlog/165-a-generated-props-reference-table-on-every.md) | A generated props reference table on every component's docs page, produced in preview/build.rs (~100 public… | OPEN -- recorded 2026-10-05 (audit); scheduled in the next-session order (step 4, after the… | M (the generator is S-M; writing ~100… | build |
| [167](backlog/167-migrate-the-52-hand-paired-listener-sites-onto.md) | Migrate the 52 hand-paired listener sites onto use_js_listeners, shrinking PAIRED_BY_HAND in… | OPEN -- recorded 2026-10-05 (follow-up of row 159, which landed 2026-10-05 in round 3); scheduled… | M (mechanical per site, 11 files) | misc |
| [170](backlog/170-cross-component-css-overrides-190-sites-in-12.md) | Cross-component CSS overrides: 190 sites in 12 sheets declare, in one component's stylesheet, a property that… | OPEN (construction started) -- recorded 2026-10-05 (round 3: the FIELD-VAR and STYLE-DELIVERY… | M (the decision and the prototype are… | theme |
| [174](backlog/174-home-overview-page-scroll-jank-re-profiled-2026.md) | Home/overview page scroll jank, re-profiled 2026-10-10 | PROFILED (partial, stopped by the owner) -- 2026-10-10; causes 2 and 4 LANDED 2026-10-10 (rows 177… | S to M per fix (order below) | misc |
| [186](backlog/186-shadcn-utilities-scroll-fade-and-shimmer-are.md) | shadcn utilities: scroll-fade and shimmer are built but not shipped as documented, public utilities | PART LANDED 2026-10-15 (the scroll-fade fade prop, owner-approved 2026-10-14); BACKLOG: the /docs… | M | docs |
| [187](backlog/187-demo-upgrades-25-components-whose-shadcn-demo.md) | Demo upgrades: 25 components whose shadcn demo is clearly richer than ours | BACKLOG -- owner item 2026-10-11, from a comparison of 61 shadcn pages (25 richer there, 36 equal… | S mostly (20 of 25 S; 5 M: combobox… | docs |
| [188](backlog/188-demo-parity-second-audit-the-owner-s-per.md) | Demo parity, second audit: the owner's per-example list against the original shadcn demos (51 components; row… | BATCHES 1 AND 2 LANDED (2026-10-11: 86 demos in 20 components; 2026-10-12: 43 demos in 30… | M per batch | docs |
| [213](backlog/213-message-scroller-parity-with-the-original-folds.md) | Message Scroller parity with the original (folds row 146) | OWNER DECISIONS 2026-10-15 (markdown, the nine questions): the recommendations -- pulldown-cmark… | M-L | virtual |
| [216](backlog/216-infra-worktrees-sharing-or-copying-one-cargo.md) | Infra: worktrees sharing or copying one CARGO_TARGET_DIR build and lint another tree's code (re-confirms row… | OPEN 2026-10-14 (workaround in CLAUDE.md; by-construction fix proposed) | S-M | process |
| [217](backlog/217-dependency-audit-follow-ups-what-users-pay-for.md) | Dependency audit follow-ups (what users pay for in compile time and supply chain) | PARTLY LANDED 2026-10-18 (lane deps 04ad7ddf): features calendar/color-picker, no build script… | S-M each | build |
| [218](backlog/218-popup-style-isolation-top-layer-popups-stay-dom.md) | Popup style isolation: top-layer popups stay DOM descendants of their trigger, so free-content rules in an… | PARTLY LANDED 2026-10-15 (lane ancestorleak 900f394 + main-loop 526b51d, beaf3b0); open: the… | M | overlays |
| [221](backlog/221-infra-session-process-lessons-from-the-2026-10.md) | Infra: session process lessons from the 2026-10-14/15 batch (memory, lock fairness, gate worktree… | OPEN 2026-10-15 (rules in force in the lane brief; constructions proposed) | S-M | process |
| [237](backlog/237-carousel-a-free-scrolling-switch-embla-dragfree.md) | Carousel: a free-scrolling switch (Embla dragFree parity) | BACKLOG (owner 2026-10-18) | M | carousel |
| [239](backlog/239-all-unset-outranks-the-theme-s-zero-specificity.md) | all: unset outranks the theme's zero-specificity baselines | BACKLOG (found 2026-10-18 by lane baseline) | S-M | theme |
| [240](backlog/240-base-ui-focus-model-what-to-lift-beyond-row-226.md) | Base UI focus model: what to lift beyond row 226 | BACKLOG (research 2026-10-18) | S-M each | a11y |
| [241](backlog/241-specs-still-read-block-demo-rows-from-a-mirror.md) | Specs still read block-demo rows from a mirror in components/mod.rs | BACKLOG (lane registry 2026-10-18) | S | tests |
| [242](backlog/242-spec-failures-that-are-red-on-the-deployed.md) | Spec failures that are red on the deployed baseline too (found by the 2026-10-18 lanes) | OPEN | S each | process |
| [243](backlog/243-button-hover-fills-3-levels-off-shadcn.md) | Button hover fills 3 levels off shadcn | BACKLOG (lane docs-site 2026-10-18) | S | theme |
| [244](backlog/244-virtualgrid-x-y-virtualization-that-windows.md) | VirtualGrid: x/y virtualization that windows rows AND columns (a data grid, an image grid) | BACKLOG (owner question 2026-10-18) | L | virtual |

## Done rows (156)

| # | Item | Updated |
|---|---|---|
| [1](backlog/done/001-scroll-lock-scrollbar-gutter-fix-permanent.md) | Scroll-lock scrollbar-gutter fix (permanent scrollbar-gutter: stable baseline) | 2026-08-31 |
| [2](backlog/done/002-phase-4-4-popover-popover-auto-top-layer-for.md) | Phase 4.4 — popover/popover="auto" top layer for non-modal overlays | unknown |
| [3](backlog/done/003-phase-4-2-native-dialog-showmodal-for-dialog.md) | Phase 4.2 — native &lt;dialog&gt;+showModal() for Dialog and AlertDialog | 2026-09-01 |
| [7](backlog/done/007-wervice-tooltip-fade-out-a704c517-upstream-pr.md) | Wervice tooltip fade-out (a704c517, upstream PR #293) | 2026-09-18 |
| [8](backlog/done/008-select-trigger-apg-select-only-combobox-role.md) | Select trigger → APG select-only-combobox role (role="combobox") | 2026-09-18 |
| [9](backlog/done/009-select-listbox-scroll-lock.md) | Select listbox scroll lock | 2026-09-18 |
| [11](backlog/done/011-phase-6-typeahead-for-menus-dq-typeahead-rs-not.md) | Phase 6 — typeahead for menus (dq typeahead.rs, not select/) | 2026-09-18 |
| [12](backlog/done/012-phase-6-pub-mod-portal-toggle-class-prop-public.md) | Phase 6 — pub mod portal, Toggle class prop, public CalendarDayState (upstream #199) | 2026-09-18 |
| [13](backlog/done/013-phase-6-rtl-dq-direction-rs-key-flip-concept.md) | Phase 6 — RTL (dq direction.rs + key-flip concept) | 2026-10-05 |
| [17](backlog/done/017-mutation-testing-cargo-mutants-scoped-to-the.md) | Mutation testing (cargo-mutants), scoped to the algorithmic modules that already carry unit tests (slider… | 2026-08-31 |
| [18](backlog/done/018-css-anchor-positioning-cross-browser-fallback.md) | CSS Anchor Positioning cross-browser fallback for anchored overlays | 2026-09-01 |
| [19](backlog/done/019-popover-tooltip-hovercard-close-fade-animation.md) | Popover/Tooltip/HoverCard close-fade animation cut short on the web (popover) arm | 2026-09-18 |
| [20](backlog/done/020-date-picker-e2e-spec-none-existed-as-of-2026-08.md) | date_picker e2e spec — none existed as of 2026-08-31 (calendar.spec.ts does not cover DatePicker) | 2026-09-19 |
| [21](backlog/done/021-residual-env-only-anchor-flake-top-layer.md) | Residual env-only anchor flake: top-layer ColorPicker CSS-path test fails ~40% in this sandbox (initial… | 2026-10-05 |
| [24](backlog/done/024-dropdownmenu-renders-role-listbox-role-option.md) | DropdownMenu renders role="listbox" / role="option" (upstream's original markup) where the APG menu-button… | 2026-09-19 |
| [25](backlog/done/025-contextmenu-s-and-menubar-s-submenu-role-menu.md) | ContextMenu's and Menubar's submenu role="menu" popups carry no aria-labelledby/aria-label at all (grepped… | 2026-09-03 |
| [26](backlog/done/026-not-reproduced-re-investigated-2026-09-04-first.md) | NOT REPRODUCED — re-investigated 2026-09-04, first session with all three engines available. Original claim… | 2026-09-04 |
| [27](backlog/done/027-shadcn-catalog-parity-round-scoped-to-the.md) | shadcn catalog parity round — scoped to the styled-only gaps (user decision 2026-09-03): Alert, Breadcrumb… | 2026-09-04 |
| [29](backlog/done/029-color-picker-spec-ts-s-clicking-outside-closes.md) | color-picker.spec.ts's "clicking outside closes the color picker popover" got measurably flakier after the… | 2026-10-04 |
| [32](backlog/done/032-styling-engine-decided-2026-09-03-landed-2026.md) | Styling engine — DECIDED 2026-09-03, LANDED 2026-09-04: (b) plain dx--prefixed CSS files without css_module… | 2026-09-04 |
| [34](backlog/done/034-axe-scans-on-more-specs-3-of-our-32-component.md) | axe scans on more specs — 3 of our 32 component specs run axe vs 12 of their 56; add an axe pass to every… | 2026-09-03 |
| [35](backlog/done/035-use-dignifiedquire-dx-components-5af3cc2-2026.md) | Use dignifiedquire/dx-components (5af3cc2, 2026-08-21, idle since; nothing upstreamed) as a references —… | 2026-09-26 |
| [36](backlog/done/036-selecttrigger-s-caller-overridable-id-was-never.md) | SelectTrigger's caller-overridable id was never resynced into ctx.selectable.trigger_id, the shared context… | 2026-09-04 |
| [37](backlog/done/037-dashboard-email-client-s-message-rows-listpane.md) | Dashboard email client's message rows (ListPane, preview/src/dashboard/views/email_client/list_pane.rs)… | 2026-09-04 |
| [38](backlog/done/038-the-component-block-name-x-variant-route-family.md) | The /component/block/?name=X&amp;variant=... route family had no page-level &lt;h1&gt;/&lt;main&gt; of its own when visited… | 2026-09-04 |
| [44](backlog/done/044-top-layer-fixture-add-toast-appeared-dead-on.md) | top_layer fixture: "Add toast" appeared dead on iOS 18 (device report 2026-09-03). Root-caused last round… | 2026-09-04 |
| [45](backlog/done/045-main-thread-responsiveness-oracle-inp-long.md) | Main-thread responsiveness oracle (INP / long tasks). Install a PerformanceObserver for longtask (and event… | 2026-09-19 |
| [46](backlog/done/046-every-ssg-prerendered-component-page-is-the.md) | Every SSG-prerendered component page is the "Component not found" shell. dx build --ssg prerenders… | 2026-09-19 |
| [47](backlog/done/047-styled-select-listbox-rendered-full-viewport.md) | styled Select listbox rendered full viewport width on the home page gallery — the "dropdown menu has full… | 2026-09-04 |
| [48](backlog/done/048-the-stylelint-gate-was-red-and-hiding-a-real.md) | the stylelint gate was red and hiding a real defect. cd preview &amp;&amp; npx stylelint "/*.css" --max-warnings=0… | 2026-09-04 |
| [49](backlog/done/049-pinneartop-scroll-helper-bug-in-top-layer-spec.md) | pinNearTop scroll-helper bug in top-layer.spec.ts, found on first execution. Rule 11(c)'s Popover case — the… | 2026-09-04 |
| [50](backlog/done/050-top-layer-overlays-inherited-the-ua-s-ink.md) | top-layer overlays inherited the UA's ink colour instead of the app's, sitewide. User-reported against the… | 2026-09-04 |
| [51](backlog/done/051-calendar-s-stylesheet-does-not-load-on-the-date.md) | Calendar's stylesheet does not load on the date_picker route — LANDED 2026-09-04. Found incidentally during… | 2026-09-04 |
| [52](backlog/done/052-menu-family-role-consolidation-stage-1-landed.md) | Menu-family role consolidation, stage 1 — LANDED 2026-09-04. menu_semantics exists, in its own module doc's… | 2026-09-04 |
| [53](backlog/done/053-menu-family-submenu-consolidation-stage-2.md) | Menu-family submenu consolidation, stage 2 — LANDED 2026-09-14, verified by execution. Following stage 1 (row… | 2026-09-14 |
| [54](backlog/done/054-nested-submenus-dropdownmenu-sub-contextmenu.md) | Nested submenus (DropdownMenu.Sub/ContextMenu.Sub) — LANDED 2026-09-04, verified by execution. Closes the… | 2026-09-04 |
| [55](backlog/done/055-sidebar-right-side-mirroring-silently-no-op-d.md) | Sidebar right-side mirroring silently no-op'd when collapsible: SidebarCollapsible::None — LANDED 2026-09-15… | 2026-09-15 |
| [56](backlog/done/056-docs-site-now-dogfoods-the-real-sidebar.md) | Docs site now dogfoods the real Sidebar component as its own navigation — LANDED 2026-09-15, user-requested.… | 2026-09-15 |
| [57](backlog/done/057-contextmenu-dropdownmenu-submenus-closed-the.md) | ContextMenu/DropdownMenu submenus closed the instant the pointer left the trigger toward the submenu's own… | 2026-09-15 |
| [58](backlog/done/058-sidebar-block-demo-iframe-rendered-empty-on.md) | Sidebar block-demo iframe rendered empty on live production; homepage Sidebar card now exposes live config… | 2026-09-15 |
| [59](backlog/done/059-block-demo-iframe-s-row-58-min-width-820px-fix.md) | Block-demo iframe's row-58 min-width: 820px fix itself broke real mobile viewports; homepage nav now closed… | 2026-09-17 |
| [60](backlog/done/060-input-otp-almost-no-hit-target-for-the-input.md) | Input OTP: almost no hit target for the input — LANDED 2026-09-17, user-reported. Not a geometry/sizing bug… | 2026-09-17 |
| [61](backlog/done/061-sidebar-toggle-icon-lived-in-its-own-strip.md) | Sidebar toggle icon lived in its own strip below the main site navbar instead of inside it — LANDED… | 2026-09-17 |
| [62](backlog/done/062-docs-sidebar-s-start-group-had-a-single.md) | /docs sidebar's "Start" group had a single "Overview" link and no icons anywhere in the site nav — LANDED… | 2026-09-17 |
| [63](backlog/done/063-input-otp-click-to-caret-positioning-did-not.md) | Input OTP click-to-caret positioning did not generalize past the first slot — LANDED 2026-09-17… | 2026-09-17 |
| [64](backlog/done/064-input-otp-clicking-a-slot-and-typing-didn-t.md) | Input OTP: clicking a slot and typing didn't correct that digit — LANDED 2026-09-17, user-reported. Row 63… | 2026-09-17 |
| [65](backlog/done/065-homepage-s-own-sections-had-no-sidebar-jump.md) | Homepage's own sections had no sidebar jump-links — LANDED 2026-09-18, user-requested (same feature as row… | 2026-09-18 |
| [66](backlog/done/066-upstream-dioxuslabs-components-main-moved-for.md) | Upstream DioxusLabs/components main moved for the first time since 2026-06-29 — three commits on… | 2026-09-18 |
| [76](backlog/done/076-dev-loop-measurements-and-tooling-dev-wait-sh.md) | Dev-loop measurements and tooling (dev-wait.sh, inspect.mjs, base-url.ts) | 2026-09-18 |
| [77](backlog/done/077-user-reported-ui-round-2026-09-18-19-batch-2.md) | User-reported UI round (2026-09-18/19, batch 2): Command palette focus, Data Table filter, Tag Group sizing… | 2026-09-18 |
| [78](backlog/done/078-the-native-dialog-un-freed-opposite-edge.md) | The native &lt;dialog&gt; un-freed-opposite-edge centering class | 2026-09-18 |
| [79](backlog/done/079-reactivity-guards-check-hooks-in-closures-sh.md) | Reactivity guards: check-hooks-in-closures.sh + check-self-subscribing-effects.sh | 2026-09-18 |
| [80](backlog/done/080-css-hot-reload-root-cause-a-compile-time-embed.md) | CSS hot reload root cause: a compile-time embed, not the folder location | 2026-09-18 |
| [81](backlog/done/081-fixtures-72-follow-ups-base-url-migration.md) | Fixtures-72 follow-ups, base-URL migration completion, and three full-suite-only test-timing fixes | 2026-09-18 |
| [82](backlog/done/082-batch-2-integration-outcome-full-suite-775-12.md) | Batch-2 integration outcome: full suite 775/12/11, net 0 unaddressed regressions | 2026-09-19 |
| [84](backlog/done/084-date-picker-a11y-ux-gaps-found-but-not-fixed.md) | Date picker — a11y/UX gaps found but not fixed this round | 2026-09-20 |
| [85](backlog/done/085-dropdownmenu-contextmenu-a-raw-dom-focus-on-a.md) | DropdownMenu/ContextMenu — a raw DOM .focus() on a sub-trigger closes the whole menu | 2026-09-20 |
| [86](backlog/done/086-chart-and-carousel-decisions-researched.md) | Chart and Carousel — decisions researched, awaiting the user (SUPERSEDED: both decided and landed, rows 88… | 2026-09-26 |
| [87](backlog/done/087-batch-3-integration-outcome-full-suite-898-6-12.md) | Batch-3 integration outcome: full suite 898/6/12, 0 unaddressed regressions | 2026-09-19 |
| [88](backlog/done/088-chart-landed-in-repo-pure-rsx-svg-engine-engine.md) | Chart landed — in-repo pure-RSX SVG engine, engine/layer seam for upstreaming, two integration a11y/visual… | 2026-10-03 |
| [89](backlog/done/089-selected-active-state-losing-to-hover-on-click.md) | Selected/active state losing to :hover on click — a user-reported hover-over-state specificity class, swept… | 2026-09-20 |
| [90](backlog/done/090-chart-stage-2-per-chart-type-galleries-shadcn.md) | Chart stage 2 — per-chart-type galleries (shadcn parity), parked mid-flight (LANDED 2026-09-25) | 2026-09-25 |
| [91](backlog/done/091-carousel-landed-css-scroll-snap-engine-tier-1.md) | Carousel landed — CSS scroll-snap engine, tier-1 APG oracle vendored, three live-verification fixes | 2026-09-20 |
| [96](backlog/done/096-scroll-snap-stop-always-caps-a-browser-driven.md) | scroll-snap-stop: always caps a browser-driven scroll but NOT this crate's own pointer-drag release | 2026-09-20 |
| [102](backlog/done/102-port-the-carousel-edge-rubber-band-into.md) | Port the carousel edge rubber-band into primitives/ as mode B3 (edge transform) — LANDED 2026-09-25 (round… | 2026-09-28 |
| [107](backlog/done/107-two-fresh-container-cold-start-traps-in-scripts.md) | Two fresh-container cold-start traps in scripts/check-attr-spread-collision.sh and its --update-baseline mode | 2026-10-11 |
| [113](backlog/done/113-calendar-spec-ts-364-a-range-s-in-between-day.md) | calendar.spec.ts:364 "a range's in-between day keeps its muted look while hovered" times out (90s) on every… | 2026-10-05 |
| [114](backlog/done/114-drag-and-drop-list-spec-ts-lifecycle-tests-284.md) | drag_and_drop_list.spec.ts lifecycle tests (:284 mouse drop to the side, :300 cancelled mouse drag) compare… | 2026-10-05 |
| [116](backlog/done/116-build-dev-loop-efficiency-release-ssg-build.md) | Build/dev loop efficiency: release SSG build takes ~18 min; lanes rebuild cold | 2026-10-04 |
| [117](backlog/done/117-chart-display-docs-ux-round-series-colors-were.md) | Chart display + docs UX round: series colors were black/grey/invisible on every SSG page, plus a docs-site UX… | 2026-10-03 |
| [118](backlog/done/118-shadcn-style-charts-kind-gallery-and-a-pointer.md) | shadcn-style /charts/&lt;kind&gt;/ gallery and a pointer-following tooltip (PR #80) | 2026-10-03 |
| [119](backlog/done/119-recharts-shadcn-chart-projection-parity-sizing.md) | Recharts/shadcn chart projection parity -- sizing, scales, curves, polar geometry, shadcn-sourced demos… | 2026-10-03 |
| [120](backlog/done/120-duplicate-id-dnd-instructions-on-the-home-page.md) | Duplicate id="dnd-instructions" on the home page: DragAndDropList hard-codes the instructions id | 2026-10-05 |
| [121](backlog/done/121-navigation-menu-spec-ts-128-the-content-s-width.md) | navigation_menu.spec.ts:128 "the content's width never changes while its open animation plays" fails on every… | 2026-10-11 |
| [122](backlog/done/122-scripts-deploy-preview-sh-hard-codes-repo-root.md) | scripts/deploy-preview.sh hard-codes $repo_root/target/dx/preview/release/web/public and ignores… | 2026-10-04 |
| [123](backlog/done/123-oracle-hydration-parity-spec-ts-hard-codes-base.md) | oracle/hydration-parity.spec.ts hard-codes BASE = "http://127.0.0.1:8090"; 5 cases fail when a base-path… | 2026-10-04 |
| [127](backlog/done/127-preview-assets-main-css-is-outside-the.md) | preview/assets/main.css is outside the stylelint gate's glob; 7 pre-existing errors | 2026-10-11 |
| [128](backlog/done/128-scripts-check-dx-class-prefix-sh-is-red-on-main.md) | scripts/check-dx-class-prefix.sh is red on main @ 62de3cb: bar_chart/style.css and line_chart/style.css… | 2026-10-03 |
| [129](backlog/done/129-preview-src-components-navbar-component-json.md) | preview/src/components/navbar/component.json lacks the theme globalAssets entry every other component has | 2026-10-05 |
| [132](backlog/done/132-dx-tabs-content-keeps-padding-var-dx-space-8.md) | .dx-tabs-content keeps padding: var(--dx-space-8) while Nova's TabsContent has none; consumers not audited | 2026-10-14 |
| [133](backlog/done/133-inputgroup-has-no-invalid-or-disabled-styling.md) | InputGroup has no invalid or disabled styling | 2026-10-13 |
| [134](backlog/done/134-nova-s-tinted-destructive-button-is-contrast.md) | Nova's tinted destructive button is contrast-failing in dark; ours deliberately is not Nova's. Widened… | 2026-10-13 |
| [135](backlog/done/135-the-computed-style-snapshot-cannot-see-closed.md) | The computed-style snapshot cannot see closed overlays or hover/focus/open states, so phase C's proof has… | 2026-10-12 |
| [139](backlog/done/139-pr-83-s-all-gates-green-was-wrong-main-was.md) | PR #83's "all gates green" was wrong: main was clippy-red from d9bcfda (primitives/src/navbar.rs:655… | 2026-10-04 |
| [140](backlog/done/140-menubar-a-pointer-selection-leaves-dom-focus-on.md) | Menubar: a pointer selection leaves DOM focus on &lt;body&gt;; the keyboard path restores it to the trigger | 2026-10-05 |
| [141](backlog/done/141-dialog-alertdialog-sheet-and-drawer-exit.md) | Dialog, AlertDialog, Sheet and Drawer exit animations play only where transition: overlay allow-discrete… | 2026-10-05 |
| [142](backlog/done/142-the-phone-theme-picker-is-missing-on-demos-and.md) | The phone theme picker is missing on /demos and /charts (no docs sidebar there) | 2026-10-05 |
| [143](backlog/done/143-virtuallist-add-a-content-visibility-mode.md) | VirtualList: add a content-visibility mode (default up to ~5k rows), quantize the scroll signal, drop the… | 2026-10-05 |
| [144](backlog/done/144-opt-in-hidden-until-found-for-collapsible.md) | Opt-in hidden="until-found" for Collapsible, Accordion and Tabs | 2026-10-05 |
| [145](backlog/done/145-unification-refactors-top-3-theme-baseline.md) | Unification refactors, top 3: theme baseline, floating-surface recipe, state recipes (incl. Combobox and… | 2026-10-13 |
| [149](backlog/done/149-scroll-lock-rs-installs-non-passive-wheel.md) | scroll_lock.rs installs non-passive wheel/touchmove listeners on window permanently after the first lock | 2026-10-05 |
| [151](backlog/done/151-a-document-link-stylesheet-can-be-recorded-as.md) | A document::Link stylesheet can be recorded as present and never inserted: a subtree torn down in the turn it… | 2026-10-04 |
| [159](backlog/done/159-js-listeners-added-through-eval-or-shipped.md) | JS listeners added through eval (or shipped scripts) and never removed: a class, widening row 149 | 2026-10-05 |
| [160](backlog/done/160-two-small-bugs-the-shadcn-comparison-found-tabs.md) | Two small bugs the shadcn comparison found: Tabs Ghost sets data-variant="ghost" that no CSS styles… | 2026-10-05 |
| [163](backlog/done/163-wrap-hover-styles-in-media-hover-hover-by.md) | Wrap hover styles in @media (hover: hover), by codemod plus a gate: 80 :hover blocks in 39 stylesheets leave… | 2026-10-14 |
| [166](backlog/done/166-hovercard-and-tooltip-delay-close-delay-props.md) | HoverCard and Tooltip delay / close_delay props (Radix and Base UI parity) | 2026-10-15 |
| [168](backlog/done/168-the-disk-full-class-lanes-running-raw-cargo.md) | The disk-full class: lanes running raw cargo with incremental compilation on regrow the shared gates target… | 2026-10-11 |
| [169](backlog/done/169-open-owner-question-should-popover-be-non-modal.md) | Open owner question: should Popover be non-modal by default (shadcn / Radix parity)? | 2026-10-13 |
| [171](backlog/done/171-drag-and-drop-list-axe-heading-order-finding-on.md) | drag_and_drop_list axe heading-order finding on an h3 | 2026-10-05 |
| [172](backlog/done/172-date-picker-spec-ts-73-and-144-anchor-placement.md) | date-picker.spec.ts :73 and :144 anchor placement (the panel opened above the trigger at 1280x720; offset… | 2026-10-05 |
| [173](backlog/done/173-the-deployed-site-has-no-404-html-unknown-urls.md) | The deployed site has no 404.html: unknown URLs (e.g. /dioxus-components/charts/no-such-chart/) get GitHub… | 2026-10-11 |
| [175](backlog/done/175-the-component-installer-dx-components-add.md) | The component installer: dx components add resolved dependencies in dx's upstream default registry, not ours | 2026-10-10 |
| [176](backlog/done/176-reduced-motion-has-a-gap-the-infinite-loops-of.md) | Reduced motion has a gap: the infinite loops of Skeleton (pulse), Progress (indeterminate sweep) and Avatar… | 2026-10-10 |
| [177](backlog/done/177-motion-nobody-can-see-must-not-run-visibility.md) | Motion nobody can see must not run: visibility gating for CSS loops, timers and entrances, by construction | 2026-10-10 |
| [178](backlog/done/178-an-infinite-animation-may-only-animate.md) | An infinite animation may only animate compositor properties | 2026-10-10 |
| [179](backlog/done/179-the-shadcn-dioxus-installer-cli-new-init-add.md) | The shadcn-dioxus installer CLI: new, init, add, remove, list, update, clean | 2026-10-10 |
| [180](backlog/done/180-the-starter-template-dx-new-myapp-template.md) | The starter template: dx new myapp --template https://github.com/MentalGear/shadcn-dioxus --subtemplate… | 2026-10-10 |
| [181](backlog/done/181-the-registry-mod-rs-fix-27-components-could-not.md) | The registry mod.rs fix: 27 components could not be installed (E0583 in the user's tree), and no gate noticed | 2026-10-10 |
| [182](backlog/done/182-the-install-smoke-script-install-every.md) | The install smoke script: install every component into a scratch app and cargo check it | 2026-10-10 |
| [183](backlog/done/183-the-home-page-card-estimate-contain-intrinsic.md) | The home page card estimate: contain-intrinsic-block-size ignored the cards' padding | 2026-10-10 |
| [184](backlog/done/184-docs-a-motion-section-and-cli-first-install.md) | Docs: a Motion section and CLI-first install | 2026-10-10 |
| [185](backlog/done/185-component-json-cargodependencies-were-missing.md) | component.json cargoDependencies were missing for 14 components; the gate and the smoke app both hid it | 2026-10-11 |
| [189](backlog/done/189-navigationmenu-a-click-on-a-trigger-whose-panel.md) | NavigationMenu: a click on a trigger whose panel hover just opened CLOSED it | 2026-10-11 |
| [190](backlog/done/190-oracle-tier2-html-demo-placeholder-links-spec.md) | oracle/tier2-html/demo-placeholder-links.spec.ts:146 (top_layer) is flaky: link.focus() times out | 2026-10-14 |
| [191](backlog/done/191-batch-2-of-the-demo-parity-work-surfaced-four.md) | Batch 2 of the demo-parity work surfaced four library classes; all fixed by construction | 2026-10-12 |
| [192](backlog/done/192-tabs-size-sm-default-lg-on-a-shared-control.md) | Tabs size (sm/default/lg) on a shared control-height scale | 2026-10-13 |
| [193](backlog/done/193-non-modal-anchored-panels-close-when-their.md) | Non-modal anchored panels close when their anchor leaves the viewport (one shared helper) | 2026-10-14 |
| [194](backlog/done/194-hover-opened-menus-never-closed-on-pointer.md) | Hover-opened menus never closed on pointer leave (Navbar) -- one shared hover-intent helper | 2026-10-14 |
| [195](backlog/done/195-anchored-overlays-overflowed-the-right-edge-by.md) | Anchored overlays overflowed the right edge by the scrollbar gutter; specs measured the same wrong box | 2026-10-14 |
| [196](backlog/done/196-build-ssg-sh-could-exit-0-for-a-broken-site.md) | build-ssg.sh could exit 0 for a broken site | 2026-10-14 |
| [197](backlog/done/197-dialog-panels-inherited-the-font-size-dependent.md) | Dialog panels inherited the font-size-dependent UA padding: 1em | 2026-10-14 |
| [198](backlog/done/198-point-anchor-and-fallback-clamps-measured-a.md) | Point-anchor and fallback clamps measured a shrink-to-fit panel at the position they were replacing | 2026-10-14 |
| [199](backlog/done/199-sidebar-triggers-inherited-other-components.md) | Sidebar triggers inherited other components' default trigger recipes | 2026-10-14 |
| [200](backlog/done/200-buttongroupseparator-was-a-horizontal-separator.md) | ButtonGroupSeparator was a horizontal separator in a horizontal group | 2026-10-14 |
| [201](backlog/done/201-calendar-the-wrapped-third-month-stretched-to.md) | Calendar: the wrapped third month stretched to the card width; calendar and date picker on Nova | 2026-10-14 |
| [202](backlog/done/202-extra-components-explain-why-they-are-extra.md) | Extra components explain why they are Extra | 2026-10-14 |
| [203](backlog/done/203-dark-dx-border-dx-input-follow-shadcn-white-10.md) | Dark --dx-border/--dx-input follow shadcn (white 10% / 15%) | 2026-10-14 |
| [204](backlog/done/204-layout-flicker-of-the-top-demo-group-at-ipad.md) | Layout flicker of the top demo group at iPad-landscape widths | 2026-10-14 |
| [205](backlog/done/205-no-sticky-hover-on-touch-press-feedback-as-a.md) | No sticky hover on touch; press feedback as a switchable system utility | 2026-10-14 |
| [206](backlog/done/206-modal-initial-focus-by-opening-modality-no.md) | Modal initial focus by opening modality (no mobile keyboard pop on tap) | 2026-10-14 |
| [207](backlog/done/207-one-edge-panel-system-for-sheet-and-drawer.md) | One edge-panel system for Sheet and Drawer | 2026-10-14 |
| [208](backlog/done/208-drawer-like-shadcn-s-demo-drawerside-auto.md) | Drawer like shadcn's demo: DrawerSide::Auto default, rounded, nested drawers | 2026-10-14 |
| [209](backlog/done/209-date-picker-apply-cancel-confirm-as-the-default.md) | Date picker: Apply/Cancel confirm as the default; verify the natural-language demo against shadcn | 2026-10-14 |
| [210](backlog/done/210-ios-the-combobox-list-must-never-cover-its.md) | iOS: the Combobox list must never cover its input -- exclusion-zone placement + geometry regression spec | 2026-10-14 |
| [211](backlog/done/211-charts-flicker-on-hydration-no-entry-animation.md) | Charts flicker on hydration: no entry animation for charts already painted | 2026-10-14 |
| [212](backlog/done/212-small-control-demos-labelled-like-shadcn-s-not.md) | Small-control demos labelled like shadcn's (not a lone control in a big frame) | 2026-10-14 |
| [214](backlog/done/214-home-masonry-column-assignment-depends-on-how.md) | Home masonry column assignment depends on how much of the ~580 KB prerendered page has been parsed | 2026-10-18 |
| [215](backlog/done/215-charts-re-lays-out-at-hydration-at-every-width.md) | /charts/ re-lays out at hydration at every width | 2026-10-18 |
| [219](backlog/done/219-preview-src-components-mod-rs-variant-registry.md) | preview/src/components/mod.rs variant registry is one shared macro list that every demo-adding lane edits | 2026-10-18 |
| [220](backlog/done/220-shared-oracle-specs-locate-demo-controls-by.md) | Shared oracle specs locate demo controls by their visible demo text, so every demo that mirrors shadcn more… | 2026-10-18 |
| [222](backlog/done/222-specs-that-measure-geometry-while-an-enter.md) | Specs that measure geometry while an enter animation is still running | 2026-10-18 |
| [223](backlog/done/223-a-modal-returns-focus-to-its-trigger-only-after.md) | A modal returns focus to its trigger only after its exit animation, so a quick keyboard reopen is lost | 2026-10-18 |
| [224](backlog/done/224-merge-attributes-dropped-one-of-two-same-named.md) | merge_attributes dropped one of two same-named event listeners | 2026-10-15 |
| [225](backlog/done/225-email-client-demo-a-view-change-keeps-the.md) | Email-client demo: a view change keeps the previous view's scroll position | 2026-10-18 |
| [226](backlog/done/226-touch-opens-a-blue-focus-ring-on-overlay-panels.md) | Touch opens: a blue focus ring on overlay panels, and the keyboard rises because a field is focused on open | 2026-10-18 |
| [227](backlog/done/227-virtual-keyboard-modal-panels-with-fields.md) | Virtual keyboard: modal panels with fields ignore the iOS/Android keyboard | 2026-10-18 |
| [228](backlog/done/228-navigation-lands-at-the-new-page-s-bottom-and.md) | Navigation lands at the new page's bottom and glides up (home gallery card links, Back) | 2026-10-18 |
| [229](backlog/done/229-interactive-elements-allow-text-selection.md) | Interactive elements allow text selection | 2026-10-18 |
| [230](backlog/done/230-attachment-first-demo-has-no-image.md) | Attachment first demo has no image | 2026-10-18 |
| [231](backlog/done/231-docs-button-page-looks-different-from-shadcn-s.md) | Docs Button page looks different from shadcn's base docs | 2026-10-18 |
| [232](backlog/done/232-docs-one-place-for-installation-the-cli.md) | Docs: one place for installation; the CLI everywhere | 2026-10-18 |
| [233](backlog/done/233-docs-pages-short-coherent-long-sections-in.md) | Docs pages: short, coherent; long sections in disclosures | 2026-10-18 |
| [234](backlog/done/234-docs-sidebar-toggle-category-a-z-a-z-default.md) | Docs sidebar: toggle category / A-Z, A-Z default | 2026-10-18 |
| [235](backlog/done/235-avatar-shadcn-s-first-demo-loading-state-flashes.md) | Avatar: shadcn's first demo; loading state flashes | 2026-10-18 |
| [236](backlog/done/236-docs-manual-installation-files-closed-by-default.md) | Docs: "Manual installation files" closed by default | 2026-10-18 |
| [238](backlog/done/238-playwright-fixtures-force-scroll-behavior-auto.md) | Playwright fixtures force scroll-behavior: auto !important on every context | 2026-10-18 |

## Notes (8)

Narrative sections of the old monolithic file, verbatim (ordering rounds, environment notes, queue evaluations).

- [Fresh-session order (2026-09-04)](backlog/notes/fresh-session-order-2026-09-04.md) (2026-09-04)
- [Fresh-session order (next session, 2026-10-05)](backlog/notes/fresh-session-order-2026-10-05-next-session.md) (2026-10-05)
- [What a fresh session should pick up next (2026-09-18)](backlog/notes/pickup-next-2026-09-18.md) (2026-09-18)
- [What a fresh session should pick up next (2026-09-25)](backlog/notes/pickup-next-2026-09-25.md) (2026-09-25)
- [What a fresh session should pick up next (2026-10-04)](backlog/notes/pickup-next-2026-10-04.md) (2026-10-04)
- [2026-09-18 — queue evaluation and what was taken](backlog/notes/queue-evaluation-2026-09-18.md) (2026-09-18)
- [General backlog — ranked, with triggers](backlog/notes/ranking-intro.md) (n/a)
- [Row 116 -- before/after (2026-10-04)](backlog/notes/row-116-before-after-2026-10-04.md) (2026-10-04)

<!-- END GENERATED -->
