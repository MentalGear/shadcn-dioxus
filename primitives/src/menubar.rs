//! Defines the [`Menubar`] component and its sub-components.
//!
//! This file is the Menubar *host* (backlog row 435): the `menubar` row of roving triggers, each
//! [`MenubarMenu`]'s trigger and anchored panel, and the keys that move between menus. The items, the checkable
//! items and the panel wiring are the menu family every host shares (`crate::menu_item`, `crate::menu_root`),
//! reading the `MenuHost` each `MenubarMenu` provides; its items select a mouse press on `pointerdown`
//! (`SelectOn::PressStart`), because the commit closes the menu and hands focus back to the trigger inside that one
//! press, and a touch or pen press on the release, so a touch that turns into a scroll selects nothing.

use crate::state_attrs;
use dioxus::prelude::*;
use dioxus_attributes::attributes;

use crate::menu_item::{
    self, MenuCheckboxItemProps, MenuGroupProps, MenuItemProps, MenuRadioGroupProps,
    MenuRadioItemProps, MenuSeparatorProps, MenuShortcutProps, SelectOn,
};
use crate::menu_root::{self, MenuHost, PanelConfig, PanelKind, PanelPlacement, PanelPopover};
use crate::merge_attributes;
use crate::{
    collection::{
        collection_item, use_collection_provider, use_deferred_collection_focus, use_item,
        use_item_index, CollectionPlacement, CollectionState, ItemIndex,
    },
    direction::{use_direction, use_direction_signal, Direction, HorizontalNav},
    dismiss::{self, Cause, Policy, Scope},
    use_id_or, use_unique_id, ContentAlign, ContentSide,
};

#[derive(Clone, Copy)]
struct MenubarContext {
    // Currently open menu index
    open_menu: Signal<Option<usize>>,
    set_open_menu: Callback<Option<usize>>,
    disabled: ReadSignal<bool>,

    // Focus state
    focus: CollectionState,

    // Text direction, for `ArrowLeft`/`ArrowRight`'s roving-focus role
    // between top-level `MenubarMenu`s -- always horizontal (a menubar is a
    // single row) -- see `direction::Direction::resolve_horizontal`'s doc.
    direction: ReadSignal<Direction>,

    // `MenubarProps::open_on_hover` -- read by `MenubarTrigger`'s
    // `onmouseenter`, the only pointer-hover behaviour a `Menubar` has.
    open_on_hover: ReadSignal<bool>,
}

/// The props for the [`Menubar`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MenubarProps {
    /// Whether the menubar is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether focus should loop around when reaching the end.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// The text direction for `ArrowLeft`/`ArrowRight` roving focus between
    /// top-level menus. Defaults to the nearest
    /// [`crate::direction::DirectionProvider`], or LTR if there is none.
    #[props(default)]
    pub dir: Option<Direction>,

    /// Whether, once one menu is open, hovering another [`MenubarTrigger`]
    /// switches to that menu without a click. Defaults to `true` (the
    /// native menubar convention, and what Radix does unconditionally).
    /// Set it to `false` for **click activation**: moving the pointer
    /// across the triggers while a menu is open does nothing -- another
    /// menu opens only on click, `Enter`, `Space` or the arrow keys.
    ///
    /// A `Menubar` never opens a menu from a hover alone (the first menu
    /// always takes a click), so this switch only governs the
    /// "hover-switch while one is open" half. Keyboard behaviour (`ArrowLeft`/
    /// `ArrowRight` moving to and opening the neighbouring menu) is identical
    /// in both modes. Same name as Base UI's `Menu.Trigger` `openOnHover`
    /// prop, which Base UI's `Menubar` has no equivalent of.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub open_on_hover: ReadSignal<bool>,

    /// Additional attributes to apply to the menubar element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the menubar component.
    pub children: Element,
}

/// # Menubar
///
/// The `Menubar` component creates a menu bar that allows users to define multiple grouped dropdowns.
/// Each dropdown menu is represented by a [`MenubarMenu`] component with an associated trigger and content.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::menubar::{
///     Menubar, MenubarContent, MenubarItem, MenubarMenu, MenubarTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Menubar {
///             MenubarMenu {
///                 MenubarTrigger { "File" }
///                 MenubarContent {
///                     MenubarItem {
///                         value: "new".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "New"
///                     }
///                     MenubarItem {
///                         value: "open".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "Open"
///                     }
///                 }
///             }
///             MenubarMenu {
///                 MenubarTrigger { "Edit" }
///                 MenubarContent {
///                     MenubarItem {
///                         value: "cut".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "Cut"
///                     }
///                     MenubarItem {
///                         value: "copy".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "Copy"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`Menubar`] component defines the following data attributes you can use to control styling:
/// - `data-disabled`: Indicates if the menubar is disabled. Values are `true` or `false`.
/// - `data-direction`: The resolved text direction. Values are `ltr` or `rtl`.
#[component]
pub fn Menubar(props: MenubarProps) -> Element {
    let mut open_menu = use_signal(|| None);
    let set_open_menu = use_callback(move |idx| open_menu.set(idx));
    let direction = use_direction(props.dir);
    let direction_signal = use_direction_signal(direction);

    let focus = use_collection_provider(props.roving_loop);
    let mut ctx = use_context_provider(|| MenubarContext {
        open_menu,
        set_open_menu,
        disabled: props.disabled,
        focus,
        direction: direction_signal,
        open_on_hover: props.open_on_hover,
    });
    use_effect(move || {
        let index = ctx.focus.focused_index();
        if ctx.open_menu.peek().is_some() {
            ctx.set_open_menu.call(index);
        }
    });

    // Owned by this component -- role/direction/disabled/roving-focus state
    // must win over a caller's own attributes (`dev-docs/backlog.md` row 93's
    // duplicate-attribute hazard).
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(div {
            role: "menubar",
            dir: direction.as_str(),
            ..state_attrs::presence(state_attrs::Flag::Disabled, (props.disabled)()),
            "data-direction": direction.as_str(),
            tabindex: (!ctx.focus.any_focused()).then_some("0"),
        }),
    ]);

    rsx! {
        div {
            // If the menu receives focus, focus the most recently focused menu item.
            onfocus: move |_| {
                ctx.focus.set_focus(Some(ctx.focus.recent_focus_or_default()));
            },

            ..attributes,

            {props.children}
        }
    }
}

