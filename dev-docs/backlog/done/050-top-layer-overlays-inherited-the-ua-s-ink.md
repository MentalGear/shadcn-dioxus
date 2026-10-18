---
id: 50
title: "top-layer overlays inherited the UA's ink colour instead of the app's, sitewide. User-reported against the…"
status: "Landed 2026-09-04"
size: "S–M"
area: build
updated: 2026-09-04
---

# Row 50: top-layer overlays inherited the UA's ink colour instead of the app's, sitewide. User-reported against the…

## Item

**LANDED 2026-09-04: top-layer overlays inherited the UA's ink colour instead of the app's, sitewide.** User-reported against the deployed preview: Menubar submenu text and the Popover menu render the wrong colour. Root cause: `[popover]` and `<dialog>` each carry a UA stylesheet rule setting `color: CanvasText` DIRECTLY on the element, and a directly-cascaded value beats an inherited one regardless of specificity or source order — so every overlay wrapper that never declares its own `color` renders UA black instead of `--secondary-color-4`. Navbar only appeared correct because `.dx-navbar-item` declares its own colour and masks the identical bug; in dark mode the defect degrades to black-on-dark-grey. **Explicitly NOT a regression from rows 31b/32** — proven by building and serving the pre-session commit `963c570` side by side for byte-identical measurements, and by `git diff 963c570..HEAD` touching zero `color:` lines in those stylesheets; the hypothesis that row 32's per-component `document::Link` changed cascade order was falsified directly, since neither `main.css` nor `dx-components-theme.css` declares a `color` rule for these elements at all. Fixed as a class, not per instance, because row 44's `ensure_toast_base_styles` had already fixed exactly this for the raw toast and never generalised it: `ensure_top_layer_ink_styles` (`primitives/src/top_layer.rs`) injects zero-specificity `:where([popover]), :where(dialog) { color: inherit }` from the only three hooks by which anything enters the top layer — `use_popover_sync`, `use_popover_shown_while_mounted` and `use_dialog_open_driver` (`lib.rs`), covering the popover-promoted and native-`<dialog>` arms. Execution then showed the wrapper rule alone was insufficient: `<button>`/`<input>`/`<select>`/`<textarea>` carry their own direct UA colour, so DatePicker, Select and Combobox stayed black (their items are real buttons, e.g. `.dx-calendar-grid-cell`); the rule was extended to those controls **inside** a top-layer surface, scoped there rather than app-wide. Two instances the shared rule cannot fix alone were also fixed: `.dx-menubar-item` never declared a colour, and the Popover demo's `<h3>` was rendered with no class so `.dx-popover-content-title` never applied. Oracle: new `playwright/oracle/tier2-html/top-layer-ink.spec.ts`, 15 surfaces x light/dark, **red-first 20 failed/10 passed before, 30/30 after**, asserting against the token read from the page rather than a hardcoded rgb so it survives a theme retune, and deliberately retaining the cases where a component sets its own colour (tooltip; select/combobox's `--secondary-color-1`) as proof the shared rule does not override them. 199 further specs across the overlay family pass. **Known residue — CLOSED, same date.** The ink baseline was injected from its own `use_effect` while the overlay's `showPopover()`/`showModal()` ran from a different effect's `document::eval`, leaving a first-paint window (observed as the 2-in-N flake described above). Fixed with the construction this repo had already proven for the identical problem: `anchor_positioning_inject_js`'s own doc records that cross-`document::eval` ordering between two effects is unreliable even when both are declared, in order, in the same function, and that the fix is to prepend the injecting script directly into the dependent script so both run as one synchronous JS program with no ordering assumption left to make. `ensure_top_layer_ink_styles`'s injection is now prepended the identical way at all three top-layer entry hooks — `use_popover_sync`, `use_popover_shown_while_mounted` (`top_layer.rs`), and `use_dialog_open_driver` (`lib.rs`) — the same three exhaustive call sites the original fix touched. `toast.rs` was checked and deliberately left unchanged: it already shares this construction, reads nothing back that the window could race, and its own ink is covered via `use_popover_shown_while_mounted`. Proven by repetition rather than one run, since the defect was itself a flake: **150/150 across five repeats** of the full `top-layer-ink.spec.ts` oracle

## Status / trigger

Landed 2026-09-04

## Size

S–M

## Detail

Cross-reference the `css_module`/`@supports`-scoping bug class (rows 31a/32/47): this is a *different* bug mechanism (UA direct-cascade beating inheritance), not another instance of that one — noted so the two aren't conflated. See row 51 for a second, unrelated defect surfaced by this same investigation
