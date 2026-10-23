---
id: 373
title: "Theme default fallbacks are shadcn Neutral; the upstream colour ramp is deleted"
status: "LANDED in lane fallbacks (batch 12). Verified on a debug SSG build: theme-defaults 6/6 (4 red on…"
size: "S"
area: theme
updated: 2026-10-23
---

# Row 373: the theme's own defaults are shadcn Neutral

## Item

Owner-approved 2026-10-23, proposals P7 and P8 of the theme contrast audit. With no preset and no `--foreground` etc. supplied, the theme resolved 32 `--dx-*` roles differently from shadcn Neutral: leftovers of the upstream Dioxus numbered ramp (`--primary-color-*`, `--secondary-color-*`, the status and error variables, `--focused-border-color`) and a pure-black ink.

## Status / trigger

LANDED in lane fallbacks (batch 12). Verified on a debug SSG build: `theme-defaults` 6/6 (4 red on the batch-12 build before), the contrast-related specs green, `test:changed` (2,535 tests) green except two failures that also fail on the batch-12 build (`virtual-list-find` needs Chromium 144's `::search-text`, the `Rule 4c: selftest:unfixed:id` test is red by design).

## Size

S

## Detail

Changed in `preview/assets/dx-components-theme.css` (token declarations only): P7 `--dx-accent`, `--dx-accent-foreground`, `--dx-destructive-foreground`, light `--dx-destructive`, `--dx-border-strong` (ink 26% over the page in `--dx-hover-mix-space`), the sidebar set, light `--dx-sidebar-primary-foreground`; P8 `--dx-foreground`, `--dx-card-foreground`, `--dx-popover-foreground`, `--dx-primary`, light `--dx-sidebar-primary` to shadcn's near-black. Every role that read a ramp step now carries its literal (same computed value), the ramp and `--focused-border-color` are deleted, and `main.css` loses the two dead `--highlight-color-main` / `--highlight-color-secondary`. Kept: `--dx-muted-foreground`, charts, docs blue, dark destructive / sidebar-primary / its ink, the contrast-mode block (its `-source` tokens only changed their shadcn-side defaults).

Guard: `playwright/theme-defaults.spec.ts` (each role resolves to shadcn's value in both modes; a bare shadcn variable still wins; the removed names resolve to nothing). Specs that hard-coded the old defaults were updated (`theme-preset`, `docs-button-parity`, `theme-all-reset`, `button-hover`, `menu-item-parts`). One comment-only edit in `primitives/src/top_layer.rs` (a CSS comment inside an eval string named `--secondary-color-1`, which `check-css-vars-defined` reads as an undefined use).
