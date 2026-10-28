//! Defines the [`NavigationMenu`] component and its sub-components.
//!
//! # Why this is a new primitive, not a `Navbar` reskin
//!
//! [`crate::navbar`] implements the APG **Menu and Menubar** pattern:
//! `role="menubar"`/`role="menu"`/`role="menuitem"` (via
//! `crate::menu_semantics`), roving `tabindex`, and the full
//! ArrowLeft/Right/Up/Down/Home/End menu keyboard contract -- and this
//! crate's docs site depends on that exact contract
//! (`playwright/navbar.spec.ts`,
//! `playwright/oracle/tier1-apg/keyboard-matrix.spec.ts`). Radix/shadcn's
//! `NavigationMenu` is deliberately a *different* APG pattern: **Disclosure
//! (Show/Hide) Navigation** -- a `nav` of plain links and
//! `button[aria-expanded][aria-controls]` disclosure triggers, with **no
//! menu role anywhere** (`content/patterns/disclosure/disclosure-pattern.html`,
//! its "WAI-ARIA Roles, States, and Properties" section: only `button`,
//! `aria-expanded`, optionally `aria-controls` -- no `aria-haspopup`, no
//! `role="menu"`; `content/patterns/disclosure/examples/
//! disclosure-navigation.html`'s own "Important" advisement is explicit:
//! "this implementation of site navigation does not use the menu role
//! because it does not provide the complex functionality that assistive
//! technologies expect in a widget that has the menu role. Typical site
//! navigation does not need all the keyboard interactions specified by the
//! menu and menubar pattern." `disclosure-navigation-hybrid.html` is the
//! same pattern with top-level links alongside the disclosure buttons,
//! cited below for its own optional-arrow-key keyboard table, which this
//! module also implements).
//!
//! A different ARIA contract is a real reason for a distinct primitive,
//! not a duplicate: this is **not** the duplication
//! `crate::menu_root`'s own module doc describes for row 53's
//! `DropdownMenu`/`ContextMenu`/`Menubar` consolidation (three hosts
//! implementing the *same* Menu-and-Menubar pattern, with identical
//! plumbing literally copy-pasted between them). `Navbar` and
//! `NavigationMenu` implement two *different, APG-distinguished* patterns
//! on purpose -- forcing them onto one shared component would either
//! smuggle menu semantics into a widget the pattern explicitly forbids
//! them from, or strip Navbar's real menubar contract out from under the
//! specs that already pin it. `navbar.rs` is still the right file to read
//! for this crate's house idioms (context struct shape, `use_id_or`,
//! `merge_attributes`, `attributes!`, `use_animated_open`, the
//! `#[cfg(feature = "web")]` `*ContentRendered` leaf split via
//! `crate::top_layer`), and this module mirrors those idioms closely --
//! just never `crate::menu_semantics`, and never a roving-tabindex
//! collection for keyboard focus (see below).
//!
//! # Tab order: native, not roving
//!
//! The disclosure-navigation example's own keyboard table is explicit that
//! Tab/Shift+Tab move through *every* top-level button/link and, once a
//! panel is open, through its links too, "in the page tab sequence" --
//! unlike a menu/menubar, "the buttons and links are not contained by an
//! element with a widget role ... that is expected to occupy only one stop
//! in the page tab sequence and manage focus for all its descendants."
//! So every [`NavigationMenuTrigger`]/[`NavigationMenuLink`] keeps its
//! element's native default `tabindex` (never `-1`) -- this crate's
//! existing precedent for "use `crate::collection`'s ordering purely to
//! move real DOM focus on an explicit key press, never to drive a roving
//! `tabindex`" is [`crate::accordion::AccordionTrigger`], which already
//! ignores its own collection item's `tabindex` and hard-codes `"0"`
//! instead; this module does the same.
//!
//! Left/Right between top-level items, Home/End to the first/last top-level
//! item, and Up/Down within an open panel's own list of links are all
//! optional in APG and all implemented here
//! (`disclosure-navigation-hybrid.html`'s own keyboard table includes every
//! one of them) -- even though Tab alone already reaches every one of
//! those same targets, exactly as this section's own citation above
//! describes. Not implemented: Home/End *within* an open panel (the
//! reference table's own Home/End rows cover this too, but Home/End is
//! scoped to top-level items only) and treating
//! ArrowUp/ArrowDown on a top-level item as aliases for Left/Right (the
//! reference table conflates them one row apiece; kept separate here so
//! ArrowDown on a top-level trigger unambiguously means "open/enter this
//! panel," never "also move to the next item" -- see
//! [`NavigationMenuTrigger`]'s own doc for that choice). Both are cheap,
//! cited and backward-compatible, and both are left out.
//!
//! # Hover intent and the pointer contract
//!
//! Opening on pointer hover (after a short delay) is, like `crate::menu_sub`'s
//! submenu hover-intent, a widely-implemented convention (Radix, native OS
//! menus) rather than an APG requirement. The whole pointer contract -- the
//! open-intent delay, the immediate switch between triggers, the close grace
//! when the pointer leaves the trigger and its panel, and the marker that
//! lets a click confirm a hover-opened panel -- lives in
//! `crate::hover_intent::HoverIntent`, shared with `crate::navbar` so the
//! opening half can never be wired without the closing half. This
//! module only reports `mouseenter`/`mouseleave` of its trigger and panel to
//! it. The timers are `crate::menu_sub::DelayedAction`s, the same primitive
//! `DropdownMenuSub`/`ContextMenuSub` use.
//!
//! Hover is an *option*, not a given: [`NavigationMenuProps::open_on_hover`]
//! (default `true`) switches the whole pointer-hover
//! contract off, leaving **click activation** -- see that prop's doc for the
//! exact semantics and the Base UI/Radix comparison. Every hover entry point
//! in this file (the trigger's and the content's `onmouseenter`/
//! `onmouseleave`) checks it first, so there is no ungated hover path.
//!
//! # Detecting "focus left the nav" without `relatedTarget`
//!
//! The disclosure-navigation example's own accessibility-features note
//! ("Moving focus out of the navigation region also closes an open
//! dropdown ... necessary to meet WCAG 2.1 1.4.13") is implemented in its
//! reference JS (`js/disclosureMenu.js`'s `onBlur`) by reading
//! `event.relatedTarget` and checking `rootNode.contains(...)`.
//! `dioxus-html` 0.7.9's own event source
//! (`dioxus-html-0.7.9/src/events/focus.rs`) shows that this framework version's
//! `FocusData` exposes no `related_target` at all, so that check is not
//! available here -- consistent with no existing component in this crate
//! reading it either (`rg relatedTarget primitives/src` finds nothing).
//! Every existing multi-item collection in this crate (`Navbar`,
//! `Accordion`, `Toolbar`, `ToggleGroup`, `Tabs`) sidesteps the same gap
//! with a *roving*-`tabindex` invariant: only one item is ever a real Tab
//! stop, so any blur it doesn't itself cause is, by construction, focus
//! leaving the widget. That invariant does not hold here (every item is
//! independently tabbable, on purpose -- see above), so this module
//! instead debounces: every focusable element's `onblur` schedules a
//! `DelayedAction` that closes the open panel, and every focusable
//! element's `onfocus` cancels it. A real browser dispatches a moving
//! focus's `blur`/`focusout` on the old element and `focus`/`focusin` on
//! the new one back-to-back in the same task, so a Tab/click that lands on
//! *any* other element this module wires (a sibling trigger, a sibling
//! link, or a link inside the panel that's opening) cancels the scheduled
//! close before `FOCUS_LEAVE_CLOSE_DELAY` elapses; a Tab/click that
//! leaves the whole `nav` has nothing left to cancel it, so the close goes
//! through.
//!
//! # Top layer
//!
//! [`NavigationMenuContent`]'s web-arm leaf promotes to the top layer
//! broadly the way `NavbarContent` does --
//! `crate::top_layer::anchored_content_attributes` with
//! `AnchorKind::NavigationMenu` (the `data-dx-anchored` attribute the shared
//! anchor-positioning stylesheet in `top_layer.rs` selects on) and
//! `crate::top_layer::use_anchor_position_fallback` -- except
//! `popover="manual"`, not `"auto"`: this module already owns its entire
//! open/close lifecycle (hover-intent timers, the focus-leave debounce
//! above, Escape, click), the same reasoning `crate::tooltip`/
//! `crate::hover_card` document for `Manual` (`top_layer.rs`'s
//! `PopoverKind::Manual` doc) -- native `auto` light-dismiss firing
//! instantly on outside focus/click would fight this module's own delayed,
//! debounced open/close instead of the two ever agreeing. Unlike
//! `NavbarContent`, this content's own popover lifecycle sync is
//! `crate::top_layer::use_popover_shown_while_mounted`, not
//! `crate::top_layer::use_popover_sync` -- see
//! `NavigationMenuContentRendered`'s own doc for why (this content, like
//! `NavbarContent`, renders through `use_animated_open` and needs its exit
//! animation not to be cut short by an instant `hidePopover()`).

