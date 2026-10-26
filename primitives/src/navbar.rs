//! Defines the [`Navbar`] component and its sub-components.

use crate::{
    collection::{
        collection_item, use_collection_provider, use_deferred_collection_focus, use_item,
        CollectionPlacement, CollectionState,
    },
    direction::{use_direction, use_direction_signal, Direction, HorizontalNav},
    has_own_accessible_name,
    hover_intent::{use_hover_intent, HoverIntent},
    merge_attributes,
    nav_link::NavLink,
    use_animated_open, use_id_or, use_unique_id,
};
use dioxus::prelude::*;
use dioxus_attributes::attributes;

#[derive(Clone, Copy)]
struct NavbarContext {
    // Currently open nav index
    open_nav: Signal<Option<usize>>,
    set_open_nav: Callback<Option<usize>>,
    disabled: ReadSignal<bool>,

    // Focus state
    focus: CollectionState,

    // Text direction, for `ArrowLeft`/`ArrowRight`'s roving-focus role
    // between top-level nav triggers -- always horizontal (a navbar is a
    // single row) -- see `direction::Direction::resolve_horizontal`'s doc.
    direction: ReadSignal<Direction>,

    // The pointer-hover contract (`NavbarProps::open_on_hover`): when a
    // pointer resting on a nav opens it, and -- the half that must never be
    // missing -- when the pointer leaving it closes it again. Owns the open
    // index's write path (`set_open_nav` is `hover.set_open()`), so the
    // opening and closing halves cannot be wired separately; see
    // `crate::hover_intent`.
    hover: HoverIntent,
}

/// The props for the [`Navbar`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NavbarProps {
    /// Whether the navbar is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether focus should loop around when reaching the end.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// The text direction for `ArrowLeft`/`ArrowRight` roving focus between
    /// top-level nav triggers. Defaults to the nearest
    /// [`crate::direction::DirectionProvider`], or LTR if there is none.
    #[props(default)]
    pub dir: Option<Direction>,

    /// Whether hovering a [`NavbarNav`] opens its dropdown (after a short
    /// hover-intent delay; once one is open, hovering another switches to it
    /// at once), and the pointer leaving the nav -- its trigger and its
    /// dropdown -- closes it again after a short grace delay, however it was
    /// opened (hover, click or keyboard), as Radix's `NavigationMenu` does.
    /// Moving from the trigger into the dropdown, across the gap between them,
    /// keeps it open. Defaults to `true`. The whole contract lives in
    /// `crate::hover_intent`.
    ///
    /// Set it to `false` for **click activation**: a dropdown then opens only
    /// when its [`NavbarTrigger`] is clicked or activated with `Enter`/
    /// `ArrowDown`/`ArrowUp`, hovering does nothing (it neither opens a
    /// dropdown nor switches between them), and the pointer leaving a
    /// click-opened dropdown does not close it. It closes like a keyboard-
    /// opened one: clicking its trigger again, `Escape`, or a press or focus
    /// outside it (native popover light dismiss). Keyboard behaviour is
    /// identical in both modes, and touch never hover-opens either way.
    ///
    /// Same name as Base UI's `Menu.Trigger` `openOnHover` prop; neither Base
    /// UI nor Radix has a switch for hover on a navigation row.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub open_on_hover: ReadSignal<bool>,

    /// Additional attributes to apply to the navbar element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the navbar component.
    pub children: Element,
}