#[derive(Clone, Copy)]
struct MenubarMenuContext {
    index: ReadSignal<usize>,
    // This menu's place in the menubar row; its trigger renders the marker.
    order: ItemIndex,
    focus: CollectionState,
    is_open: Memo<bool>,
    disabled: ReadSignal<bool>,
    initial_focus: Signal<Option<CollectionPlacement>>,

    // The current `MenubarContent`'s own element id for *this* menu, kept
    // in sync by that component -- mirrors `DropdownMenuContext::content_id`
    // (`dropdown_menu.rs`). `MenubarTrigger`'s `anchor-name` must key off
    // this signal (not some menubar-wide id) so each menu's content anchors
    // to *its own* trigger, not another menu's -- see
    // `DropdownMenuContext::content_id`'s doc for the exact bug this guards
    // against if trigger and content ever named different ids.
    content_id: Signal<String>,

    // This menu's own `MenubarTrigger` element id, so `MenubarContent` can
    // label itself `aria_labelledby` from it -- dev-docs/backlog.md row 25 (an
    // APG menu requires an accessible name from either aria-labelledby or
    // aria-label; this crate's `DropdownMenu`/`ContextMenu` already do the
    // same, keyed off their own `trigger_id`). Set once by `MenubarTrigger`
    // via `use_unique_id`, read by `MenubarContentRendered`.
    trigger_id: Signal<String>,

    // Closes this menu for a `Cause` (Escape, a press outside, focus leaving): the one body the dismiss layer and
    // the trigger's Escape arm for the gap before the layer is on the page both call.
    dismiss: Callback<Cause>,
}

/// The props for the [`MenubarMenu`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MenubarMenuProps {
    /// The menu's place in the menubar's keyboard order. Leave it unset: menus are ordered as
    /// they are created, which is tree order. Set it on every menu only when one is inserted
    /// between menus that are already mounted, or a keyed list reorders them.
    #[props(default)]
    pub index: ReadSignal<Option<usize>>,

    /// Whether this menu is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Additional attributes to apply to the menu element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the menu component.
    pub children: Element,
}

