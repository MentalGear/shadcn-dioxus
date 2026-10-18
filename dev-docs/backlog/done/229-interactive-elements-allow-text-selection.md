---
id: 229
title: "Interactive elements allow text selection"
status: "LANDED 2026-10-18 (lane baseline 9eb2e91d), merged on the session branch; deploy pending -- was…"
size: "S"
area: theme
updated: 2026-10-18
---

# Row 229: Interactive elements allow text selection

## Item

Interactive elements allow text selection

## Status / trigger

**LANDED 2026-10-18 (lane baseline 9eb2e91d), merged on the session branch; deploy pending** -- was: **Scheduled 2026-10-18 in `dev-docs/research/next-round-plan-2026-10-15.md` (lane baseline)** -- was: **PROPOSED for the 2026-10-15 round (lane `baseline`); waiting for the owner's go**

## Size

S

## Detail

Owner 2026-10-17: buttons etc. must not be text-selectable. Inventory `research/user-select-inventory-2026-10-17.md`: 18 declarations in 12 sheets, one theme rule (`:where([data-dx-menu-item])`); buttons, toggles, tab/accordion triggers, checkbox/switch/radio, badges, pagination, sidebar items, calendar days, select trigger, overlay triggers and close buttons are selectable; no `-webkit-user-select` anywhere (Safari needs the prefix). shadcn base/radix Button carry `select-none`. **Construction:** the theme's one zero-specificity interactive baseline (`dx-components-theme.css:830-867`, the touch-action list) also carries `-webkit-user-select: none; user-select: none` for buttons, `[role=button]`, menu/option/tab/switch/checkbox/radio roles, `summary`; excluded: text inputs, textarea, select, contenteditable/`[role=spinbutton]` segments, the `[tabindex]` catch-all (prose, code scrollers) and `a[href]` (shadcn leaves links selectable); per-component duplicates removed; `check-theme-baseline.sh` extended (no component sheet re-declares it; the rule must exist).