/// # Navbar
///
/// The `Navbar` component creates a navigation bar that allows users to navigate through different sections with keyboard and pointer interactions.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::navbar::{Navbar, NavbarContent, NavbarItem, NavbarNav, NavbarTrigger};
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Navbar {
///             aria_label: "Components",
///             NavbarNav { index: 0usize,
///                 NavbarTrigger {
///                     "Inputs"
///                 }
///                 NavbarContent {
///                     NavbarItem {
///                         index: 0usize,
///                         value: "calendar".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=calendar",
///                         "Calendar"
///                     }
///                     NavbarItem {
///                         index: 1usize,
///                         value: "slider".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=slider",
///                         "Slider"
///                     }
///                 }
///             }
///             NavbarNav { index: 1usize,
///                 NavbarTrigger {
///                     "Information"
///                 }
///                 NavbarContent {
///                     NavbarItem {
///                         index: 0usize,
///                         value: "toast".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=toast",
///                         "Toast"
///                     }
///                     NavbarItem {
///                         index: 1usize,
///                         value: "tabs".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=tabs",
///                         "Tabs"
///                     }
///                 }
///             }
///             NavbarItem {
///                 index: 2usize,
///                 value: "home".to_string(),
///                 to: "https://dioxuslabs.github.io/components",
///                 "Home"
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`Navbar`] component defines the following data attributes you can use to control styling:
/// - `data-disabled`: Indicates if the navbar is disabled. Values are `true` or `false`.
/// - `data-direction`: The resolved text direction. Values are `ltr` or `rtl`.
#[component]
pub fn Navbar(props: NavbarProps) -> Element {
    let open_nav = use_signal(|| None);
    let hover = use_hover_intent(open_nav, props.open_on_hover);
    let set_open_nav = hover.set_open();
    let direction = use_direction(props.dir);
    let direction_signal = use_direction_signal(direction);

    let focus = use_collection_provider(props.roving_loop);
    let mut ctx = use_context_provider(|| NavbarContext {
        open_nav,
        set_open_nav,
        disabled: props.disabled,
        focus,
        direction: direction_signal,
        hover,
    });
    use_effect(move || {
        let index = ctx.focus.focused_index();
        if ctx.open_nav.peek().is_some() {
            ctx.set_open_nav.call(index);
        }
    });

    let aria_label = props
        .attributes
        .iter()
        .find_map(|attr| (attr.name == "aria-label").then(|| attr.value.clone()));

    // Owned by this component -- role/direction/disabled/roving-focus state
    // must win over a caller's own attributes (`dev-docs/backlog.md` row 93's
    // duplicate-attribute hazard).
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(div {
            // Deliberately NOT routed through `menu_semantics`: this is
            // the Menu and Menubar pattern's own top-level container
            // role, the same element `Menubar`'s identical bare
            // `role: "menubar"` literal renders (`menubar.rs`) --
            // `menu_semantics`'s module doc states this exact case is
            // out of its scope ("`menubar`'s own top-level container
            // role ... a distinct element of the Menu and Menubar
            // pattern, not shared with a menu-button's popup"). Leaving
            // it inline here keeps `Navbar` consistent with `Menubar`
            // rather than widening the module to cover a role it
            // explicitly disclaims.
            role: "menubar",
            dir: direction.as_str(),
            "data-disabled": (props.disabled)(),
            "data-direction": direction.as_str(),
            tabindex: (!ctx.focus.any_focused()).then_some("0"),
        }),
    ]);

    rsx! {
        div {
            role: "navigation",
            display: "content",
            aria_label,
            div {
                // If the menu receives focus, focus the most recently focused menu item.
                onfocus: move |_| {
                    ctx.focus.set_focus(Some(ctx.focus.recent_focus_or_default()));
                },
                onkeydown: move |event: Event<KeyboardData>| {
                    match event.key() {
                        Key::Escape => {
                            // Escape refocuses the trigger (dev-docs/backlog.md
                            // row 53): `Navbar` fully supports keyboard-driven
                            // opening (`NavbarNav`'s ArrowDown/ArrowUp move
                            // focus into the open content exactly the way
                            // `MenubarMenu`'s do), so it needs the same
                            // "return focus to the trigger the open menu
                            // belongs to" contract `MenubarMenu`'s own
                            // Escape arm already implements
                            // (`menubar.rs`). Read the open index BEFORE
                            // closing (`set_open_nav.call(None)` below
                            // clears it), then re-focus that trigger via
                            // the collection -- `ctx.focus` never actually
                            // left that trigger's own index while keyboard
                            // focus roamed *within* the open nav's own
                            // content (`nav_ctx.focus`, a separate
                            // collection, tracks that), so a plain
                            // `set_focus` would be a same-value no-op that
                            // never re-runs `control_mount_focus` -- clear
                            // first to force a real transition, mirroring
                            // `MenubarMenu`'s identical comment. A no-op
                            // when nothing was open (Escape pressed with no
                            // nav open).
                            let open_index = *ctx.open_nav.peek();
                            ctx.set_open_nav.call(None);
                            if let Some(index) = open_index {
                                ctx.focus.clear_focus();
                                ctx.focus.set_focus(Some(index));
                            }
                        }
                        Key::ArrowLeft | Key::ArrowRight => {
                            match (ctx.direction)().resolve_horizontal(&event.key()) {
                                Some(HorizontalNav::Prev) => ctx.focus.focus_prev(),
                                Some(HorizontalNav::Next) => ctx.focus.focus_next(),
                                None => {}
                            }
                        }
                        Key::Home => ctx.focus.focus_first(),
                        Key::End => ctx.focus.focus_last(),
                        _ => return,
                    }
                    event.prevent_default();
                },

                ..attributes,

                {props.children}
            }
        }
    }
}

#[derive(Clone, Copy)]
struct NavbarNavContext {
    index: ReadSignal<usize>,
    focus: CollectionState,
    is_open: Memo<bool>,
    disabled: ReadSignal<bool>,
    initial_focus: Signal<Option<CollectionPlacement>>,

    // The current `NavbarContent`'s own element id for *this* nav, kept in
    // sync by that component -- mirrors `MenubarMenuContext::content_id`
    // (`menubar.rs`). `NavbarTrigger`'s `anchor-name` must key off this
    // signal (not some navbar-wide id) so each nav's content anchors to
    // *its own* trigger, not another nav's -- see that doc for the exact
    // bug this guards against if trigger and content ever named different
    // ids.
    content_id: Signal<String>,

    // This nav's own `NavbarTrigger` element id, so `NavbarContent` can
    // label itself `aria_labelledby` from it -- dev-docs/backlog.md row 41,
    // mirroring `MenubarMenuContext::trigger_id`'s identical role
    // (`menubar.rs`, dev-docs/backlog.md row 25: an APG menu requires an
    // accessible name from either aria-labelledby or aria-label; this
    // crate's `DropdownMenu`/`ContextMenu`/`Menubar` already do the same,
    // keyed off their own `trigger_id`). Read by `NavbarContentRendered`;
    // kept in sync with `NavbarTrigger`'s own (possibly caller-overridden)
    // `id` via `use_id_or` there -- see that component's doc
    // (dev-docs/backlog.md row 36) -- rather than set once via `use_unique_id`,
    // so a caller-supplied id cannot desync from the id `NavbarContent` reads.
    trigger_id: Signal<String>,
}

impl NavbarNavContext {
    fn focus_next(&mut self) {
        self.focus.focus_next();
    }

    fn focus_prev(&mut self) {
        self.focus.focus_prev();
    }
}

/// The props for the [`NavbarNav`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NavbarNavProps {
    /// The index of this nav item in the navbar. This is used to define the focus order for keyboard navigation.
    pub index: ReadSignal<usize>,

    /// Whether this nav item is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Additional attributes to apply to the nav element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the nav component.
    pub children: Element,
}

