---
id: 167
title: "Migrate the 52 hand-paired listener sites onto use_js_listeners, shrinking PAIRED_BY_HAND in…"
status: "OPEN -- recorded 2026-10-05 (follow-up of row 159, which landed 2026-10-05 in round 3); scheduled…"
size: "M (mechanical per site, 11 files)"
area: misc
updated: 2026-10-05
---

# Row 167: Migrate the 52 hand-paired listener sites onto use_js_listeners, shrinking PAIRED_BY_HAND in…

## Item

Migrate the 52 hand-paired listener sites onto `use_js_listeners`, shrinking `PAIRED_BY_HAND` in `scripts/check-js-listeners.sh` to empty

## Status / trigger

**OPEN** -- recorded 2026-10-05 (follow-up of row 159, which landed 2026-10-05 in round 3); scheduled late in the next-session order (step 4, after rows 137, 146, 155's menu group and 154's Carousel research, because they edit the same files)

## Size

M (mechanical per site, 11 files)

## Detail

Row 159 built the scope-owned helper (`primitives/src/js_listener.rs`, `use_js_listeners`) and the gate (`scripts/check-js-listeners.sh`), and grandfathered the registrations that already had a correct hand-written add/remove pair in `PAIRED_BY_HAND`, a ratchet that only shrinks. The audit counted **52 hand-paired sites in the 11 files** the map names: `carousel.rs`, `collapsible.rs`, `context_menu.rs`, `drag_and_drop_list.rs`, `drawer.rs`, `input_otp.rs`, `menu_sub.rs`, `message_scroller.js`, `navigation_menu.rs`, `top_layer.rs` and `virtual_list.rs`. Each is correct today, but each is a hand-wired teardown (an `await dioxus.recv()` sentinel and the matching removes) that a later edit can break, which is the class row 159 exists to close. **Migration (mechanical, per the gate's own header):** `x.addEventListener(t, h, o)` -> `listen(x, t, h, o)`, delete the `await dioxus.recv()` and the matching removes, wrap in `use_js_listeners`; one file per lane step; delete the file's `PAIRED_BY_HAND` line in the same change (the gate reports a file that no longer needs its entry); the target is an empty map. `message_scroller.js` is a shipped script whose `attach()` returns its own teardown: decide per file whether it moves or stays with a reasoned marker. **Two sites need a marker, not just a move:** `primitives/src/context_menu.rs:801` and `:960` register `window` `wheel`/`touchmove` listeners with `{ capture: true, passive: false }` (they cancel scrolling outside the open menu), which `check-blocking-scroll-listeners.sh` reports once they go through `listen(...)` with `passive: false`: add `blocking-ok: <reason>` (cancelling the wheel is the feature; installed only while a menu is open and removed with it) as a JS comment on the line above inside the migrated script, the way `scroll_lock.rs:458` and `:482` do. **Collisions:** `carousel.rs` waits for row 154's Carousel research, `navigation_menu.rs` is row 137, `message_scroller.js` row 146, `context_menu.rs` row 155 group 2, `drawer.rs` rows 141 and 156: take those files after those rows merge, or last. Verify each migrated site with its component spec, `playwright/oracle/tier2-html/listener-inventory.spec.ts` and the three listener gates.