use std::rc::Rc;
use std::time::Duration;

use dioxus::prelude::*;
use dioxus_attributes::attributes;

use crate::collection::{
    collection_item, use_collection_provider, use_item, use_item_index, CollectionOptions,
    CollectionPlacement, CollectionState, ItemIndex,
};
use crate::direction::{use_direction, use_direction_signal, Direction};
use crate::hover_intent::{use_hover_intent_with_delays, HoverDelays, HoverIntent};
use crate::menu_sub::{use_delayed_action, DelayedAction};
use crate::{
    has_own_accessible_name, merge_attributes, use_animated_open, use_id_or, use_unique_id,
};

/// Debounce before a blur that nothing inside the nav cancels closes the
/// open panel -- see this module's doc, "Detecting 'focus left the nav'
/// without `relatedTarget`". Deliberately tiny but nonzero, so it is never
/// perceptible as a delay yet still gives a same-task compensating
/// `onfocus` elsewhere in the nav a chance to cancel it first.
const FOCUS_LEAVE_CLOSE_DELAY: Duration = Duration::from_millis(1);

#[derive(Clone, Copy)]
struct NavigationMenuContext {
    /// The currently open item's index, if any. One open at a time, the
    /// same shape as `NavbarContext::open_nav`.
    open: Signal<Option<usize>>,
    /// The ONE write path for `open` ([`HoverIntent::set_open`]). It also
    /// clears the hover marker in [`Self::hover`], so that marker can only
    /// ever describe a panel the pointer opened and nothing since has
    /// re-decided.
    set_open: Callback<Option<usize>>,
    /// The pointer-hover contract: hover-intent open/switch, the close grace on
    /// pointer leave, the "opened by hover, not yet claimed by a click" marker
    /// (a click on the trigger of such a panel keeps it open and claims it
    /// instead of toggling it shut -- the pointer path hover -> click must not
    /// flash the panel open then closed, `dev-docs/backlog.md` row 189), and the
    /// `open_on_hover` gate on every one of those. `set_open` above is its
    /// write path. Shared with `crate::navbar`; see `crate::hover_intent`.
    hover: HoverIntent,
    disabled: ReadSignal<bool>,
    /// Top-level items only (every [`NavigationMenuTrigger`] and every
    /// top-level [`NavigationMenuLink`]), keyed by [`NavigationMenuItem`]'s
    /// own `index` -- Left/Right's move target and the mount-ref registry
    /// Escape uses to return focus to a trigger. Never drives `tabindex`
    /// (see this module's doc).
    focus: CollectionState,
    /// The "focus left the nav" debounce -- see this module's doc.
    blur_close: DelayedAction,
    /// The [`NavigationMenuList`] element, which the indicator measures against.
    list: Signal<Option<Rc<MountedData>>>,
    /// The `li` of the open item, with its index: each open [`NavigationMenuItem`] publishes it
    /// (the index lets the indicator ignore the previous item's handle for the one render
    /// between the open index changing and the new item publishing).
    open_item: Signal<Option<(usize, Rc<MountedData>)>>,
}

/// The props for the [`NavigationMenu`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuProps {
    /// Whether the whole navigation menu is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether hovering a [`NavigationMenuTrigger`] opens its panel (after a
    /// short hover-intent delay), and the pointer leaving the trigger and
    /// panel closes it again. Defaults to `true`.
    ///
    /// Set it to `false` for **click activation**: a panel then opens only
    /// when its trigger is clicked or activated with `Enter`/`Space`/
    /// `ArrowDown`, hovering a trigger does nothing, and the pointer
    /// leaving a click-opened panel does *not* close it. It closes the way
    /// a keyboard-opened one does: clicking the trigger again, `Escape`,
    /// choosing a link, focus leaving the navigation menu, or a pointer
    /// press anywhere outside it. Keyboard behaviour is identical in both
    /// modes, and touch never hover-opens either way (a tap is a click).
    ///
    /// Same name as Base UI's `Menu.Trigger` `openOnHover` prop. Base UI's
    /// own `NavigationMenu` and Radix's `NavigationMenu` have no such
    /// switch (they only tune the hover delay: Base UI `delay`/`closeDelay`,
    /// Radix `delayDuration`/`skipDelayDuration`) -- this crate's delays
    /// stay fixed constants.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub open_on_hover: ReadSignal<bool>,

    /// Additional attributes to apply to the root `nav` element. An
    /// accessible name (`aria_label`/`aria_labelledby`) is required by the
    /// disclosure-navigation pattern's own accessibility-features note --
    /// pass one of these.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the navigation menu -- typically a
    /// [`NavigationMenuList`] of [`NavigationMenuItem`]s.
    pub children: Element,
}