/// # NavbarNav
///
/// The `NavbarNav` component represents a single navigation dropdown within a navbar. It contains a [`NavbarTrigger`] and [`NavbarContent`] that can be opened and closed.
///
/// This must be used inside a [`Navbar`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::navbar::{Navbar, NavbarContent, NavbarItem, NavbarNav, NavbarTrigger};
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Navbar {
///             aria_label: "Components",
///             NavbarNav { index: 0usize,
///                 NavbarTrigger {
///                     "Inputs"
///                 }
///                 NavbarContent {
///                     NavbarItem {
///                         index: 0usize,
///                         value: "calendar".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=calendar",
///                         "Calendar"
///                     }
///                     NavbarItem {
///                         index: 1usize,
///                         value: "slider".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=slider",
///                         "Slider"
///                     }
///                 }
///             }
///             NavbarNav { index: 1usize,
///                 NavbarTrigger {
///                     "Information"
///                 }
///                 NavbarContent {
///                     NavbarItem {
///                         index: 0usize,
///                         value: "toast".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=toast",
///                         "Toast"
///                     }
///                     NavbarItem {
///                         index: 1usize,
///                         value: "tabs".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=tabs",
///                         "Tabs"
///                     }
///                 }
///             }
///             NavbarItem {
///                 index: 2usize,
///                 value: "home".to_string(),
///                 to: "https://dioxuslabs.github.io/components",
///                 "Home"
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`NavbarNav`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates if the nav is open or closed. Values are `open` or `closed`.
/// - `data-disabled`: Indicates if the nav is disabled. Values are `true` or `false`.
#[component]
pub fn NavbarNav(props: NavbarNavProps) -> Element {
    let mut ctx: NavbarContext = use_context();
    let is_open = use_memo(move || (ctx.open_nav)() == Some(props.index.cloned()));
    let focus = use_collection_provider(ctx.focus.loop_signal());
    let initial_focus = use_signal(|| None);

    let disabled = move || (ctx.disabled)() || (props.disabled)();

    // Placeholder value until `NavbarContent` mounts and syncs its own id in
    // -- see `NavbarNavContext::content_id`'s doc.
    let content_id = use_unique_id();
    // This nav's own trigger id -- see `NavbarNavContext::trigger_id`'s doc.
    let trigger_id = use_unique_id();

    let mut nav_ctx = use_context_provider(|| NavbarNavContext {
        index: props.index,
        focus,
        is_open,
        disabled: props.disabled,
        initial_focus,
        content_id,
        trigger_id,
    });

    use_effect(move || {
        if !is_open() {
            nav_ctx.focus.clear_focus();
            nav_ctx.initial_focus.set(None);
        }
    });

    // Owned by this component -- see `Navbar`'s identical construction
    // above (`dev-docs/backlog.md` row 93).
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(div {
            // Pattern-class popup role, shared with `MenubarMenu`'s
            // identical always-rendered wrapper div (`menubar.rs`) -- see
            // `menu_semantics`'s module doc for why this is the `menu` half
            // of "role of either `menu` or `menubar`", not a hand-written
            // literal.
            role: crate::menu_semantics::MENU_ROLE,
            "data-state": if is_open() { "open" } else { "closed" },
            "data-disabled": (ctx.disabled)() || (props.disabled)(),
        }),
    ]);

    rsx! {
        div {
            // The nav wraps its trigger AND its (top-layer) content, which is a
            // DOM descendant, so these two events cover the whole hover region:
            // the pointer crossing the gap between the trigger and the content
            // leaves and re-enters it, which the close-grace timer absorbs. Both
            // are no-ops when `open_on_hover` is `false` (click activation) --
            // `HoverIntent` checks it. See `crate::hover_intent` and
            // `NavbarProps::open_on_hover`.
            onmouseenter: move |_| {
                if !disabled() {
                    ctx.hover.enter(nav_ctx.index.cloned());
                }
            },
            onmouseleave: move |_| {
                ctx.hover.leave(nav_ctx.index.cloned());
            },
            onkeydown: move |event: Event<KeyboardData>| {
                // A key press is a deliberate action: a pending hover open or
                // close must not overrule it.
                ctx.hover.cancel();
                match event.key() {
                    Key::Enter if !disabled() => {
                        ctx.set_open_nav.call((!is_open()).then(&*props.index));
                    }
                    Key::ArrowDown if !disabled() => {
                        if !is_open() {
                            nav_ctx.initial_focus.set(Some(CollectionPlacement::First));
                            ctx.set_open_nav.call(Some(props.index.cloned()));
                        } else {
                            nav_ctx.focus_next();
                        }
                    },
                    Key::ArrowUp if !disabled() => {
                        if is_open() {
                            nav_ctx.focus_prev();
                        } else {
                            nav_ctx.initial_focus.set(Some(CollectionPlacement::Last));
                            ctx.set_open_nav.call(Some(props.index.cloned()));
                        }
                    },
                    _ => return,
                }
                event.prevent_default();
            },

            ..attributes,
            {props.children}
        }
    }
}

/// The props for the [`NavbarTrigger`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NavbarTriggerProps {
    /// The ID of the trigger button. If not provided, an internally
    /// generated ID is used -- see `NavbarTrigger`'s use of `use_id_or`
    /// for why this is a typed field (dev-docs/backlog.md rows 36 and 41).
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes to apply to the trigger element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the trigger component.
    pub children: Element,
}

