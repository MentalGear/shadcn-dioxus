---
id: 330
title: "Audit 4: a policy cascade layer for theme rules that must win (@layer dx-base, dx-components, dx-policy)"
status: "CLOSED 2026-10-21: spike verdict \"do not build as specified\" (no gate arm retired); the two cheap…"
size: "L"
area: theme
updated: 2026-10-21
---

# Row 330: Audit 4: a policy cascade layer for theme rules that must win (@layer dx-base, dx-components, dx-policy)

## Item

Audit 4: a policy cascade layer for theme rules that must win (@layer dx-base, dx-components, dx-policy).

## Status / trigger

**CLOSED 2026-10-21: spike verdict "do not build as specified" (no gate arm retired); the two cheap salvages are noted in Detail.** 

## Size

L

## Detail

Audit section 2 #4 and section 3: several gates exist only because zero-specificity theme baselines lose to component rules -- `check-theme-baseline` arms 4-5b, `check-press-under`, parts of `check-focus-outline-reset` -- and the press wash had to leave `:where()` to win (row 295). Rules in a later cascade layer win by construction regardless of specificity. No `@layer` exists in `preview/` today; Safari >= 15.4 supports it (row 15). Also consider CSS `@scope (.x) to ([popover], dialog)` for the popup boundary (replaces the `:not(:where([popover], dialog) *)` suffix `check-popup-boundary.sh` requires; check Firefox and older-WebKit support first). Doc drift to fix with it: `dx-components-theme.css:896` cites `scripts/check-touch-action.sh`, which no longer exists (now `check-theme-baseline.sh` arm 4).


## Spike (2026-10-21, lane b-layer, branch `claude/layer-spike`, debug SSG of dd14365e)

Prototype: `main.css` starts with `@layer dx-base, dx-components, dx-policy;`; `scripts/ssg-css-bundle.mjs` wraps the
bundle body in `@layer dx-components { ... }`; the theme's touch-action, user-select and press-feedback rules went into
`@layer dx-policy`, the popup-root reset into `@layer dx-base`.

**(a) The pipeline keeps layers: yes.** dx's CSS minifier leaves them intact. Built CSS: `main-*.css` begins
`@layer dx-base,dx-components,dx-policy;`, the theme has `@layer dx-policy{...}` and `@layer dx-base{...}`, the bundle
`dx-components-all-*.css` begins `@layer dx-components {`. CSSOM in Chromium: statement + three layer blocks (dx-policy 86
rules, dx-base 1, dx-components 1825).

**(b) Policy beats a higher-specificity component rule: yes.** A probe `html body button.dx-button.dx-button.dx-button {
touch-action: pan-x; user-select: text }` injected inside `@layer dx-components` left the button at `manipulation` / `none`;
the same rule UNLAYERED won (`user-select: text`; touch-action stayed `manipulation` only because `main.css` carries an
`!important` backstop). So an unlayered app rule beats the whole library, which is the right user-override direction.

**(c) Nothing else changes: no.** 13 specs (first-paint-styles, popup-style-isolation, theme-all-reset, press-feedback,
touch-double-tap, touch-focus-zoom, touch-open-focus, user-select, focus-ring, slider, drawer, resizable,
color-picker-touch): 512 passed, 6 failed on the layered build. On the unlayered baseline build of the same commit
the `/component/bubble/` test and `press-feedback` pass and `theme-all-reset` fails only `cp-area-thumb` (red on main
already, not the layer's), so the layer added 5 reds: bubble, press-feedback, and theme-all-reset switch, slider-thumb
and cp-slider-thumb.
- `theme-all-reset` (switch, slider-thumb, cp-slider-thumb; cp-area-thumb is red on main too) and `press-feedback` "press lands instantly
  where a component has its own transition": the policy `transition: scale ...` replaced every component's own
  `transition` list (Switch `background-color, box-shadow` became `scale`; Button 0.15s became 0.2s).
- `first-paint-styles` `/component/bubble/`: `.dx-toast-container ol { gap }` (component layer) now loses to an
  unlayered `main.css` rule, so JS-off 12px vs hydrated 8px. Wrapping the bundle means EVERY unlayered docs-site rule
  now beats every component rule, a docs-site cascade change that has to be audited rule by rule.

**Why it does not hold.** The theme rules are not policies, they are baselines with deliberate, reasoned exceptions:
touch-action `none`/`pan-*` on slider, color area, resizable handle, drawer, chart and toast (9 sites in 6 sheets), the 15
`USER_SELECT_ALLOW` drag surfaces, components that own their `transition` or `:active` scale, and the popup-root reset
(a root's own `font-size` must beat it). A layer that wins by construction removes those exceptions, so each needs an
opt-out attribute in the policy selector (`:not([data-dx-touch])` ...), which is a per-exception list in the theme
instead of a per-sheet gate arm, not a retirement. Specificity was never the only reason a rule lost; replacement is the
other: `check-press-under` exists because the wash REPLACES `background-image`, which a layer does not change. And the
layer is only real where the component sheets are layered: the SSG bundle yes, but `dx serve` and `dx components add`
consumers load each `style.css` through `document::Link` UNLAYERED, so there the policy would lose to every component
(worse than today's baseline) unless every `style.css` wraps itself in `@layer dx-components { }` (83 sheets, and the
~20 gate scripts that parse sheets assume top-level rules).

**Verdict: do not build as specified.** Gates arms kept: all of `check-theme-baseline`, `check-press-under`,
`check-focus-outline-reset`. Salvageable and cheap if wanted later: (1) only the `!important` cases already in the theme
could move into `@layer`, no behaviour change; (2) the app-override layer of row 170 item 2 needs the docs `main.css`
audit above first. The stale `scripts/check-touch-action.sh` comment at `dx-components-theme.css:896` is fixed in this branch.