/// # NavigationMenu
///
/// The root of a disclosure-pattern site navigation: a `nav` landmark
/// containing top-level links and disclosure triggers whose panels expand
/// below them. See this module's doc for why this is a distinct primitive
/// from [`crate::navbar::Navbar`] (a different APG pattern, not a
/// duplicate).
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::navigation_menu::{
///     NavigationMenu, NavigationMenuContent, NavigationMenuItem, NavigationMenuLink,
///     NavigationMenuList, NavigationMenuTrigger,
/// };
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         NavigationMenu { aria_label: "Main",
///             NavigationMenuList {
///                 NavigationMenuItem {
///                     NavigationMenuTrigger { "Components" }
///                     NavigationMenuContent {
///                         NavigationMenuLink { href: "/calendar", "Calendar" }
///                         NavigationMenuLink { href: "/slider", "Slider" }
///                     }
///                 }
///                 NavigationMenuItem {
///                     NavigationMenuLink { href: "/docs", "Docs" }
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`NavigationMenu`] component defines the following data attributes
/// you can use to control styling:
/// - `data-disabled`: Indicates if the navigation menu is disabled. Values
///   are `true` or `false`.
#[component]
pub fn NavigationMenu(props: NavigationMenuProps) -> Element {
    let open = use_signal(|| None);
    let hover = use_hover_intent_with_delays(
        open,
        props.open_on_hover,
        HoverDelays::BASE_UI_NAVIGATION_MENU,
    );
    let set_open = hover.set_open();

    // Non-looping (matches the reference implementation's clamped
    // `Math.max`/`Math.min` indices, `js/disclosureMenu.js`'s
    // `controlFocusByKey` -- Left/Right stop at the first/last item rather
    // than wrapping).
    let focus = use_collection_provider(ReadSignal::new(Signal::new(false)));

    // Each `use_delayed_action()` call is itself a hook (it wraps
    // `use_signal`, `menu_sub.rs`), so -- like `open`/`focus` above -- it
    // must run directly in this component's own body, never nested inside
    // another hook's closure. Calling it inline inside
    // `use_context_provider`'s initializer below panics with a `BorrowMutError`
    // ("The hook list is already borrowed ... hook inside a hook") on every
    // real render (the wasm panic never surfaces in `cargo check`/`clippy`/
    // `cargo test`, since Dioxus's Rules of Hooks are enforced at runtime,
    // not by the type system).
    let blur_close = use_delayed_action();
    let list = use_signal(|| None);
    let open_item = use_signal(|| None);

    use_context_provider(|| NavigationMenuContext {
        open,
        set_open,
        hover,
        disabled: props.disabled,
        focus,
        blur_close,
        list,
        open_item,
    });

    // Owned by this component -- disabled state must win over a caller's
    // own attributes (`dev-docs/backlog.md` row 93's duplicate-attribute
    // hazard).
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(nav {
            "data-disabled": (props.disabled)(),
        }),
    ]);

    rsx! {
        nav {
            ..attributes,
            {props.children}
        }
    }
}

/// The props for the [`NavigationMenuList`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuListProps {
    /// Additional attributes to apply to the list element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the list -- [`NavigationMenuItem`]s.
    pub children: Element,
}