/// # NavbarTrigger
///
/// The `NavbarTrigger` component is a button that opens and closes a [`NavbarNav`]. It controls if the associated [`NavbarContent`] is visible or not.
///
/// This must be used inside a [`NavbarNav`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::navbar::{Navbar, NavbarContent, NavbarItem, NavbarNav, NavbarTrigger};
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Navbar {
///             aria_label: "Components",
///             NavbarNav { index: 0usize,
///                 NavbarTrigger {
///                     "Inputs"
///                 }
///                 NavbarContent {
///                     NavbarItem {
///                         index: 0usize,
///                         value: "calendar".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=calendar",
///                         "Calendar"
///                     }
///                     NavbarItem {
///                         index: 1usize,
///                         value: "slider".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=slider",
///                         "Slider"
///                     }
///                 }
///             }
///             NavbarNav { index: 1usize,
///                 NavbarTrigger {
///                     "Information"
///                 }
///                 NavbarContent {
///                     NavbarItem {
///                         index: 0usize,
///                         value: "toast".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=toast",
///                         "Toast"
///                     }
///                     NavbarItem {
///                         index: 1usize,
///                         value: "tabs".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=tabs",
///                         "Tabs"
///                     }
///                 }
///             }
///             NavbarItem {
///                 index: 2usize,
///                 value: "home".to_string(),
///                 to: "https://dioxuslabs.github.io/components",
///                 "Home"
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn NavbarTrigger(props: NavbarTriggerProps) -> Element {
    let mut ctx: NavbarContext = use_context();
    let nav_ctx: NavbarNavContext = use_context();
    let disabled = move || (ctx.disabled)() || (nav_ctx.disabled)();
    // The nav's trigger is the focusable element, so it registers the nav in the
    // parent navbar collection.
    let item = use_item(collection_item(ctx.focus, nav_ctx.index).disabled(disabled));
    let is_focused = move || item.focused() && !nav_ctx.focus.any_focused();
    let onmounted = item.onmounted();
    let is_open = nav_ctx.is_open;
    // See the `onpointerdown`/`onclick` pair below (dev-docs/backlog.md row 40).
    let mut open_at_down: Signal<Option<bool>> = use_signal(|| None);

    // dev-docs/backlog.md row 36: `nav_ctx.trigger_id` is the same signal
    // `NavbarContent`'s `aria-labelledby` reads back (row 41, below) -- a
    // caller-supplied `id` here must resolve into it, not just win the
    // render. Feeding it straight into `use_id_or` as the generated-id
    // signal does that: `use_id_or`'s own effect writes a caller override
    // back into its `gen_id` argument, which *is* `nav_ctx.trigger_id`
    // here -- mirrors `DialogTitle`'s identical
    // `use_id_or(ctx.dialog_labelledby, props.id)` (`dialog.rs`).
    let id = use_id_or(nav_ctx.trigger_id, props.id);

    // Merged (caller-wins, deduped) rather than a plain explicit-attribute +
    // `..props.attributes` spread: `id` below and a caller-supplied `id` in
    // `props.attributes` would otherwise both render, the same
    // duplicate-attribute hydration hazard `merge_attributes`'s own doc
    // warns about (dev-docs/conformance-harness.md Rule 4) -- see
    // `MenubarTrigger`'s identical construction (`menubar.rs`).
    let attributes = merge_attributes(vec![
        attributes!(button {
            // This nav's own trigger id, so `NavbarContent` can label
            // itself `aria_labelledby` from it -- dev-docs/backlog.md row 41.
            id: id.cloned(),
        }),
        props.attributes,
    ]);
    // Ties this trigger to the web-arm content's `position-anchor`
    // (`NavbarContentRendered`) so its anchor-positioned placement resolves
    // relative to *this* trigger once promoted to the top layer -- keyed on
    // `nav_ctx.content_id`, not this nav's own index, for the same reason
    // `MenubarTrigger` keys off `menu_ctx.content_id` (see that doc).
    // Not a bare `style: anchor_name_style(..)` literal directly on
    // this element, beside `..attributes` below: a caller supplying their
    // own `style` would then produce two `style` attributes on this same tag
    // (`dev-docs/conformance-harness.md` hydration-parity Rule 4;
    // `scripts/check-attr-spread-collision.sh`). Folding it into
    // `attributes` here, via `top_layer::anchored_trigger_attributes`,
    // avoids that and (unlike plain `merge_attributes`, which folds a plain
    // `style` attribute but leaves shorthand style props -- `padding:
    // ".."`, namespace `Some("style")` -- as separate attributes) renders
    // exactly one `style` carrying the anchor binding plus whatever the
    // caller supplied; see that function's own doc.
    let attributes =
        crate::top_layer::anchored_trigger_attributes(&nav_ctx.content_id.cloned(), attributes);
    // Owned by this component -- role/type/roving-focus state must win
    // over a caller's own attributes (`dev-docs/backlog.md` row 93's
    // duplicate-attribute hazard).
    let attributes = merge_attributes(vec![
        attributes,
        attributes!(button {
            // Pattern-class activatable-item role -- see
            // `MenubarTrigger`'s identical `crate::menu_semantics::
            // MENU_ITEM_ROLE` (`menubar.rs`): this trigger is itself an
            // item within the parent navbar's `role="menubar"` container.
            role: crate::menu_semantics::MENU_ITEM_ROLE,
            type: "button",
            tabindex: if is_focused() { "0" } else { "-1" },
        }),
    ]);

    rsx! {
        button {
            onmounted,
            // dev-docs/backlog.md row 40: the toggle happens on `click`, NOT on
            // `pointerdown`. `NavbarContent` is `popover="auto"`, and the
            // browser's native light dismiss reads only `pointerdown` +
            // `pointerup` (WHATWG HTML "popover light dismiss"): a popover
            // shown between a gesture's `pointerdown` and its `pointerup`
            // (any hold longer than a render + effect -- every real
            // finger) sees `null === null` at `pointerup` and is hidden by
            // the very gesture that opened it. `click` fires after
            // `pointerup`, so opening here cannot be self-dismissed on any
            // engine. Contract: an `auto` popover is shown only from
            // `click`/`pointerup`/keyboard/timer, never from `pointerdown`
            // (see `top_layer::PopoverKind::Auto`).
            onpointerdown: move |event| {
                if !disabled() {
                    // Suppress the synthesized focus shift so that tapping a child
                    // menuitem on mobile doesn't blur the trigger and trip onblur,
                    // which would close the nav mid-tap and detach the menuitem
                    // before Playwright (or a real user) can complete the tap.
                    event.prevent_default();
                    // Snapshot the state this gesture started from. On touch the
                    // compat `mouseenter` (`NavbarNav`'s hover-open) fires
                    // between `pointerup` and `click`, so by `click` time
                    // `is_open()` is already true -- toggling off that would
                    // close the menu the tap was opening. Desktop hover-open
                    // happens before `pointerdown`, so the snapshot is `true`
                    // there and the click closes it.
                    open_at_down.set(Some(is_open()));
                }
            },
            onclick: move |_| {
                // A click is a deliberate action: a still-pending hover open
                // (the pointer arrived <150ms ago) must not fire after it and
                // a pending close must not undo it.
                ctx.hover.cancel();
                // `None`: a click with no preceding pointerdown (Space
                // button activation, assistive tech) -- toggle live state.
                let was_open = open_at_down.write().take().unwrap_or_else(&*is_open);
                if !disabled() {
                    let new_open = if was_open { None } else { Some(nav_ctx.index.cloned()) };
                    ctx.set_open_nav.call(new_open);
                }
            },
            // Any key activation invalidates a stale pointerdown snapshot
            // (a pointerdown that never produced a click, e.g. dragged off).
            onkeydown: move |_| {
                open_at_down.set(None);
            },
            onblur: move |_| {
                if is_focused() {
                    ctx.focus.clear_focus();
                    ctx.set_open_nav.call(None);
                }
            },
            ..attributes,
            {props.children}
        }
    }
}

/// The props for the [`NavbarContent`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NavbarContentProps {
    /// The id of the content element.
    pub id: ReadSignal<Option<String>>,
    /// Additional attributes to apply to the content element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the content component.
    pub children: Element,
}

