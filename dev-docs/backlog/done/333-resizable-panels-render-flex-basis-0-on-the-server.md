---
id: 333
title: "Resizable panels render flex-basis 0% on the server and get their real size only in an effect"
status: "LANDED 2026-10-21"
size: "S"
area: components
updated: 2026-10-21
---

# Row 333: Resizable panels render flex-basis 0% on the server and get their real size only in an effect

## Item

Resizable panels render flex-basis 0% on the server and get their real size only in an effect.

## Status / trigger

**LANDED 2026-10-21**

## Size

S

## Detail

Found 2026-10-21 by the row-329 parity spike (Round 4 thread): the server HTML of the resizable panels has `flex-basis: 0%`, and an effect sets 50%/40% after hydration, so the first paint of every resizable demo collapses the panels and then jumps -- a layout shift and the row 306 class. Fix by computing the default sizes on the server from the panels' `default_size` props.

## Resolution (lane ssr-parity, 2026-10-21)

`ResizablePanel` records its constraints into a non-reactive `render_defaults` list on first render; `panel_size`/`panel_at` use it until the effect-filled registry covers the panel, so server markup has the real `flex-basis` and the handles' `aria-valuenow/min/max`. A panel without `default_size` takes the equal split among the panels rendered up to it (exact when every panel has a default, as all demos do; corrected when the registry fills). Three resizable demos plus `/component/resizable/` left the parity baseline.
