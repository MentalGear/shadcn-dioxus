use dioxus::prelude::*;
use dioxus_primitives::alert_dialog::{AlertDialogContent, AlertDialogRoot, AlertDialogTitle};
use dioxus_primitives::dialog::{DialogContent, DialogRoot, DialogTitle};
use dioxus_primitives::context_menu::{ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger};
use dioxus_primitives::dropdown_menu::{DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger};
use dioxus_primitives::hover_card::{HoverCard, HoverCardContent, HoverCardTrigger};
use dioxus_primitives::menubar::{Menubar, MenubarContent, MenubarItem, MenubarMenu, MenubarTrigger};
use dioxus_primitives::navbar::{Navbar, NavbarContent, NavbarItem, NavbarNav, NavbarTrigger};
use dioxus_primitives::popover::{PopoverContent, PopoverRoot, PopoverTrigger};
// Themed wrapper, not the raw primitive: the Navigation Menu fixture family
// (dev-docs/backlog.md row 72) composes `crate::components::navigation_menu::*`
// specifically, unlike every other overlay in this file (this file's own
// composition-rule exemption in `scripts/check-preview-composition.sh` covers
// either choice).
use crate::components::navigation_menu::{
    NavigationMenu, NavigationMenuContent, NavigationMenuItem, NavigationMenuLink,
    NavigationMenuList, NavigationMenuTrigger,
};
// Themed `Dialog` for the scroll-lock handoff fixture below: the real exit animation is what
// keeps the closing dialog (and its lock) alive while the next one opens.
use crate::components::dialog as themed_dialog;
use dioxus_primitives::tooltip::{Tooltip, TooltipContent, TooltipTrigger};
use dioxus_primitives::select::{Select, SelectList, SelectOption, SelectTrigger, SelectValue};
use dioxus_primitives::combobox::{Combobox, ComboboxInput, ComboboxList, ComboboxOption};
use dioxus_primitives::toast::{use_toast, ToastOptions, ToastProvider};
use dioxus_primitives::ContentSide;

#[css_module("/src/components/top_layer/style.css")]
struct Styles;

