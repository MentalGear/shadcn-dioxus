---
id: 43
title: "iOS 18 Safari device review, item 2 — top_layer oracle fixture removed from the / home-page gallery — landed…"
status: "CLOSED 2026-10-19 (bookkeeping: status already said done). — (landed)"
size: "S"
area: tests
updated: 2026-10-19
---

# Row 43: iOS 18 Safari device review, item 2 — top_layer oracle fixture removed from the / home-page gallery — landed…

## Item

**iOS 18 Safari device review, item 2 — `top_layer` oracle fixture removed from the `/` home-page gallery — landed 2026-09-03.** `preview/src/main.rs`'s `ComponentGallery` now filters `top_layer` out of `components::DEMOS` before rendering the gallery grid (it is not an installable component, just a large `playwright/oracle/tier2-html/top-layer.spec.ts`/`native-dialog.spec.ts` probe surface); it stays reachable at its own `/component/?name=top_layer&` page and still lists under "Overlays" in `DocsSidebar` (`components::DEMOS` itself, and the sidebar's own iteration over it, are unchanged — only the gallery grid's iteration is filtered). Fallout fixed by construction: `oracle/hydration-parity.spec.ts` Rule 4b's subject (the fixture's `ToastProvider` `aria_label` override) no longer renders on `/`, so it was rewritten against the `avatar` component's own always-mounted gallery card instead (`ImageAvatar`'s alt-derived default vs. its caller's `aria_label` override, the same "explicit default + caller override, merged caller-wins" shape one layer up from a primitive) — see `dev-docs/conformance-harness.md`'s hydration-parity section. No prior backlog row asked for this specific change (the request came directly from the device review, not a standing row) — filed here as the landed record rather than force-fitting an earlier row number

## Status / trigger

CLOSED 2026-10-19 (bookkeeping: status already said done). — (landed)

## Size

S

## Detail

`preview.spec.ts`'s homepage axe scan, `touch-focus-zoom.spec.ts`, and `global-stylesheet.spec.ts` were checked and did not need re-pointing: none of them asserted anything specific to the `top_layer` fixture's presence on `/` (`preview.spec.ts` is a generic scan; the other two already target `/component/?name=top_layer&` or don't reach the fixture through `/` at all). `top-layer.spec.ts`/`native-dialog.spec.ts` already navigated to `/component/?name=top_layer&` directly, unaffected
