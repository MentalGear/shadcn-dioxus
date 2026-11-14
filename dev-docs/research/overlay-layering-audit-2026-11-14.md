# Overlay layering audit

Owner's question: "Shouldn't there be a global general component for this? Are all those compoents as far abstracted to a general layer/component as is optimal ?"
Date: 2026-11-14. A read-only audit of `primitives/src` at `38c04d61`; every line count and hour figure is inferred from reading, not measured by diff.
Paths are relative to `primitives/src` unless they start with `dev-docs/` or `playwright/`.

## 1. The shared layers

| Module (lines) | Owns | Used by |
|---|---|---|
| `top_layer/` (1441 Rust, 208 css, about 400 test lines) | The top-layer and anchor engine: `PopoverKind` and the popover-attribute sync, anchor attributes, CSS anchor positioning, the JS-measured fallback, point anchor, close-on-scroll, injected stylesheets | The submodules below |
| `top_layer/popover_sync.rs` (321) | `use_popover_sync` (`:110`, hides at once when `open` goes false) and `use_popover_shown_while_mounted` (`:245`, never hides; DOM removal ends it, so the exit animation plays) | While-mounted: Tooltip `tooltip.rs:490`, HoverCard `hover_card.rs:648`, Popover non-modal `popover.rs:732`, Navbar `navbar.rs:924`, NavigationMenu `navigation_menu.rs:1002`, Select list `select/components/list.rs:356`, Combobox list `combobox/components/list.rs:145`, Toast `toast.rs:882` (8). Immediate: `menu_root.rs:277` only, so Dropdown, Context and Menubar panels and submenus |
| `top_layer/anchor_fallback.rs` (241) | `use_anchor_position_fallback` (`:228`), `use_placement` (`:175`), `Anchor::Element` and `Anchor::Point` | Tooltip, HoverCard, Popover non-modal, Navbar, NavigationMenu, Select list, Combobox list, menu panels (`menu_root.rs:284`); the point form only by ContextMenu |
| `top_layer/attributes.rs` (236) | The one writer of `data-dx-anchored` (`AnchorKind`), `anchored_trigger_attributes`, `anchored_content_attributes` (one folded `style`) | Every anchored trigger (9) and content (11) |
| `top_layer/scroll_clamp.rs` (194) | `use_close_on_scroll` (`:145`, only `ScrollClose::AnyScroll`), `use_point_anchor_placement` (`:177`) | Close on scroll: Tooltip only (`tooltip.rs:500`) |
| `top_layer/styles.rs` (369) | Per-window injection of the anchor stylesheet and the ink baseline | All of the above through `window_once` |
| `hover_intent.rs` (230) | `HoverIntent`: one-at-a-time, index-keyed hover open and close; 50 ms open intent, 50 ms close grace; its own touch detection (a page listener per instance, around `:105`) | NavigationMenu, Navbar |
| `use_hover_timers` in `hover_card.rs:59`, `HoverTimers` at `:44-57` (about 160 lines with docs, `:25-190`) | A boolean hover and focus surface: `delay` and `close_delay` props, `hold`, focus open at once, Escape in the gap | HoverCard, Tooltip (`tooltip.rs:3` imports it from `hover_card.rs`) |
| `menu_sub.rs:232-300` (`SubMenuState`) | A third hover timing: 100 ms open, 0 ms close, safe triangle | Submenus of Dropdown, Context, Menubar |
| `safe_triangle.rs` (345) | `Triangle` geometry and `SafeClose` | `hover_intent.rs:60`, `menu_sub.rs:37` |
| `delayed_action.rs` (170) | The one cancellable delayed task, with a gate (`tests::no_site_keeps_its_own_delayed_task`) | HoverTimers, HoverIntent, SafeClose, SubMenuState, ContextMenu long press (`context_menu.rs:393,395`), Select typeahead clear (`select/context.rs:53`), DatePicker (`date_picker/context.rs:99`), typeahead, dismiss, carousel autoplay |
| `dismiss.rs` (501) | The dismiss core (row 487): layer-aware Escape, outside press with the touch tap rule, focus leaving; `Policy::{ESCAPE, ESCAPE_AND_PRESS, ESCAPE_PRESS_AND_FOCUS, MENU, MENU_SUB, NATIVE, NATIVE_BACKDROP}` (`:125-177`); `escape_in_gap` (`:284`) | 12 files: HoverCard (and Tooltip through `use_hover_dismiss`), Popover, Dialog, AlertDialog, Dropdown, Context, Menubar, menu_sub, Navbar, NavigationMenu, Select list, Combobox list |
| `escape.rs` (108) | Not the Escape key: escaping ids into CSS and JavaScript text | `top_layer/*`, `modality.rs`, `select/components/list.rs`, carousel, `lib.rs` |
| `scroll_lock.rs` (599) | Refcounted body scroll lock, one page-wide count | Dialog, AlertDialog, Popover modal arm, ContextMenu |
| `portal.rs` (183) | `PortalIn` and `PortalOut` | Toast only |
| `menu_root.rs` (411) | `MenuHost`, `use_menu_panel` (`:262-330`, popover, placement and ARIA for a menu panel), `use_open_focus_sync`, `use_menu_content_lifecycle` | Dropdown, Context, Menubar |
| `menu_item.rs` (1379), `menu_sub.rs` (616), `menu_semantics.rs` (130) | Shared items, submenu, ARIA literals | The 3 menu hosts; `menu_semantics` also Navbar and NavigationMenu |
| `listbox.rs` (135), `selectable.rs`, `collection.rs` (1177) | Listbox id and render hooks; the select-like open state (`selectable.rs:305`); roving collection | Select, Combobox, Command (listbox); collection is used well beyond overlays |
| `modality.rs` (428) | Interaction modality on `<html>`, the open-focus policy, `focus_attributes`, `use_close_focus` | Dialog, AlertDialog, Popover, Command, DatePicker context, Dropdown, Select context |
| `window_once.rs` (104) | Per-window one-time work | `scroll_lock`, Toast, `top_layer/styles`, `virtual/grid` |
| `lib.rs` helpers | `use_controlled` (`:281`), `use_animated_open` (`:849`, exit wait for popover-family panels), `use_dialog_open_driver` (`:616`, `showModal()` and `close()` after the exit settles), `use_dialog_close_sync` (`:787`), `use_refocus_on_close_unless` (`:509`) | Driver: Dialog, AlertDialog, Popover modal. `use_animated_open`: 11 sites. Refocus: Dropdown, Context, Select |
| `bridge/` behaviours | `bridge/behaviours.rs`: the `Behaviour` enum has one variant, `Fixture`, which no component uses; `ts/behaviours/behaviours.gen.ts` has `KINDS = ["fixture"]` | No overlay uses a bridge behaviour; every overlay page script is a legacy `document::eval` string |

