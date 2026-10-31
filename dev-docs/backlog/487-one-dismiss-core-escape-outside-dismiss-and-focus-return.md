---
id: 487
title: "One dismiss core: Escape, outside dismiss and focus return for every overlay"
status: "PHASE 1 on lane/dismiss-core-487 (2026-10-31), not yet merged; phase 2 (the menu family) waits for…"
size: "M-L"
area: overlays
updated: 2026-10-31
---

# Row 487: One dismiss core: Escape, outside dismiss and focus return for every overlay

## Item

Every overlay closes the same three ways (Escape, a press outside, focus handed back to the trigger), yet each does it by its own code: placement and top-layer sync are shared already, dismissal is not. Construction: one dismiss core that takes a per-overlay policy (which keys, which outside presses, whether focus returns, whether a nested layer owns the Escape), so a rule such as "a touch press outside is a tap, not a touch" lands once and reaches every overlay.

## Status / trigger

PHASE 1 on `lane/dismiss-core-487` (2026-10-31), not yet merged; phase 2 (the menu family) waits for [row 435](./435-one-menu-family-behind-three-hosts.md) to merge. Prioritised with row 435 (owner 2026-10-31): the single overlay engine is finished before the Rust ports. [Row 456](./456-port-3-ov-overlay-core-escape-dismiss-open-focus.md) then ports one core instead of the scattered handlers.

**Phase 1, what exists.** `primitives/src/dismiss.rs` is the core: an overlay passes a `Policy` (`escape`: `Ignore` / `Close` / `Native`; `press`; `focus`) and a root (an element id, or the nearest `closest` ancestor of it) to `use_dismiss`, and gets an `on_dismiss(Cause)` call (`EscapeWithin`, `Escape`, `OutsidePress`, `FocusLeft`). The page half is one legacy script (`DISMISS_JS`, `lib.rs`) that replaces three: the global Escape listener, the outside-press listener and NavigationMenu's own. Layers register on one page-side list (`window.__dxDismissLayers`) ordered by when they opened, so the topmost layer takes Escape and the decision is made inside the key's dispatch; a `Close` layer cancels the key's default action, a `Native` layer (a modal `<dialog>`, a `popover="auto"`) stays in the order and leaves the key to the browser. A mouse press dismisses on `pointerdown`; a touch or pen press is a tap (row 477's rule), for every surface that has a press arm. `use_outside_dismiss` keeps its signature (it is `Policy::OUTSIDE` on the core), and `RefocusTrigger` / `use_refocus_on_close_unless` are unchanged for the menu hosts.

**Moved onto the core (policy per surface, also the table in `dismiss.rs`).**

| surface | Escape | press outside | focus leaving | Escape sites removed |
|---|---|---|---|---|
| Tooltip | close | no | no (blur closes it) | trigger `onkeydown` |
| HoverCard | close | no | no (blur closes it) | trigger and content `onkeydown` (web and native arms) |
| NavigationMenu | close, focus back to the trigger when it was inside the nav | yes, hover and click activation (the tap rule now reaches it) | no (its own blur debounce) | trigger and content `onkeydown` (both arms), `NavigationMenuOutsidePressDismiss` and its listener |
| Navbar | close, focus back to the trigger when it was inside the content | native `popover="auto"` light dismiss, measured equivalent (spec) | no | the root `onkeydown` Escape arm |
| Select | close (also from the trigger after Alt+ArrowDown) | yes | yes | list and trigger arms (kept on the arm without script only) |
| Combobox | close | no: a page control may act on the options while the list stays open (`combobox.spec.ts` "dynamic option removal"), the input's blur closes it | no | the input's open-list arm (kept on the arm without script only) |
| Popover, Dialog, AlertDialog (web) | `Native`: the browser's `cancel` / light dismiss, in the order | native (`popover="auto"`; Dialog and Popover: the backdrop click, `use_dialog_backdrop_dismiss`; AlertDialog: none) | no | none (`use_global_escape_listener` is gone from the native arms) |
| Popover, Dialog (arms without script) | close | yes | yes | `use_global_escape_listener` |
| AlertDialog (arm without script) | close | no | no | `use_global_escape_listener` |

Escape is now document-wide for Tooltip and HoverCard (WAI-ARIA tooltip: "Escape dismisses the tooltip" wherever focus is) and nested: a Select, a Popover or a Tooltip inside a Dialog takes the first Escape, the Dialog the second.

**Proof (phase 1).** `playwright/dismiss-parity.spec.ts` (52 cases: Tooltip, HoverCard, Popover, Select, Combobox, Navbar, NavigationMenu in both activation modes, Dialog, AlertDialog, DropdownMenu, Menubar; Escape and focus return, mouse press outside, focus leaving, a touch scroll versus a tap, nested layers on a Dialog fixture in `top_layer`). On the pre-change build (`pub-docs422`, no nested fixture) 47 passed and 7 failed: the four nested cases (fixture absent), the Combobox and DropdownMenu table rows that the spec now records as measured policy, and NavigationMenu click activation, whose touch scroll that starts outside closed the panel (its own `pointerdown` listener); on the phase 1 build all 52 pass. JS volume: `js_lit` 3,866 -> 3,860 and `JsListeners::install` sites 44 -> 42.

## Size

M-L

## Detail

