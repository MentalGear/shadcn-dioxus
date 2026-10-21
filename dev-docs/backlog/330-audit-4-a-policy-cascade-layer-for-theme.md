---
id: 330
title: "Audit 4: a policy cascade layer for theme rules that must win (@layer dx-base, dx-components, dx-policy)"
status: "BACKLOG (owner 2026-10-21: do it if it holds up; overlaps row 170)"
size: "L"
area: theme
updated: 2026-10-21
---

# Row 330: Audit 4: a policy cascade layer for theme rules that must win (@layer dx-base, dx-components, dx-policy)

## Item

Audit 4: a policy cascade layer for theme rules that must win (@layer dx-base, dx-components, dx-policy).

## Status / trigger

**BACKLOG (owner 2026-10-21: do it if it holds up; overlaps row 170)**

## Size

L

## Detail

Audit section 2 #4 and section 3: several gates exist only because zero-specificity theme baselines lose to component rules -- `check-theme-baseline` arms 4-5b, `check-press-under`, parts of `check-focus-outline-reset` -- and the press wash had to leave `:where()` to win (row 295). Rules in a later cascade layer win by construction regardless of specificity. No `@layer` exists in `preview/` today; Safari >= 15.4 supports it (row 15). Also consider CSS `@scope (.x) to ([popover], dialog)` for the popup boundary (replaces the `:not(:where([popover], dialog) *)` suffix `check-popup-boundary.sh` requires; check Firefox and older-WebKit support first). Doc drift to fix with it: `dx-components-theme.css:896` cites `scripts/check-touch-action.sh`, which no longer exists (now `check-theme-baseline.sh` arm 4).