## 2. Matrix

"UC" is `use_controlled`. Shared module names are bare; own code gives file:lines.

| Component | Open state | Trigger and ARIA | Hover timing | Focus open | Placement | Top-layer entry | Outside press | Escape | Scroll | Focus management | Exit wait | Nested | Typeahead / roving |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Tooltip | UC `tooltip.rs:164` | own trigger `:254-370`; `aria-describedby` always | HoverTimers | HoverTimers.focus; pressed guard own `:285-310` | `use_anchor_position_fallback` (4 px) | `popover=manual`, `shown_while_mounted` | none | `use_hover_dismiss` (ESCAPE) | `use_close_on_scroll` | n/a | `use_animated_open` | n/a | n/a |
| HoverCard | UC `hover_card.rs:319` | own trigger `:410-470`; `role=button`; describedby while open | HoverTimers | HoverTimers.focus | fallback (10 px) | `manual`, `shown_while_mounted`; `force_mount` dormant | none | `use_hover_dismiss` | none | n/a | `use_animated_open` | n/a | n/a |
| Popover | UC `popover.rs:244` | own trigger `:895-950`; `aria-haspopup=dialog`, `aria-expanded` | n/a | n/a | fallback (8 px) non-modal | non-modal `popover=auto` + `shown_while_mounted` (`:704-785`); modal `<dialog>` (`:561-700`) | non-modal NATIVE; modal NATIVE_BACKDROP | dismiss | modal `use_scroll_lock` (`:493,586`) | `modality::use_close_focus`; modal native trap | non-modal `use_animated_open`; modal driver | n/a | n/a |
| Dialog | UC `dialog.rs:234` | own trigger | n/a | n/a | n/a | modal `<dialog>` via driver; non-modal arm `:417-475` | NATIVE_BACKDROP (`:575`); non-modal ESCAPE_PRESS_AND_FOCUS | dismiss | `use_scroll_lock` (`:371`), `use_early_scroll_capture` (`:238`) | `focus_attributes`, native trap | driver | n/a | n/a |
| AlertDialog | UC `alert_dialog.rs:117` | own | n/a | n/a | n/a | same driver | none by design (NATIVE, `:319`) | dismiss | `use_scroll_lock` | `focus_attributes` | driver | n/a | n/a |
| Sheet / Drawer | via DialogRoot (`drawer.rs:146`) | via Dialog | n/a | n/a | CSS | via Dialog | via Dialog | via Dialog | via Dialog | via Dialog | via Dialog | n/a | n/a |
| Command (dialog form) | query only (`command.rs:268`); open via DialogRoot | via Dialog | n/a | n/a | n/a | via Dialog | via Dialog | via Dialog | via Dialog | modality | via Dialog | n/a | `use_listbox_id` |
| Select | `selectable.rs:305` (UC) | own trigger `select/components/trigger.rs:189` | n/a | n/a | popper: fallback (4 px); item-aligned: own `use_item_aligned` | `manual`, `shown_while_mounted` (`list.rs:356`) | `use_dismiss` ESCAPE_PRESS_AND_FOCUS (`list.rs` about `:371`) | dismiss | stays open | `use_refocus_on_close_unless` (`select.rs:364`) | `use_animated_open` | n/a | own matcher `select/text_search.rs` |
| Combobox | `selectable.rs:305` | own input `combobox/components/input.rs:147` | n/a | n/a | fallback (4 px) | `manual`, `shown_while_mounted` | `use_dismiss` | dismiss | stays open | focus stays on the input | `use_animated_open` | n/a | listbox, collection |
| DropdownMenu | UC `dropdown_menu.rs:200` | own trigger `:509` | n/a | `use_open_focus_sync` | `use_menu_panel` | `popover=auto`, `use_popover_sync` | MENU | dismiss | stays open | `use_refocus_on_close_unless` (`:288`) | `use_menu_content_lifecycle` | `menu_sub` | typeahead, collection |
| ContextMenu | UC `context_menu.rs:188` | own (right click, long press `:393`) | n/a | `use_open_focus_sync` | point anchor | `popover=manual` | MENU | dismiss | `ScrollLockGuard` (`:734`) | `use_refocus_on_close_unless` (`:257`) | same | `menu_sub` | same |
| Menubar | own open index (no UC in `menubar.rs`) | own | n/a | n/a | `use_menu_panel` | `popover=auto` | MENU (`:379`) | dismiss | stays open | own | same | `menu_sub` | same |
| NavigationMenu | `HoverIntent.open` | own `navigation_menu.rs:711` | HoverIntent | own blur debounce (`:175`) | fallback | `manual`, `shown_while_mounted` (`:1002`) | ESCAPE_AND_PRESS, `Scope::Closest("nav")` (`:927`) | dismiss | stays open | own | `use_animated_open` | safe triangle via HoverIntent | collection |
| Navbar | `HoverIntent` | own | HoverIntent | n/a | fallback | `popover=auto`, `shown_while_mounted` (`:924`) | native light dismiss | ESCAPE (`:828`) | stays open | own | `use_animated_open` | safe triangle | collection |
| DatePicker | PopoverRoot (`date_picker/popover.rs`) | via Popover | n/a | n/a | via Popover | via Popover | via Popover | via Popover | non-modal | via Popover | via Popover | n/a | `picker.rs` collection |
| ColorPicker | no overlay in `color_picker.rs` | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| Toast | own queue | n/a | pause on hover and focus-in (`toast.rs:431-460`), own | F6 own (`:589`) | fallback | `popover=manual` + `shown_while_mounted` (`:882,920`) | none | none | n/a | own region | own | n/a | n/a |
| Chart tooltip | `ctx.active_index`; open is `active.is_some()` (`chart/components/tooltip.rs:262`) | none; `aria-hidden` | none (the Chart wrapper drives it) | `chart.rs:713` | own `transform: translate` from `place_tooltip` inside the chart box (`tooltip.rs:332-347`) | none (inline, always rendered) | `onpointerleave`, `onblur` clear (`chart.rs:724-735`) | own (`chart.rs:750`) | n/a | n/a | CSS only | n/a | arrow keys own (`chart.rs:736-770`) |