/// # NavigationMenuList
///
/// The `ul` that holds a [`NavigationMenu`]'s top-level
/// [`NavigationMenuItem`]s -- matches the disclosure-navigation example's
/// own list structure ("the top-level set ... is in a list ... and the set
/// of links controlled by each button is contained within a list nested
/// inside the item with the controlling button"). Purely structural: this
/// component holds no state of its own.
///
/// This must be used inside a [`NavigationMenu`] component.
#[component]
pub fn NavigationMenuList(props: NavigationMenuListProps) -> Element {
    let mut ctx: NavigationMenuContext = use_context();
    rsx! {
        ul {
            onmounted: move |event| ctx.list.set(Some(event.data())),
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`NavigationMenuIndicator`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuIndicatorProps {
    /// Additional attributes to apply to the indicator element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// What the indicator draws (the theme's arrow); empty by default.
    pub children: Element,
}

/// # NavigationMenuIndicator
///
/// The part Radix and shadcn call `NavigationMenuIndicator`: a decoration that sits under the
/// open item's trigger and follows it as the open item changes. Place it inside
/// [`NavigationMenuList`], after the items. It is purely visual (`aria-hidden`, presentational).
///
/// The primitive measures the open item against the list (their client rects, which agree with
/// the layout offsets because neither is transformed) and writes `--dx-nav-indicator-x` (the inline-start offset,
/// so it is right in RTL) and `--dx-nav-indicator-width` on the element's own `style`, so the theme positions it with plain
/// CSS. The measurement runs when the open item changes. It is not re-run on a window resize
/// while a panel stays open: a resize closes nothing here, so a narrow-to-wide resize with a
/// panel open leaves it at the old offset until the next switch.
///
/// This must be used inside a [`NavigationMenu`] component.
///
/// ## Data attributes
/// - `data-state`: `visible` while an item is open, `hidden` otherwise.
#[component]
pub fn NavigationMenuIndicator(props: NavigationMenuIndicatorProps) -> Element {
    let ctx: NavigationMenuContext = use_context();
    let open = ctx.open;
    // The open item's inline-start offset in the list and its width, in px. Kept after the item
    // closes: the indicator fades out where it was instead of sliding back to the start.
    let mut placement = use_signal(|| None::<(f64, f64)>);
    // A signal, so the effect below re-measures when the direction changes.
    let direction = use_direction_signal(use_direction(None));

    use_effect(move || {
        let direction = direction();
        let Some(index) = open() else { return };
        let (Some(list), Some((item_index, item))) = ((ctx.list)(), (ctx.open_item)()) else {
            return;
        };
        if item_index != index {
            return;
        }
        spawn(async move {
            let (Ok(list), Ok(item)) = (list.get_client_rect().await, item.get_client_rect().await)
            else {
                return;
            };
            // Neither the list nor an item is transformed (the list has no border or padding),
            // so the rect difference is the layout offset the stylesheet positions by.
            let x = match direction {
                Direction::Ltr => item.min_x() - list.min_x(),
                Direction::Rtl => list.max_x() - item.max_x(),
            };
            let next = Some((x, item.width()));
            if *placement.peek() != next {
                placement.set(next);
            }
        });
    });

    // Written as the element's own `style` (merged with a caller's), so the theme positions the
    // indicator with plain CSS from two custom properties.
    let measured: Vec<Attribute> = match placement() {
        Some((x, width)) => {
            let style =
                format!("--dx-nav-indicator-x: {x}px; --dx-nav-indicator-width: {width}px;");
            attributes!(li { style: "{style}" })
        }
        None => Vec::new(),
    };

    let attributes = merge_attributes(vec![
        props.attributes,
        measured,
        attributes!(li {
            role: "presentation",
            aria_hidden: "true",
            "data-state": if open().is_some() { "visible" } else { "hidden" },
        }),
    ]);

    rsx! {
        li { ..attributes, {props.children} }
    }
}

#[derive(Clone, Copy)]
struct NavigationMenuItemContext {
    index: ReadSignal<usize>,
    /// This item's place in the top-level row; the element that registers
    /// it (its trigger, or its top-level link) renders the marker.
    order: ItemIndex,
    is_open: Memo<bool>,
    disabled: ReadSignal<bool>,
    /// This item's own [`NavigationMenuContent`]'s current element id, kept
    /// in sync by that component -- mirrors `NavbarNavContext::content_id`
    /// (`navbar.rs`): `NavigationMenuTrigger`'s `anchor-name`/`aria-controls`
    /// must key off this item's own content, not another item's.
    content_id: Signal<String>,
    /// This item's own [`NavigationMenuTrigger`] element id, so
    /// [`NavigationMenuContent`] can label itself `aria-labelledby` from it,
    /// and so Escape-from-content can return focus to it. Mirrors
    /// `NavbarNavContext::trigger_id`.
    trigger_id: Signal<String>,
    /// Set by the trigger's own ArrowDown handler when it opens a
    /// previously-closed panel (or finds one already open); consumed once
    /// `content_focus` has at least one link registered -- see
    /// [`NavigationMenuContent`]'s own consumer effect. This is the whole
    /// mechanism behind ArrowDown jumping into the panel's first link, for
    /// either case.
    focus_first_link: Signal<bool>,
    /// Roving-focus collection for this item's own OPEN content's links --
    /// ArrowUp/ArrowDown move within them
    /// (`disclosure-navigation-hybrid.html`'s own keyboard table, "Up
    /// Arrow"/"Down Arrow" rows: "moves focus to the previous/next link"),
    /// and it is also how `focus_first_link` above finds and focuses the
    /// first one (`content_focus.first_available_index()` +
    /// `content_focus.set_focus`, the exact mechanism Home/Left/Right
    /// already use on the top-level collection below). Distinct from
    /// `NavigationMenuContext::focus` (this item's own top-level
    /// Left/Right/Home/End collection, shared across every
    /// [`NavigationMenuItem`]) -- this one is scoped to a single item's own
    /// panel, one instance per [`NavigationMenuItem`]. Never drives
    /// `tabindex` (this module's own "Tab order: native, not roving" doc);
    /// [`NavigationMenuLink`] reads only the focus-moving half of
    /// `crate::collection::use_item`'s return value, exactly like the
    /// top-level collection already does.
    content_focus: CollectionState,
}

/// The props for the [`NavigationMenuItem`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuItemProps {
    /// The item's place in the top-level keyboard order (Left/Right, Home/End). Leave it unset:
    /// items are ordered as they are created, which is tree order. Set it on every item only when
    /// one is inserted between items that are already mounted, or a keyed list reorders them.
    #[props(default)]
    pub index: ReadSignal<Option<usize>>,

    /// Whether this item is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Additional attributes to apply to the `li` element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the item -- either a [`NavigationMenuTrigger`] +
    /// [`NavigationMenuContent`] pair, or a single top-level
    /// [`NavigationMenuLink`].
    pub children: Element,
}

/// # NavigationMenuItem
///
/// One top-level entry: either a disclosure trigger with its panel, or a
/// plain top-level link. Provides the context its children key off.
///
/// This must be used inside a [`NavigationMenu`] (or [`NavigationMenuList`])
/// component.
///
/// ## Styling
///
/// The [`NavigationMenuItem`] component defines the following data
/// attributes you can use to control styling:
/// - `data-state`: Indicates if this item's panel is open or closed.
///   Values are `open` or `closed`.
/// - `data-disabled`: Indicates if the item is disabled. Values are `true`
///   or `false`.
#[component]
pub fn NavigationMenuItem(props: NavigationMenuItemProps) -> Element {
    let mut ctx: NavigationMenuContext = use_context();
    let order = use_item_index(ctx.focus, props.index);
    let index = order.index;
    let is_open = use_memo(move || (ctx.open)() == Some(index.cloned()));
    let content_id = use_unique_id();
    let trigger_id = use_unique_id();
    let content_focus_id = use_unique_id();

    // Not `use_collection_provider` (which also provides `CollectionState`
    // as its own bare context type): nothing here needs `content_focus`
    // reachable that way, only as this struct's own field, and this item's
    // panel content is unmounted/remounted far more often than this
    // component itself, so the collection's identity must survive that
    // (`use_hook`, exactly like `NavigationMenuLink`'s own per-instance
    // collection construction below).
    let content_focus = use_hook(|| {
        CollectionState::new(
            ReadSignal::new(Signal::new(false)),
            CollectionOptions::default(),
            content_focus_id,
        )
    });

    let mut item_ctx = use_context_provider(|| NavigationMenuItemContext {
        index,
        order,
        is_open,
        disabled: props.disabled,
        content_id,
        trigger_id,
        focus_first_link: Signal::new(false),
        content_focus,
    });

    use_effect(move || {
        if !is_open() {
            item_ctx.focus_first_link.set(false);
        }
    });

    // The indicator measures the open item's `li`: publish it while this item is the open one.
    let mut li = use_signal(|| None::<Rc<MountedData>>);
    use_effect(move || {
        if is_open() {
            if let Some(element) = li() {
                ctx.open_item.set(Some((index.cloned(), element)));
            }
        }
    });

    // Owned by this component -- open/disabled state must win over a
    // caller's own attributes (`dev-docs/backlog.md` row 93's
    // duplicate-attribute hazard).
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(li {
            "data-state": if is_open() { "open" } else { "closed" },
            "data-disabled": (ctx.disabled)() || (props.disabled)(),
        }),
    ]);

    rsx! {
        li {
            onmounted: move |event| li.set(Some(event.data())),
            ..attributes,
            {props.children}
        }
    }
}

/// Provided only by [`NavigationMenuContent`], so a [`NavigationMenuLink`]
/// can tell whether it is a top-level item or a link inside an open panel
/// (context-type presence, the same technique `NavbarItem` uses
/// `try_use_context::<NavbarNavContext>()` for -- `navbar.rs`). Carries no
/// data of its own -- a content link's shared state
/// (`content_focus`/`focus_first_link`) already lives on
/// [`NavigationMenuItemContext`], reachable from both the trigger and every
/// link regardless of nesting; this type exists purely so its *presence*
/// (not any field on it) answers "am I nested in content."
#[derive(Clone, Copy)]
struct NavigationMenuContentContext;

/// The props for the [`NavigationMenuTrigger`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuTriggerProps {
    /// The ID of the trigger button. If not provided, an internally
    /// generated ID is used and kept in sync with a caller override (the
    /// same `use_id_or` construction as `NavbarTrigger`, `navbar.rs`).
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes to apply to the trigger element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the trigger.
    pub children: Element,
}

