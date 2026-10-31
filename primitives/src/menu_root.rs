//! What the three menu hosts -- [`crate::dropdown_menu`], [`crate::context_menu`] and [`crate::menubar`] -- share
//! below their own trigger, root and positioning: the host context the shared parts read, the open/focus sync, the
//! content lifecycle and the content panel.
//!
//! # One family, three hosts
//!
//! A host keeps what makes it a different widget: its trigger (a toggle button, a right-click or long-press target,
//! a roving row of triggers), its root and the keys that open it, and how its panel is placed (anchored to the
//! trigger, at a pointer point, under a menubar trigger). Everything an open menu does is one implementation:
//!
//! - [`MenuHost`] -- the small context a host provides and the shared parts read: whether the menu is open, whether
//!   it is disabled, the collection its items register in, the text direction, how a selection commits
//!   ([`crate::menu_item::SelectOn`]), what closes the whole tree, and what a blurred item does. The items
//!   (`crate::menu_item`), the submenu (`crate::menu_sub`) and the panel below are written once against it.
//! - [`use_menu_panel`] -- the content panel: role, state, accessible name, popover promotion and placement (the
//!   last two web-only), as one attribute list a host spreads on its panel element.
//! - [`nav_key`] -- the arrow, Home, End and typeahead keys of an open menu: one rule for the root panel and every
//!   submenu.
//! - [`use_open_focus_sync`] -- keeps a host's `open` boolean in step with whether its roving-focus collection holds
//!   DOM focus (Dropdown and Context; the Menubar keys its open state by menu index instead).
//! - [`use_menu_content_lifecycle`] -- id resolution plus `use_animated_open`'s close-animation hold.
//! - [`content_labelledby_attributes`] -- the panel's accessible name from its trigger.
//!
//! Escape, outside dismissal and focus return stay with the host's call sites of `use_outside_dismiss` and
//! `use_refocus_on_close_unless` (`lib.rs`).
//!
//! `Navbar` and `NavigationMenu` are not menus in this sense (their items are links) and use none of this.

use dioxus::prelude::*;
use dioxus_attributes::attributes;

use crate::collection::CollectionState;
use crate::direction::Direction;
use crate::menu_item::SelectOn;
use crate::typeahead::TypeaheadState;
use crate::{has_own_accessible_name, state_attrs, ContentAlign, ContentSide};

/// The host-side half of the shared menu parts: what an item, a submenu or a panel needs to know about the menu it
/// sits in. Each host provides one from its root (`DropdownMenu`, `ContextMenu`) or, for the Menubar, from each
/// `MenubarMenu`, because every menubar menu has its own item collection.
///
/// All fields are `Copy` handles, so a host builds it once from its own context and the shared parts copy it freely.
#[derive(Clone, Copy)]
pub(crate) struct MenuHost {
    /// Whether the menu this host provides is open.
    pub(crate) open: Memo<bool>,
    /// Whether the whole menu is disabled: its items cannot be selected.
    pub(crate) disabled: ReadSignal<bool>,
    /// The collection the menu's own items register in (a submenu has its own, see
    /// `crate::menu_sub::SubMenuState::focus`).
    pub(crate) focus: CollectionState,
    /// The text direction, for the submenu open/close arrow keys and which side a submenu opens on.
    pub(crate) direction: ReadSignal<Direction>,
    /// How many submenus are open. A submenu's own item legitimately holds DOM focus while its trigger's
    /// collection looks unfocused, so [`use_open_focus_sync`] must not read that as "focus left the menu".
    pub(crate) submenu_open_count: Signal<usize>,
    /// How a pointer press on an item selects it.
    pub(crate) select_on: SelectOn,
    /// Closes the whole menu tree after a selection: clears the roving focus and closes (the Menubar also hands
    /// focus back to the menu's trigger).
    pub(crate) close: Callback<()>,
    /// A root-level item that held the roving focus lost DOM focus.
    pub(crate) blur: Callback<()>,
}

/// Keeps `open` in sync with whether `focus`'s roving-focus collection currently holds DOM focus, skipped while
/// `submenu_open_count` is nonzero (see [`MenuHost::submenu_open_count`]).
pub(crate) fn use_open_focus_sync(
    focus: CollectionState,
    open: Memo<bool>,
    set_open: Callback<bool>,
    submenu_open_count: Signal<usize>,
) {
    use_effect(move || {
        let focused = focus.any_focused();
        if *open.peek() != focused && *submenu_open_count.peek() == 0 {
            set_open.call(focused);
        }
    });
}