Other surfaces that open a `popover` or `<dialog>`: Toast (`toast.rs:920`), Navbar, and the Select and Combobox lists. A search for `popover=`, `popover:` and `<dialog` across `primitives/src` finds no other file.

## 3. Duplications and seams

1. **Tooltip and HoverCard are one component twice.** Root `tooltip.rs:142-232` and `hover_card.rs:311-350`; trigger `tooltip.rs:254-370` and `hover_card.rs:410-470`; content `tooltip.rs:398-440` and `hover_card.rs:550-600`; browser arm `tooltip.rs:474-558` and `hover_card.rs:628-706`; native arm `tooltip.rs:561-613` and `hover_card.rs:707-760`. About 330 lines each, about 80 percent the same recipe. Differences: Tooltip has pointer-down-closes with a `pressed` focus guard (`tooltip.rs:285-310`), `aria-describedby` always, `use_close_on_scroll`, a 4 px gap and an `r#as` trigger; HoverCard has `force_mount` with a dormant instance (`hover_card.rs:567-585`), `role=button`, describedby while open, a 10 px gap and no scroll close. The pressed guard and `force_mount` have comments and rows (166, 425), so they are deliberate; the 4 versus 10 px gap and the scroll close have no row or spec found. HoverCard content carries `role="tooltip"` (`hover_card.rs:672`), which has no stated reason.
2. **Hover timing exists three times.**