/// # NavigationMenuTrigger
///
/// A disclosure button (`aria-expanded`, `aria-controls`) that toggles its
/// sibling [`NavigationMenuContent`] on click/Enter/Space, and -- unless
/// [`NavigationMenuProps::open_on_hover`] is `false` -- on pointer hover
/// after a short delay (see this module's doc).
///
/// This must be used inside a [`NavigationMenuItem`] component.
#[component]
pub fn NavigationMenuTrigger(props: NavigationMenuTriggerProps) -> Element {
    let mut ctx: NavigationMenuContext = use_context();
    let item_ctx: NavigationMenuItemContext = use_context();
    let direction = use_direction(None);
    let index = item_ctx.index;
    let disabled = move || (ctx.disabled)() || (item_ctx.disabled)();
    let is_open = item_ctx.is_open;

    // The trigger is the focusable element, so it registers this item in
    // the top-level collection -- mirrors `NavbarTrigger` registering
    // `nav_ctx.index` in `ctx.focus`. Its `tabindex` is never read (see
    // this module's doc): this button keeps its native default tabindex.
    let item = use_item(collection_item(ctx.focus, index).disabled(disabled));
    let onmounted = item.onmounted();

    let id = use_id_or(item_ctx.trigger_id, props.id);

    // Merged (caller-wins, deduped) rather than a bare literal alongside
    // `..props.attributes` -- see `NavbarTrigger`'s identical construction
    // (`navbar.rs`) for the duplicate-attribute hazard this avoids.
    let attributes = merge_attributes(vec![
        attributes!(button { id: id.cloned() }),
        props.attributes,
    ]);
    // Ties this trigger to the web-arm content's `position-anchor` once
    // promoted to the top layer -- see `crate::top_layer::anchor_name_style`
    // (inert off the web arm). Not a bare `style: anchor_name_style(
    // ..)` literal directly on this element, beside `..attributes` below: a
    // caller supplying their own `style` would then produce two `style`
    // attributes on this same tag (`dev-docs/conformance-harness.md`
    // hydration-parity Rule 4; `playwright/oracle/attr-synth`'s
    // `navigation_menu:trigger:style` case covers it). Folding it into
    // `attributes` here, via
    // `top_layer::anchored_trigger_attributes`, avoids that and (unlike
    // plain `merge_attributes`, which folds a plain `style` attribute but
    // leaves shorthand style props -- `padding: ".."`, namespace
    // `Some("style")` -- as separate attributes) renders exactly one
    // `style` carrying the anchor binding plus whatever the caller
    // supplied; see that function's own doc.
    let attributes =
        crate::top_layer::anchored_trigger_attributes(&item_ctx.content_id.cloned(), attributes);
    // Owned by this component -- disclosure semantics (`aria-expanded`/
    // `aria-controls`) and state must win over a caller's own attributes
    // (`dev-docs/backlog.md` row 93's duplicate-attribute hazard).
    let attributes = merge_attributes(vec![
        attributes,
        item_ctx.order.marker_attribute(),
        attributes!(button {
            type: "button",
            aria_expanded: is_open(),
            aria_controls: item_ctx.content_id.cloned(),
            "data-state": if is_open() { "open" } else { "closed" },
            "data-disabled": disabled(),
        }),
    ]);

    rsx! {
        button {
            onmounted,
            onclick: move |_| {
                if disabled() {
                    return;
                }
                ctx.hover.cancel();
                // The pointer opened this panel by hovering and this is the
                // first click since: the user is confirming it, not asking
                // to dismiss it. Keep it open and claim it, so the NEXT
                // click toggles it shut like any click-opened panel. Radix
                // would toggle it shut here (its `wasClickCloseRef` only
                // stops hover re-opening it); Base UI's `stickIfOpen` and
                // this crate keep it open (`dev-docs/backlog.md` row 189).
                if is_open.cloned() && ctx.hover.opened_by_hover() {
                    ctx.hover.claim();
                    return;
                }
                let want_open = !is_open.cloned();
                ctx.set_open.call(want_open.then(|| index.cloned()));
            },
            // Both hover handlers are no-ops when `open_on_hover` is
            // `false` (click activation) -- `HoverIntent` checks it, see
            // `NavigationMenuProps::open_on_hover`. The trigger and its panel
            // are one hover region: this item's panel reports the same
            // enter/leave pair (`NavigationMenuContentRendered`).
            onmouseenter: move |_| {
                if !disabled() {
                    ctx.hover.enter(index.cloned());
                }
            },
            onmouseleave: move |_| {
                ctx.hover.leave(index.cloned());
            },
            onfocus: move |_| {
                ctx.blur_close.cancel();
                // Sync native focus (Tab, a click, or a screen reader's own
                // navigation -- anything that isn't this collection's own
                // `focus_next`/`focus_prev`/`set_focus` call) back into
                // `ctx.focus`'s own `recent`/`focused` state -- the same
                // construction `toolbar.rs`/`tabs.rs` already use on their
                // own identical collections (`ctx.focus.set_focus(Some(
                // (props.index)()))`). Without this, `focus_next()`/
                // `focus_prev()` compute their "current position" from
                // whatever this collection last *itself* set focus to,
                // which can go stale the moment focus arrives some other
                // way -- for example Tab into a top-level item,
                // then Left/Right, where the first press would silently be a
                // same-spot no-op (`next_index_after`/`prev_index_before`
                // treat an unknown/`None` current position as "before the
                // first item", so the first press only catches the
                // collection up to where focus already was).
                ctx.focus.set_focus(Some(index.cloned()));
            },
            onblur: move |_| {
                ctx.blur_close.schedule(FOCUS_LEAVE_CLOSE_DELAY, move || {
                    ctx.set_open.call(None);
                });
            },
            onkeydown: move |event: Event<KeyboardData>| {
                // RTL mirrors the horizontal pair (`Direction::ltr_equivalent`).
                match direction.ltr_equivalent(&event.key()) {
                    // "If a dropdown is open ... pressing Esc will close the
                    // dropdown" -- disclosure-navigation.html's own
                    // accessibility-features note. Closes whatever is open
                    // (mirroring the reference JS's own
                    // `toggleExpand(this.openIndex, false)`), not only when
                    // it is this trigger's own panel, matching
                    // `onButtonKeyDown` exactly.
                    Key::Escape if (ctx.open)().is_some() => {
                        ctx.set_open.call(None);
                    }
                    Key::ArrowLeft => ctx.focus.focus_prev(),
                    Key::ArrowRight => ctx.focus.focus_next(),
                    // disclosure-navigation-hybrid.html's own keyboard
                    // table, Home/End rows: "moves focus to the
                    // first/last item" -- this collection is already
                    // non-looping (see `NavigationMenu`'s own doc comment
                    // on `focus`), so these are exactly `focus_first`/
                    // `focus_last`, the same pair `NavigationMenuLink`'s
                    // own top-level branch below (and, elsewhere in this
                    // crate, `context_menu.rs`'s top-level Home/End) already
                    // use for an identical collection.
                    Key::Home => ctx.focus.focus_first(),
                    Key::End => ctx.focus.focus_last(),
                    // "if focus is on a button and its dropdown is
                    // expanded, moves focus to the first link in the
                    // dropdown" -- disclosure-navigation-hybrid.html's
                    // keyboard table. Extended here, past that one
                    // sentence, to also OPEN a still-*closed* panel first
                    // -- this module's own choice, not the reference
                    // table's literal text for a collapsed trigger (that
                    // row instead sends a collapsed trigger's ArrowDown to
                    // the *next* top-level item, same as ArrowRight;
                    // deliberately not mirrored, since ArrowRight already
                    // covers that and doing both would make the two keys
                    // redundant with each other for half of what ArrowDown
                    // does). A keyboard user pressing ArrowDown on a
                    // collapsed trigger far more plausibly means "open this
                    // and take me into it" than "skip it, take me to
                    // whatever's next" -- the same one-key affordance a
                    // native `<select>`/combobox already gives ArrowDown.
                    Key::ArrowDown => {
                        if !is_open.cloned() {
                            ctx.hover.cancel();
                            ctx.set_open.call(Some(index.cloned()));
                        }
                        item_ctx.focus_first_link.clone().set(true);
                    }
                    _ => return,
                }
                event.prevent_default();
            },

            ..attributes,
            {props.children}
        }
    }
}

