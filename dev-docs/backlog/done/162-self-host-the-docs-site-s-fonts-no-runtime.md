---
id: 162
title: "Self-host the docs site's fonts (no runtime requests to Google) and give the library --dx-font-sans /…"
status: "LANDED 2026-10-19 (lane fonts)"
size: "S-M"
area: docs
updated: 2026-10-19
---

# Row 162: Self-host the docs site's fonts (no runtime requests to Google) and give the library --dx-font-sans /…

## Item

Self-host the docs site's fonts (no runtime requests to Google) and give the library `--dx-font-sans` / `--dx-font-mono` tokens, so an app chooses its own fonts

## Status / trigger

**LANDED 2026-10-19 (lane fonts)**

## Size

S-M

## Detail

**Facts (measured 2026-10-05).** The docs site loaded Geist (wght 100-900) and Geist Mono from Google Fonts at runtime: ~52.5 KB
of Latin woff2 plus an 8.5 KB Google CSS file and two Google hosts, preloaded with `display=optional` (the font CLS fix:
`/component/form/` 0.117 to 0.000, `/component/carousel/` 0.048 to 0). The library sets no font of its own on purpose (components
inherit); shadcn apps self-host through `next/font`. The owner picked option (a) on 2026-10-05: self-host Geist and Geist Mono
(option (b), Geist only plus a system mono, was the recommendation).

**Landed in two steps.**

1. *Round 3, 2026-10-05: the hard-coded stacks.* `dialog`, `alert_dialog`, `sheet`, `drawer` and `calendar` hard-coded a system
   sans stack, so the app font never reached overlays or the calendar. They now inherit, and `scripts/check-component-fonts.sh`
   (in `run-gates.sh`) makes the class inexpressible: a component `style.css` `font-family` may only be `inherit` or
   `var(--dx-font-<name>)`, `font` only `font: inherit`, and `@font-face` is flagged. Facts and numbers:
   [round-3 note](../../research/round3-2026-10-05.md), section 6.
2. *Lane fonts, 2026-10-19.* Geist and Geist Mono (Latin subset, variable woff2, 29.4 and 23.1 KB) live in
   `preview/assets/fonts/` with the OFL text beside them. `GlobalHead` writes the two `@font-face` rules (`font-display:
   optional`, latin `unicode-range`) and the two `crossorigin` preloads from the SAME `asset!` constants, because `asset!` hashes
   file names and does not rewrite a stylesheet `url()` (so the rules cannot sit in `main.css`). The Google preconnect and
   stylesheet links are gone. Tokens `--dx-font-sans` and `--dx-font-mono` in `dx-components-theme.css` inside a zero-specificity
   `:where(:root)` (system-stack defaults, any app override wins whatever the sheet order); `main.css` overrides both with Geist
   and its 14 literal mono stacks plus the inline `monospace` stacks now read the tokens. Gates: `check-demo-remote-assets.sh` now
   also flags a remote font host, a remote `@import` and a remote `document::Stylesheet` (seeded self-test fails);
   `check-css-delivery.sh` holds the preload, `font-display: optional` and no remote URL in GlobalHead. Spec:
   `playwright/fonts-self-hosted.spec.ts`. Left: `hero.css` still names `"Geist Mono"` literally (works; should read the token).

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/162-self-host-the-docs-site-s-fonts-no-runtime.md`.*