| | `HoverTimers` (`hover_card.rs:44`) | `HoverIntent` (`hover_intent.rs:27`) | `SubMenuState` (`menu_sub.rs:232-300`) |
|---|---|---|---|
| Shape | one boolean surface | one open index among siblings | one boolean per submenu |
| Delays | props `delay` (default 600) and `close_delay` (default 0), 0 is immediate | constants 50 ms and 50 ms | constants 100 ms and 0 ms |
| Switching | n/a | a sibling while one is open switches at once | n/a |
| Safe triangle | none; content `hold` and `close_delay` cover the gap | yes | yes |
| Touch rule | per event, `pointer_type() != "touch"` | per instance, a window `pointerover` and `pointerdown` listener (`hover_intent.rs` about `:105`) | none (`open_on_hover` prop only) |
| Focus and Escape | owns focus, blur and `escape_in_gap` | none (host) | none |
| Delay source | Base UI defaults (`tooltip.rs:94`, `hover_card.rs:258`) | Base UI and Radix (`hover_intent.rs:66-73`) | Base UI (`menu_sub.rs:76-87`) |

   The index-keyed versus boolean shape and the delay values are deliberate (documented). The touch rule written two ways is accidental: nothing pins that they agree. All three sit on `DelayedAction`, so the timer is shared.