/// The props for the [`NavigationMenuContent`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuContentProps {
    /// The id of the content element.
    pub id: ReadSignal<Option<String>>,
    /// Additional attributes to apply to the content element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the content -- typically a grid of
    /// [`NavigationMenuLink`]s.
    pub children: Element,
}

/// # NavigationMenuContent
///
/// The panel a [`NavigationMenuTrigger`] discloses: anchored below its
/// trigger and, on the web arm, promoted to the top layer (see this
/// module's doc). Never carries a menu role (R4: this pattern has none).
///
/// This must be used inside a [`NavigationMenuItem`] component, after its
/// [`NavigationMenuTrigger`].
///
/// ## Styling
///
/// The [`NavigationMenuContent`] component defines the following data
/// attributes you can use to control styling:
/// - `data-state`: Indicates if this panel is open or closed. Values are
///   `open` or `closed`.
#[component]
pub fn NavigationMenuContent(props: NavigationMenuContentProps) -> Element {
    let mut item_ctx: NavigationMenuItemContext = use_context();

    let unique_id = use_unique_id();
    let id = use_id_or(unique_id, props.id);

    // Keep `item_ctx.content_id` in sync -- mirrors `NavbarContent`'s
    // identical `nav_ctx.content_id.set(id())` (`navbar.rs`).
    use_effect(move || item_ctx.content_id.set(id()));

    let render = use_animated_open(id, item_ctx.is_open);

    // Consume a pending "focus the first link" request (the trigger's own
    // ArrowDown) once `content_focus` has at least one link registered --
    // see `NavigationMenuItemContext::focus_first_link`'s doc. Moves focus
    // through the collection (`use_item`'s own `control_mount_focus`), the
    // mechanism Home/Left/Right use on the top-level collection. The panel's
    // links are created in reverse of source order for this shape, so
    // "first" is the collection's document order, which
    // `try_focus_placement` waits for (`crate::collection::use_item_index`).
    use_effect(move || {
        if !(item_ctx.focus_first_link)() {
            return;
        }
        let mut content_focus = item_ctx.content_focus;
        if content_focus.try_focus_placement(CollectionPlacement::First) {
            item_ctx.focus_first_link.set(false);
        }
    });

    use_context_provider(|| NavigationMenuContentContext);

    let ctx: NavigationMenuContext = use_context();

    rsx! {
        if render() {
            NavigationMenuContentRendered {
                id: id.cloned(),
                attributes: props.attributes,
                children: props.children,
            }
            // Click activation only: with no pointer-leave close, a press
            // outside the nav is what dismisses a click-opened panel on
            // engines that don't focus a button on click (Safari, macOS
            // Firefox), where the trigger's own blur never fires.
            if !ctx.hover.enabled() {
                NavigationMenuOutsidePressDismiss {}
            }
        }
    }
}

/// Click-activation mode's "press outside the navigation menu closes the
/// open panel" -- mounted by [`NavigationMenuContent`] only while its panel
/// is rendered *and* [`NavigationMenuProps::open_on_hover`] is `false`, so
/// the default (hover) mode and every closed panel pay nothing for it.
///
/// Why a document-level listener at all: a hover-opened panel is closed by
/// the pointer leaving it (the hover-close grace timer), and a keyboard- or
/// Chromium-click-opened one by the trigger's `onblur`. A click-opened panel
/// with hover switched off has only the latter, and Safari/macOS Firefox do
/// not give a `<button>` focus on click -- so no blur ever arrives and the
/// panel would stay open under a click on the page background. The check is
/// "the target is outside the `nav` that holds this item's trigger" (found
/// via the trigger's own id, `closest('nav')`), so a press on a sibling
/// trigger or on this panel's own links is never a dismiss (the sibling
/// trigger's own click switches panels). The focus half is already covered
/// by the trigger/link `onblur` debounce, so this listens to `pointerdown`
/// only.
#[component]
fn NavigationMenuOutsidePressDismiss() -> Element {
    let ctx: NavigationMenuContext = use_context();
    let item_ctx: NavigationMenuItemContext = use_context();
    let trigger_id = item_ctx.trigger_id;
    let index = item_ctx.index;

    crate::js_listener::use_js_listeners(move || {
        Some(
            crate::js_listener::JsListeners::install(
                "const triggerId = await dioxus.recv();
                listen(document, 'pointerdown', (e) => {
                    const trigger = document.getElementById(triggerId);
                    const nav = trigger && trigger.closest('nav');
                    if (nav && !nav.contains(e.target)) dioxus.send(true);
                }, true);",
            )
            .arg(trigger_id.cloned())
            .on_message(move |_: bool| {
                if (ctx.open)() == Some(index.cloned()) {
                    ctx.set_open.call(None);
                }
            }),
        )
    });

    rsx! {}
}