/// # MenubarMenu
///
/// The `MenubarMenu` component represents a single menu within a menubar. It contains a [`MenubarTrigger`]
/// to open the menu and a [`MenubarContent`] that holds the menu items. Its `index` is optional and
/// sets the menu's place in the keyboard order (default: tree order); it does not move the menu on screen.
///
/// This must be used inside a [`Menubar`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::menubar::{
///     Menubar, MenubarContent, MenubarItem, MenubarMenu, MenubarTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Menubar {
///             MenubarMenu {
///                 MenubarTrigger { "File" }
///                 MenubarContent {
///                     MenubarItem {
///                         value: "new".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "New"
///                     }
///                     MenubarItem {
///                         value: "open".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "Open"
///                     }
///                 }
///             }
///             MenubarMenu {
///                 MenubarTrigger { "Edit" }
///                 MenubarContent {
///                     MenubarItem {
///                         value: "cut".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "Cut"
///                     }
///                     MenubarItem {
///                         value: "copy".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "Copy"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`MenubarMenu`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates if the menu is open or closed. Values are `open` or `closed`.
/// - `data-open` / `data-closed`: Present (empty) while open / closed, absent otherwise.
/// - `data-disabled`: Indicates if the menu is disabled. Values are `true` or `false`.
#[component]
pub fn MenubarMenu(props: MenubarMenuProps) -> Element {
    let mut ctx: MenubarContext = use_context();
    let order = use_item_index(ctx.focus, props.index);
    let index = order.index;
    let is_open = use_memo(move || (ctx.open_menu)() == Some(index.cloned()));
    let focus = use_collection_provider(ctx.focus.loop_signal());
    let initial_focus = use_signal(|| None);
    let disabled = move || (ctx.disabled)() || (props.disabled)();
    // Placeholder value until `MenubarContent` mounts and syncs its own id
    // in -- see `MenubarMenuContext::content_id`'s doc.
    let content_id = use_unique_id();
    // This menu's own trigger id -- see `MenubarMenuContext::trigger_id`'s doc.
    let trigger_id = use_unique_id();
    // This menu's own outer wrapping element id (the `div` `MenubarMenu` renders, enclosing both its
    // `MenubarTrigger` and `MenubarContent`): the root of the dismiss layer below, which answers "did focus leave
    // this menu" by asking the DOM directly (`dev-docs/backlog.md` row 85), instead of the signal-based checks an
    // `onblur` handler on `MenubarTrigger` or `MenubarItem` would make (neither has one).
    let root_id = menu_root::use_root_id(&props.attributes);

    // Escape closes the menu and, APG menubar: "Escape: ... sets focus to the menubar", moves focus to this menu's
    // own trigger. `ctx.focus` never stopped being this trigger's own index while keyboard focus roamed *within*
    // the open menu (`menu_ctx.focus` tracks that separately), so `set_focus` alone would be a same-value no-op that
    // never re-runs `control_mount_focus`: the roving focus is cleared first to force a real transition. A close
    // caused by acting somewhere else leaves focus where the user put it. A menu that is no longer the recorded
    // one (arrow-key navigation moved straight to a sibling) is left alone.
    let dismiss = use_callback(move |cause: Cause| {
        let mut ctx = ctx;
        if (ctx.open_menu)() != Some(index.cloned()) {
            return;
        }
        ctx.focus.clear_focus();
        ctx.set_open_menu.call(None);
        if !cause.interacted_outside() {
            ctx.focus.set_focus(Some(index.cloned()));
        }
    });

    let mut menu_ctx = use_context_provider(|| MenubarMenuContext {
        index,
        order,
        focus,
        is_open,
        disabled: props.disabled,
        initial_focus,
        content_id,
        trigger_id,
        dismiss,
    });

    // One layer for the open menu, on the menu's wrapper (its trigger and its content): the policy every menu host
    // shares (`Policy::MENU`).
    dismiss::use_dismiss(
        root_id,
        Scope::Element,
        Policy::MENU,
        Some(is_open.into()),
        move |cause| dismiss.call(cause),
    );

    use_effect(move || {
        if !is_open() {
            menu_ctx.focus.clear_focus();
            menu_ctx.initial_focus.set(None);
        }
    });

    // What the shared menu family (`menu_item`) reads from this menu. A selection closes the menu and hands focus
    // back to this menu's own trigger (APG menubar close-focus rule): `ctx.focus` never left the trigger's index
    // while item-level keyboard focus roamed this menu's own collection, so `set_focus` alone would be a
    // same-value no-op -- the roving focus is cleared first to force a real transition. The Menubar has no
    // submenus and its items take no blur action: whether focus left the menu is decided by the DOM in
    // `MenubarContentRendered`.
    let close = use_callback(move |()| {
        let mut ctx = ctx;
        ctx.set_open_menu.call(None);
        ctx.focus.clear_focus();
        ctx.focus.set_focus(Some(index.cloned()));
    });
    let blur = use_callback(|()| {});
    let submenu_open_count = use_signal(|| 0usize);
    use_context_provider(|| MenuHost {
        open: is_open,
        disabled: ctx.disabled,
        focus,
        direction: ctx.direction,
        submenu_open_count,
        select_on: SelectOn::PressStart,
        close,
        blur,
    });

    // dev-docs/backlog.md row 11 (typeahead): one buffer per
    // `MenubarMenu`, scoped to this menu's own items (`menu_ctx.focus`),
    // never the menubar's row of triggers (`ctx.focus`) -- the APG rule
    // this implements ("Move focus to the next item in the *current menu*
    // whose label begins with that printable character") is about the open
    // menu's own items, not the always-visible trigger row; see
    // `DropdownMenu`'s identical call (`dropdown_menu.rs`) for the general
    // rationale.
    let mut typeahead = crate::typeahead::use_typeahead_state();

    // Merged (caller-wins, deduped), not set-then-spread-over: see
    // `DropdownMenu`'s identical root `div` (`dropdown_menu.rs`) for the
    // full account of why an explicit `id:` literal (or `role:`, which is
    // just as reachable through `..props.attributes` -- `MenubarMenuProps`
    // has no typed field claiming either name) followed by a trailing
    // `..props.attributes` spread on the same element emits *both* into
    // the SSR'd HTML instead of the caller's override "still winning" --
    // a WHATWG duplicate-attribute parse error,
    // `dev-docs/conformance-harness.md` hydration-parity Rule 4.
    // `ContextMenu`'s own root `div` (`context_menu.rs`) merges
    // this same shape via `merge_attributes`.
    let attributes = merge_attributes(vec![
        attributes!(div {
            id: root_id.cloned(),
            role: crate::menu_semantics::MENU_ROLE,
            ..state_attrs::open(is_open()),
            ..state_attrs::presence(state_attrs::Flag::Disabled, (ctx.disabled)() || (props.disabled)()),
        }),
        props.attributes,
    ]);

    rsx! {
        div {
            onkeydown: move |event: Event<KeyboardData>| {
                match event.key() {
                    Key::Enter if !disabled() => {
                        // APG menubar: "Enter: ... opens the submenu and
                        // places focus on its first item." Route through the
                        // same open-with-focus path ArrowDown (below) uses,
                        // rather than toggling `open_menu` alone -- that would
                        // leave `initial_focus` unset, stranding focus on
                        // this trigger. See
                        // oracle/tier1-apg/keyboard-matrix.spec.ts.
                        if !is_open() {
                            menu_ctx.initial_focus.set(Some(CollectionPlacement::First));
                            ctx.set_open_menu.call(Some(index.cloned()));
                        } else {
                            ctx.set_open_menu.call(None);
                        }
                    }
                    Key::Character(c) if !disabled() && c == " " => {
                        // APG menubar (Optional): "Space: ... opens the
                        // submenu and places focus on its first item." Wired
                        // explicitly here (keydown), not left to a
                        // synthesized native-button click --
                        // `MenubarTrigger` wires only `onpointerup`,
                        // never `onclick`, so that click has no listener to
                        // act on it at all.
                        if !is_open() {
                            menu_ctx.initial_focus.set(Some(CollectionPlacement::First));
                            ctx.set_open_menu.call(Some(index.cloned()));
                        }
                    }
                    // Escape is the dismiss layer's (`Policy::MENU`); the trigger takes it for the gap before the
                    // layer is on the page (`MenubarTrigger`).
                    Key::ArrowLeft | Key::ArrowRight => {
                        match (ctx.direction)().resolve_horizontal(&event.key()) {
                            Some(HorizontalNav::Prev) => ctx.focus.focus_prev(),
                            Some(HorizontalNav::Next) => ctx.focus.focus_next(),
                            None => {}
                        }
                    }
                    Key::ArrowDown if !disabled() => {
                        if !is_open() {
                            menu_ctx.initial_focus.set(Some(CollectionPlacement::First));
                            ctx.set_open_menu.call(Some(index.cloned()));
                        } else {
                            menu_root::nav_key(&event, menu_ctx.focus, &mut typeahead);
                        }
                    },
                    Key::ArrowUp if !disabled() => {
                        if is_open() {
                            menu_root::nav_key(&event, menu_ctx.focus, &mut typeahead);
                        } else {
                            menu_ctx.initial_focus.set(Some(CollectionPlacement::Last));
                            ctx.set_open_menu.call(Some(index.cloned()));
                        }
                    },
                    Key::Home => ctx.focus.focus_first(),
                    Key::End => ctx.focus.focus_last(),
                    // APG Menu and Menubar pattern, "Keyboard Interaction"
                    // (h2), same pinned commit as this file's other APG
                    // citations: "Any key that corresponds to a printable
                    // character (Optional): Move focus to the next item in
                    // the current menu whose label begins with that
                    // printable character." Only while this menu is open
                    // and not disabled, matching the ArrowDown/ArrowUp arms
                    // just above -- `menu_ctx.focus` has no items to search
                    // while closed (`MenubarContent` mounts items only
                    // once `render()` is true).
                    Key::Character(_) if !disabled() && is_open() => {
                        if !menu_root::nav_key(&event, menu_ctx.focus, &mut typeahead) {
                            return;
                        }
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

/// The props for the [`MenubarTrigger`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MenubarTriggerProps {
    /// The ID of the trigger button. If not provided, an internally
    /// generated ID is used -- see `MenubarTrigger`'s use of `use_id_or`
    /// for why this is a typed field rather than left to
    /// `GlobalAttributes` alone (dev-docs/backlog.md row 36).
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes to apply to the trigger element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the trigger component.
    pub children: Element,
}

/// # MenubarTrigger
///
/// The `MenubarTrigger` component is a button that opens and closes a [`MenubarMenu`] when clicked.
///
/// This must be used inside a [`MenubarMenu`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::menubar::{
///     Menubar, MenubarContent, MenubarItem, MenubarMenu, MenubarTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Menubar {
///             MenubarMenu {
///                 MenubarTrigger { "File" }
///                 MenubarContent {
///                     MenubarItem {
///                         value: "new".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "New"
///                     }
///                     MenubarItem {
///                         value: "open".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "Open"
///                     }
///                 }
///             }
///             MenubarMenu {
///                 MenubarTrigger { "Edit" }
///                 MenubarContent {
///                     MenubarItem {
///                         value: "cut".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "Cut"
///                     }
///                     MenubarItem {
///                         value: "copy".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "Copy"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn MenubarTrigger(props: MenubarTriggerProps) -> Element {
    let mut ctx: MenubarContext = use_context();
    let menu_ctx: MenubarMenuContext = use_context();
    let disabled = move || (ctx.disabled)() || (menu_ctx.disabled)();
    // The menu's trigger is the focusable element, so it registers the menu in
    // the parent menubar collection.
    let item = use_item(collection_item(ctx.focus, menu_ctx.index).disabled(disabled));
    let onmounted = item.onmounted();
    let is_open = menu_ctx.is_open;
    let index = menu_ctx.index;
    let is_focused = move || item.focused() && !menu_ctx.focus.any_focused();

    // dev-docs/backlog.md row 36: `menu_ctx.trigger_id` is the same signal
    // `MenubarContent`'s `aria-labelledby` reads back (row 25, below) -- a
    // caller-supplied `id` here must resolve into it, not just win the
    // render. Feeding it straight into `use_id_or` as the generated-id
    // signal does that: `use_id_or`'s own effect writes a caller override
    // back into its `gen_id` argument, which *is* `menu_ctx.trigger_id`
    // here -- mirrors `DialogTitle`'s identical
    // `use_id_or(ctx.dialog_labelledby, props.id)` (`dialog.rs`).
    let id = use_id_or(menu_ctx.trigger_id, props.id);

    // Merged (caller-wins, deduped) rather than a plain explicit-attribute +
    // `..props.attributes` spread: `id` below and a caller-supplied `id` in
    // `props.attributes` would otherwise both render, the same
    // duplicate-attribute hydration hazard `merge_attributes`'s own doc
    // warns about (dev-docs/conformance-harness.md Rule 4).
    let attributes = merge_attributes(vec![
        attributes!(button {
            // This menu's own trigger id, so `MenubarContent` can label
            // itself `aria_labelledby` from it -- dev-docs/backlog.md row 25.
            id: id.cloned(),
        }),
        props.attributes,
    ]);
    // Ties this trigger to the web-arm content's `position-anchor`
    // (`MenubarContentRendered`) so its anchor-positioned placement
    // resolves relative to *this* trigger once promoted to the top layer --
    // keyed on `menu_ctx.content_id`, not this menu's own index, for the
    // same reason `DropdownMenuTrigger` keys off `ctx.content_id` (see that
    // doc). It is folded into `attributes` here via
    // `top_layer::anchored_trigger_attributes`, not written as a bare
    // `style: anchor_name_style(..)` literal on this element beside
    // `..attributes` below: a caller supplying their own `style` would then
    // produce two `style` attributes on this same tag
    // (`dev-docs/conformance-harness.md` hydration-parity Rule 4;
    // `scripts/check-attr-spread-collision.sh`). Folding it (unlike plain
    // `merge_attributes`, which folds a plain
    // `style` attribute but leaves shorthand style props -- `padding:
    // ".."`, namespace `Some("style")` -- as separate attributes) renders
    // exactly one `style` carrying the anchor binding plus whatever the
    // caller supplied; see that function's own doc.
    let attributes =
        crate::top_layer::anchored_trigger_attributes(&menu_ctx.content_id.cloned(), attributes);
    // Owned by this component -- role/type/roving-focus state must win
    // over a caller's own attributes (`dev-docs/backlog.md` row 93's
    // duplicate-attribute hazard).
    let attributes = merge_attributes(vec![
        attributes,
        menu_ctx.order.marker_attribute(),
        attributes!(button {
            role: crate::menu_semantics::MENU_ITEM_ROLE,
            type: "button",
            tabindex: if is_focused() { "0" } else { "-1" },
        }),
    ]);

    rsx! {
        button {
            onmounted,
            // The layer registers a moment after the menu opens; an Escape in that gap is the trigger's, which still
            // holds focus (`dismiss::escape_in_gap`).
            onkeydown: move |event: Event<KeyboardData>| {
                dismiss::escape_in_gap(&event, is_open(), || {
                    menu_ctx.dismiss.call(Cause::EscapeWithin)
                });
            },
            onpointerup: move |_| {
                if !disabled() {
                    let new_open = if is_open() { None } else { Some(index.cloned()) };
                    ctx.set_open_menu.call(new_open);
                    ctx.focus.set_focus(Some(index.cloned()));
                }
            },
            // Hover-switch between menus once one is open -- off when
            // `MenubarProps::open_on_hover` is `false` (click activation).
            onmouseenter: move |_| {
                if (ctx.open_on_hover)() && !disabled() && (ctx.open_menu)().is_some() {
                    ctx.focus.set_focus(Some(index.cloned()));
                }
            },
            // No `onblur` here (`dev-docs/backlog.md` row 85): closing this
            // menu on blur would synchronously read the row's own roving-
            // focus signal (`item.focused() && !menu_ctx.focus.any_focused()`),
            // which is current for every focus move this crate's own
            // click/hover/keyboard code drives (all of which update
            // `ctx.focus`/`menu_ctx.focus` *before* moving DOM focus) but
            // stale for a focus move arriving any other way: arrow-down into
            // a `MenubarItem` inside this menu's open content (so it
            // genuinely holds DOM focus), then a raw `.focus()` call
            // directly on this trigger, would close the content immediately.
            // `MenubarMenu`'s dismiss layer on `root_id` gives this
            // menu the DOM-truth-based construction `DropdownMenu`/
            // `ContextMenu` use (see `DropdownMenuTrigger`'s comment in
            // `dropdown_menu.rs` for the root cause).
            ..attributes,
            {props.children}
        }
    }
}

/// The props for the [`MenubarContent`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MenubarContentProps {
    /// The id of the content element.
    pub id: ReadSignal<Option<String>>,
    /// Additional attributes to apply to the content element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the content component.
    pub children: Element,
}

/// # MenubarContent
///
/// The `MenubarContent` component defines the content of a [`MenubarMenu`]. It will only be rendered if the menu is open.
///
/// This must be used inside a [`MenubarMenu`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::menubar::{
///     Menubar, MenubarContent, MenubarItem, MenubarMenu, MenubarTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Menubar {
///             MenubarMenu {
///                 MenubarTrigger { "File" }
///                 MenubarContent {
///                     MenubarItem {
///                         value: "new".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "New"
///                     }
///                     MenubarItem {
///                         value: "open".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "Open"
///                     }
///                 }
///             }
///             MenubarMenu {
///                 MenubarTrigger { "Edit" }
///                 MenubarContent {
///                     MenubarItem {
///                         value: "cut".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "Cut"
///                     }
///                     MenubarItem {
///                         value: "copy".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "Copy"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`MenubarContent`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates if the menu is open or closed. Values are `open` or `closed`.
/// - `data-open` / `data-closed`: Present (empty) while open / closed, absent otherwise.
#[component]
pub fn MenubarContent(props: MenubarContentProps) -> Element {
    let mut menu_ctx: MenubarMenuContext = use_context();

    // dev-docs/backlog.md row 53 (the "content lifecycle/positioning pipeline"
    // band): id resolution + `use_animated_open`'s render gate, identical
    // across all three menu-family hosts' `*Content` components -- see
    // `menu_root::use_menu_content_lifecycle`'s own doc.
    let (id, render) = crate::menu_root::use_menu_content_lifecycle(props.id, menu_ctx.is_open);

    // Keep `menu_ctx.content_id` in sync with this content's actual id --
    // see `MenubarMenuContext::content_id`'s doc. Mirrors
    // `DropdownMenuContent`'s identical `ctx.content_id.set(id())`
    // (`dropdown_menu.rs`).
    use_effect(move || menu_ctx.content_id.set(id()));

    use_deferred_collection_focus(menu_ctx.focus, menu_ctx.initial_focus, render);

    rsx! {
        if render() {
            MenubarContentRendered {
                id: id.cloned(),
                attributes: props.attributes,
                children: props.children,
            }
        }
    }
}

/// The menu panel. On the web it is promoted to the top layer via `popover="auto"`, anchored to its own trigger --
/// the same mechanism `DropdownMenuContentRendered` uses (`dropdown_menu.rs`), which see for the general
/// `popover`/CSS-anchor wiring. Without the popover API it is a plain, always-in-flow `div`.
///
/// `auto`: each `MenubarMenu` has exactly the shape `DropdownMenu` does -- a trigger button plus content anchored
/// to it -- and `auto`'s native light dismiss (Escape, outside pointerdown) is free, spec-correct insurance beside
/// this crate's own DOM-truth dismissal, not a competing mechanism.
///
/// Escape is the dismiss layer's `MenubarMenu` registers (`Policy::MENU`, which cancels the key so the popover's own
/// close does not also run): APG requires Escape to move focus to *this specific menu's own trigger*
/// (`ctx.focus.set_focus(Some(index))`), which native light dismiss has no way to know how to do. There is exactly
/// one trigger for a `DropdownMenu`, so "return focus to the trigger" can be deferred there; here it cannot.
///
/// The close echo of the native `toggle` and the outside dismissal both act *only when this menu is still the one
/// recorded as open*. `Menubar`'s effect can move `ctx.open_menu` straight from `Some(this index)` to
/// `Some(another index)` (arrow-key navigation between triggers while a menu is open), and showing the new menu's
/// `auto` popover natively closes this now-unrelated sibling, firing the `toggle` callback with `false` *after*
/// `ctx.open_menu` already points at the other menu. An unconditional `set_open_menu(None)` would then close the
/// menu the user just switched to.
///
/// The panel is anchored below its trigger, start-aligned, 8px away (`side`/`align` and the gap match the themed
/// CSS's `top: 100%; left: 0; margin-top: 0.5rem`).
#[component]
fn MenubarContentRendered(
    /// The DOM id of the menu's content element.
    id: String,
    /// Extra HTML attributes for the menu's content element.
    attributes: Vec<Attribute>,
    /// The menu items.
    children: Element,
) -> Element {
    let ctx: MenubarContext = use_context();
    let menu_ctx: MenubarMenuContext = use_context();
    let index = menu_ctx.index;

    let on_native_toggle = use_callback(move |is_open: bool| {
        if !is_open && (ctx.open_menu)() == Some(index.cloned()) {
            ctx.set_open_menu.call(None);
        }
    });
    let attributes = menu_root::use_menu_panel(
        PanelConfig {
            id: id.clone(),
            open: menu_ctx.is_open,
            trigger_id: menu_ctx.trigger_id.cloned(),
            on_native_toggle,
            popover: PanelPopover::Auto,
            placement: PanelPlacement::Anchored {
                kind: PanelKind::Menubar,
                side: ContentSide::Bottom,
                align: ContentAlign::Start,
                offset: 8,
            },
            owned: Vec::new(),
        },
        attributes,
    );
    rsx! {
        div {
            id,
            // A mouse press on an item commits on `pointerdown` and closes the menu right there; the mousedown that follows must not
            // focus the pressed (about to unmount) item, or focus ends on `<body>` instead of the trigger the
            // commit just refocused (dev-docs/backlog.md row 140) -- see `menu_root::keep_focus_on_pointerdown`.
            onpointerdown: menu_root::keep_focus_on_pointerdown,
            ..attributes,
            {children}
        }
    }
}

/// The props for the [`MenubarItem`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuItemProps`] (a `String`
/// value here).
pub type MenubarItemProps = MenuItemProps<String>;

/// # MenubarItem
///
/// The `MenubarItem` component represents a selectable item within a menu. In addition to calling the
/// [`MenubarItemProps::on_select`] callback, the menu will close when the item is selected.
///
/// This must be used inside a [`MenubarContent`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::menubar::{
///     Menubar, MenubarContent, MenubarItem, MenubarMenu, MenubarTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Menubar {
///             MenubarMenu {
///                 MenubarTrigger { "File" }
///                 MenubarContent {
///                     MenubarItem {
///                         value: "new".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "New"
///                     }
///                     MenubarItem {
///                         value: "open".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "Open"
///                     }
///                 }
///             }
///             MenubarMenu {
///                 MenubarTrigger { "Edit" }
///                 MenubarContent {
///                     MenubarItem {
///                         value: "cut".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "Cut"
///                     }
///                     MenubarItem {
///                         value: "copy".to_string(),
///                         on_select: move |value| {
///                             tracing::info!("Selected value: {}", value);
///                         },
///                         "Copy"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`MenubarItem`] component defines the following data attributes you can use to control styling:
/// - `data-disabled`: Indicates if the item is disabled. Values are `true` or `false`.
/// - `data-highlighted`: Present (empty) while this item holds the roving focus, absent otherwise.
#[component]
pub fn MenubarItem(props: MenubarItemProps) -> Element {
    menu_item::render_item(props, "MenubarItem")
}

/// The props for the [`MenubarCheckboxItem`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuCheckboxItemProps`].
pub type MenubarCheckboxItemProps = MenuCheckboxItemProps;

/// The props for the [`MenubarRadioGroup`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuRadioGroupProps`].
pub type MenubarRadioGroupProps = MenuRadioGroupProps;

/// The props for the [`MenubarRadioItem`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuRadioItemProps`].
pub type MenubarRadioItemProps = MenuRadioItemProps;

/// # MenubarCheckboxItem
///
/// An item within a [`MenubarContent`] that can be checked and unchecked
/// independently of its siblings -- shadcn's `MenubarCheckboxItem`. Renders
/// `role="menuitemcheckbox"` with an always-present `aria-checked` (see
/// `crate::menu_semantics`).
///
/// Controlled with `checked` + `on_checked_change`, or uncontrolled with
/// `default_checked`. Pressing it with the pointer, or pressing Enter or
/// Space while it has focus, toggles it. Like Base UI's `closeOnClick =
/// false`, that **keeps the menu open** so several items can be toggled in one
/// visit (APG: "changes the state without closing the menu"); set
/// `close_on_select: true` for Radix's `onSelect` behaviour, which closes it.
/// Disabled items cannot be toggled and are skipped by arrow keys and
/// typeahead. A checkbox item has no `value`, so give it a `text_value` to
/// make it a typeahead target.
///
/// This must be used inside a [`MenubarContent`] component.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::menubar::{
///     Menubar, MenubarCheckboxItem, MenubarContent, MenubarMenu, MenubarTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     let mut auto_save = use_signal(|| true);
///     rsx! {
///         Menubar {
///             MenubarMenu {
///                 MenubarTrigger { "File" }
///                 MenubarContent {
///                     MenubarCheckboxItem {
///                         checked: Some(auto_save()),
///                         on_checked_change: move |checked| auto_save.set(checked),
///                         text_value: "Auto Save",
///                         "Auto Save"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`MenubarCheckboxItem`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Whether the item is checked. Values are `checked` or `unchecked`.
/// - `data-checked` / `data-unchecked`: Present (empty) while checked / unchecked, absent otherwise.
/// - `data-disabled`: Indicates if the item is disabled. Values are `true` or `false`.
/// - `data-highlighted`: Present (empty) while this item holds the roving focus, absent otherwise.
#[component]
pub fn MenubarCheckboxItem(props: MenubarCheckboxItemProps) -> Element {
    let state = menu_item::use_checkbox_state(
        props.checked,
        props.default_checked,
        props.on_checked_change,
    );
    menu_item::render_checkable(state, props.into())
}

/// # MenubarRadioGroup
///
/// A set of mutually exclusive [`MenubarRadioItem`]s -- shadcn's
/// `MenubarRadioGroup`. Renders `role="group"`; give it an accessible name
/// with `aria-label` or `aria-labelledby`. Controlled with `value` +
/// `on_value_change`, or uncontrolled with `default_value`.
///
/// This must be used inside a [`MenubarContent`] component.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::menubar::{
///     Menubar, MenubarContent, MenubarMenu, MenubarRadioGroup, MenubarRadioItem,
///     MenubarTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     let mut profile = use_signal(|| "benoit".to_string());
///     rsx! {
///         Menubar {
///             MenubarMenu {
///                 MenubarTrigger { "Profiles" }
///                 MenubarContent {
///                     MenubarRadioGroup {
///                         aria_label: "Profile",
///                         value: Some(profile()),
///                         on_value_change: move |value| profile.set(value),
///                         MenubarRadioItem {
///                             value: "andy".to_string(),
///                             "Andy"
///                         }
///                         MenubarRadioItem {
///                             value: "benoit".to_string(),
///                             "Benoit"
///                         }
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn MenubarRadioGroup(props: MenubarRadioGroupProps) -> Element {
    menu_item::render_radio_group(props)
}

/// # MenubarRadioItem
///
/// One choice in a [`MenubarRadioGroup`] -- shadcn's `MenubarRadioItem`.
/// Renders `role="menuitemradio"` with an always-present `aria-checked`; it
/// is checked while the group's value equals its own `value`, and choosing
/// it (pointer, Enter or Space) sets the group's value and, like Base UI's
/// `closeOnClick = false`, **keeps the menu open** unless `close_on_select` is
/// `true`. Falls back to `value` as its typeahead label. Disabled items
/// cannot be chosen and are skipped by arrow keys and typeahead.
///
/// This must be used inside a [`MenubarRadioGroup`] component.
///
/// ## Styling
///
/// The [`MenubarRadioItem`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Whether the item is checked. Values are `checked` or `unchecked`.
/// - `data-checked` / `data-unchecked`: Present (empty) while checked / unchecked, absent otherwise.
/// - `data-disabled`: Indicates if the item is disabled. Values are `true` or `false`.
/// - `data-highlighted`: Present (empty) while this item holds the roving focus, absent otherwise.
#[component]
pub fn MenubarRadioItem(props: MenubarRadioItemProps) -> Element {
    let state = menu_item::use_radio_state(props.value);
    menu_item::render_checkable(state, props.into())
}

/// The props for the [`MenubarSeparator`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuSeparatorProps`].
pub type MenubarSeparatorProps = MenuSeparatorProps;

/// The props for the [`MenubarGroup`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuGroupProps`].
pub type MenubarGroupProps = MenuGroupProps;

/// The props for the [`MenubarShortcut`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuShortcutProps`].
pub type MenubarShortcutProps = MenuShortcutProps;

/// # MenubarSeparator
///
/// A divider between groups of items -- shadcn's `MenubarSeparator`. Renders
/// `role="separator"` with `aria-orientation="horizontal"`. It is not
/// focusable and takes no `index`: arrow keys, Home/End and typeahead skip
/// it.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::menubar::{MenubarSeparator};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         MenubarSeparator {}
///     }
/// }
/// ```
#[component]
pub fn MenubarSeparator(props: MenubarSeparatorProps) -> Element {
    menu_item::render_separator(props)
}

/// # MenubarGroup
///
/// A `role="group"` around related items -- shadcn's `MenubarGroup`. It adds
/// no behaviour: its items still take part in the enclosing menu's keyboard
/// navigation and typeahead. Name it with `aria-label` when it has a visible
/// heading (a label).
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::menubar::{MenubarGroup};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         MenubarGroup { aria_label: "File", "..." }
///     }
/// }
/// ```
#[component]
pub fn MenubarGroup(props: MenubarGroupProps) -> Element {
    menu_item::render_group(props)
}

/// # MenubarShortcut
///
/// A keyboard-shortcut hint at the inline end of an item -- shadcn's
/// `MenubarShortcut`. Text only: it renders a `span` and does not bind any
/// key.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::menubar::{MenubarShortcut};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         MenubarShortcut { "⌘T" }
///     }
/// }
/// ```
#[component]
pub fn MenubarShortcut(props: MenubarShortcutProps) -> Element {
    menu_item::render_shortcut(props)
}

/// `dev-docs/backlog.md` row 85 / hydration-parity Rule 4 coverage:
/// proves `MenubarMenu`'s own wrapping `div` renders a caller-supplied
/// `id` exactly once, with no second, internally-generated `dxc-N` id
/// also present -- the identical contract `dropdown_menu.rs`'s own
/// `ssr_tests::callers_own_id_on_the_root_is_not_duplicated` proves for
/// `DropdownMenu`'s root. Without routing this element's attributes through
/// `merge_attributes` both the generated id and `id="my-menu-root"` would be
/// present (2 total `id="` occurrences).
#[cfg(test)]
mod ssr_tests {
    use super::*;

    #[component]
    fn MenubarWithOwnMenuId() -> Element {
        rsx! {
            Menubar {
                MenubarMenu { id: "my-menu-root", "content" }
            }
        }
    }

    fn render(component: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(component);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn callers_own_id_on_the_menu_wrapper_is_not_duplicated() {
        let html = render(MenubarWithOwnMenuId);
        assert_eq!(
            html.matches(" id=\"").count(),
            1,
            "expected exactly one `id` attribute on the menu wrapper, got: {html}"
        );
        assert!(html.contains(r#"id="my-menu-root""#), "html: {html}");
        assert!(
            !html.contains("dxc-"),
            "the internally-generated id must not also render: {html}"
        );
    }
}