/// Resolves a content's own (possibly caller-overridden) element id, then gates its mount/unmount on `open`
/// through `use_animated_open`'s close-animation hold. Returns `(id, render)`; `render()` is what every call
/// site's own `if render() { ...ContentRendered { ... } }` branch keys off.
///
/// `open` is generic over anything `Readable<Target = bool>`, so this works whether a host's own `open` is the
/// menu's controlled state or a derived per-menu memo (`MenubarContent`'s `menu_ctx.is_open`).
pub(crate) fn use_menu_content_lifecycle(
    id_prop: ReadSignal<Option<String>>,
    open: impl Readable<Target = bool> + Copy + 'static,
) -> (Memo<String>, impl Fn() -> bool + Copy) {
    let unique_id = crate::use_unique_id();
    let id = crate::use_id_or(unique_id, id_prop);
    let render = crate::use_animated_open(id, open);
    (id, render)
}

/// An APG menu needs an accessible name (menu-and-menubar-pattern, "WAI-ARIA Roles, States, and Properties"):
/// labelled by its own trigger's id, unless the caller already named the content some other way
/// (`has_own_accessible_name`). Returns the `aria-labelledby` attribute as its own `merge_attributes` input
/// (present only when applicable, an empty `Vec` otherwise), never a bare literal beside `..attributes`: two
/// entries for one attribute name is the duplicate-attribute hazard `merge_attributes` exists to prevent
/// (`dev-docs/conformance-harness.md` hydration-parity Rule 4).
pub(crate) fn content_labelledby_attributes(
    attributes: &[Attribute],
    trigger_id: &str,
) -> Vec<Attribute> {
    if has_own_accessible_name(attributes) {
        Vec::new()
    } else {
        attributes!(div {
            aria_labelledby: "{trigger_id}"
        })
    }
}

/// The one definition of "a pointer press inside an open menu's content moves no DOM focus": cancel the press's
/// default action (the browser's mousedown focus change onto the pressed element or, failing that, its nearest
/// focusable ancestor) and keep the event from reaching any ancestor handler.
///
/// Every menu panel wires this as its `onpointerdown`. Two reasons, one construction:
/// - the menu must stay open across a click's pointerdown-to-pointerup gap (an item focus change would otherwise
///   blur the trigger and trip the blur-driven close before `onclick`/`on_select` fire);
/// - a menu whose items **commit on `pointerdown`** (`SelectOn::PressStart`, the Menubar) closes and refocuses
///   the trigger *inside* that same pointerdown. The mousedown that follows it would then focus the pressed item,
///   which is already being unmounted, and focus would end on `<body>` instead of the trigger (backlog row 140).
///   With the default cancelled, the refocus the commit made is the last focus change.
pub(crate) fn keep_focus_on_pointerdown(event: Event<PointerData>) {
    event.prevent_default();
    event.stop_propagation();
}

/// The navigation keys of an open menu, for one collection: ArrowDown/ArrowUp move the roving focus, Home/End jump
/// to the first/last item, and a printable character is APG's typeahead ("Move focus to the next item in the
/// current menu whose label begins with that printable character"), matched against `focus`'s own items with
/// `typeahead`'s buffer. Returns whether the key was handled, so the caller prevents its default action.
///
/// The same rule serves the root panel and every submenu: each passes its own collection and its own typeahead
/// buffer, so a search typed in a submenu never touches the enclosing menu's items.
pub(crate) fn nav_key(
    event: &Event<KeyboardData>,
    mut focus: CollectionState,
    typeahead: &mut TypeaheadState,
) -> bool {
    match event.key() {
        Key::ArrowDown => focus.focus_next(),
        Key::ArrowUp => focus.focus_prev(),
        Key::Home => focus.focus_first(),
        Key::End => focus.focus_last(),
        Key::Character(_) => {
            let entries = focus.text_entries();
            if !typeahead.handle_key(focus, &entries, event) {
                return false;
            }
        }
        _ => return false,
    }
    true
}

/// Which popover mode a panel is promoted with on the web.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    not(feature = "web"),
    allow(
        dead_code,
        reason = "only the web build promotes a panel to the top layer"
    )
)]
pub(crate) enum PanelPopover {
    /// `popover="auto"`: the platform light-dismisses on Escape and an outside press.
    Auto,
    /// `popover="manual"`: the host owns every dismissal.
    Manual,
}

/// Which stylesheet family anchors a panel to its trigger on the web.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    not(feature = "web"),
    allow(dead_code, reason = "only the web build anchors a panel")
)]
pub(crate) enum PanelKind {
    /// A dropdown menu, and every submenu of a dropdown or context menu.
    Dropdown,
    /// A menubar menu.
    Menubar,
}