/// Web arm: promote to the top layer via `popover="manual"`, anchored to
/// this item's own trigger -- see this module's doc, "Top layer", for why
/// `manual` (not `auto`, unlike `NavbarContent`).
///
/// # Exit animation: `use_popover_shown_while_mounted`, not `use_popover_sync`
///
/// This content renders through [`use_animated_open`] (in
/// [`NavigationMenuContent`] above), which deliberately keeps the element
/// mounted with `data-state="closed"` for its whole exit transition (plus a
/// settle hold) before actually unmounting it -- exactly the shape
/// `crate::top_layer::use_popover_shown_while_mounted`'s own doc describes
/// for `SelectList`/`ComboboxList`/`Toast`/`Popover`/`HoverCard`/`Tooltip`.
/// The plain `crate::top_layer::use_popover_sync` does not fit this shape:
/// its "signal -> browser" effect calls `hidePopover()` the
/// instant `open` goes `false`, which (per the UA popover stylesheet's
/// `[popover]:not(:popover-open) { display: none }`) sets `display: none`
/// on the content *before* `use_animated_open`'s own rAF-deferred
/// `getAnimations()` check ever runs, so that check observes zero
/// running animations and finishes the close cycle immediately -- the exit
/// fade/scale would be skipped outright: the panel would vanish within one
/// 40ms sample of its `data-state` flipping to `"closed"`, instead of
/// playing the ~150ms fade-and-scale-down this file's stylesheet declares,
/// plus its ~250ms settle hold. This is `top_layer.rs`'s own documented
/// "Bug 1 (animation race)" on `use_popover_shown_while_mounted`'s doc --
/// the exact bug class that hook exists to close. (`NavbarContent` also
/// renders through `use_animated_open`; its use of the plain hook is out
/// of this module's scope, see the module doc.) Using the animated-exit
/// variant here closes it the same way it
/// does for every other `use_animated_open`-rendered, `popover`-
/// promoted consumer: `hidePopover()` is never called on this component's
/// own closing path at all, only a real DOM removal (once
/// `use_animated_open` itself has decided the animation and its hold are
/// done) ever takes this content out of the top layer, so the exit
/// animation plays out undisturbed on an element that is still very much
/// `:popover-open` the whole time.
#[cfg(feature = "web")]
#[component]
fn NavigationMenuContentRendered(
    /// The DOM id of the content element.
    id: String,
    /// Extra HTML attributes for the content element.
    attributes: Vec<Attribute>,
    /// The content of the panel, typically a grid of links.
    children: Element,
) -> Element {
    let mut ctx: NavigationMenuContext = use_context();
    let item_ctx: NavigationMenuItemContext = use_context();
    let open = item_ctx.is_open;
    let index = item_ctx.index;

    crate::top_layer::use_popover_shown_while_mounted(
        id.clone(),
        open,
        Callback::new(move |is_open: bool| {
            if !is_open && (ctx.open)() == Some(index.cloned()) {
                ctx.set_open.call(None);
            }
        }),
    );
    // JS-measured static positioning fallback for engines without CSS
    // Anchor Positioning -- anchored below the trigger, left-aligned,
    // matching `NavbarContentRendered`'s identical bottom/start convention.
    crate::top_layer::use_anchor_position_fallback(
        id.clone(),
        id.clone(),
        open,
        crate::ContentSide::Bottom,
        crate::ContentAlign::Start,
        8,
    );
    // Any scroll closes an open navigation panel (`top_layer::use_close_on_scroll`'s doc).
    crate::top_layer::use_close_on_scroll(
        id.clone(),
        id.clone(),
        open,
        crate::top_layer::ScrollClose::AnyScroll,
        Callback::new(move |_: bool| {
            if (ctx.open)() == Some(index.cloned()) {
                ctx.set_open.call(None);
            }
        }),
    );

    // An APG-adjacent panel still needs an accessible name; labelled by its
    // own trigger unless the caller already named it some other way --
    // mirrors `MenubarContent`/`NavbarContent`'s identical construction.
    let labelledby: Vec<Attribute> = if has_own_accessible_name(&attributes) {
        Vec::new()
    } else {
        attributes!(div {
            aria_labelledby: "{item_ctx.trigger_id}"
        })
    };
    let attributes = merge_attributes(vec![
        attributes,
        labelledby,
        // Owned by this component -- top-layer wiring + state must win
        // over a caller's own attributes (`dev-docs/backlog.md` row 93's
        // duplicate-attribute hazard).
        attributes!(div {
            popover: crate::top_layer::PopoverKind::Manual.as_str(),
            "data-state": if open() { "open" } else { "closed" },
        }),
    ]);
    let attributes = crate::top_layer::anchored_content_attributes(
        crate::top_layer::AnchorKind::NavigationMenu,
        &id,
        attributes,
    );

    rsx! {
        div {
            id: id.clone(),
            // No-ops when `open_on_hover` is `false`: a click-opened panel
            // is not closed by the pointer leaving it.
            onmouseenter: move |_| {
                ctx.hover.hold();
            },
            onmouseleave: move |_| {
                ctx.hover.leave(index.cloned());
            },
            onkeydown: move |event: Event<KeyboardData>| {
                // "closes it and sets focus on the button that controls
                // that dropdown" -- disclosure-navigation-hybrid.html's
                // keyboard table, Escape row. Mirrors the reference JS's
                // `onMenuKeyDown`: focus the trigger via the top-level
                // collection (registered by `NavigationMenuTrigger`).
                if event.key() == Key::Escape {
                    ctx.set_open.call(None);
                    // `clear_focus()` immediately before `set_focus` --
                    // not merely `set_focus(Some(index.cloned()))` alone --
                    // because `ctx.focus`'s own `focused` field can already
                    // equal this trigger's index from *before* focus ever
                    // left it for `content_focus` (a link inside the panel
                    // never touches `ctx.focus` at all, per this module's
                    // two-collection design -- see this module's own doc,
                    // "Tab order"), and `CollectionState::set_focus` only
                    // notifies/re-drives DOM focus when the value actually
                    // *changes* (its own doc: "A redundant clear ... must
                    // not wake effects"). Setting the same already-current
                    // value is therefore a silent no-op that leaves real
                    // DOM focus exactly where it was: with real DOM focus on a link inside the
                    // panel, `ctx.focus.set_focus(Some(trigger_index))`
                    // alone would never re-focus the trigger (focus would fall
                    // through to `<body>` once the closing content was
                    // actually removed from the DOM, several hundred ms
                    // later, past this animated exit's own hold). Clearing
                    // first forces a real `None -> Some` transition, so the
                    // very next line's `set_focus` is never a no-op.
                    ctx.focus.clear_focus();
                    ctx.focus.set_focus(Some(index.cloned()));
                    event.prevent_default();
                }
            },

            ..attributes,
            {children}
        }
    }
}

/// Native (Blitz) arm: a plain positioned `div` -- Blitz has no popover-API
/// support, matching `NavbarContentRendered`'s native arm.
#[cfg(not(feature = "web"))]
#[component]
fn NavigationMenuContentRendered(
    /// The DOM id of the content element.
    id: String,
    /// Extra HTML attributes for the content element.
    attributes: Vec<Attribute>,
    /// The content of the panel, typically a grid of links.
    children: Element,
) -> Element {
    let mut ctx: NavigationMenuContext = use_context();
    let item_ctx: NavigationMenuItemContext = use_context();
    let index = item_ctx.index;

    let labelledby: Vec<Attribute> = if has_own_accessible_name(&attributes) {
        Vec::new()
    } else {
        attributes!(div {
            aria_labelledby: "{item_ctx.trigger_id}"
        })
    };
    let attributes = merge_attributes(vec![
        attributes,
        labelledby,
        // Owned by this component -- see the web arm's identical
        // construction above (`dev-docs/backlog.md` row 93).
        attributes!(div {
            "data-state": if (item_ctx.is_open)() { "open" } else { "closed" },
        }),
    ]);

    rsx! {
        div {
            id,
            // No-ops when `open_on_hover` is `false`: a click-opened panel
            // is not closed by the pointer leaving it.
            onmouseenter: move |_| {
                ctx.hover.hold();
            },
            onmouseleave: move |_| {
                ctx.hover.leave(index.cloned());
            },
            onkeydown: move |event: Event<KeyboardData>| {
                if event.key() == Key::Escape {
                    ctx.set_open.call(None);
                    // `clear_focus()` before `set_focus` -- see the web
                    // arm's identical Escape handler, just above in this
                    // file, for the full account of why `set_focus` alone
                    // can silently no-op here.
                    ctx.focus.clear_focus();
                    ctx.focus.set_focus(Some(index.cloned()));
                    event.prevent_default();
                }
            },

            ..attributes,
            {children}
        }
    }
}

