---
id: 134
title: "Nova's tinted destructive button is contrast-failing in dark; ours deliberately is not Nova's. Widened…"
status: "LANDED 2026-10-13/14 (phase C T0) -- solid destructive stays (owner decision 5; a tinted variant…"
size: "S-M"
area: theme
updated: 2026-10-13
---

# Row 134: Nova's tinted destructive button is contrast-failing in dark; ours deliberately is not Nova's. Widened…

## Item

Nova's tinted destructive button is contrast-failing in dark; ours deliberately is not Nova's. **Widened 2026-10-05 into one `--dx-destructive-text` token:** Alert's and Field's destructive-as-text is 2.61:1 in dark too

## Status / trigger

**LANDED 2026-10-13/14 (phase C T0)** -- solid destructive stays (owner decision 5; a tinted variant may come later); destructive TEXT is the one clamped role `--dx-destructive-text` (>= 4.5:1 on every surface, both modes) enforced by `check-destructive-text.sh`; alert, field, bubble, attachment, form read it; the two axe known findings (alert, field error, site-home) no longer reproduce and were deleted from `all-routes.spec.ts`

## Size

S-M

## Detail

`button/style.css` (`.dx-button[data-style="destructive"]`, comment at `:183`) keeps a solid destructive fill with `--dx-destructive-foreground`; Nova's `bg-destructive/10 text-destructive` needs a text-grade destructive colour that `--dx-destructive` (a dark-red fill) is not. Dark fill is `color-mix(--dx-destructive 60%, transparent)`, unmeasured against axe. **Next:** with phase C's colour-contrast re-run, measure the destructive button, badge and alert in light and dark; if a tinted style is wanted, add a `--dx-destructive-text`-style token first (a role shadcn lacks, like `--dx-border-strong`) and pass 4.5:1 before repainting anything. **Widened 2026-10-05 (audit): one token, not a button decision.** The missing role shows up outside the button: destructive-as-text is `color: var(--dx-destructive)` in `field/style.css` (~`:92`, `:98`) and `alert/style.css` (~`:56`, `:60`, `:70`, `:74`), and the dashboard demo's `email_client.css:881`, and it measures **2.61:1 in dark** (text needs 4.5:1), because `--dx-destructive` is a fill colour (the theme already has `--dx-destructive-fill` for the fill role). **Construction:** define `--dx-destructive-text` once in `preview/assets/dx-components-theme.css` (light and dark values that pass 4.5:1 on the surfaces they sit on: page, card, popover), move every destructive-as-text use onto it (Alert, Field, the dashboard demo, and the tinted button if one is wanted), and consider a gate that fails on `color: var(--dx-destructive)` so the fill token cannot be used as text again (the shape of the other `check-*.sh` markers; add it to `scripts/run-gates.sh` and CLAUDE.md's gate sentence). Re-measure with axe in light and dark before and after.