/// # NavbarContent
///
/// The `NavbarContent` component defines the content of a [`NavbarNav`] that appears when the
/// [`NavbarTrigger`] is activated. It contains the [`NavbarItem`]s inside the navbar dropdown.
///
/// This must be used inside a [`NavbarNav`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::navbar::{Navbar, NavbarContent, NavbarItem, NavbarNav, NavbarTrigger};
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Navbar {
///             aria_label: "Components",
///             NavbarNav { index: 0usize,
///                 NavbarTrigger {
///                     "Inputs"
///                 }
///                 NavbarContent {
///                     NavbarItem {
///                         index: 0usize,
///                         value: "calendar".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=calendar",
///                         "Calendar"
///                     }
///                     NavbarItem {
///                         index: 1usize,
///                         value: "slider".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=slider",
///                         "Slider"
///                     }
///                 }
///             }
///             NavbarNav { index: 1usize,
///                 NavbarTrigger {
///                     "Information"
///                 }
///                 NavbarContent {
///                     NavbarItem {
///                         index: 0usize,
///                         value: "toast".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=toast",
///                         "Toast"
///                     }
///                     NavbarItem {
///                         index: 1usize,
///                         value: "tabs".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=tabs",
///                         "Tabs"
///                     }
///                 }
///             }
///             NavbarItem {
///                 index: 2usize,
///                 value: "home".to_string(),
///                 to: "https://dioxuslabs.github.io/components",
///                 "Home"
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`NavbarContent`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates if the nav is open or closed. Values are `open` or `closed`.
/// - `data-open-menu-direction`: Indicates the direction of the open menu relative to this content. Values are `start`, `end`, `open`, or `closed`.
#[component]
pub fn NavbarContent(props: NavbarContentProps) -> Element {
    let ctx: NavbarContext = use_context();
    let mut nav_ctx: NavbarNavContext = use_context();
    let index = nav_ctx.index.cloned();
    let open_direction = match (ctx.open_nav)() {
        Some(open_index) if open_index > index => "start",
        Some(open_index) if open_index < index => "end",
        Some(_) => "open",
        None => "closed",
    };

    let unique_id = use_unique_id();
    let id = use_id_or(unique_id, props.id);

    // Keep `nav_ctx.content_id` in sync with this content's actual id --
    // see `NavbarNavContext::content_id`'s doc. Mirrors `MenubarContent`'s
    // identical `menu_ctx.content_id.set(id())` (`menubar.rs`).
    use_effect(move || nav_ctx.content_id.set(id()));

    let render = use_animated_open(id, nav_ctx.is_open);
    use_deferred_collection_focus(nav_ctx.focus, nav_ctx.initial_focus, render);

    rsx! {
        if render() {
            NavbarContentRendered {
                id: id.cloned(),
                open_direction,
                attributes: props.attributes,
                children: props.children,
            }
        }
    }
}

/// Web arm: promote each nav's content to the top
/// layer via `popover="auto"`, anchored to its own trigger -- the same
/// mechanism `MenubarContentRendered`/`DropdownMenuContentRendered` use
/// (`menubar.rs`/`dropdown_menu.rs`), which see for the general
/// `popover`/CSS-anchor wiring this mirrors. A content left a plain `div`
/// positioned by ordinary CSS (`position: absolute; top: 100%; left: 0`
/// inside `.dx-navbar-nav`'s `position: relative` wrapper,
/// `preview/src/components/navbar/style.css`) would have no viewport-edge
/// collision handling of any kind -- unlike `DropdownMenu`/`Menubar`/
/// `Select`, whose menus all flip when they would run off-viewport. So the
/// Navbar menu auto-flips when it gets cut off at the bottom/top like the
/// other menus: anchored the same way, the same shared, engine-injected
/// anchor-positioning stylesheet (`top_layer::ensure_anchor_positioning_
/// styles`) and `use_anchor_position_fallback`'s JS fallback both apply
/// `position-try-fallbacks: flip-block, flip-inline` (or its JS-measured
/// equivalent) to this content exactly as they already do for those three.
///
/// `auto`, not `manual`: same shape and reasoning as `MenubarContentRendered`
/// -- a persistent trigger per nav, plus this component's own existing
/// blur-driven close (`NavbarTrigger`/`NavbarItem`'s `onblur`) as a backstop
/// that native light dismiss only ever supplements, never competes with.
///
/// `open_direction` (Radix-style cosmetic slide transition between sibling
/// navs, unrelated to viewport-edge collision) is threaded straight through
/// from `NavbarContent` above -- unlike `side`/`align`, it depends on
/// `NavbarContext`/the nav's own index, not anything this component or its
/// fallback needs to compute itself.
#[cfg(feature = "web")]
#[component]
fn NavbarContentRendered(
    /// The DOM id of the content element.
    id: String,
    /// Where the open menu sits relative to this one: `start` if it comes after, `end` if it comes before, `open` if this is it, or `closed` when none is open.
    open_direction: &'static str,
    /// Extra HTML attributes for the content element.
    attributes: Vec<Attribute>,
    /// The content of the nav menu.
    children: Element,
) -> Element {
    let ctx: NavbarContext = use_context();
    let nav_ctx: NavbarNavContext = use_context();
    let open = nav_ctx.is_open;
    let index = nav_ctx.index;

    // Drive `showPopover()` from `open`, and sync a native close back into
    // `ctx.open_nav` -- but *only* when this nav is still the one recorded
    // as open. Same guard, and the same reason, as `MenubarContentRendered`'s
    // identical callback (`menubar.rs`, see its doc): `Navbar`'s own effect
    // reacting to focus changes can move `ctx.open_nav` straight from
    // `Some(this index)` to `Some(another index)` in one step, and per
    // WHATWG HTML, showing the *new* nav's `auto` popover natively closes
    // this (now-unrelated) sibling `auto` popover for us, firing this same
    // `toggle` callback with `is_open: false` *after* `ctx.open_nav` already
    // points at the other nav.
    //
    // `use_popover_shown_while_mounted`, not the plain `use_popover_sync` --
    // this content, like `NavigationMenuContent`
    // (`navigation_menu.rs`), renders through [`use_animated_open`] above,
    // which deliberately keeps the element mounted with `data-state="closed"`
    // for its whole exit animation (plus a settle hold) before actually
    // unmounting it. `use_popover_sync`'s "signal -> browser" effect calls
    // `hidePopover()` the instant `open` goes `false`, which (per the UA
    // popover stylesheet's `[popover]:not(:popover-open) { display: none }`)
    // sets `display: none` on the content before `use_animated_open`'s own
    // rAF-deferred `getAnimations()` check ever runs -- the exact defect of
    // `dev-docs/backlog.md` row 77 (the identical `NavigationMenu` bug):
    // `opacity`/`transform` jump straight from the open resting value to the
    // closed target between two consecutive animation frames, with the
    // content disconnected from the DOM one frame later -- no
    // transition frame ever runs. `PopoverKind::Auto` (unlike
    // `NavigationMenuContent`'s `Manual`) is proven safe with this hook
    // already: `popover.rs`'s modal-less `Popover` pairs the identical
    // `Auto` + `use_popover_shown_while_mounted` combination (see that
    // hook's own doc, "Bug 2 (reopen-while-still-mounted, auto only)",
    // which is reasoned against exactly this pairing).
    crate::top_layer::use_popover_shown_while_mounted(
        id.clone(),
        open,
        Callback::new(move |is_open: bool| {
            if !is_open && (ctx.open_nav)() == Some(index.cloned()) {
                ctx.set_open_nav.call(None);
            }
        }),
    );
    // JS-measured static positioning fallback for engines without CSS
    // Anchor Positioning -- see `top_layer::use_anchor_position_fallback`'s
    // doc. `side`/`align`/gap match this component's themed CSS
    // (`../../preview/src/components/navbar/style.css`'s `top: 100%; left:
    // 0; margin-top: 0.5rem` -- 0.5rem == 8px at the default root font
    // size): anchored below the trigger, left-aligned with it, the same
    // bottom/start convention `MenubarContentRendered`/
    // `DropdownMenuContentRendered` use for the identical visual shape.
    crate::top_layer::use_anchor_position_fallback(
        id.clone(),
        id.clone(),
        open,
        crate::ContentSide::Bottom,
        crate::ContentAlign::Start,
        8,
    );

    // dev-docs/backlog.md row 41: an APG menu requires an accessible name from
    // either aria-labelledby or aria-label -- labelled by this nav's own
    // trigger, mirroring `MenubarContent`'s identical row-25 pattern
    // (`menubar.rs`). Only contributed when the caller
    // hasn't already named this content some other way
    // (`has_own_accessible_name`'s own doc), and as its own
    // `merge_attributes` input -- present only when applicable -- rather
    // than a bare `aria_labelledby: ...` literal alongside `..attributes`:
    // a caller attribute list can carry a same-named
    // `aria-labelledby`/`aria-label` with an empty/`AttributeValue::None`
    // value, and two entries for one attribute name is exactly the
    // duplicate-attribute hazard `merge_attributes` exists to prevent
    // (`dev-docs/conformance-harness.md` hydration-parity Rule 4).
    let labelledby: Vec<Attribute> = if has_own_accessible_name(&attributes) {
        Vec::new()
    } else {
        attributes!(div {
            aria_labelledby: "{nav_ctx.trigger_id}"
        })
    };
    let attributes = merge_attributes(vec![
        attributes,
        labelledby,
        // Owned by this component -- menu semantics + top-layer wiring
        // must win over a caller's own attributes (`dev-docs/backlog.md` row
        // 93's duplicate-attribute hazard).
        attributes!(div {
            // Pattern-class popup role -- see `MenubarContentRendered`'s
            // identical web-arm `crate::menu_semantics::MENU_ROLE`
            // (`menubar.rs`), which this mirrors.
            role: crate::menu_semantics::MENU_ROLE,
            popover: crate::top_layer::PopoverKind::Auto.as_str(),
            "data-state": if open() { "open" } else { "closed" },
            "data-open-menu-direction": "{open_direction}",
        }),
    ]);
    // Folds the caller's own `style` together with the anchor binding into
    // one `style` attribute -- see `top_layer::anchored_content_attributes`'s
    // doc for why a bare `style: position_anchor_style(&id)` literal
    // alongside `..attributes` is the duplicate-`style` hazard
    // (`dev-docs/conformance-harness.md` hydration-parity Rule 4). The
    // `top_layer` fixture's `NavbarContent { style: "min-height: 100px;" }`
    // is the exerciser: a bare literal beside the spread would emit two
    // `style` attributes (SSR keeps the anchor binding, the client the
    // caller's).
    let attributes = crate::top_layer::anchored_content_attributes(
        crate::top_layer::AnchorKind::Navbar,
        &id,
        attributes,
    );

    rsx! {
        div {
            id: id.clone(),
            ..attributes,
            {children}
        }
    }
}

/// Native (Blitz) arm: a plain `div` -- Blitz has no popover-API
/// support at all, so this is the functional floor, matching
/// `MenubarContentRendered`'s identical native arm. It carries the
/// row-41 `aria-labelledby` below, the one piece of the web arm that
/// applies here too (unlike `popover`/anchor positioning, which are
/// web-only).
#[cfg(not(feature = "web"))]
#[component]
fn NavbarContentRendered(
    /// The DOM id of the content element.
    id: String,
    /// Where the open menu sits relative to this one: `start` if it comes after, `end` if it comes before, `open` if this is it, or `closed` when none is open.
    open_direction: &'static str,
    /// Extra HTML attributes for the content element.
    attributes: Vec<Attribute>,
    /// The content of the nav menu.
    children: Element,
) -> Element {
    let nav_ctx: NavbarNavContext = use_context();

    // See the web arm's identical construction above (dev-docs/backlog.md row
    // 41) for why this is conditional and routed through `merge_attributes`
    // rather than a bare literal alongside `..attributes`.
    let labelledby: Vec<Attribute> = if has_own_accessible_name(&attributes) {
        Vec::new()
    } else {
        attributes!(div {
            aria_labelledby: "{nav_ctx.trigger_id}"
        })
    };
    let attributes = merge_attributes(vec![
        attributes,
        labelledby,
        // Owned by this component -- see the web arm's identical
        // construction above (`dev-docs/backlog.md` row 93).
        attributes!(div {
            // Pattern-class popup role -- see `MenubarContentRendered`'s
            // identical native-arm `crate::menu_semantics::MENU_ROLE`
            // (`menubar.rs`), which this mirrors.
            role: crate::menu_semantics::MENU_ROLE,
            "data-state": if (nav_ctx.is_open)() { "open" } else { "closed" },
            "data-open-menu-direction": "{open_direction}",
        }),
    ]);

    rsx! {
        div {
            id,
            ..attributes,
            {children}
        }
    }
}

/// The props for the [`NavbarItem`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NavbarItemProps {
    /// The index of this item within the nav. This is used to define the focus order for keyboard navigation.
    pub index: ReadSignal<usize>,

    /// The value associated with this nav item. This will be passed to the [`Self::on_select`] callback when the item is selected.
    pub value: String,

    /// Whether this nav item is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Callback fired when the item is selected. The [`Self::value`] will be passed to this callback when the item is selected.
    #[props(default)]
    pub on_select: Callback<String>,

    /// The class attribute for the `a` tag.
    pub class: Option<String>,

    /// A class to apply to the generate HTML anchor tag if the `target` route is active.
    pub active_class: Option<String>,

    /// When [`true`], the `target` route will be opened in a new tab.
    ///
    /// This does not change whether the link is active or not.
    #[props(default)]
    pub new_tab: bool,

    /// The onclick event handler.
    pub onclick: Option<EventHandler<MouseEvent>>,

    /// The onmounted event handler.
    /// Fired when the `<a>` element is mounted.
    pub onmounted: Option<EventHandler<MountedEvent>>,

    #[props(default)]
    /// Whether the default behavior should be executed if an `onclick` handler is provided.
    ///
    /// 1. When `onclick` is [`None`] (default if not specified), `onclick_only` has no effect.
    /// 2. If `onclick_only` is [`false`] (default if not specified), the provided `onclick` handler
    ///    will be executed after the links regular functionality.
    /// 3. If `onclick_only` is [`true`], only the provided `onclick` handler will be executed.
    pub onclick_only: bool,

    /// The rel attribute for the generated HTML anchor tag.
    ///
    /// For external `a`s, this defaults to `noopener noreferrer`.
    pub rel: Option<String>,

    /// The navigation target. Roughly equivalent to the href attribute of an HTML anchor tag.
    #[props(into)]
    pub to: NavigationTarget,

    /// Additional attributes to apply to the item element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children to render within the generated HTML anchor tag.
    pub children: Element,
}

