---
id: 302
title: "Charts page tab bar: keep the pill on the active tab"
status: "OPEN (owner review 2026-10-19)."
size: "S (static) / M (animated)"
area: preview
updated: 2026-10-19
---

# Row 302: Charts page tab bar: keep the pill on the active tab

## Item

On `/charts/` the rounded highlight you see on a tab while pressing doesn't stay on the active tab. Why not, and should hover show it and should it animate?

## Status / trigger

OPEN (owner review 2026-10-19).

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