3. **The anchored panel recipe is copied 8 times** (`use_popover_shown_while_mounted`, `use_anchor_position_fallback`, `merge_attributes` for `role`, `popover`, `data-side`, `data-align` and open state, `anchored_content_attributes`): Tooltip `tooltip.rs:490-540`, HoverCard `hover_card.rs:648-695`, Popover non-modal `popover.rs:732-760`, Navbar `navbar.rs:924-1000`, NavigationMenu `navigation_menu.rs:1002-1050`, Select list `select/components/list.rs:356-460`, Combobox list `combobox/components/list.rs:145-185`, Toast `toast.rs:882-925`. About 25 code lines each plus long comments, about 200 code lines. The menu family already generalises it: `use_menu_panel` (`menu_root.rs:262-330`) is the same recipe once for Dropdown, Context, Menubar and submenus. The Manual versus Auto and while-mounted versus immediate choices are deliberate (`popover_sync.rs`, `tooltip.rs:452-472`); the repeated explanatory comments are accidental.
4. **Two exit-wait implementations:** `use_animated_open` (`lib.rs:849`, 250 ms hold after `getAnimations`) and the dialog driver (`lib.rs:616`, waits for all animations, bounded by `DIALOG_EXIT_TIMEOUT_MS`, generation-guarded). Deliberate (`lib.rs:555-615` explains the top-layer reason).
5. **The modal dialog recipe is written 3 times:** `use_dialog_close_sync`, `use_dialog_open_driver`, `use_dismiss(NATIVE_BACKDROP)`, `use_scroll_lock` and `focus_attributes` at `dialog.rs:544-600`, `alert_dialog.rs:292-340`, `popover.rs:561-620`. About 25 lines each; the differences are deliberate (AlertDialog has no outside dismissal; the Popover modal gates the lock on `is_modal`).
6. **Id and trigger plumbing:** `use_unique_id` with `use_id_or` appears 52 times, and the content-id-to-trigger-id sync effect is at `tooltip.rs:404-407`. Trigger attribute merging (`merge_attributes` and `anchored_trigger_attributes`) is repeated in 9 triggers with different orders. Accidental, small per site.
7. **Controlled and uncontrolled open:** `use_controlled` is shared (23 sites); the one-line call repeated for 7 overlays is not worth an abstraction.
8. **Shared code inside a component file:** `HoverTimers` and `use_hover_dismiss` (`hover_card.rs:44-190`, `:613`) are imported by `tooltip.rs:3`; `overlay_off` and `aria_modal` (`dialog.rs:118,125`) serve AlertDialog and Popover; `use_dialog_open_driver` and `use_animated_open` live in the 1900-line `lib.rs`.
9. **Components that re-implement a shared module:** none for dismissal (all 12 overlays that dismiss use `use_dismiss`); Toast and the chart tooltip have no dismiss layer, and the chart handles Escape itself (`chart.rs:750`). Toast's pause-on-hover timing (`toast.rs:431-460`) and Select's `use_item_aligned` are own code by design.

## 4. Reference libraries (from memory, unverified)

| Reference | This repo |
|---|---|
| Radix `Popper` (floating-ui) | `top_layer/anchor_fallback.rs` and CSS anchor positioning (`top_layer/anchor.css`): present, a different engine |
| Radix `DismissableLayer` | `dismiss.rs`: present, layer-aware |
| Radix `FocusScope` | Missing as a Rust module: modals get a native trap from `<dialog>`; non-modal surfaces use the `modality.rs` open-focus policy; the native-arm trap is removed (row 448, first half) |
| Radix `Presence` | `use_animated_open` and `use_dialog_open_driver` (two implementations, 3.4) |
| Radix `Portal` | The top layer (`popover` attribute) replaces it; `portal.rs` has one user, Toast |
| Radix `Menu` shared by Dropdown, Context, Menubar | `menu_root.rs`, `menu_item.rs`, `menu_sub.rs` (row 435, done) |
| Radix roving focus group | `collection.rs` |
| Base UI Positioner and Popup parts; floating hooks (`useHover`, `useFocus`, `useDismiss`, `useRole`, `FloatingFocusManager`, `safePolygon`) | The hover trio (3.2), `dismiss.rs`, `safe_triangle.rs`; missing: one popup-part recipe that Tooltip, HoverCard, Popover, Select and Combobox share |

## 5. Rows already planned