/// The props for the [`NavigationMenuLink`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuLinkProps {
    /// Whether this link represents the current page. Reflected as
    /// `data-active` and, per the disclosure-navigation example's own
    /// `aria-current="page"` row, as `aria-current`.
    #[props(default)]
    pub active: bool,

    /// Whether this link is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// The link's place in its panel's keyboard order (ArrowUp/ArrowDown). Leave it unset: links
    /// are ordered as they appear in the document. Set it on every link of the panel only when one
    /// is inserted between links that are already mounted, or a keyed list reorders them. Ignored
    /// for a top-level link (a direct child of [`NavigationMenuItem`]), which takes its place from
    /// its [`NavigationMenuItem`].
    #[props(default)]
    pub index: ReadSignal<Option<usize>>,

    /// Called when the link is activated (after this primitive's own
    /// close-the-open-panel handling runs).
    pub onclick: Option<EventHandler<MouseEvent>>,

    /// Additional attributes to apply to the `a` element -- pass `href`
    /// here. This module renders a plain anchor rather than
    /// `dioxus_router::Link`, so it needs no `router` feature.
    #[props(extends = GlobalAttributes)]
    #[props(extends = a)]
    pub attributes: Vec<Attribute>,
    /// The children of the link.
    pub children: Element,
}

/// # NavigationMenuLink
///
/// A plain `a[href]`, usable either as a top-level item (a direct child of
/// [`NavigationMenuItem`], no [`NavigationMenuTrigger`]/[`NavigationMenuContent`]
/// pair) or nested inside a [`NavigationMenuContent`] panel.
///
/// This must be used inside a [`NavigationMenuItem`] component.
///
/// ## Styling
///
/// The [`NavigationMenuLink`] component defines the following data
/// attributes you can use to control styling:
/// - `data-active`: Indicates if this link represents the current page.
///   Values are `true` or `false`.
/// - `data-disabled`: Indicates if the link is disabled. Values are `true`
///   or `false`.
#[component]
pub fn NavigationMenuLink(props: NavigationMenuLinkProps) -> Element {
    let mut ctx: NavigationMenuContext = use_context();
    let mut item_ctx: NavigationMenuItemContext = use_context();
    let content_ctx: Option<NavigationMenuContentContext> = try_use_context();
    let is_top_level = content_ctx.is_none();
    let direction = use_direction(None);
    let disabled = move || (ctx.disabled)() || (item_ctx.disabled)() || (props.disabled)();

    // `use_item` is always called (Rules of Hooks: this component's hook
    // order must never depend on a runtime value) -- but only a top-level
    // link is meant to participate in the top-level Left/Right/Home/End
    // collection (`ctx.focus`, shared across every top-level item, keyed by
    // its own `NavigationMenuItem`'s `index` prop). A content link instead
    // participates in *this item's own* `content_focus` collection
    // (ArrowUp/ArrowDown, and the trigger's own ArrowDown-to-first-link --
    // see `NavigationMenuItemContext::content_focus`'s doc), in document
    // order (`crate::collection::use_item_index`; both hooks run for every
    // link, and a top-level link's unused content slot is never rendered).
    // `is_top_level` cannot change for a mounted instance
    // (it is fixed by where this component sits in the tree), so which of
    // the two ends up used is itself stable across this instance's
    // re-renders.
    let content_order = use_item_index(item_ctx.content_focus, props.index);
    let (collection, order) = if is_top_level {
        (ctx.focus, item_ctx.order)
    } else {
        (item_ctx.content_focus, content_order)
    };
    let own_index = order.index;
    let item = use_item(collection_item(collection, own_index).disabled(disabled));
    let mut onmounted_item = item.onmounted();

    // Owned by this component -- active/disabled state must win over a
    // caller's own attributes (`dev-docs/backlog.md` row 93's
    // duplicate-attribute hazard).
    let attributes = merge_attributes(vec![
        props.attributes,
        order.marker_attribute(),
        attributes!(a {
            aria_current: props.active.then_some("page"),
            // `<a>` has no native disabled semantics (unlike
            // `<button disabled>`), so `data-disabled` alone would look,
            // to assistive tech, like a perfectly normal active link --
            // same fix as `NavbarItem`/`DropdownMenuItem` (dev-docs/backlog.md
            // row 39).
            aria_disabled: disabled(),
            "data-active": props.active,
            "data-disabled": disabled(),
        }),
    ]);

    rsx! {
        a {
            onmounted: move |evt: MountedEvent| {
                onmounted_item(evt.clone());
            },
            onclick: move |event: MouseEvent| {
                if disabled() {
                    event.prevent_default();
                    return;
                }
                ctx.hover.cancel();
                ctx.set_open.call(None);
                if let Some(onclick) = props.onclick {
                    onclick.call(event);
                }
            },
            onfocus: move |_| {
                ctx.blur_close.cancel();
                // Sync native focus back into whichever collection this
                // link belongs to -- see `NavigationMenuTrigger`'s
                // identical addition for the full account (Tab landing on
                // a link never otherwise informs `content_focus`/`ctx.focus`
                // that it is now the "current" item, so ArrowUp/ArrowDown
                // -- or Left/Right for a top-level link -- would otherwise
                // waste their first press catching the collection up
                // instead of moving).
                let mut collection = collection;
                collection.set_focus(Some(own_index.cloned()));
            },
            onblur: move |_| {
                ctx.blur_close.schedule(FOCUS_LEAVE_CLOSE_DELAY, move || {
                    ctx.set_open.call(None);
                });
            },
            onkeydown: move |event: Event<KeyboardData>| {
                if is_top_level {
                    match direction.ltr_equivalent(&event.key()) {
                        Key::ArrowLeft => ctx.focus.focus_prev(),
                        Key::ArrowRight => ctx.focus.focus_next(),
                        // disclosure-navigation-hybrid.html's own keyboard
                        // table, Home/End rows: "moves focus to the
                        // first/last item" -- mirrors
                        // `NavigationMenuTrigger`'s identical addition (a
                        // top-level item is a trigger or a plain link
                        // interchangeably, and both share this same
                        // collection).
                        Key::Home => ctx.focus.focus_first(),
                        Key::End => ctx.focus.focus_last(),
                        _ => return,
                    }
                } else {
                    match event.key() {
                        // disclosure-navigation-hybrid.html's own keyboard
                        // table, "Up Arrow"/"Down Arrow" rows: "If focus is
                        // on a link within an expanded dropdown, and it is
                        // not the first/last link, moves focus to the
                        // previous/next link" -- explicitly conditional on
                        // "not the first/last link", i.e. non-looping
                        // (stops at the ends) rather than wrapping, the
                        // same choice (and the same citation) this
                        // module's top-level collection already makes --
                        // see `NavigationMenu`'s own doc comment on
                        // `focus`. `content_focus` is constructed
                        // non-looping for the identical reason.
                        Key::ArrowUp => item_ctx.content_focus.focus_prev(),
                        Key::ArrowDown => item_ctx.content_focus.focus_next(),
                        _ => return,
                    }
                }
                event.prevent_default();
            },

            ..attributes,
            {props.children}
        }
    }
}
