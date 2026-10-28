---
id: 477
title: "Anchored overlays are placed in document coordinates (absolute top-layer panels, transform-only), CSS anchoring kept where supported"
status: "IMPLEMENTED on lane/placement-477, device run open. Measured on the owner's iPhone (iOS 18.7…"
size: "L"
area: overlays
updated: 2026-10-28
---

# Row 477: Anchored overlays are placed in document coordinates

## Item

Every anchored overlay (Popover, Tooltip, HoverCard, DropdownMenu, ContextMenu, Menubar, NavigationMenu, Navbar, Select in both modes, Combobox, and the date picker through Popover) is placed by one shared rule that carries the panel with the page instead of tracking it. The root cause of the iOS overlay drift class (row 296) is in the Detail; the rule was measured on a device.

## Status / trigger

IMPLEMENTED on lane/placement-477, device run open. Measured on the owner's iPhone (iOS 18.7 Safari) on 2026-10-28; the owner's verdict on the lab runs: "all works as it should and version P is the best". Ordered before rows [452](./452-port-3-pl-placement-anchor-fallback-and-the-paint-safe-gate.md) and [456](./456-port-3-ov-overlay-core-escape-dismiss-open-focus.md): this row changes what they port. Source: [`radix-vs-base-ui-2026-10-28.md`](../research/radix-vs-base-ui-2026-10-28.md).

IMPLEMENTED on branch `lane/placement-477` (2026-10-28), proof open: the device run on the iPhone. What landed:

- `primitives/src/top_layer/anchor_fallback.rs`: one placement script (`PLACE_JS`) for every anchored overlay and for the ContextMenu point (`use_placement`, `Anchor::{Element, Point}`); `scroll_clamp.rs`: the shared geometry (`PLACEMENT_HELPERS_JS`: `visible()`, `docRect()`, `writer()`), `use_point_anchor_placement`, and `use_close_on_scroll` cut to the one `AnyScroll` policy. The `visualViewport` offset tracking, the settle loop, the measure-at-full-width probe and the `AnchorLeavesViewport` policy are gone. CSS anchor positioning (`anchor.css`, `position: fixed` + `anchor()`) stays the first placement; the script takes a panel over when it matches no contract spot, lies outside the visible region or covers the anchor, and keeps it for that open.
- `select/components/list.rs` (item-aligned) and `context_menu.rs` (`pageX`/`pageY`, the `visual_viewport_offset` eval is gone) use the same helpers. `lib.rs` `use_outside_dismiss`: a mouse press dismisses on `pointerdown`, a touch or pen press is a tap (`pointerup` within 10 px, `pointercancel` drops it).
- Anchored overlays no longer close on scroll: the call is removed from Popover, Menubar, DropdownMenu, Combobox, Select and ContextMenu. `use_close_on_scroll` stays for Tooltip, HoverCard and NavigationMenu (their files belong to other lanes; HoverCard and NavigationMenu drop the call under the owner decision, then the hook can go).
- Proof so far: `playwright/anchored-placement.spec.ts` (phone viewport, pinch zoom plus wheel pan, Chromium's and iOS WebKit's rect frame, engine and script path, gap within 1 px; transform-only writes; context menu document point; touch scroll vs tap) passes on the new build and fails 6 of its pan cases on the base build; `scripts/check-anchored-placement.sh` is the gate of the Gate idea. JS in Rust strings 4,356 -> 3,878 lines.
- Open: the iPhone device run (keyboard up, scrolling with each overlay open); the release-build run of the timing specs; DropdownMenu and Select still lock page scroll (row 479), which is why a touch drag outside them ends in the platform's own light dismiss.

## Size

L

## Detail

**Root cause (row 296 and the whole class).** Anchored panels are top-layer `popover` elements with `position: fixed`, placed by JS (`primitives/src/top_layer/anchor_fallback.rs`) that adds `visualViewport` offsets on `visualViewport` events. iOS fires those events about every 260 ms, so with the keyboard up the panel drifts by up to `vv.offsetTop` (measured 176 px; the "fixed landed delta" equalled the maximum `vv.offsetTop`, 354 px). Mixing coordinate frames is wrong by construction: on iOS WebKit `getBoundingClientRect` is relative to the visual viewport, in Chromium to the layout viewport, so adding scroll or `visualViewport` offsets to a rect is correct in one engine and wrong in the other. iOS 18.7 has no CSS anchor positioning, so the JS path is what iOS users get. Base UI (shadcn `base-nova`) avoids all of it by positioning in document coordinates (`positionMethod='absolute'`, the default in `@base-ui/react` 1.9.0 `internals/useAnchorPositioning`); Radix uses `fixed`.

**The placement rule (rule P).**