/// A fixture for the top-layer oracle (Phase 4.4, dev-docs/plan.md)
/// (`playwright/oracle/tier2-html/top-layer.spec.ts`).
///
/// Every rule the oracle checks is calibrated against a native reference --
/// a plain `<div popover>` + `<button popovertarget>`, the browser's own
/// implementation of the exact same WHATWG HTML feature our components
/// use -- placed in the same fixture, tier-2 style (see
/// `dev-docs/conformance-harness.md` and `preview/src/components/form` for the
/// established pattern).
///
/// Reachable only at its own page, `/component/top_layer/` --
/// `preview/src/site/gallery.rs`'s `ComponentGallery` deliberately excludes
/// `top_layer` from the `/` home-page catalog grid (it is not an
/// installable component, just this large oracle probe surface), while
/// `components::DEMOS`/`DocsSidebar` still list it under "Overlays" so it
/// stays navigable.
#[component]
pub fn TopLayerFixture() -> Element {
    // Fixture for the native-dialog oracle
    // (`playwright/oracle/tier2-html/native-dialog.spec.ts`): the modal
    // `Dialog` renders a real `<dialog>` + `showModal()` on the web arm
    // (dev-docs/plan.md Phase 4.2). Calibrated against a plain native `<dialog>`
    // opened by a single `showModal()` call -- there is no fully declarative
    // opener for `<dialog>` here (the Invoker Commands API's
    // `command="show-modal"` would give one, but its cross-engine support
    // is not verified) -- so the reference below
    // has exactly one line of JS to open it and closes natively via
    // `<form method="dialog">`, same spirit as the `popovertarget` reference
    // above: no library state or logic under test, just the platform
    // feature.
    let mut dialog_clip_open = use_signal(|| false);
    let mut dialog_inert_open = use_signal(|| false);
    let mut bg_clicks = use_signal(|| 0u32);
    // Fixture for the two-engine overlay architecture
    // (modal `Popover` -> native `<dialog>`/`showModal()` on the web arm):
    // its own background-inertness click counter, mirroring `bg_clicks`
    // above but kept separate so the two sections' assertions can never
    // cross-contaminate each other's counts.
    let mut popover_modal_bg_clicks = use_signal(|| 0u32);
    // Scroll-lock handoff (backlog row 375): two SIBLING dialogs, one click closes A and opens B.
    let mut handoff_a_open = use_signal(|| false);
    let mut handoff_b_open = use_signal(|| false);

    rsx! {
        div { class: Styles::dx_top_layer_fixture,

            section { class: Styles::dx_top_layer_section,
                h2 { "Clipping escape" }
                p { class: Styles::dx_top_layer_hint,
                    "Every trigger below sits inside "
                    code { "#clip-box" }
                    ", an ancestor with "
                    code { "overflow: hidden; height: 60px; transform: translateZ(0)" }
                    ". Each content panel sets an explicit "
                    code { "min-height: 100px" }
                    ", taller than the clip -- open one and it should render fully "
                    "visible outside the clip (WHATWG HTML §popover, top layer), not "
                    "cut off at 60px."
                }
                div { id: "clip-box", class: Styles::dx_top_layer_clip_box,

                    Tooltip {
                        TooltipTrigger { id: "clip-tooltip-trigger", "Tooltip trigger" }
                        TooltipContent {
                            id: "clip-tooltip-content",
                            style: "min-height: 100px;",
                            "Tooltip content for the clipping-escape rule."
                        }
                    }

                    HoverCard {
                        HoverCardTrigger { id: "clip-hovercard-trigger", "HoverCard trigger" }
                        HoverCardContent {
                            id: "clip-hovercard-content",
                            style: "min-height: 100px;",
                            "HoverCard content for the clipping-escape rule."
                        }
                    }

                    PopoverRoot { id: "clip-popover-root", is_modal: false,
                        PopoverTrigger { id: "clip-popover-trigger", "Popover trigger" }
                        PopoverContent {
                            "data-dx-surface": "floating",
                            id: "clip-popover-content",
                            style: "min-height: 100px;",
                            "Popover content for the clipping-escape rule."
                        }
                    }

                    // DropdownMenu (dev-docs/backlog.md item 2): web-arm
                    // `popover="auto"`, like the three exemplars above -- see
                    // this rule in `top-layer.spec.ts` (a plain, non-`popover`
                    // `DropdownMenuContent` div would clip at the ancestor).
                    DropdownMenu { id: "clip-dropdown-menu-root",
                        DropdownMenuTrigger { id: "clip-dropdown-menu-trigger", "DropdownMenu trigger" }
                        DropdownMenuContent {
                            id: "clip-dropdown-menu-content",
                            style: "min-height: 100px;",
                            DropdownMenuItem::<String> {
                                value: "one".to_string(),
                                on_select: move |_: String| {},
                                "Item one"
                            }
                        }
                    }

                    // ContextMenu's web arm is `popover="manual"`
                    // (`ContextMenuContentRendered`, `context_menu.rs`). The
                    // case is red against a plain, `position: fixed`-only
                    // div, which clips at the 60px ancestor. Opened by
                    // right-click, like every other `ContextMenu` in this
                    // workspace.
                    ContextMenu { id: "clip-context-menu-root",
                        ContextMenuTrigger { id: "clip-context-menu-trigger", "ContextMenu trigger (right-click)" }
                        ContextMenuContent {
                            id: "clip-context-menu-content",
                            style: "min-height: 100px;",
                            ContextMenuItem {
                                value: "one".to_string(),
                                on_select: move |_: String| {},
                                "Item one"
                            }
                        }
                    }

                    // Menubar menus' web arm is `popover="auto"`, anchored
                    // to their own trigger (`MenubarContentRendered`,
                    // `menubar.rs`). The case is red against a plain,
                    // `position: absolute`-only div, which clips at the
                    // 60px ancestor.
                    Menubar { id: "clip-menubar-root",
                        MenubarMenu {
                            MenubarTrigger { id: "clip-menubar-trigger", "Menubar trigger" }
                            MenubarContent {
                                id: "clip-menubar-content",
                                style: "min-height: 100px;",
                                MenubarItem {
                                    value: "one".to_string(),
                                    on_select: move |_: String| {},
                                    "Item one"
                                }
                            }
                        }
                    }

                    // Select's web arm is `popover="auto"`
                    // (`SelectListRendered`, `list.rs`). The case is red
                    // against a plain div, which clips at the 60px ancestor
                    // like the others.
                    Select::<String> { id: "clip-select-root", item_to_label: |value: String| value,
                        SelectTrigger { id: "clip-select-trigger", SelectValue { placeholder: "Select trigger" } }
                        SelectList {
                            id: "clip-select-content",
                            style: "min-height: 100px;",
                            SelectOption::<String> {
                                value: "one",
                                "Option one"
                            }
                        }
                    }

                    // Combobox's web arm is `popover="auto"`
                    // (`ComboboxListRendered`, `list.rs`). The case is red
                    // against a plain div, which clips at the 60px ancestor
                    // like the others.
                    Combobox::<String> { id: "clip-combobox-root", item_to_label: |value: String| value,
                        ComboboxInput { id: "clip-combobox-trigger", aria_label: "Combobox trigger" }
                        ComboboxList {
                            id: "clip-combobox-content",
                            style: "min-height: 100px;",
                            ComboboxOption::<String> {
                                value: "one",
                                "Option one"
                            }
                        }
                    }

                    // `NavbarNav`'s web arm is `popover="auto"`
                    // (`NavbarContentRendered`, `navbar.rs`), so `Navbar`
                    // gets clipping escape, top-layer stacking and (Rule
                    // 5-style below) viewport-edge flip the way
                    // `DropdownMenu`/`Menubar`/`Select` do. A plain,
                    // `position: absolute`-only div would clip at the 60px
                    // ancestor.
                    Navbar { id: "clip-navbar-root", aria_label: "Clip navbar test",
                        NavbarNav { index: 0usize,
                            NavbarTrigger { id: "clip-navbar-trigger", "Navbar trigger" }
                            NavbarContent {
                                id: "clip-navbar-content",
                                style: "min-height: 100px;",
                                NavbarItem {
                                    index: 0usize,
                                    value: "one".to_string(),
                                    to: crate::Route::home(),
                                    "Item one"
                                }
                            }
                        }
                    }

                    // dev-docs/backlog.md row 72 (Navigation Menu oracle
                    // coverage): `NavigationMenuContent`'s web arm is
                    // `popover="manual"`, anchored to its own trigger
                    // (`NavigationMenuContentRendered`, `navigation_menu.rs`)
                    // -- the same top-layer engine `NavbarContent` above
                    // uses (marker class `dx-anchor-navigation-menu`,
                    // already wired alongside `dx-anchor-navbar` in
                    // `top_layer.rs`'s shared anchor-positioning
                    // stylesheet). A *different* APG pattern from `Navbar`
                    // (Disclosure Navigation, not Menu/Menubar -- see
                    // `navigation_menu.rs`'s module doc), but the same
                    // hover/click-opened, trigger-anchored top-layer shape
                    // this rule checks, so it gets an identical clip-*
                    // fixture instance right alongside Navbar's.
                    NavigationMenu { id: "clip-navigation-menu-root", aria_label: "Clip navigation menu test",
                        NavigationMenuList {
                            NavigationMenuItem {
                                NavigationMenuTrigger { id: "clip-navigation-menu-trigger", "Navigation menu trigger" }
                                NavigationMenuContent {
                                    id: "clip-navigation-menu-content",
                                    style: "min-height: 100px;",
                                    NavigationMenuLink { href: "#", "Item one" }
                                }
                            }
                        }
                    }

                    // Native reference: the browser's own implementation of the
                    // same WHATWG HTML feature, with no Dioxus involvement at
                    // all -- the calibration control for this rule.
                    button {
                        id: "clip-native-trigger",
                        popovertarget: "clip-native-content",
                        "Native trigger"
                    }
                    div {
                        id: "clip-native-content",
                        popover: "auto",
                        style: "min-height: 100px; min-width: 200px; border: 1px solid; padding: 0.5rem; background: Canvas; color: CanvasText;",
                        "Native reference content for the clipping-escape rule."
                    }
                }
            }

            section { class: Styles::dx_top_layer_section,
                h2 { "Light dismiss, Escape, and stacking" }
                p { class: Styles::dx_top_layer_hint,
                    "The red panel is a high-"
                    code { "z-index" }
                    " sibling positioned where each popover opens. Opening a "
                    code { "popover=\"auto\"" }
                    " (or a non-modal "
                    code { "Popover" }
                    ") should render its content above that sibling (top layer "
                    "stacking). Clicking elsewhere, or pressing Escape, should "
                    "close it and sync back to "
                    code { "data-state" }
                    " on the root/trigger -- the Rust signal must never strand."
                }
                div { class: Styles::dx_top_layer_stack_area,
                    div { id: "stack-sibling", class: Styles::dx_top_layer_stack_sibling, "High z-index sibling" }

                    div { class: Styles::dx_top_layer_stack_row,
                        PopoverRoot { id: "stack-popover-root", is_modal: false,
                            PopoverTrigger { id: "stack-popover-trigger", "Stacking popover trigger" }
                            PopoverContent {
                                id: "stack-popover-content",
                                "data-dx-surface": "floating",
                                "Stacking popover content"
                            }
                        }

                        button {
                            id: "stack-native-trigger",
                            popovertarget: "stack-native-content",
                            "Native stacking trigger"
                        }
                        div {
                            id: "stack-native-content",
                            // Plain, hand-written marker class -- never run
                            // through `#[css_module]`'s hashing, deliberately:
                            // that hashing only rewrites selectors it finds as
                            // flat top-level rules, not ones nested inside an
                            // `@supports` block, so the hashed class would
                            // land on this element while the `@supports`-scoped
                            // rule kept referencing the *unhashed* name and
                            // silently never match. Same reasoning and fix as
                            // `primitives/src/tooltip.rs`'s `dx-anchor-tooltip`
                            // (see its comment) -- this fixture just needs its
                            // own instance since it isn't a primitives
                            // component.
                            class: "dx-anchor-stack-native",
                            popover: "auto",
                            style: "min-width: 200px; border: 1px solid; padding: 0.5rem; background: Canvas; color: CanvasText;",
                            "Native stacking content"
                        }
                    }

                    // An element well away from both triggers, for the
                    // light-dismiss ("click outside") assertions.
                    button { id: "outside-click-target", class: Styles::dx_top_layer_outside, "Click outside target" }
                }
            }

            section { class: Styles::dx_top_layer_section,
                h2 { "Toast top-layer stacking" }
                p { class: Styles::dx_top_layer_hint,
                    "The small red panel is a high-"
                    code { "z-index" }
                    " sibling pinned to the same fixed viewport corner the toast "
                    "below will render at. Toasts never light-dismiss and never "
                    "anchor to a trigger -- "
                    code { "ToastProvider" }
                    "'s region is `popover=\"manual\"` purely so it always paints "
                    "in the top layer, above whatever else is on the page "
                    "(top-layer stacking, the same rule as the section above). "
                    "Click the trigger to add a toast and cover the sibling."
                }
                // `position: fixed`, not `absolute`, on both -- this matters,
                // it is not a style preference: moving an
                // element into the top layer changes its containing block
                // to the *initial* containing block regardless of its own
                // `position` value (see `crate::top_layer::anchor_name_
                // style`'s doc in `primitives/src/top_layer/attributes.rs` for the
                // general rule this is an instance of), and for `position:
                // absolute` specifically that initial containing block is
                // sized like the viewport but anchored to the *document's*
                // origin, not the current scroll position -- so `position:
                // absolute; top: 0; left: 0` on the (popover-promoted) toast
                // region nested inside a `position: relative` wrapper would
                // render correctly relative to that wrapper only when the
                // page is scrolled to the very top, and hundreds of pixels
                // off-screen otherwise (the "toast not findable" failure).
                // `position: fixed` sidesteps this: fixed's containing block is the viewport
                // itself either way, top layer or not, so `top: 0; left: 0`
                // reliably means the actual visible viewport corner
                // regardless of scroll -- matching this component's own
                // real-world CSS (`preview/src/components/toast/style.css`'s
                // `.dx-toast-container` is `position: fixed` for the same
                // reason).
                div {
                    id: "toast-stack-sibling",
                    style: "position: fixed; bottom: 0; right: 0; z-index: 99999; width: 40px; height: 20px; background: crimson;",
                }
                // No `id` here, deliberately: `ToastProviderProps` has no
                // dedicated `id` field (the region's id is
                // purely internal, generated by `use_unique_id` inside
                // `toast.rs`, since the F6-focus-region wiring needs one
                // but nothing caller-visible does) -- an `id` passed here
                // would be absorbed by its `#[props(extends =
                // GlobalAttributes)]` catch-all into `attributes`.
                // `ToastRegionRendered` drops any `id` found there
                // explicitly before merging (see its doc, "Attribute-
                // override dedup") rather than relying on spread order;
                // spread order alone would let an id passed here override
                // the generated one, stranding
                // `use_popover_shown_while_mounted`'s `document.
                // getElementById`, so the toast would never become
                // `:popover-open`.
                //
                // `top: auto; left: auto; margin: 0;` alongside `bottom`/
                // `right` -- also necessary, and the exact reason `preview/src/components/toast/style.css`'s
                // real `.dx-toast-container[popover]` rule resets all four
                // sides plus `margin`, not just the two this fixture cares
                // about: the WHATWG popover UA stylesheet's `[popover] {
                // inset: 0; margin: auto; ... }` still supplies `top: 0;
                // left: 0` for the two sides this element's own style never
                // mentions, and combined with the UA's own `margin: auto`
                // (a non-popover element's initial `margin` is `0`) that is
                // exactly the four-sides-plus-auto-margin shape CSS's own
                // centering algorithm looks for -- setting only
                // `bottom`/`right` renders the toast dead-centered in the
                // viewport instead of at the corner, nowhere near the
                // sibling.
                ToastProvider {
                    style: "position: fixed; top: auto; right: 0; bottom: 0; left: auto; margin: 0;",
                    // The app shell already mounts a toast region on every
                    // page; two `role="region"` landmarks sharing the
                    // primitive's default "N notifications" name fail axe's
                    // `landmark-unique` on the all-components page. The
                    // primitive merges this override with its own default
                    // `aria_label` via `merge_attributes` (caller-wins,
                    // deduped -- see `ToastRegionRendered`'s doc in
                    // `toast.rs`, "Attribute-override dedup"), so this
                    // value wins on both the CSR and SSR/SSG lanes, not
                    // just in the live DOM.
                    aria_label: "Top-layer fixture notifications",
                    ToastStackTrigger {}
                }
            }

            section { class: Styles::dx_top_layer_section,
                h2 { "Point-positioned vs. anchored scroll behavior" }
                p { class: Styles::dx_top_layer_hint,
                    "Both controls below sit normally in-flow (no clip ancestor, plenty of "
                    "room on every side) so a modest scroll cannot itself cross a "
                    "viewport-edge flip threshold -- these rules aren't about "
                    "flipping (Rules 5-7 already cover that), they're about what "
                    "happens to already-open content while the page scrolls. "
                    code { "ContextMenu" }
                    " opens at a raw click point with no anchor -- its content "
                    "stays at the click's "
                    code { "viewport" }
                    "-relative position, i.e. it does not move on screen as the "
                    "page scrolls underneath it (same as a native OS context "
                    "menu). "
                    code { "Menubar" }
                    "'s content, by contrast, is anchored to its own trigger "
                    "(like " code { "DropdownMenu" } "'s Rule 8 case) and must "
                    "keep tracking that trigger's position through a scroll."
                }
                ContextMenu { id: "scroll-context-menu-root",
                    ContextMenuTrigger { id: "scroll-context-menu-trigger", "Scroll test: right-click here" }
                    ContextMenuContent { id: "scroll-context-menu-content",
                        ContextMenuItem {
                            value: "one".to_string(),
                            on_select: move |_: String| {},
                            "Item one"
                        }
                    }
                }
                Menubar { id: "scroll-menubar-root",
                    MenubarMenu {
                        MenubarTrigger { id: "scroll-menubar-trigger", "Scroll test menu" }
                        MenubarContent { id: "scroll-menubar-content",
                            MenubarItem {
                                value: "one".to_string(),
                                on_select: move |_: String| {},
                                "Item one"
                            }
                        }
                    }
                }
            }

            section { class: Styles::dx_top_layer_section,
                h2 { "Near-viewport-edge flip (CSS position-try-fallbacks)" }
                p { class: Styles::dx_top_layer_hint,
                    "W3C CSS Anchor Positioning's "
                    code { "position-try-fallbacks" }
                    " ("
                    a {
                        href: "https://www.w3.org/TR/css-anchor-position-1/#fallback-var",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "spec"
                    }
                    ") -- a CSS-spec citation, not WHATWG HTML, unlike the rest of "
                    "this fixture; see the oracle spec's header for why this rule "
                    "still lives in this tier-2 file. Every trigger below is "
                    "pinned by fixed positioning right at a viewport edge, with "
                    "its preferred "
                    code { "side" }
                    " pointing off-viewport -- the bottom row's triggers request "
                    code { "side=\"bottom\"" }
                    " while sitting a few pixels from the bottom edge; the right "
                    "column's request "
                    code { "side=\"right\"" }
                    " a few pixels from the right edge -- so content can only "
                    "render fully on-screen by flipping to the opposite side on "
                    "the relevant axis. A native "
                    code { "<div popover>" }
                    " + "
                    code { "position-try-fallbacks" }
                    " reference sits in each row/column too: the browser's own "
                    "implementation of the identical spec feature, no library "
                    "code involved (CALIBRATION)."
                }
                div { class: Styles::dx_top_layer_edge_bottom_row,
                    Tooltip {
                        TooltipTrigger { id: "edge-bottom-tooltip-trigger", "Bottom tooltip" }
                        TooltipContent {
                            id: "edge-bottom-tooltip-content",
                            side: ContentSide::Bottom,
                            "Flips above its trigger when the preferred side runs off-viewport."
                        }
                    }

                    HoverCard {
                        HoverCardTrigger { id: "edge-bottom-hovercard-trigger", "Bottom hover card" }
                        HoverCardContent {
                            id: "edge-bottom-hovercard-content",
                            side: ContentSide::Bottom,
                            "Flips above its trigger when the preferred side runs off-viewport."
                        }
                    }

                    PopoverRoot { id: "edge-bottom-popover-root", is_modal: false,
                        PopoverTrigger { id: "edge-bottom-popover-trigger", "Bottom popover" }
                        PopoverContent {
                            "data-dx-surface": "floating",
                            id: "edge-bottom-popover-content",
                            side: ContentSide::Bottom,
                            "Flips above its trigger when the preferred side runs off-viewport."
                        }
                    }

                    // Unlike `Tooltip`/`HoverCard`/
                    // `PopoverContent` above, `NavbarContent` has no `side`
                    // prop -- its placement is always below/start-aligned,
                    // the same fixed convention `MenubarContent`/
                    // `DropdownMenuContent` use (see `NavbarContentRendered`'s
                    // doc, `navbar.rs`). So the flip case here is driven by
                    // pinning the *trigger* at the bottom edge instead of by
                    // an explicit `side` prop -- the same shape a real page
                    // footer or bottom nav bar would put a `Navbar` in.
                    // A plain `position: absolute; top: 100%`-only div would
                    // never flip: the content would render straight off the
                    // bottom of the viewport.
                    Navbar { id: "edge-bottom-navbar-root", aria_label: "Edge bottom navbar test",
                        NavbarNav { index: 0usize,
                            NavbarTrigger { id: "edge-bottom-navbar-trigger", "Bottom navbar" }
                            NavbarContent {
                                id: "edge-bottom-navbar-content",
                                NavbarItem {
                                    index: 0usize,
                                    value: "one".to_string(),
                                    to: crate::Route::home(),
                                    "Flips above its trigger when the preferred side runs off-viewport."
                                }
                            }
                        }
                    }

                    // dev-docs/backlog.md row 72 (Navigation Menu oracle coverage). Like `Navbar` just
                    // above, `NavigationMenuContent` has no `side` prop
                    // (always below/start-aligned -- see
                    // `navigation_menu.rs`'s `NavigationMenuContentRendered`,
                    // hardcoded `ContentSide::Bottom`/`ContentAlign::Start`),
                    // so this pins the *trigger* at the bottom edge instead,
                    // the identical construction as the Navbar case.
                    NavigationMenu { id: "edge-bottom-navigation-menu-root", aria_label: "Edge bottom navigation menu test",
                        NavigationMenuList {
                            NavigationMenuItem {
                                NavigationMenuTrigger { id: "edge-bottom-navigation-menu-trigger", "Bottom navigation menu" }
                                NavigationMenuContent {
                                    id: "edge-bottom-navigation-menu-content",
                                    NavigationMenuLink { href: "#", "Flips above its trigger when the preferred side runs off-viewport." }
                                }
                            }
                        }
                    }

                    // Native reference (CALIBRATION): the browser's own
                    // `position-try-fallbacks: flip-block` on a plain
                    // `<div popover>`, anchored via the `dx-top-layer-edge-
                    // native-trigger-bottom` class's `anchor-name` (see
                    // style.css). No Dioxus positioning logic involved.
                    button {
                        id: "edge-bottom-native-trigger",
                        class: Styles::dx_top_layer_edge_native_trigger_bottom,
                        popovertarget: "edge-bottom-native-content",
                        "Native bottom trigger"
                    }
                    div {
                        id: "edge-bottom-native-content",
                        // Plain, hand-written marker class -- see this
                        // file's `dx-anchor-stack-native` comment above for
                        // why (`@supports`-nested selectors aren't scoped
                        // by `#[css_module]`).
                        class: "dx-anchor-edge-bottom-native",
                        popover: "auto",
                        "data-side": "bottom",
                        style: "min-height: 60px; min-width: 200px; border: 1px solid; padding: 0.5rem; background: Canvas; color: CanvasText;",
                        "Native reference: flips above via position-try-fallbacks: flip-block."
                    }
                }

                div { class: Styles::dx_top_layer_edge_right_col,
                    Tooltip {
                        TooltipTrigger { id: "edge-right-tooltip-trigger", "Right tooltip" }
                        TooltipContent {
                            id: "edge-right-tooltip-content",
                            side: ContentSide::Right,
                            "Flips left of its trigger when the preferred side runs off-viewport."
                        }
                    }

                    HoverCard {
                        HoverCardTrigger { id: "edge-right-hovercard-trigger", "Right hover card" }
                        HoverCardContent {
                            id: "edge-right-hovercard-content",
                            side: ContentSide::Right,
                            "Flips left of its trigger when the preferred side runs off-viewport."
                        }
                    }

                    PopoverRoot { id: "edge-right-popover-root", is_modal: false,
                        PopoverTrigger { id: "edge-right-popover-trigger", "Right popover" }
                        PopoverContent {
                            "data-dx-surface": "floating",
                            id: "edge-right-popover-content",
                            side: ContentSide::Right,
                            "Flips left of its trigger when the preferred side runs off-viewport."
                        }
                    }

                    // Native reference (CALIBRATION): `position-try-
                    // fallbacks: flip-inline` counterpart of the row above.
                    button {
                        id: "edge-right-native-trigger",
                        class: Styles::dx_top_layer_edge_native_trigger_right,
                        popovertarget: "edge-right-native-content",
                        "Native right trigger"
                    }
                    div {
                        id: "edge-right-native-content",
                        class: "dx-anchor-edge-right-native",
                        popover: "auto",
                        "data-side": "right",
                        style: "min-width: 200px; border: 1px solid; padding: 0.5rem; background: Canvas; color: CanvasText;",
                        "Native reference: flips left via position-try-fallbacks: flip-inline."
                    }
                }
            }

            section { class: Styles::dx_top_layer_section,
                h2 { "Native <dialog> clipping escape" }
                p { class: Styles::dx_top_layer_hint,
                    "The modal "
                    code { "Dialog" }
                    " below sits inside "
                    code { "#dialog-clip-box" }
                    ", the same "
                    code { "overflow: hidden; height: 60px; transform: translateZ(0)" }
                    " ancestor as the section above. Its content is taller than "
                    "the clip, so it must render fully visible outside it -- same "
                    "top-layer rule, now for "
                    code { "showModal()" }
                    " rather than "
                    code { "popover=" }
                    "."
                }
                div { id: "dialog-clip-box", class: Styles::dx_top_layer_clip_box,
                    button {
                        id: "dialog-clip-trigger",
                        onclick: move |_| dialog_clip_open.set(true),
                        "Dialog clip trigger"
                    }
                    DialogRoot {
                        id: "dialog-clip-root",
                        open: dialog_clip_open(),
                        on_open_change: move |v| dialog_clip_open.set(v),
                        DialogContent {
                            id: "dialog-clip-content",
                            style: "min-height: 140px; min-width: 220px; border: 1px solid; padding: 1rem; background: Canvas; color: CanvasText;",
                            DialogTitle { "Clip-escape dialog" }
                            p { "Content deliberately taller than the 60px clip ancestor." }
                            button {
                                id: "dialog-clip-close",
                                onclick: move |_| dialog_clip_open.set(false),
                                "Close"
                            }
                        }
                    }

                    // Native reference: a plain `<dialog>` opened with
                    // `showModal()` -- see this function's doc comment for
                    // why the trigger needs one line of JS.
                    button {
                        id: "dialog-clip-native-trigger",
                        onclick: move |_| {
                            document::eval(
                                "document.getElementById('dialog-clip-native-content').showModal();",
                            );
                        },
                        "Native dialog clip trigger"
                    }
                    dialog {
                        id: "dialog-clip-native-content",
                        style: "min-height: 140px; min-width: 220px; border: 1px solid; padding: 1rem; background: Canvas; color: CanvasText;",
                        p { "Native dialog reference content for the clipping-escape rule." }
                        form { method: "dialog",
                            button { id: "dialog-clip-native-close", "Close" }
                        }
                    }
                }
            }

            section { class: Styles::dx_top_layer_section,
                h2 { "Native <dialog> background inertness" }
                p { class: Styles::dx_top_layer_hint,
                    "While the alert dialog below is open, the background "
                    "button/input here must be behaviourally inert: a real "
                    "click must not reach the button's handler, and calling "
                    code { ".focus()" }
                    " on the input directly must not move "
                    code { "document.activeElement" }
                    ". (Chromium does not reflect this onto "
                    code { "Element.inert" }
                    " for "
                    code { "<body>" }
                    " -- dev-docs/archive/research/phase4-spike-findings.md -- so this fixture is measured "
                    "behaviourally, not via that property.) Uses "
                    code { "AlertDialog" }
                    ", not "
                    code { "Dialog" }
                    ", deliberately: "
                    code { "Dialog" }
                    "'s own backdrop-click dismiss (dialog.spec.ts) would "
                    "otherwise close the dialog the instant a background "
                    "click lands outside it, which is a correct interaction "
                    "but confounds this specific probe -- "
                    code { "AlertDialog" }
                    " never light-dismisses, so it stays open through the "
                    "whole check."
                }
                div {
                    div { id: "dialog-inert-bg-count", "{bg_clicks()}" }
                    button {
                        id: "dialog-inert-bg-button",
                        onclick: move |_| bg_clicks.set(bg_clicks() + 1),
                        "Background button"
                    }
                    input { id: "dialog-inert-bg-input", placeholder: "Background input" }
                }
                button {
                    id: "dialog-inert-trigger",
                    onclick: move |_| dialog_inert_open.set(true),
                    "Open inertness dialog"
                }
                AlertDialogRoot {
                    id: "dialog-inert-root",
                    open: dialog_inert_open(),
                    on_open_change: move |v| dialog_inert_open.set(v),
                    AlertDialogContent { id: "dialog-inert-content",
                        AlertDialogTitle { "Inertness dialog" }
                        button {
                            id: "dialog-inert-close",
                            onclick: move |_| dialog_inert_open.set(false),
                            "Close"
                        }
                    }
                }
            }

            section { class: Styles::dx_top_layer_section,
                h2 { "Native <dialog> modal Popover" }
                p { class: Styles::dx_top_layer_hint,
                    "The modal "
                    code { "Popover" }
                    " below (explicit "
                    code { "is_modal: true" }
                    "; the primitive's default is non-modal) renders a real "
                    code { "<dialog>" }
                    " opened with "
                    code { "showModal()" }
                    " on the web arm, driven by the same open-driver/"
                    "close-sync/backdrop-dismiss trio as modal "
                    code { "Dialog" }
                    " -- except, unlike "
                    code { "Dialog" }
                    ", a modal "
                    code { "Popover" }
                    " stays anchored to its own trigger rather than "
                    "viewport-centered. Three checks share the "
                    code { "#popover-modal-clip-box" }
                    "/inertness/anchor sub-fixtures below: clipping escape "
                    "(same "
                    code { "overflow: hidden" }
                    " ancestor as the plain-Dialog case above), background "
                    "inertness (click/focus must not reach the background "
                    "button/input), and trigger-anchored placement -- the "
                    "anchor trigger sits pinned near the top-left viewport "
                    "corner, far from the viewport center, so a "
                    "viewport-centered "
                    code { "showModal()" }
                    " (the UA default) would "
                    "measurably disagree with a trigger-anchored one."
                }
                div { id: "popover-modal-clip-box", class: Styles::dx_top_layer_clip_box,
                    PopoverRoot { id: "popover-modal-clip-root", is_modal: true,
                        PopoverTrigger { id: "popover-modal-clip-trigger", "Modal popover clip trigger" }
                        PopoverContent {
                            "data-dx-surface": "floating",
                            id: "popover-modal-clip-content",
                            // An explicit `width` (not just `min-width`) --
                            // a native modal `<dialog>` (UA `dialog:modal {
                            // width: fit-content; ... }`, un-reset by this
                            // component's own CSS) sizes to its content's
                            // natural line width by default. Without a cap,
                            // this fixture's sentence-length text renders
                            // wide enough that `align: Center`'s
                            // `transform: translateX(-50%)` -- centering it
                            // under a trigger sitting near this box's own
                            // left edge -- pushes a meaningful chunk of the
                            // content off the left edge of the *viewport*,
                            // confusing this rule's clip-escape probe
                            // (which needs the content to still land
                            // somewhere `elementFromPoint` can see) with a
                            // symptom that looks like clipping but is
                            // actually just off-canvas.
                            style: "min-height: 140px; width: 220px;",
                            "Modal popover content, taller than the clip ancestor."
                        }
                    }
                }

                div {
                    div { id: "popover-modal-inert-bg-count", "{popover_modal_bg_clicks()}" }
                    button {
                        id: "popover-modal-inert-bg-button",
                        onclick: move |_| popover_modal_bg_clicks.set(popover_modal_bg_clicks() + 1),
                        "Background button"
                    }
                    input { id: "popover-modal-inert-bg-input", placeholder: "Background input" }
                }
                PopoverRoot { id: "popover-modal-inert-root", is_modal: true,
                    PopoverTrigger { id: "popover-modal-inert-trigger", "Open inertness popover" }
                    PopoverContent { id: "popover-modal-inert-content", "data-dx-surface": "floating",
                        "Modal popover inertness content."
                    }
                }

                // Pinned well away from the viewport center (Escape/reopen
                // and focus-restore cycles, plus the trigger-anchored
                // placement measurement, all share this one trigger/content
                // pair).
                //
                // `top: calc(var(--dx-navbar-height) + 20px)`, not a plain
                // `40px` (`oracle/tier2-html/native-dialog.spec.ts`
                // 6c/6d/6e, SSG lane only): a plain `40px` sits partly underneath
                // `.dx-preview-navbar` (`position: sticky; top: 0`), which is an
                // ordinary fixed-position element that `scrollIntoViewIfNeeded`
                // cannot correct (the sticky-nav `scroll-padding-top` in
                // `main.css` only helps things that scroll into place), so the
                // trigger needs its own clearance. The clearance is not specific
                // to the SSG build: on the CSR dev server the nav can still be
                // `position: static` for a moment after load (the Google Fonts
                // `@import` and this page's own `document::Link` head injection
                // delay its CSS), which only masks the same overlap.
                //
                // `z-index: var(--dx-z-top, 9999)`: every route renders
                // `DocsLayout`'s `Sidebar` (`preview/src/components/sidebar/style.css`'s
                // `.dx-sidebar-container`, `position: fixed; z-index:
                // var(--dx-z-local-sm)` = 10, occupying `left: 0` through
                // `width: var(--dx-sidebar-width)` = 16rem), and this trigger's
                // `left: 40px` places it inside that column, where with no `z-index`
                // of its own it loses hit-testing to the Sidebar
                // (`oracle/tier2-html/native-dialog.spec.ts` Rule 6c/6d/6e). A
                // `top`/`left` nudge would only dodge that one element and leave the
                // same mechanism free to recur against whatever permanently-mounted
                // chrome comes next, so the fix is a `z-index` raise using this
                // fixture's "must clear ordinary page chrome" convention
                // (`--dx-z-top`, also used by `.dx-top-layer-stack-sibling` in
                // style.css; see style.css's matching comment on
                // `.dx-top-layer-edge-bottom-row`/`-edge-right-col`): far above *any*
                // normal-range chrome z-index, with this trigger's actual position --
                // the geometry Rule 6e's trigger-anchored placement measurement and
                // the Escape/focus-restore cycles depend on -- left untouched.
                div {
                    style: "position: fixed; top: calc(var(--dx-navbar-height, 60px) + 20px); left: 40px; z-index: var(--dx-z-top, 9999);",
                    PopoverRoot { id: "popover-modal-anchor-root", is_modal: true,
                        PopoverTrigger { id: "popover-modal-anchor-trigger", "Modal popover anchor trigger" }
                        PopoverContent { id: "popover-modal-anchor-content", "data-dx-surface": "floating",
                            "Modal popover anchored content."
                        }
                    }
                }
            }

            section { class: Styles::dx_top_layer_section,
                h2 { "Select scroll_lock opt-out (dev-docs/backlog.md rows 9/72)" }
                p { class: Styles::dx_top_layer_hint,
                    "The "
                    code { "scroll_lock: ReadSignal<bool>" }
                    " prop (default "
                    code { "true" }
                    ") on "
                    code { "Select" }
                    "/"
                    code { "SelectMulti" }
                    " opts out of the page scroll lock. The trigger below sets "
                    code { "scroll_lock=false" }
                    " explicitly, so opening its "
                    "listbox must leave page scroll unlocked (a wheel event "
                    "still scrolls the page), unlike every other overlay on "
                    "this page and unlike Select's own default-"
                    code { "true" }
                    " instance ("
                    code { "playwright/oracle/tier3-radix/scroll-lock.spec.ts" }
                    "'s default case, on Select's own gallery page)."
                }
                Select::<String> {
                    id: "scroll-lock-select-root",
                    item_to_label: |value: String| value,
                    scroll_lock: ReadSignal::new(Signal::new(false)),
                    SelectTrigger {
                        id: "scroll-lock-select-trigger",
                        SelectValue { placeholder: "Scroll-unlocked select" }
                    }
                    SelectList {
                        id: "scroll-lock-select-content",
                        SelectOption::<String> {
                            value: "one",
                            "Option one"
                        }
                    }
                }
            }

            section { class: Styles::dx_top_layer_section,
                h2 { "Scroll-lock handoff between sibling dialogs (backlog row 375)" }
                p { class: Styles::dx_top_layer_hint,
                    "One click closes dialog A and opens dialog B, a sibling, not a nested dialog. "
                    "A keeps its lock through its exit animation while B already holds one; the page "
                    "must not move under B when A's lock lets go ("
                    code { "playwright/oracle/tier3-radix/scroll-lock.spec.ts" }
                    ")."
                }
                button {
                    id: "handoff-a-trigger",
                    onclick: move |_| handoff_a_open.set(true),
                    "Open dialog A"
                }
                themed_dialog::Dialog {
                    open: handoff_a_open(),
                    on_open_change: move |v| handoff_a_open.set(v),
                    themed_dialog::DialogHeader {
                        themed_dialog::DialogTitle { "Handoff dialog A" }
                        themed_dialog::DialogDescription { "Continue closes this dialog and opens dialog B." }
                    }
                    button {
                        id: "handoff-a-continue",
                        onclick: move |_| {
                            handoff_a_open.set(false);
                            handoff_b_open.set(true);
                        },
                        "Continue"
                    }
                }
                themed_dialog::Dialog {
                    open: handoff_b_open(),
                    on_open_change: move |v| handoff_b_open.set(v),
                    themed_dialog::DialogHeader {
                        themed_dialog::DialogTitle { "Handoff dialog B" }
                        themed_dialog::DialogDescription { "Opened by the same click that closed dialog A." }
                    }
                    button {
                        id: "handoff-b-close",
                        onclick: move |_| handoff_b_open.set(false),
                        "Done"
                    }
                }
            }
        }
    }
}

/// Adds a permanent toast on click -- a plain, unstyled use of
/// [`use_toast`], split into its own component only because `use_toast`
/// must be called from a descendant of `ToastProvider`, not `ToastProvider`
/// itself.
#[component]
fn ToastStackTrigger() -> Element {
    let toast_api = use_toast();

    rsx! {
        button {
            id: "toast-stack-trigger",
            // `ToastProvider`'s own children (this button) render in normal
            // flow, wherever `ToastProvider` sits in the tree; the toast
            // region itself (and the sibling above) are both `position:
            // fixed` to the bottom-right viewport corner, well away from
            // wherever this button's own normal-flow position happens to
            // land, so nothing here needs to work around an overlap.
            onclick: move |_| {
                toast_api
                    .info(
                        "Stacking test".to_string(),
                        ToastOptions::new().permanent(true),
                    );
            },
            "Add toast"
        }
    }
}
