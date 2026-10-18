---
id: 203
title: "Dark --dx-border/--dx-input follow shadcn (white 10% / 15%)"
status: "LANDED 2026-10-14 (theme lane 118cc9b)"
size: "S"
area: theme
updated: 2026-10-14
---

# Row 203: Dark --dx-border/--dx-input follow shadcn (white 10% / 15%)

## Item

Dark `--dx-border`/`--dx-input` follow shadcn (white 10% / 15%)

## Status / trigger

**LANDED 2026-10-14 (theme lane 118cc9b)**

## Size

S

## Detail

Owner decision 2026-10-14 (row 111 decision 2). Two dark-arm tokens; own commit first so the snapshot attributes it alone; re-run dark-surface-contrast + axe dark. Item 11. Plan: `dev-docs/research/next-round-plan-2026-10-14.md`. **Landed:** dark `--dx-border` / `--dx-input` are `oklch(100% 0 0deg / 10%)` / `15%` (the notation stylelint wants); the light arm and host hooks are unchanged. `dark-surface-contrast` 14/14, `theme-preset-contrast` 24/24. Derived values got lighter: dark input/textarea/native-select/combobox fills ~4.5% / 7.5% white (= shadcn `dark:bg-input/30` and `/50`), dark switch track 12% white, drag_and_drop_list item hover a 10% white wash.