State measured 2026-10-31 on `248f35cb`, after rows 477-479 (PR #156).

- **Outside dismiss, today.** `use_outside_dismiss` (`primitives/src/lib.rs:391`) is used by Popover, DropdownMenu, ContextMenu, Menubar, Select and Dialog, and carries row 477's touch tap rule (a mouse press dismisses on `pointerdown`; a touch or pen press is a tap, `pointerup` within 10 px, `pointercancel` drops it). `use_sub_outside_dismiss` (`menu_sub.rs:420`) is the submenu variant. Not on it: Tooltip, HoverCard, Combobox, NavigationMenu (its own document `pointerdown` listener, `navigation_menu.rs:~947-955`, so the tap rule does not reach it: a scroll that starts outside a NavigationMenu panel closes it on a phone) and Navbar (native `popover=auto` light dismiss).
- **Escape, today.** Handled by hand in 12+ places: `tooltip.rs`, `hover_card.rs` (3), `navigation_menu.rs` (3), `navbar.rs`, `combobox/components/input.rs`, Select's trigger and list, `dropdown_menu.rs` (3), `context_menu.rs` (5), `menubar.rs`, `menu_sub.rs`, `menu_item.rs`, plus the dialog and popover roots (`use_global_escape_listener`, `lib.rs:334`). Which layer takes the Escape of a nested stack is decided per host.
- **Focus return, today.** `RefocusTrigger` (`lib.rs:455`; `Signal<String>` for Select, `TriggerHandle` for the menus) is wired per host, with the touch/pen "never refocus by script" rule living in the trait impls only.
- **Construction.** One core in `primitives/src/` (module name chosen by the lane) owning: the Escape key (layer-aware, so the innermost open layer closes first), the outside press (the tap rule of row 477, mouse versus touch/pen) with an "inside" predicate (root, trigger, nested layers) and a focus-leaves arm where the surface wants it, and the focus return (`RefocusTrigger`, modality rule). Each overlay passes a policy value; nothing else is per-host.
- **Brings under the tap rule:** NavigationMenu first (its own listener is deleted), then Tooltip, HoverCard and Combobox where an outside press applies (Tooltip and HoverCard are hover surfaces, so the policy may say "Escape and focus only"; the lane records the choice per surface). **Navbar** keeps its native light dismiss only if the lane shows it equivalent (Escape, outside mouse press, outside tap versus scroll, focus return); otherwise it joins the core.
- **Subsumes:** the 12+ Escape handlers, the NavigationMenu listener, `use_sub_outside_dismiss` as a separate function, the per-host `RefocusTrigger` wiring. **Does not subsume:** the scroll lock (row 459/479), open focus (`OPEN_FOCUS_JS`, row 456), hover intent and its delays (row 478 item 4), the menu item and submenu code (row 435; the two rows touch the same host files, so land 435 step 1 first or coordinate the files per lane).
- **Row 456 follows.** 456 ports JavaScript; it does not unify. After this row it ports one dismiss core (L3/L6/L7/L8/L9) instead of the scattered sites.
- **Proof:** one dismiss parity spec iterating every overlay (Popover, Dialog, AlertDialog, Sheet/Drawer where they dismiss, DropdownMenu, ContextMenu, Menubar, Select, Combobox, NavigationMenu, Navbar, Tooltip, HoverCard, DatePicker): Escape (nested stack closes innermost first), outside mouse press, outside tap versus scroll on touch, focus return to the trigger (and none after touch/pen). The existing `playwright/anchored-placement.spec.ts` touch cases stay green. A release-build run, since it touches every overlay.

## Phase 2 (after row 435 merges; the menu hosts and `menu_sub.rs` are that row's files)

1. DropdownMenu, ContextMenu, Menubar, `menu_sub.rs`, `menu_item.rs`: each host's hand-written Escape arms (3, 5, 1, 1, 1) become one `use_dismiss` layer per open panel (`Close` for a `manual` popover, `Native` for a `popover="auto"` one), a submenu a layer above its parent so only the submenu closes first.
2. `use_sub_outside_dismiss` (`menu_sub.rs:420`) becomes `use_dismiss` with a root that includes the sub-trigger; its script goes. `use_outside_dismiss` calls in the three hosts become `use_dismiss` calls with the policy of the surface (the wrapper then goes).
3. The per-host `RefocusTrigger` wiring (`interacted_outside` signals set by hand in the hosts) becomes the `Cause` the core hands to `on_dismiss`; `use_refocus_on_close_unless` takes it. The touch/pen "never refocus by script" rule stays in the two `RefocusTrigger` impls.
4. Parity gap the spec recorded: Menubar closes when focus moves to a control outside, DropdownMenu does not (`dismiss-parity.spec.ts`, `focusOutside`); phase 2 gives them one policy.
5. Not moved in phase 1: the backdrop-click dismiss of a modal `<dialog>` (`use_dialog_backdrop_dismiss`: the target is the dialog itself, so a rect test and a press-started-on-the-backdrop rule stand in for "outside"); it becomes a press mode of the core with the tap rule. The `cancel` route of `use_dialog_close_sync` (Escape through state, the opener hand-back of row 223) joins the `Native` mode with row 456.
6. A Combobox tap on plain page does not close its list (the input does not blur on a touch tap, and the `dynamic` demo needs a press elsewhere not to close it): an owner decision whether a touch-only outside tap should close it.
7. Row 456 ports `DISMISS_JS`, the last script of the core, together with the open-focus and `cancel` scripts.