- **449** (proof port B) rewrites `popover_sync.rs` so the hooks return the attribute to spread, and edits the spread line and the `onbeforetoggle` handler at 13 call sites (`tooltip.rs:452`, `dropdown_menu.rs:765,1927`, `select/components/list.rs:378`, `hover_card.rs:600`, `navbar.rs` and more; 16 h inferred in the row). It collides with the 8-fold recipe in 3.3: each copy is edited unless the copies are one first.
- **447** (order runtime) precedes 448 and 449. **Row 515 is on `lane/order-runtime-447`, not on this base** (it lands with the next batch); `dev-docs/STATUS.md` says it holds what row 447 leaves to the rows that own the consumers.
- **448:** the native focus trap is removed (PR #182); the second half is `otpFilter`, unrelated to overlays except that it shares `lib.rs` order with `dialog.rs`, `alert_dialog.rs` and `popover.rs`.
- **456** (3.OV) "ports JavaScript to Rust or TS; it does not unify"; it owns `hover_intent.rs`, `menu_sub.rs`, `navbar.rs`, `navigation_menu.rs` and `modality.rs` lines and depends on 449. A hover merge or panel recipe lands before it or is coordinated with it.
- **460** (3.HV, conditional) replaces frame-rate vdom handlers including `chart.rs:701` and `context_menu.rs:531`; relevant to a pointer-following chart tooltip.
- **462** (roving focus, conditional): no overlap.
- **516** (in progress) is one entry-motion recipe in the theme CSS for the overlays; it removes no Rust duplication.
- **517** (proposed, follows 449): the chart tooltip on the Tooltip's content layer; its construction matches section 6.
- **452** (3.PL) ports the placement JavaScript under `anchor_fallback.rs`, consuming the executor and `popover_sync.rs`; a Rust-side panel recipe over those hooks does not collide with it.
- Done and relevant: 435 (menu family), 487 (dismiss core), 499 (safe triangle), 166 (delay props), 19 (`use_popover_shown_while_mounted`), 432 (one anchor attribute).

No open row removes the Tooltip and HoverCard duplication (3.1), the hover trio (3.2) or the 8-fold panel recipe (3.3).

## 6. Judgement

**The layering is not optimal; the lower layers are good.** Dismissal, the top-layer and anchor engine, the menu family, the safe triangle, the delayed task and the scroll lock are each one implementation with gates. The gap is one level up: no shared popup part composes them, so every non-menu overlay re-assembles the same five calls, and the hover logic sits in a component file next to two siblings.

**The one restructuring first: `top_layer::use_anchored_panel`, the generalised `use_menu_panel`.** One hook and one config struct (`id`, `open`, `set_open`, `kind: Manual | Auto`, `sync: WhileMounted | Immediate`, anchor id, `side`, `align`, gap, owned attributes) that returns the attribute list to spread, as `use_menu_panel` does at `menu_root.rs:262-330`.
- **Moves onto it:** Tooltip, HoverCard, Popover non-modal, Navbar, NavigationMenu, Select list, Combobox list, Toast (8); `use_menu_panel` becomes a thin caller, so the three menu hosts share it too.
- **Deletes:** about 200 code lines of the 8 copies against about 80 for the hook, so about 120 to 180 net (inferred), plus the repeated comments. Row 449's 13 call-site edits become one.
- **Could change behaviour:** the per-site differences (Manual versus Auto, while-mounted versus immediate, gaps 4, 8 and 10, HoverCard's dormant instance, Select's item-aligned case, which carries no engine attribute at `list.rs:430-460`). Pinned by (filenames only; content unread) `playwright/tooltip.spec.ts`, `hover-card.spec.ts`, `hover-card-force-mount.spec.ts`, `popover.spec.ts`, `navigation_menu.spec.ts`, `anchored-close-clip.spec.ts`, `dismiss-parity.spec.ts`, `escape-gap.spec.ts`, `oracle/tier2-html/top-layer.spec.ts`, and the unit tests `tooltip.rs:641` (an SSR open tooltip with one folded `style`), `hover_card.rs:784`, `top_layer/tests.rs`.
- **Order:** before row 449, or as its first step, because 449 rewrites the same lines. Independent of 447.
- **Size (inferred):** 8 to 10 h, one lane, medium risk.
- **Free precursor (inferred 0.5 h, no behaviour change):** move `use_hover_timers`, `HoverTimers` and `use_hover_dismiss` out of `hover_card.rs` into their own module.

**Further candidates:**
1. `HoverSurface`: Tooltip and HoverCard as one root, trigger and content with a config struct (aria mode, press closes, scroll close, gap, force mount); about 250 lines (inferred), about 5 h, low risk, right after the panel recipe.
2. One hover-intent engine under `HoverTimers`, `HoverIntent` and `SubMenuState` (one touch rule, one set of delay constants); about 100 to 150 lines, medium risk (`navigation_menu.spec.ts`, `safe-triangle.spec.ts`), after row 456.
3. `use_modal_dialog` for the dialog quartet (3.5); about 60 lines, low risk; touches row 448's second half only through `lib.rs` order.

**Already well layered, leave alone:** `dismiss.rs`, the menu family, `safe_triangle.rs`, `delayed_action.rs`, `scroll_lock.rs`, `window_once.rs`, `top_layer/attributes.rs`, `use_controlled`. `use_dialog_open_driver` and `use_animated_open` differ on purpose (top-layer exit); keep them separate unless row 449's executor absorbs both.

**The chart tooltip as the library's `Tooltip` with different config and data.** Feasible, mostly with options that exist; it is row 517's construction. The chart wrapper already owns the trigger-like events (`chart.rs:700-770`: `onpointermove`, `onpointerdown`, `onfocus`, `onpointerleave`, `onblur`, `onkeydown` with Escape), and `active_index` is the open state. The tooltip is inline, always rendered, and positioned by `transform` from `place_tooltip` inside the chart box (`chart/components/tooltip.rs:332-347`), with `data-placed` and `data-motion` driving a slide.

| What stands in the way | Status |
|---|---|
| Controlled `open` | Exists (`tooltip.rs:164`); the chart passes `open: active.is_some()` and maps `on_open_change(false)` to `ctx.clear()` |
| `delay` 0 is on contact, `close_delay` | Exist (`TooltipProps`, `HoverTimers::pointer_enter`, `hover_card.rs:84-107`) |
| Trigger hover and focus handlers on an inert anchor | Small prop: `TooltipTrigger` always sets `tabindex=0` and pointer, focus, blur and keydown handlers (`tooltip.rs:297-318`); caller attributes merge after the base (`:319`) but cannot drop handlers |
| `aria-describedby` and `role="tooltip"` on an inert anchor | Small prop: the chart tooltip is `aria-hidden` and the hidden table is the accessible path (`chart/components/tooltip.rs:193-201`); `TooltipTrigger` writes `aria-describedby` unconditionally (`tooltip.rs:312`) and the content always has `role="tooltip"` (`:520`) |
| Pointer-follow mode, an anchor that moves | Real work. Placement re-runs only when its deps change (`anchor_fallback.rs:175-200`), so a `Point` anchor re-installs the page script on every move. With CSS anchor positioning the content follows an anchor element moved by Rust, so the engine path may need no script; the JavaScript fallback (Firefox, Safari) does not track an anchor that moves while open (it listens to scroll, resize and `animationend`), so it needs an observe-the-anchor option |
| The slide between points (`data-motion`) | Open question (from memory, unverified: anchor positioning follows the anchor's laid-out box, and a transition on the anchor's transform is not guaranteed to be followed); test first if the slide is required |
| SSR markup | No obstacle: `TooltipContent` renders nothing while closed and `tooltip.rs:641` pins an SSR open tooltip with one folded `style`. The chart tooltip is always mounted for its fade; `Tooltip` mounts through `use_animated_open` (row 516 covers entry fade) |
| Per-point content | No obstacle: `children` carries the rows |
| Escape scope | Behaviour decision: `use_hover_dismiss` registers a document-level Escape layer while open (`hover_card.rs:613-624`), so Escape anywhere closes the tooltip, not only with focus in the chart (`chart.rs:750`) |
| Flip and clamp inside the chart box versus the viewport, `data-side-x` and `data-side-y` | Owner decision (row 517): the engine clamps to the viewport and publishes `data-placed-side`; themed CSS changes |
| Close on scroll | Exists for `Tooltip` (`ScrollClose::AnyScroll`); row 517 hides on scroll in the fallback |

Net: three small props (inert anchor, no ARIA linkage, no trigger handlers), one decision (Escape scope), one real piece of work (an anchor that moves while open, in the fallback) and one open question (the slide).

## Not examined

- Playwright specs and Rust test bodies: filenames and one SSR test location only; what they assert is unread.
- Toast internals beyond its popover wiring and F6 and pause code; `select/components/list.rs` item-aligned placement script.
- `top_layer/styles.rs` and `anchor.css` rules (whether `data-dx-anchored="tooltip"` supports a side and align the chart needs; whether a CSS anchor follows a transform).
- Whether Menubar hover switching has its own timing (a search for `DelayedAction` finds none; assumed none).
- The Combobox `use_dismiss` policy value (the call exists in `combobox/components/list.rs`; its `Policy` is unread).
- Bridge and legacy internals beyond `bridge/behaviours.rs` (`bridge/legacy.rs`, `session.rs`, `transport.rs`) and the `decision-kernel` crate.
- ColorPicker's composition in `preview/` (`color_picker.rs` has no overlay).
- Rows 436, 438, 442 to 446, 450, 509, 513 and 514 beyond their mentions in `dev-docs/STATUS.md`; row 515's content (it is on `lane/order-runtime-447`).
- Every "duplicated lines" figure is an estimate from reading structure, not a diff.