/// # NavbarItem
///
/// The `NavbarItem` component represents a link within a navbar. It accepts a superset of the props
/// from the router's `Link` component, rendered through [`NavLink`] (one `aria-current`, row 126).
///
/// This must be used inside a [`NavbarContent`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::navbar::{Navbar, NavbarContent, NavbarItem, NavbarNav, NavbarTrigger};
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Navbar {
///             aria_label: "Components",
///             NavbarNav { index: 0usize,
///                 NavbarTrigger {
///                     "Inputs"
///                 }
///                 NavbarContent {
///                     NavbarItem {
///                         index: 0usize,
///                         value: "calendar".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=calendar",
///                         "Calendar"
///                     }
///                     NavbarItem {
///                         index: 1usize,
///                         value: "slider".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=slider",
///                         "Slider"
///                     }
///                 }
///             }
///             NavbarNav { index: 1usize,
///                 NavbarTrigger {
///                     "Information"
///                 }
///                 NavbarContent {
///                     NavbarItem {
///                         index: 0usize,
///                         value: "toast".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=toast",
///                         "Toast"
///                     }
///                     NavbarItem {
///                         index: 1usize,
///                         value: "tabs".to_string(),
///                         to: "https://dioxuslabs.github.io/components/component/?name=tabs",
///                         "Tabs"
///                     }
///                 }
///             }
///             NavbarItem {
///                 index: 2usize,
///                 value: "home".to_string(),
///                 to: "https://dioxuslabs.github.io/components",
///                 "Home"
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`NavbarItem`] component defines the following data attributes you can use to control styling:
/// - `data-disabled`: Indicates if the item is disabled. Values are `true` or `false`.
#[component]
pub fn NavbarItem(mut props: NavbarItemProps) -> Element {
    let mut ctx: NavbarContext = use_context();
    let mut nav_ctx: Option<NavbarNavContext> = try_use_context();

    let disabled = move || {
        (ctx.disabled)()
            || nav_ctx
                .map(|nav_ctx| (nav_ctx.disabled)())
                .unwrap_or_default()
            || (props.disabled)()
    };
    let focused = move || {
        nav_ctx.map_or_else(
            || ctx.focus.is_focused(props.index.cloned()),
            |nav_ctx| nav_ctx.focus.is_focused(props.index.cloned()) && (nav_ctx.is_open)(),
        )
    };

    // Resolve the innermost collection: the parent nav's when inside a
    // NavbarNav, otherwise the top-level navbar's.
    let collection: CollectionState = use_context();
    let mut onmounted =
        use_item(collection_item(collection, props.index).disabled(disabled)).onmounted();

    props.attributes.push(onkeydown({
        let value = props.value.clone();
        let to = props.to.clone();
        move |event: Event<KeyboardData>| {
            if event.key() == Key::Enter || event.key() == Key::Character(" ".to_string()) {
                if !disabled() {
                    props.on_select.call(value.clone());
                    ctx.set_open_nav.call(None);
                    let navigator = navigator();
                    navigator.push(to.clone());
                }
                event.prevent_default();
                event.stop_propagation();
            }
        }
    }));

    props.attributes.push(onpointerdown(move |_| {
        if !disabled() {
            if let Some(mut nav_ctx) = nav_ctx {
                nav_ctx.focus.set_focus(Some(props.index.cloned()));
            }
        }
    }));

    props.attributes.push(onblur(move |_| {
        if focused() {
            if let Some(nav_ctx) = &mut nav_ctx {
                nav_ctx.focus.clear_focus();
            }
            ctx.focus.clear_focus();
            ctx.set_open_nav.call(None);
        }
    }));

    let tabindex = if focused()
        || (nav_ctx.is_none() && ctx.focus.recent_focus_or_default() == props.index.cloned())
    {
        "0"
    } else {
        "-1"
    };

    // The item's own role, disabled state and roving tabindex go through the
    // same merge as the caller's attributes (merged last, so they win), never
    // as literals beside a forwarded `attributes:` that could carry the same
    // key a second time.
    let is_disabled = disabled();
    let item_attributes = merge_attributes(vec![
        std::mem::take(&mut props.attributes),
        attributes!(a {
            // Pattern-class activatable-item role -- see `MenubarItem`'s
            // identical `crate::menu_semantics::MENU_ITEM_ROLE`
            // (`menubar.rs`), which this mirrors.
            role: crate::menu_semantics::MENU_ITEM_ROLE,
            // The disabled state of this pattern class (dev-docs/backlog.md
            // row 39): see `dropdown_menu.rs`'s identical `DropdownMenuItem`
            // wiring for the full account -- this renders as an `<a>`, which
            // (unlike a `<button disabled>`) has no native disabled
            // semantics at all, so `data-disabled` alone would leave this
            // item looking, to assistive tech, like a normal active link.
            aria_disabled: is_disabled,
            "data-disabled": is_disabled,
            tabindex: tabindex,
        }),
    ]);

    rsx! {
        NavLink {
            class: props.class,
            active_class: props.active_class,
            new_tab: props.new_tab,
            onclick_only: props.onclick_only,
            rel: props.rel,
            to: props.to,

            onclick: {
                let value = props.value.clone();
                move |mouse_event| {
                    if !disabled() {
                        props.on_select.call(value.clone());
                        ctx.set_open_nav.call(None);
                    }
                    if let Some(onclick) = props.onclick {
                        onclick.call(mouse_event);
                    }
                }
            },

            onmounted: move |evt: MountedEvent| {
                onmounted(evt.clone());
                if let Some(onmounted) = &props.onmounted {
                    onmounted.call(evt);
                }
            },

            attributes: item_attributes,
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    //! Backlog row 126: a `NavbarItem` must emit exactly one `aria-current`,
    //! even when the caller passes its own (`dioxus_router::Link` writes its
    //! literal beside `..attributes`, so the attribute would come out twice).
    use super::*;

    #[derive(Routable, Clone, PartialEq)]
    enum TestRoute {
        #[route("/?:q")]
        Home { q: Option<String> },
        #[route("/other")]
        Other {},
    }

    #[component]
    fn Home(q: Option<String>) -> Element {
        let _ = q;
        rsx! {
            Navbar { aria_label: "t",
                NavbarItem {
                    index: 0usize,
                    value: "home".to_string(),
                    to: "/?".to_string(),
                    aria_current: "page",
                    "Home"
                }
                NavbarItem {
                    index: 1usize,
                    value: "plain".to_string(),
                    to: "/",
                    "Plain"
                }
                NavbarItem {
                    index: 2usize,
                    value: "other".to_string(),
                    to: "/other",
                    "Other"
                }
            }
        }
    }

    #[component]
    fn Other() -> Element {
        rsx! { "other" }
    }

    #[test]
    fn navbar_item_emits_exactly_one_aria_current() {
        fn app() -> Element {
            rsx! { Router::<TestRoute> {} }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        let anchors: Vec<&str> = html
            .match_indices("<a ")
            .map(|(i, _)| {
                let end = html[i..].find('>').unwrap();
                &html[i..i + end]
            })
            .collect();
        assert_eq!(anchors.len(), 3, "{html}");
        for a in &anchors {
            assert!(a.matches("aria-current").count() <= 1, "duplicate: {a}");
        }
        // Home (query-suffixed target) and Plain are both the page; Other is not.
        assert!(
            anchors[0].contains("aria-current=\"page\""),
            "{}",
            anchors[0]
        );
        assert!(
            anchors[1].contains("aria-current=\"page\""),
            "{}",
            anchors[1]
        );
        assert!(!anchors[2].contains("aria-current"), "{}", anchors[2]);
    }
}