/// Where a panel sits on the web (a native panel is always in flow, or fixed by the host).
#[cfg_attr(
    not(feature = "web"),
    allow(dead_code, reason = "only the web build places a panel")
)]
pub(crate) enum PanelPlacement {
    /// Anchored to the trigger, `offset` px away on `side`.
    Anchored {
        kind: PanelKind,
        side: ContentSide,
        align: ContentAlign,
        offset: u32,
    },
    /// At a document point, with no anchor element (the context menu).
    Point { x: i32, y: i32 },
}

/// Everything a host says about its panel to [`use_menu_panel`].
#[cfg_attr(
    not(feature = "web"),
    allow(dead_code, reason = "only the web build reads the popover wiring")
)]
pub(crate) struct PanelConfig {
    /// The panel element's id.
    pub(crate) id: String,
    /// Whether the panel is open.
    pub(crate) open: Memo<bool>,
    /// The id of the trigger that names the panel (`aria-labelledby`).
    pub(crate) trigger_id: String,
    /// The echo of a browser-side close or open (light dismiss, Escape, `hidePopover()`): called with the new
    /// state so the host can keep its own state in step.
    pub(crate) on_native_toggle: Callback<bool>,
    /// The popover mode.
    pub(crate) popover: PanelPopover,
    /// The placement.
    pub(crate) placement: PanelPlacement,
    /// Attributes the host owns on its panel (`dir`, `aria-orientation`, `tabindex`, ...); they win over the
    /// caller's.
    pub(crate) owned: Vec<Attribute>,
}

/// The wiring and attributes of a menu panel, shared by the root panel of every host and by every submenu.
///
/// On the web it promotes the panel to the top layer (`use_popover_sync` drives `showPopover()` / `hidePopover()`
/// from `open` and echoes the browser's own `toggle` to `on_native_toggle`, so the Rust state can never strand) and
/// places it (anchored to the trigger, or at a point). On a renderer without the popover API the panel stays in
/// flow. Either way the returned list carries the primitive-owned attributes -- `role="menu"`, the open state, the
/// accessible name from the trigger -- merged over the caller's, so a caller cannot override them
/// (`dev-docs/backlog.md` row 93's duplicate-attribute hazard), and on the web the single folded `style` of an
/// anchored panel (`top_layer::anchored_content_attributes`).
///
/// A hook: call it unconditionally, once per render, from the host's panel component, and spread the result on the
/// panel element.
pub(crate) fn use_menu_panel(config: PanelConfig, attributes: Vec<Attribute>) -> Vec<Attribute> {
    let PanelConfig {
        id,
        open,
        trigger_id,
        on_native_toggle,
        popover,
        placement,
        owned,
    } = config;

    #[cfg(feature = "web")]
    {
        crate::top_layer::use_popover_sync(id.clone(), open, on_native_toggle);
        match &placement {
            PanelPlacement::Anchored {
                side,
                align,
                offset,
                ..
            } => crate::top_layer::use_anchor_position_fallback(
                id.clone(),
                id.clone(),
                open,
                *side,
                *align,
                *offset,
            ),
            PanelPlacement::Point { x, y } => {
                crate::top_layer::use_point_anchor_placement(id.clone(), *x, *y, open);
            }
        }
    }
    #[cfg(not(feature = "web"))]
    let _ = (on_native_toggle, popover, &placement);

    let labelledby = content_labelledby_attributes(&attributes, &trigger_id);
    // Owned by the panel, so they win over a caller's own attributes.
    let panel = attributes!(div {
        role: crate::menu_semantics::MENU_ROLE,
        ..state_attrs::open(open()),
    });
    #[cfg(feature = "web")]
    let popover_value = match popover {
        PanelPopover::Auto => crate::top_layer::PopoverKind::Auto,
        PanelPopover::Manual => crate::top_layer::PopoverKind::Manual,
    }
    .as_str();
    #[cfg(feature = "web")]
    let promoted = attributes!(div {
        popover: popover_value
    });
    #[cfg(not(feature = "web"))]
    let promoted: Vec<Attribute> = Vec::new();
    let attributes = crate::merge_attributes(vec![attributes, labelledby, panel, promoted, owned]);

    // Folds the caller's own `style` together with the anchor binding into one `style` attribute.
    #[cfg(feature = "web")]
    if let PanelPlacement::Anchored { kind, .. } = placement {
        return crate::top_layer::anchored_content_attributes(
            match kind {
                PanelKind::Dropdown => crate::top_layer::AnchorKind::DropdownMenu,
                PanelKind::Menubar => crate::top_layer::AnchorKind::Menubar,
            },
            &id,
            attributes,
        );
    }
    let _ = id;
    attributes
}
