---
id: 302
title: "Charts page tab bar: keep the pill on the active tab"
status: "LANDED 2026-10-20 (lane charts-page). Corrected cause: the pill seen while pressing is the theme's…"
size: "S (static) / M (animated)"
area: preview
updated: 2026-10-20
---

# Row 302: Charts page tab bar: keep the pill on the active tab

## Item

On `/charts/` the rounded highlight you see on a tab while pressing doesn't stay on the active tab. Why not, and should hover show it and should it animate?

## Status / trigger

**LANDED 2026-10-20 (lane `charts-page`).** Corrected cause: the pill seen while pressing is the theme's **press-feedback wash** (`background-image` + `scale` on `a[href].dx-*:active`, row 205) on a `border-radius: full` link, not the focus ring (measured: after a mouse click the ring token resolves to `0 0 #0000` under `data-dx-modality="mouse"`, no ring is drawn; screenshots in `/home/user/tgt/shots-charts-page/before-tabs-*`). It vanished on release because the resting tab had no background. Fix in `main.css` (`.dx-charts-tab`): active (`aria-current="page"`) gets `background-color: var(--dx-muted)` as in shadcn's ExamplesNav (`data-[active=true]:bg-muted`), hover (inside `@media (hover: hover) and (pointer: fine)`) a 50% `color-mix(in var(--dx-hover-mix-space), muted, transparent)` pill, transition on `color`/`background-color` (plus the press `scale` so the press stays smooth). No sliding pill (the owner's sub-item, left out: needs measuring JS or anchor positioning). The shared Tabs component is untouched. Spec: `charts_gallery.spec.ts` "chart-type tabs keep the pill on the active tab" (3 tests). Originally: OPEN (owner review 2026-10-19).

## Size

S (static) / M (animated)

## Detail

- **Why:** that pill isn't a pressed or active style at all. It's the **focus ring** (`border-radius: full` plus `box-shadow: var(--dx-ring)` on `:focus-visible`, `main.css:2493`).
- The bar is plain `NavLink`s (`charts_gallery.rs:407,424`), not the Tabs component. Active (`aria-current="page"`) and hover change text colour only (`main.css:2526-2532`).
- The pill vanishes as soon as focus moves or the route re-renders.
- **Target:** give the active tab a `bg-muted` pill, with a lighter pill on hover inside the hover media query and a short background-colour transition.
  - As far as I remember, shadcn's charts page uses a static pill; not checked.
  - Also hide the focus ring after a pointer click (`:focus-visible` only).
- My take: yes on hover. On animation, a colour fade is cheap. A pill that slides between tabs looks nice but is beyond shadcn, needs measuring JS or anchor positioning, and makes this an M.
- Owner default (2026-10-19): static pill plus a fade; the sliding pill becomes a sub-item.
