# Test architecture audit, 2026-10-21

Owner question (2026-10-20): which tests are redundant, or hide a weak architecture that could be solved by construction at the root cause?

Scope: read-only. Nothing was built or run. Every second figure comes from `playwright/durations.json` (meta: debug SSG build, 2 workers, 217 files, 4,126 tests, 15,359 worker-seconds, budget 17,000; release total 9,776). A "saved" figure is my estimate unless stated; it is marked **unverified** where I could not split navigation time from probe time. Wall-clock is roughly half of worker-seconds at 2 workers.

Method: read the spec headers and the CLAUDE.md gate list, then the backlog rows the specs cite (152, 204, 215, 218, 222, 252, 306, 109, 164, 295). The typography section (end of file) uses a small CSS parser I wrote in the scratchpad; its counts are heuristic and are labelled as such.

Classes: A = masks an architectural weakness; B = redundant with another test or gate; C = keep.

## 1. Summary table

| Spec or gate (s, tests) | Class | Root cause | Construction | Shrinks to | Saved (s) |
|---|---|---|---|---|---|
| `all-routes.spec.ts` axe + 390px overflow + forced colors (2308 s, 517; 3 families x 2 schemes, one fresh `openRoute` per test, `all-routes.spec.ts:~205-230`) | A (harness) | One page load per probe; each route is navigated 6 times (3 families x 2 schemes) | One load per (route, scheme) running all three probes in sequence (`emulateMedia` and `setViewportSize` can switch mid-page); this is the "shared-page fixture" still open in row 152 | ~170 route-scheme loads instead of ~500 | ~1000 (unverified: nav vs axe split not measured) |
| `all-routes` forced-colors family (~170 of the 517 tests) | A | Per-control focus outline only proven by walking every route; the theme has ONE `@media (forced-colors: active)` outline rule (spec header, `all-routes.spec.ts:~25-32`) | Gate that the theme rule exists and beats `outline: none` (`!important` or a policy layer) plus `check-focus-outline-reset.sh`; test then walks one route per control signature | ~10 routes | ~450 (unverified) |
| `popup-style-isolation.spec.ts` (592 s, 85) | A | Popups are DOM children of the trigger and inherit typography (`popup-style-isolation.spec.ts:8-12`, `ALLOWED` at :54-64; row 218 recurred 2026-10-20) | Theme reset of inherited properties at every popup root, plus a gate that the reset covers every inheritable property, plus the declared-typography gate (section 4) | one popup per component | ~430 |
| `first-paint-styles` section 2 (JS-off vs hydrated per route, 2 contexts, ~76 routes) (part of 1086 s, 166 tests) | A | SSR render differs from the first client render (rows 252, 306, 215; `dynamic-style-shorthand` gate) | Native "render twice" parity test in `cargo test` (SSR HTML vs HTML after effects flush); see construction 3 | cargo test + 3 browser routes | ~270 (unverified) |
| `first-paint-text.spec.ts` (180 s, 84) | A | Same class (row 306: label from a client-only registry) | Same native parity test also compares text, `value`, `aria-label` | 1 non-vacuity browser test | ~170 |
| `first-paint-styles` sections 1, 3, 4, 5 (rest of 1086 s) | C/B | Section 1 (head carries the bundle) duplicates `check-css-delivery.sh`; section 3 (client nav to every component, `first-paint-styles.spec.ts:408-` per slug) is the browser half of the bundle construction | Keep section 1 test "bundle is complete" (:310) as the generic proof; section 3 to 5 routes | ~12 routes | ~250 (unverified) |
| `hydration-parity.spec.ts` "Rule 2/4 extended" (every prerendered route; file 91.5 s) | B | Same enumeration as `first-paint-styles` section 2 and `first-paint-text` | Fold the hydration-error and duplicate-attribute probes into the one SSR-vs-hydrated visit | merged | ~40 |
| `layout-stability.spec.ts` (819 s, 19) | A | Cause 1 SSR != hydrated (avatar `data-state`); cause 2 `content-visibility: auto` estimates (spec header :4-20); no gate bans the latter (grep of `scripts/` finds none) | Native parity test (cause 1); gate: `content-visibility: auto` only with `contain-intrinsic-size` and never on a box that sets sibling flow (cause 2, already fixed by `MasonrySlot`, header :55-59) | key widths (`KEY_WIDTHS`, :126, 34 widths) minus the step-32 sweep | ~300 (unverified) |
| `charts-hydration-stability.spec.ts` (262 s, 8) | A | "Server lays out from an assumption the measured render corrects" (header :4-9); fixed by CSS-only box | Same parity test + keep 1 width x 3 breakpoints | 3 widths | ~150 |
| `theme-picker-phone.spec.ts` (446 s, 38; 4 red in the duration run) | A | Two entry paths chosen by context presence (`SidebarCtx`), so every route type must be swept (header :7-18) | One unconditional header entry that opens the same drawer on a phone; no per-route-type branch | 1 route per shell + 1 desktop | ~300 (unverified; depends on how much of the 11.7 s/test is navigation) |
| `theme-preset-contrast.spec.ts` (168 s, 27) | A | Contrast is a pure function of token values; browser used only to resolve `color-mix`/oklch | Node/contrast gate over `theme-presets.css` for the semantic token pairs (like `check-destructive-text.sh`, which already clamps one role) | 1 spot-check test | ~150 (unverified: needs a resolver for `color-mix`) |
| `demo-forms-no-navigation.spec.ts` (191 s, 6) | B | `check-demo-forms.sh` already forces every form to be `DemoForm` (`method="dialog"` + `prevent_default`); platform rule is proven by the calibration tests (spec header "3. calibration") | Keep calibration (raw form navigates; `method=dialog` does not) + one DemoForm route, JS-off and wasm-blocked | 3 tests | ~165 |
| `rtl-toggle.spec.ts` "every component under the toggle" (`rtl-toggle.spec.ts:103-130`, ~70 of 83 tests, 168 s) | B (also vacuous) | The overflow assertion reads `documentElement.scrollWidth - clientWidth`, which `all-routes.spec.ts` header (:17-20) says proves nothing because `html`/`body` are `overflow-x: clip` (unverified in a browser); "no errors" and "toggle flips" are covered by the first 3 describe blocks and by `check-direction-keys.sh` | Keep the behavioural describes; drop or cut the sweep to a sample | ~12 tests | ~110 |
| `user-select.spec.ts` sweep (`user-select.spec.ts:54-66`, 36 pages; file 86 s, 43) | B | `check-theme-baseline.sh` arm 4 (`:181-204`) bans re-declaring `user-select: none`; arm 5b (`:242-260`) pins the one theme rule and its inclusions/exclusions | Extend arm 4 to ban any `user-select` on a covered element; sweep to 3 pages; keep the behaviour tests (dblclick/drag/Select All) | 3 pages + behaviour | ~50 |
| `touch-double-tap.spec.ts` (103 s, 57) | B | Theme has the one `touch-action: manipulation` rule (`dx-components-theme.css:946`); the theme comment (:896) names `scripts/check-touch-action.sh`, which does not exist (its arm moved into `check-theme-baseline.sh`; stale comment) | Gate arm + home page + the 3 overlay tests (`:228-248`) | ~5 tests | ~70 |
| `touch-focus-zoom.spec.ts` (127 s, 58) | A | Every text-entry element must compute >= 16px on coarse pointers; checked per route because each component sheet sets its own size (header lists 5 instance families: Combobox, language select, top_layer fixtures, native selects, calendar selects) | One theme rule in `@media (pointer: coarse)` flooring `input, textarea, select, [contenteditable]` at 16px (needs `!important` or a policy layer to beat component specificity) | calibration + home + dashboard | ~90 |
| `dx-class-migration.spec.ts` (87 s, 52) + `stylesheets-present.spec.ts` (36.7 s, 15) | B | Delivery is now by construction: one render-blocking bundle in every head (`check-css-delivery.sh`, `scripts/ssg-css-bundle.mjs`), and `first-paint-styles.spec.ts:310` proves the bundle is complete; "unhashed class" is a static grep (`check-dx-class-prefix.sh`) | Keep both on 3 routes each (they still guard the `dx serve` and copy-out `document::Link` path, which the bundle does not cover) | 6 tests | ~100 |
| `listener-inventory.spec.ts` (172 s, 22) | B (partial) | `check-js-listeners.sh` + `primitives/src/js_listener.rs listen` already derive removal from scope; blocking-scroll gate covers rule 2 | Keep scroll-lock timing and width-shift (browser only); inventory to 3 surfaces | ~12 tests | ~80 (unverified) |
| `press-feedback.spec.ts` shorthand loop (`:157`, plus `:235` specificity test) (45 s, 15) | A | Zero-specificity theme baselines lose to component rules; each loss was a separate row (205, 229, 239, 295) | Policy cascade layer, construction 4 | 3 tests | ~25 |
| `theme-all-reset.spec.ts` (16 s, 9) | B | Half (a) is `check-theme-baseline.sh` banning `all:`; half (b) is a golden snapshot "captured on the pre-change build" (header :6-10), a one-time migration oracle | Delete after one release | 0 | 16 |
| Geometry-at-rest (`measureAtRest`/`readAtRest`, 42 spec files, 196 uses; `check-spec-at-rest.sh`, `check-spec-viewport-bound.sh`) | A | Specs measure while an enter animation runs (row 222) | Default `reducedMotion: "reduce"` in the Playwright configs; the theme already swaps panel animations out under `prefers-reduced-motion: reduce` (`dx-components-theme.css:812`, `:1249`); only animation specs opt into `no-preference` | helper stays for animation specs only | ~60 (flake prevention is the real gain; unverified that reduced-motion keeps the `use_animated_open` hold) |
| Hydration wait (`gotoHydrated`, `hydration.ts:47`; 965 static call sites; `check-spec-goto-hydrated.sh`) | A | Prerendered markup is inert until wasm hydrates and early events are dropped (row 109); the wait is a per-call convention policed by a gate | Fixture-level `page.goto` default (row 164's proposal, never done: `fixtures.ts` has no wrapper); wait on `load` + `data-hydrated` instead of `networkidle` (500 ms idle window per navigation) | gate deleted | **unverified**: up to 0.5 s per navigation, measure `load` vs `networkidle` on one file first |
| `keyboard-matrix` (147 s, 78), `main-thread` (91 s release), `top-layer` (275 s, 99), `combobox-placement` (162 s, 59), carousel family, date-picker, charts, `modal-*` | C | Browser-only behaviour (APG keys, long tasks, anchored geometry, scrolling physics) | none | none | 0 |
| `check-hover-media.sh`, `check-css-state-recipes.sh`, `check-hover-mix-space.sh`, `check-focus-outline-reset.sh`, `check-destructive-text.sh` | C | Authoring-time policy on hand-written sheets that ship raw to consumers via `dx components add`, so a bundler transform cannot replace them | none | none | 0 |

Totals of the listed savings: roughly 3,900 worker-seconds (about 25% of 15,359), before the unmeasured `networkidle` effect. The first four rows hold about 60% of it.

## 2. Ranked shortlist: top 5 constructions

Ranking = estimated seconds saved x bugs prevented (instances already filed).

1. **Visit-once route harness plus fixture-level navigation** (~1000-1500 s, plus the `networkidle` effect; bugs: rows 109, 164, 222, 112, 115 and every "forgot to wait" flake). One fixture: `page.goto` waits for `data-hydrated` on `load` (no `networkidle`), `reducedMotion: "reduce"` is the config default with an explicit opt-in for animation specs, and `all-routes` runs its three probes on one load per (route, scheme). Deletes `check-spec-goto-hydrated.sh` and most `readAtRest` calls as a convention.
2. **Declared typography at roots and popup roots** (popup-style-isolation ~430 s; bugs: row 218 recurred three times in one week, Select scroll-button chevrons, docs home cards at 14 vs 16px). Section 4 below.
3. **Native SSR-vs-first-render parity test** (~900 s across `first-paint-styles` section 2, `first-paint-text`, `layout-stability` cause 1, `charts-hydration-stability`; bugs: rows 204, 215, 252, 306, `dynamic-style-shorthand`, avatar). A `cargo test` that renders every demo with `dioxus-ssr`, once before and once after the effects flush, and diffs structure, text, `value`, `aria-label` and the `style` attribute. **Unverified feasibility**: depends on whether `dioxus-core` 0.7.9 runs `use_effect` in a native `VirtualDom` (`wait_for_work`/`render_immediate`) so the second render is meaningful; needs a one-hour spike. Geometry that comes from a browser measurement (ResizeObserver) stays with a reduced browser test.
4. **Policy cascade layer for theme rules that must beat components** (~250 s of specs; bugs: rows 205, 229, 239, 295, 50 pasted `touch-action` sites). The theme's zero-specificity baselines (user-select, touch-action, press wash, coarse-pointer 16px floor, forced-colors outline) lose to any component rule, which is why each loss became its own gate arm (`check-theme-baseline.sh` arms 1-5, `check-press-under.sh`) and its own route sweep. `@layer dx-base, dx-components, dx-policy` makes the policy layer win by position, with a data-attribute opt-out for the few drag surfaces. No `@layer` exists in `preview/assets` or `preview/src` today (grep returned nothing). Caveat: copy-out consumers who only take a component sheet lose the policy, which is already true of the zero-specificity theme rules.
5. **Delete the redundant sweeps (zero new construction)** (~445 s: `demo-forms-no-navigation` 165, `rtl-toggle` sweep 110, `dx-class-migration`/`stylesheets-present` 100, `user-select` sweep 50, `touch-double-tap` 70 less overlap, `theme-all-reset` 16). Cheapest and safe because each has a named static gate that carries the same assertion.

Next by value, not in the top 5: single theme-picker entry (~300 s, row 142's two-path design), static preset contrast (~150 s), `layout-stability` width reduction (~300 s).

## 3. Gates that are themselves instance-patches over a deeper cause

| Gate(s) | Deeper cause | Deeper construction |
|---|---|---|
| `check-spec-goto-hydrated`, `check-spec-at-rest`, `check-spec-viewport-bound`, `check-demo-text-locators`, `check-playwright-fixture-import` | Specs are hand-written against raw `page`; five gates police five "use the helper" conventions | The `page` fixture does the right thing by default (hydrated goto, reduced motion, fixed-viewport box); only `check-playwright-fixture-import` remains, so the fixture cannot be bypassed |
| `check-dynamic-style-shorthand` (and the `first-paint-styles` finds: carousel dot, virtual_grid minimap) | The Dioxus 0.7.x interpreter drops a shorthand-with-`var()` on a re-set `style` attribute | Inline `style:` strings may carry only custom properties and numbers, the rest in a sheet keyed by class or `data-*`; subsumes the shorthand ban and most inline-style SSR/client drift. Or patch the interpreter upstream |
| `check-theme-baseline` arms 4 and 5b, `check-press-under`, `check-focus-outline-reset`, `check-css-state-recipes` | Baselines are zero-specificity defaults but behave as policies | Policy layer (construction 4) |
| `check-popup-boundary` (`:not(:where([popover], dialog) *)` suffix) plus `popup-style-isolation` | A popup is a DOM descendant of its trigger, so ancestor rules and inheritance reach it | Popup root reset (worked example) and the declared-typography gate; the descendant-selector suffix remains for selectors (not inheritance) |
| `check-cross-component-overrides` (shrink-only baseline `.baseline.tsv`) | Components style each other's elements, so cascade order across per-route sheets decides | Already steered to slot variables; the bundle (`check-css-delivery`) made order deterministic but not meaningful. Long-term: components expose only custom-property slots to parents; the gate then needs no baseline |
| `check-hover-media`, `check-hover-mix-space`, `check-duplicate-keyframes`, `check-anchored-keyframes`, `check-infinite-animations` | Each is "do not hand-write what the theme owns once" | Keep as gates (consumers get raw sheets); lowest priority to replace |
| `check-js-listeners`, `check-uncleared-intervals`, `check-blocking-scroll-listeners`, `check-motion-gating` | JS side effects are installed from `document::eval` strings with no owner | Already constructed through `listen`, `use_interval_while` and `use_motion`; the gates guard bypass. Fine; `listener-inventory` can shrink |
| `check-css-delivery` plus `stylesheets-present`, `dx-class-migration`, `first-paint-styles` 1 and 3 | A late `<link>` is not render-blocking | Constructed (the bundle). Remaining gap: consumers' `dx components add` path still uses `document::Link` per component |
| `check-slow-spec-budget` (`playwright/durations.json`) | Suite cost grows with the number of routes x probes | Not an instance-patch; it is the brake that makes the savings above visible. Re-measure `durations.json` after each shrink |

## 4. Declared typography (in-flow components)

Owner question: why can any component take styles from its host page? In-flow components inheriting `font-family`, `color` and `direction` is deliberate (as in shadcn); size, weight and line-height are part of the design and should be declared.

### 4.1 Measurement (heuristic, scripted parse of the 83 `preview/src/components/*/style.css`)

- 83 sheets, 508 distinct `dx-` part classes. A part counts as text- or icon-bearing if one of its rules declares `color`, `font-weight`, `text-align`, `text-transform`, `letter-spacing`, `white-space`, `text-overflow`, `text-decoration` or `font-style`; sr-only and slider/switch thumbs excluded. That gives about 200 parts.
- Of those 200, **68 never declare `font-size`** (as literal or `var()`) and **151 never declare `line-height`** (64 declare neither). `font-size: inherit` and `font: inherit` do not count as declared.
- That part-level number overstates the risk: a part that inherits from its OWN component root is fine when the root declares a size (Table: root `table/style.css:22` declares `var(--dx-text-sm)`, so `.dx-table-head` at :44 inherits it by design). The question that matters is the ROOT.
- **Roots.** Of 64 components whose sheet defines the root class `dx-<dir>`, **20 declare `font-size` and 5 declare `line-height`**. (The other 19 have no class by that name, for example charts, sidebar, scroll_area, top_layer; unmeasured.)
- **Popup roots** (the case that actually bites, because the popup is a DOM child of the host): of 17 popup-root classes I checked, `popover` (`popover/style.css:12`), `tooltip` (:11, also line-height), `hover_card` (:11) and `dialog` (:119) declare a size; **12 do not**: `dx-select-list` (`select/style.css:114`), `dx-dropdown-menu-content` (`dropdown_menu/style.css:78`), `dx-alert-dialog` (`alert_dialog/style.css:86`), `dx-sheet` (`sheet/style.css:71`), `dx-drawer` (`drawer/style.css:86`), `dx-combobox-list` (`combobox/style.css:122`), `dx-menubar-content` (:58), `dx-navigation-menu-content` (:102), `dx-command-dialog` (:118), `dx-toast` (:90), `dx-calendar` (:26), `dx-context-menu-content` (:1). Some may declare it under a differently named selector; treat 12 as an upper bound (unverified per root).

Five examples of parts that look right only by accident:

| File:line | Part | What it declares | What it inherits |
|---|---|---|---|
| `select/style.css:213` | `.dx-select-scroll-button` | `height` as a token, `color`, `display: flex` | `font-size`, `line-height` from the list, which inherits from the host (the 14px chevron case) |
| `select/style.css:114` | `.dx-select-list` (popup root) | layout only | host typography |
| `dropdown_menu/style.css:78` | `.dx-dropdown-menu-content` | layout only | host typography (the `ALLOWED` entries in `popup-style-isolation.spec.ts:60-63`) |
| `breadcrumb/style.css:29` | `.dx-breadcrumb-link` | `color: inherit`, `text-decoration` | size, weight, line-height from the list (intentional only if the root declares them: it does not) |
| `sheet/style.css:110` | `.dx-sheet-close` | position, `width` token | `font-size` for its icon box |
| `combobox/style.css:111` | `.dx-combobox-expand-icon` | `color` | icon box size from the input's font size |

### 4.2 The gate: how a static check identifies the parts

Enforce: every component ROOT and every popup root declares `font-size` and `line-height` through a token or slot (`var(--dx-text-*)`, `var(--dx-label-font-size, ...)`); `font-family`, `color`, `direction` may inherit; any part that sets its own `font-size` does so through a token.

Recommended identification, in order of reliability:

1. **From the Rust, not from the CSS.** Parse `preview/src/components/*/component.rs` for `class: "dx-..."` on (a) the outermost element each `#[component]` renders and (b) any element carrying `popover`, a `<dialog>`, `role=menu|listbox|dialog|tooltip` (the popup-boundary gate already has `POPUP_BOUNDARY` for the CSS side, `scripts/check-popup-boundary.sh`). Those classes are the roots; each must have a `font-size` and a `line-height` in its rules in `style.css`, directly or through a slot variable set on the same rule.
2. **CSS-side fallback for parts.** A part is text- or icon-bearing when its element in the rsx has a string literal child, an interpolated `{label}`/`{children}`, or an icon component (`ChevronDown {`, `Check {`), or when a descendant rule styles `svg`. A text-bearing part that is NOT inside its own component root's declared size (it is its own root, a portal, or the parent is another component's element) must declare its own.
3. **Escape hatch.** Allowlist with reasons and stale-entry failure, like every other gate: inline-flow typography components that deliberately follow the surrounding text (Markdown, Typography-like links) carry `typography-inherit: <reason>`.
4. Also bans `font-size: inherit` and `font: inherit` on a root, which is how "a button resets the UA size but still takes the host's" slips through (button reset rules).

By construction beside the gate: a zero-specificity theme rule `:where([data-dx-root], dialog[class^="dx-"], [popover][class^="dx-"]) { font-size: var(--dx-root-font-size, var(--dx-text-sm)); line-height: var(--dx-root-line-height, 1.5) }`. Components retune through the two slot variables; the gate only checks that every root class is reachable by that rule (needs a `data-dx-root` marker or the class list). This is the typography half of row 218 plus the in-flow case; the popup-root reset being built separately covers the popup subset.

Caveat: line-height has a cost: only 5 of 64 roots declare it, so the first run of the gate will fail about 50 roots; start with `font-size` and a shrink-only baseline file (the `check-cross-component-overrides.baseline.tsv` pattern).

### 4.3 Browser tests that shrink if the gate exists

| Test | Effect |
|---|---|
| `popup-style-isolation.spec.ts` (592 s) | Typography properties (`font-*`, `line-height`, `letter-spacing`, `text-*`) drop out of the per-popup diff (`:85`, `:94`); the three `ALLOWED` entries (`:60-63`) go; the remaining diff is ancestor selectors and custom properties, already covered by `check-popup-boundary`. Shrinks to one popup per component |
| `touch-focus-zoom.spec.ts` (127 s) | A declared size on text-entry roots plus the coarse-pointer floor rule makes the per-route sweep unnecessary |
| `computed-style-snapshot.spec.ts`, `menu-item-parts.spec.ts`, `menu-touch-targets.spec.ts`, `alert/attachment/item/message/tag_group` font-size assertions | Per-component "its text is 14px" assertions become the gate; the specs keep only geometry that depends on layout |
| `first-paint-styles` sections 2 and 3 | Not helped: they compare SSR with hydrated, not host with component |
| `theme-all-reset` | Not helped (cascade specificity, not inheritance) |

Estimated saving ~430-500 s (mostly popup-style-isolation); the larger value is that the "latent" inheritance stops producing new instances.

### 4.4 Candidate backlog row

- **Declared typography: every component root and popup root declares font-size (and then line-height) through a token or slot; only font-family, color, direction inherit. Gate `check-declared-typography.sh` with a shrink-only baseline (about 50 roots at the start, 12 popup roots first), roots read from component.rs, escape hatch `typography-inherit:`; shrinks `popup-style-isolation` to one popup per component.**

## 5. Candidate backlog rows (one line each)

1. Test harness: `page` fixture navigates hydrated by default (`load` + `data-hydrated`, no `networkidle`), config default `reducedMotion: "reduce"` with opt-in for animation specs; retire `check-spec-goto-hydrated`, shrink `check-spec-at-rest`; first measure `load` vs `networkidle` on one file (row 164's undone half).
2. `all-routes`: one load per (route, scheme) running axe, 390px overflow and forced-colors in sequence (row 152's open "shared-page fixture"), forced-colors reduced to one route per control signature once the theme rule has a gate.
3. Native SSR-vs-first-render parity test in `cargo test` over every demo (structure, text, value, aria-label, `style` attribute), spike first; then cut `first-paint-text`, `first-paint-styles` section 2, `layout-stability` cause-1 and `charts-hydration-stability` to one width.
4. Declared typography gate and root reset (section 4.4); closes row 218's remaining half and shrinks `popup-style-isolation`.
5. Policy cascade layer (`@layer dx-base, dx-components, dx-policy`) for user-select, touch-action, press wash, coarse-pointer 16px floor and forced-colors outline; retire `check-theme-baseline` arms 4-5b, `check-press-under`, the `user-select` and `touch-double-tap` sweeps, `theme-all-reset`.
6. Delete or shrink the redundant sweeps: `demo-forms-no-navigation` to calibration + 1 route, `rtl-toggle` per-component sweep (its overflow check reads `scrollWidth` under `overflow-x: clip`), `dx-class-migration` and `stylesheets-present` to 3 routes, `touch-double-tap` to 5 tests.
7. Coarse-pointer 16px floor as one theme rule for `input, textarea, select, [contenteditable]`; `touch-focus-zoom` to calibration + home + dashboard.
8. Theme picker: one unconditional header entry (no `SidebarCtx` branch); `theme-picker-phone` to one route per shell.
9. Static preset contrast gate over `theme-presets.css` (like `check-destructive-text.sh`); `theme-preset-contrast` to a spot check.
10. `layout-stability`: gate `content-visibility: auto` (only with `contain-intrinsic-size`, never on boxes whose height feeds sibling placement); drop the step-32 width sweep to `KEY_WIDTHS` plus 2 samples.
11. Inline `style:` strings carry only custom properties and numbers (subsumes `check-dynamic-style-shorthand`; stops style drift between SSR and client).
12. Doc drift: `dx-components-theme.css:896` names `scripts/check-touch-action.sh`, which does not exist (its job is `check-theme-baseline.sh` arm 4).

## 6. Not verified / open questions

- No spec was run. Seconds saved for rows marked "unverified" depend on nav vs probe split; `durations.json` has per-file totals only. A one-off per-test JSON report of `all-routes` and `first-paint-styles` would settle rows 1, 4 and 8 of the table.
- Whether the `rtl-toggle` overflow assertion is vacuous depends on `html { overflow-x: clip }` as stated in `all-routes.spec.ts:17-20`; I did not open the CSS to confirm.
- The typography counts come from a heuristic parse (selectors split on commas, last compound's first `dx-` class, `@media` flattened); root-class lookup by naming convention misses 19 components. Treat as order of magnitude.
- Whether reduced motion changes behaviour (animated-open hold, `use_animated_open` ~250 ms) in a way geometry specs would miss is unverified.
- Whether a native `VirtualDom` runs `use_effect` for the parity test is unverified (spike needed).
