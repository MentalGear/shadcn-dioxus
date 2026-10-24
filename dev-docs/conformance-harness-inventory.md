# Conformance harness: spec inventory

The current oracle specs under `playwright/oracle/`, one row each (tier, rule, backlog row). How the harness works and
where rules come from: [`conformance-harness.md`](./conformance-harness.md). How each spec landed:
[`archive/rounds/conformance-harness-narrative.md`](./archive/rounds/conformance-harness-narrative.md). A new oracle spec
adds its row here.

| Spec | Tier | Rule (source) | Row |
|---|---|---|---|
| `tier1-apg/focus-restore-reference` | 1 | Focus returns to the trigger on Escape, calibrated on the vendored menu-button and combobox pages | 3.1 |
| `tier1-apg/keyboard-matrix` | 1 | Per-component keyboard rows quoted from APG "Keyboard Interaction" | 24 |
| `tier1-apg/menu-roles`, `menu-submenu` | 1 | Menu pattern roles and the nested-submenu keys (shared `menu_semantics`) | 24, 52, 54 |
| `tier1-apg/select-only-combobox` | 1 | Select trigger's combobox role against the vendored select-only page | 8 |
| `tier1-apg/sortable-table`, `disclosure-navigation`, `window-splitter`, `carousel` | 1 | APG Sortable Table, Disclosure Navigation, Window Splitter (prose only), Carousel | 101 |
| `tier2-html/form-participation`, `form-attribute`, `form-label-association` | 2 | Entry list, `required`/disabled, reset, `form=`, `<label>`: each against a native control | 5 |
| `tier2-html/top-layer` | 2 | Clip escape, flip, anchored self-overlap, inline shift, fit-content width, trigger-id publish (Rules 1-14) | 36, 41, 72, 109 |
| `tier2-html/native-dialog`, `auto-popover-opening-gesture` | 2 | `<dialog>`/`showModal` contract; an auto popover opened between `pointerdown` and `pointerup` | 40, 109 |
| `tier2-html/top-layer-ink` | 2 | Popover and dialog text resolves the app's ink token, not UA `CanvasText` | 50 |
| `tier2-html/global-stylesheet`, `stylesheets-present` | 2 | `main.css` and every component sheet reach `document.styleSheets` on every route | 46 |
| `tier2-html/first-paint-styles`, `first-paint-text` | 2 | JS-off and hydrated pages agree on computed style and value text | 306 |
| `tier2-html/touch-focus-zoom`, `touch-double-tap` | 2 | Text entry `font-size >= 16px` and `touch-action: manipulation` at a coarse pointer | 42 |
| `tier2-html/main-thread` | perf | Long tasks and event timing during overlay open/close and gestures (release builds) | 45 |
| `tier2-html/scroll-main-thread`, `home-scroll-stability`, `anchor-landing` | perf | Scroll smoothness by counts, overview height stability, same-page anchor landing | 150, 174 |
| `tier2-html/listener-inventory`, `motion-gating` | 2 | Listeners owned by their scope; invisible motion paused | 159, 176, 255 |
| `tier2-html/demo-forms-no-navigation`, `demo-placeholder-links` | 2 | A demo submit or `href="#"` never navigates, reloads or scrolls | 190, 331 |
| `tier3-radix/scroll-lock`, `rtl`, `rtl-toggle`, `chart` | 3 | Scroll lock and RTL mirrored from Radix; the page-level direction toggle; Chart (no Radix original) | 9, 13, 137 |
| `hydration-parity` | none | SSG markup parity: Rules 1-6, Rule 4 duplicate attributes, all prerendered routes | 22, 46 |

Not oracles but harness guards: `axe.ts` scans in 37 specs (two-state convention), `accordion-animation` and `assert-fade-out`
(per-frame samplers), `computed-style-snapshot` (a tool, skipped unless `DX_SNAPSHOT` is set).