1. The panel stays a top-layer popover with inline `position:absolute; inset:auto; margin:0; top:0; left:0`. The containing block of an absolutely positioned top-layer element is the initial containing block, so the page carries the panel on scroll and nothing tracks it.
2. Coordinates are document coordinates, frame-independent: the anchor rect minus the origin of `document.documentElement.getBoundingClientRect()`. A point anchor (ContextMenu) uses `pageX`/`pageY`. A content anchor (item-aligned Select) puts the selected option over the trigger: document Y = trigger top minus the option's `offsetTop`, clamped to the visible region, with the list's `scrollTop` adjusted for the clamp.
3. Position is written only as `transform: translate(x, y)`. The top side adds `translateY(-100%)`, so no panel-height measurement is needed.
4. The visible region comes from a cached `position:fixed` zero-size probe: `visibleTop = probe.top + vv.offsetTop`, height `vv.height` (browser-independent).
5. On window scroll and `visualViewport` scroll/resize (one `requestAnimationFrame` per frame): choose the side (the preferred side unless it lacks the natural height and the other side has more room; 24 px hysteresis); write the transform only when the side flips; write `max-height` = the available space on the chosen side (live list height, rounded, deduplicated). The owner asked to keep the live height.
6. Outside dismiss on touch is a tap, not a touch: record `pointerdown` while open, close on a `pointerup` within 10 px, drop on `pointercancel`; a press that began before the panel opened never closes it (the ContextMenu long press). The scan found that the non-modal surfaces dismiss on outside `pointerdown` plus `focusin` (`primitives/src/lib.rs`, `use_outside_dismiss`, about line 381), which closes a panel as soon as a scroll starts on a phone once close-on-scroll is gone.
7. What it removes: the `visualViewport` offset tracking in `anchor_fallback.rs`; close-on-scroll as a placement workaround (`ScrollClose::AnyScroll` in `top_layer/scroll_clamp.rs`; neither upstream closes on scroll, except Radix's tooltip); and the "lock or close" policy idea (withdrawn). Anchored overlays stay open and scrollable while the page scrolls (owner decision 2026-10-28; Base UI and Radix lock modal menus, this library does not): only ContextMenu and the modal surfaces lock scroll (row 479).

**Device numbers** (Combobox, keyboard up, `vv.offsetTop` to 354 px): the old method drifts 176 px; rule P drifts 0 px over 2,059 frames, and 0 px with the live height over 1,284 frames, smooth by the owner's eye; Floating UI absolute (the Base UI mirror) is 0 px but writes the position about 1,095 times per session against P's writes only on flips. A popover with a field, a dropdown, a context menu (corner at the press point, 0 px) and an item-aligned select (0 px alignment) are all smooth.

**Reference implementation.** The temporary device lab was `dev-docs/lab/overlay-lab.html`, deleted after the runs; its last commit is `63190af3` (history `1302f37d` to `63190af3` on branch `claude/quirky-einstein-p2vcec`). Read it with `git show 63190af3:dev-docs/lab/overlay-lab.html`: the functions `vis`, `adoc`, `place`, `placeSel`, `pdown`/`pup` are rule steps 4, 2, 5, 2 (Select) and 6.

**Scope.** Implement the rule in the shared placement code for every anchored overlay listed in the Item. CSS anchor positioning stays where supported (Safari 26 and Chromium); rule P is the fallback. Files: `primitives/src/top_layer/{anchor_fallback,scroll_clamp,mod}.rs`, `top_layer/anchor.css`, `select/components/list.rs` (item-aligned), the ContextMenu call site, `lib.rs` `use_outside_dismiss` (touch tap rule).

**Order and effect on other rows.** Before rows 452 and 456. Row 452 ports the placement blocks to Rust; it ports rule P, not the visualViewport tracking, so the port is smaller and has less to witness. Row 456 ports the overlay core including outside dismiss; the tap rule lands here first. Row 296 closes with this row. Row 435 (one menu engine) is independent: the placement call per host is unchanged.

**Gate idea.** No overlay writes `top`/`left` or uses `position: fixed` for an anchored panel (a source scan of the anchored-panel styles and `anchor_fallback.rs`), registered in `scripts/run-gates.sh`.

- **Proof:** a device run on the owner's iPhone (keyboard up, scrolling with each overlay open, the numbers above as the baseline); Playwright specs on the top-layer, popover, select, combobox and context-menu surfaces at a phone viewport with the keyboard emulated through a `visualViewport` offset (panel gap to the anchor stays within 0.5 px while the page scrolls); release-build run for the timing specs.
- **Does not subsume:** the scroll-lock method (row 479), the Base UI defaults of row 478, the keyboard inset of the Drawer (row 480).
