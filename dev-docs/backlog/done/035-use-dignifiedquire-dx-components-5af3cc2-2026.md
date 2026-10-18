---
id: 35
title: "Use dignifiedquire/dx-components (5af3cc2, 2026-08-21, idle since; nothing upstreamed) as a references —…"
status: "SUPERSEDED 2026-09-26 (all six items landed) -- was: when any of those items is picked up…"
size: "—"
area: menus
updated: 2026-09-26
---

# Row 35: Use dignifiedquire/dx-components (5af3cc2, 2026-08-21, idle since; nothing upstreamed) as a references —…

## Item

Use `dignifiedquire/dx-components` (`5af3cc2`, 2026-08-21, idle since; nothing upstreamed) as a **references — licence permits lifting (dual MIT/Apache-2.0, same as upstream and us) with the `lifting-from-forks.md` §1 provenance header; the obstacle is architectural, not legal: their Command/Navigation Menu/Drawer are entangled with their own popper + focus_scope modules (Shape C, port the concept, no header needed), while Input OTP and Table look self-contained enough for a Shape B module lift with attribution — check per item** for the six primitive-needing shadcn gaps it already implements — `resizable.rs`, `command.rs`, `navigation_menu.rs`, `drawer.rs`, `table.rs`, `input_otp.rs` — and its `floating-ui` port for row 10's residual shift/size clamping

## Status / trigger

**SUPERSEDED 2026-09-26 (all six items landed)** -- was: when any of those items is picked up (component-backlog.md ranked queue)

## Size

—

## Detail

**2026-09-26 (audit correction): SUPERSEDED** — all six items (Resizable, Command, Navigation Menu, Drawer, Table, Input OTP) have landed, confirmed present in `preview/src/components/` (+ `primitives/src/` for the five that need a primitive; Table is styled-only by design, no primitive file). Kept for its shape/edge-case research value only. Per `lifting-from-forks.md` hygiene: read for shape and edge cases, never copy (no provenance story). Their native-arm `popper.rs` hardcodes a 1024×768 viewport and has no Blitz story — do not take positioning code from it
