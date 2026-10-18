---
id: 227
title: "Virtual keyboard: modal panels with fields ignore the iOS/Android keyboard"
status: "LANDED 2026-10-18 (lane keyboard 24e1ea78), merged on the session branch; deploy pending -- was…"
size: "M"
area: virtual
updated: 2026-10-18
---

# Row 227: Virtual keyboard: modal panels with fields ignore the iOS/Android keyboard

## Item

Virtual keyboard: modal panels with fields ignore the iOS/Android keyboard

## Status / trigger

**LANDED 2026-10-18 (lane keyboard 24e1ea78), merged on the session branch; deploy pending** -- was: **Scheduled 2026-10-18 in `dev-docs/research/next-round-plan-2026-10-15.md` (lane keyboard)** -- was: **BACKLOG (after 226)**

## Size

M

## Detail

Research `research/mobile-keyboard-2026-10-17.md`: shadcn itself has no keyboard code (only `max-h-(--available-height)` and `text-base md:text-sm`, both already ported); anchored panels already follow the visual viewport (row 210). Gap: Drawer bottom and centred Dialog. **Lift** Base UI 1.6 `Drawer.VirtualKeyboardProvider`'s mechanism: keyboard open when `innerHeight - visualViewport.height > 60` and not pinch-zoomed, publish `--dx-keyboard-inset` (beside `VIEWPORT_METRICS_JS` in `top_layer.rs`, listener via `js_listener::listen`, driven from the dialog open driver), scroll slack + centre the focused field; Drawer `max-height` consumes it. Skip vaul's imperative bottom/height writes and body-fixed lock. Learn (apply only if a real iPhone shows a jump): focusing inside `touchend`. Optional demo: shadcn's legacy responsive Combobox (Popover on desktop, Drawer on mobile). Proof: a Chromium visual-viewport-shrink spec plus a manual iPhone step.
