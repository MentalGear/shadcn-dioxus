//! Defines the [`Menubar`] component and its sub-components.

use crate::state_attrs;
use dioxus::prelude::*;
use dioxus_attributes::attributes;

use crate::menu_item::{
    self, CheckableCommon, CheckableState, MenuCheckboxItemProps, MenuGroupProps, MenuItemVariant,
    MenuRadioGroupProps, MenuRadioItemProps, MenuSeparatorProps, MenuShortcutProps,
};
use crate::merge_attributes;
use crate::{
    collection::{
        collection_item, use_collection_provider, use_deferred_collection_focus, use_item,
        use_item_index, CollectionPlacement, CollectionState, ItemIndex,
    },
    direction::{use_direction, use_direction_signal, Direction, HorizontalNav},
    use_id_or, use_outside_dismiss, use_unique_id,
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
            ..state_attrs::disabled((props.disabled)()),
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

    // This menu's own outer wrapping element id (set on the `div`
    // `MenubarMenu` itself renders, the one enclosing both its
    // `MenubarTrigger` and `MenubarContent`) -- mirrors
    // `ContextMenuCtx::root_id`/`DropdownMenuContext::root_id`
    // (`context_menu.rs`/`dropdown_menu.rs`) exactly, for
    // `dev-docs/backlog.md` row 85: `MenubarContentRendered`'s own
    // `use_outside_dismiss(menu_ctx.root_id, ...)` call answers "did focus
    // leave this menu" by asking the DOM directly, instead of the
    // signal-based checks an `onblur` handler on `MenubarTrigger` or
    // `MenubarItem` would make (neither has one -- see each one's own note).
    root_id: Signal<String>,

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
}

impl MenubarMenuContext {
    fn focus_next(&mut self) {
        self.focus.focus_next();
    }

    fn focus_prev(&mut self) {
        self.focus.focus_prev();
    }
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
    // This menu's own outer wrapping element id -- see
    // `MenubarMenuContext::root_id`'s doc.
    let root_id = use_unique_id();

    let mut menu_ctx = use_context_provider(|| MenubarMenuContext {
        index,
        order,
        focus,
        is_open,
        disabled: props.disabled,
        initial_focus,
        content_id,
        trigger_id,
        root_id,
    });

    use_effect(move || {
        if !is_open() {
            menu_ctx.focus.clear_focus();
            menu_ctx.initial_focus.set(None);
        }
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
            ..state_attrs::disabled((ctx.disabled)() || (props.disabled)()),
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
                    Key::Escape => {
                        ctx.set_open_menu.call(None);
                        // APG menubar: "Escape: ... sets focus to the
                        // menubar." Use the existing collection-focus
                        // mechanism (not the shared refocus-on-close hook --
                        // see dev-docs/plan.md Phase 3.1) to move focus back to
                        // this menu's own trigger item. `ctx.focus` never
                        // stopped being this trigger's own index while
                        // keyboard focus roamed *within* the open submenu
                        // (`menu_ctx.focus` tracks that separately), so
                        // `set_focus` alone would be a same-value no-op that
                        // never re-runs `control_mount_focus` -- clear first
                        // to force a real transition.
                        ctx.focus.clear_focus();
                        ctx.focus.set_focus(Some(index.cloned()));
                    }
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
                            menu_ctx.focus_next();
                        }
                    },
                    Key::ArrowUp if !disabled() => {
                        if is_open() {
                            menu_ctx.focus_prev();
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
                        let entries = menu_ctx.focus.text_entries();
                        if !typeahead.handle_key(menu_ctx.focus, &entries, &event) {
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
            // `MenubarContentRendered`'s own `use_outside_dismiss(menu_ctx.
            // root_id, ...)` call gives this menu the DOM-truth-based
            // construction `DropdownMenu`/`ContextMenu` use (see
            // `DropdownMenuTrigger`'s comment in `dropdown_menu.rs` for the
            // root cause).
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

/// Web arm: promote each menu's content to the top
/// layer via `popover="auto"`, anchored to its own trigger -- the same
/// mechanism `DropdownMenuContentRendered` uses (`dropdown_menu.rs`), which
/// see for the general `popover`/CSS-anchor wiring this mirrors.
///
/// `auto`, not `manual`: unlike `ContextMenu` (point-opened, no persistent
/// trigger to key dismissal off of), each `MenubarMenu` already has exactly
/// the shape `DropdownMenu` does -- a trigger button plus content anchored
/// to it -- and `auto`'s native light dismiss (Escape, outside pointerdown)
/// is free, spec-correct insurance alongside this crate's own blur-driven
/// close (`MenubarTrigger`/`MenubarItem`'s `onblur` below), not a
/// competing mechanism: closing this content -- by any means, native or
/// Rust-driven -- makes the previously-focused item within it not
/// focusable (`[popover]:not(:popover-open) { display: none }`), which the
/// UA blurs synchronously as part of hiding it, so the *existing*
/// `onblur`-driven close-and-focus-clear logic still fires and still owns
/// what happens next; native light dismiss only ever supplies the same
/// "something outside happened" signal blur already does, for the rare
/// outside interaction that would not otherwise cause a blur (see the
/// `use_popover_sync` callback below).
///
/// Escape is handled entirely by `MenubarMenu`'s own `onkeydown`
/// (which calls `prevent_default()` unconditionally): APG
/// requires Escape to move focus to *this specific menu's own trigger*
/// (`ctx.focus.set_focus(Some(index))`), which native light dismiss
/// has no way to know how to do (it has no notion of "which trigger opened
/// this"), so this component keeps driving that close+refocus itself
/// rather than deferring to the platform the way `DropdownMenuContent`
/// does -- there is exactly one trigger for `DropdownMenu`, so "return
/// focus to the trigger" needs no such per-index bookkeeping and can be
/// deferred; here it cannot, so `prevent_default()` staying unconditional
/// (no compile-time wasm skip, unlike `DropdownMenu`'s Escape arm) is the
/// correct choice, not an oversight.
#[cfg(feature = "web")]
#[component]
fn MenubarContentRendered(
    /// The DOM id of the menu's content element.
    id: String,
    /// Extra HTML attributes for the menu's content element.
    attributes: Vec<Attribute>,
    /// The menu items.
    children: Element,
) -> Element {
    let mut ctx: MenubarContext = use_context();
    let menu_ctx: MenubarMenuContext = use_context();
    let open = menu_ctx.is_open;
    let index = menu_ctx.index;

    // Drive `showPopover()`/`hidePopover()` from `open`, and sync a native
    // close back into `ctx.open_menu` -- but *only* when this menu is still
    // the one recorded as open. Necessary, not defensive
    // boilerplate: `Menubar`'s own effect
    // (`Menubar`'s `use_effect` reacting to `ctx.focus.focused_index()`)
    // can move `ctx.open_menu` straight from `Some(this index)` to
    // `Some(another index)` in one step (arrow-key navigation between
    // triggers while a menu is open) -- and per WHATWG HTML, showing the
    // *new* menu's `auto` popover natively closes this (now-unrelated)
    // sibling `auto` popover for us, firing this same `toggle` callback
    // with `is_open: false` *after* `ctx.open_menu` already points at the
    // other menu. An unconditional `ctx.set_open_menu.call(None)` here
    // would then wrongly close the menu the user just switched to. Guarding
    // on "is `ctx.open_menu` still `Some(this index)`" makes the callback a
    // no-op in that case, while still correctly closing on every close that
    // *is* this menu's own (Escape, item-select, an actual outside
    // interaction) -- all of which either already set `ctx.open_menu` to
    // `None` themselves (making this call idempotent) or never touched it
    // (the rare native-only outside-close case this sync exists for).
    crate::top_layer::use_popover_sync(
        id.clone(),
        open,
        Callback::new(move |is_open: bool| {
            if !is_open && (ctx.open_menu)() == Some(index.cloned()) {
                ctx.set_open_menu.call(None);
            }
        }),
    );
    // `dev-docs/backlog.md` row 85: "did focus leave this menu" is never
    // decided by an `onblur` on `MenubarTrigger` or `MenubarItem` that
    // synchronously checks a Rust-tracked roving-focus signal
    // (`menu_ctx.focus.any_focused()`/`ctx.focus`) -- the defect
    // `DropdownMenuTrigger` documents (see its comment in
    // `dropdown_menu.rs` for the root cause), most directly reproducible
    // here: arrow-down into a `MenubarItem`
    // (so it genuinely holds DOM focus), then a raw `.focus()` call
    // directly on `MenubarTrigger` would close this content immediately,
    // with no mouse-opened precondition needed at all.
    // `use_outside_dismiss` asks the DOM directly instead
    // (`root_id` covers this menu's own trigger *and* this content, since
    // `MenubarMenu` wraps both in one element) -- same guard as the
    // `use_popover_sync` callback just above, for the same race (arrow-key
    // hover-switching between triggers can move `ctx.open_menu` to another
    // index while this content is still mid-close-animation).
    use_outside_dismiss(menu_ctx.root_id, move || {
        if (ctx.open_menu)() == Some(index.cloned()) {
            ctx.focus.clear_focus();
            ctx.set_open_menu.call(None);
        }
    });
    // JS-measured static positioning fallback for engines without CSS
    // Anchor Positioning -- see `top_layer::use_anchor_position_fallback`'s
    // doc. `side`/`align` and the 8px gap match this menu's themed
    // CSS (`../menubar/style.css`'s `top: 100%; left: 0; margin-top:
    // 0.5rem` -- 0.5rem == 8px at the default root font size): anchored
    // below the trigger, left-aligned with it, matching
    // `DropdownMenuContentRendered`'s identical bottom/start choice for the
    // same visual shape.
    crate::top_layer::use_anchor_position_fallback(
        id.clone(),
        id.clone(),
        open,
        crate::ContentSide::Bottom,
        crate::ContentAlign::Start,
        8,
    );
    // dev-docs/backlog.md row 25: an APG menu requires an accessible name from
    // either aria-labelledby or aria-label -- labelled by this menu's own
    // trigger, mirroring `DropdownMenuContent`'s identical pattern
    // (`dropdown_menu.rs`). Only contributed when the caller hasn't already
    // named this content some other way (`has_own_accessible_name`'s own
    // doc), and as its own `merge_attributes` input -- present only when
    // applicable -- rather than a bare `aria_labelledby: ...` literal
    // alongside `..attributes`: a caller attribute list can carry a
    // same-named `aria-labelledby`/`aria-label` with an
    // empty/`AttributeValue::None` value, and two entries for one
    // attribute name is exactly the duplicate-attribute hazard
    // `merge_attributes` exists to prevent (`dev-docs/conformance-harness.md`
    // hydration-parity Rule 4). dev-docs/backlog.md row 53 (the "trigger
    // id-plumbing" band): this block is
    // `menu_root::content_labelledby_attributes`, shared with
    // `DropdownMenu`/`ContextMenu`'s identical construction.
    let labelledby =
        crate::menu_root::content_labelledby_attributes(&attributes, &menu_ctx.trigger_id.cloned());
    let attributes = merge_attributes(vec![
        attributes,
        labelledby,
        // Owned by this component -- menu semantics + top-layer wiring
        // must win over a caller's own attributes (`dev-docs/backlog.md` row
        // 93's duplicate-attribute hazard).
        attributes!(div {
            role: crate::menu_semantics::MENU_ROLE,
            popover: crate::top_layer::PopoverKind::Auto.as_str(),
            ..state_attrs::open(open()),
        }),
    ]);
    // Folds the caller's own `style` together with the anchor binding into
    // one `style` attribute -- see `top_layer::anchored_content_attributes`'s
    // doc for why a bare `style: position_anchor_style(&id)` literal
    // alongside `..attributes` is the duplicate-`style` hazard
    // (`dev-docs/conformance-harness.md` hydration-parity Rule 4).
    let attributes = crate::top_layer::anchored_content_attributes(
        crate::top_layer::AnchorKind::Menubar,
        &id,
        attributes,
    );

    rsx! {
        div {
            id: id.clone(),
            // Items commit on `pointerdown` and close the menu right there;
            // the mousedown that follows must not focus the pressed (about
            // to unmount) item, or focus ends on `<body>` instead of the
            // trigger the commit just refocused (dev-docs/backlog.md row 140) --
            // see `menu_root::keep_focus_on_pointerdown`.
            onpointerdown: crate::menu_root::keep_focus_on_pointerdown,
            ..attributes,
            {children}
        }
    }
}

/// Native (Blitz) arm: Blitz has no popover-API support at all, so this is
/// the functional floor, a plain, always-in-flow `div`.
#[cfg(not(feature = "web"))]
#[component]
fn MenubarContentRendered(
    /// The DOM id of the menu's content element.
    id: String,
    /// Extra HTML attributes for the menu's content element.
    attributes: Vec<Attribute>,
    /// The menu items.
    children: Element,
) -> Element {
    let mut ctx: MenubarContext = use_context();
    let menu_ctx: MenubarMenuContext = use_context();
    let index = menu_ctx.index;

    // `dev-docs/backlog.md` row 85 -- see the web arm's identical call above
    // for the full construction.
    use_outside_dismiss(menu_ctx.root_id, move || {
        if (ctx.open_menu)() == Some(index.cloned()) {
            ctx.focus.clear_focus();
            ctx.set_open_menu.call(None);
        }
    });

    // See the web arm's identical construction above (dev-docs/backlog.md row
    // 25/53) for why this is conditional and routed through
    // `menu_root::content_labelledby_attributes` rather than a bare literal
    // alongside `..attributes`.
    let labelledby =
        crate::menu_root::content_labelledby_attributes(&attributes, &menu_ctx.trigger_id.cloned());
    let attributes = merge_attributes(vec![
        attributes,
        labelledby,
        // Owned by this component -- see the web arm's identical
        // construction above (`dev-docs/backlog.md` row 93).
        attributes!(div {
            role: crate::menu_semantics::MENU_ROLE,
            ..state_attrs::open((menu_ctx.is_open)()),
        }),
    ]);

    rsx! {
        div {
            id,
            // See the web arm's identical guard above (dev-docs/backlog.md row
            // 140).
            onpointerdown: crate::menu_root::keep_focus_on_pointerdown,
            ..attributes,
            {children}
        }
    }
}

/// The props for the [`MenubarItem`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MenubarItemProps {
    /// The item's place in keyboard order. Leave it unset: items are ordered as they are
    /// created, which is tree order. Set it on every item only when one is inserted between
    /// items that are already mounted, or a keyed list reorders them.
    #[props(default)]
    pub index: ReadSignal<Option<usize>>,

    /// The value associated with this menu item. This value will be passed to the [`Self::on_select`] callback when the item is selected.
    pub value: String,

    /// Whether this menu item is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Explicit label text used for typeahead search (`dev-docs/backlog.md`
    /// row 11 -- APG's "Any key that corresponds to a printable character"
    /// rule). When not set, falls back to [`Self::value`].
    #[props(default)]
    pub text_value: ReadSignal<Option<String>>,

    /// Callback fired when the item is selected. The [`Self::value`] will be passed as an argument.
    #[props(default)]
    pub on_select: Callback<String>,

    /// Whether the item is drawn in the destructive ink -- shadcn's
    /// `variant="destructive"`. The role stays `menuitem`; the item carries
    /// `data-variant`. See [`crate::menu_item::MenuItemVariant`].
    #[props(default)]
    pub variant: MenuItemVariant,

    /// Reserves the leading indicator slot without drawing anything -- shadcn's
    /// `inset` -- so a menu that mixes plain and checkable items keeps its
    /// labels on one edge. The item carries `data-inset="true"`.
    #[props(default)]
    pub inset: bool,

    /// Additional attributes to apply to the item element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the item component.
    pub children: Element,
}

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
    let mut ctx: MenubarContext = use_context();
    let menu_ctx: MenubarMenuContext = use_context();

    let disabled = move || (ctx.disabled)() || (props.disabled)();
    // Cloned once up front (not moved), the same `props.value.clone()`
    // pattern this component's own `onpointerdown`/`onkeydown` closures
    // already use below -- `String` isn't `Copy`, so each closure needing
    // it clones its own copy rather than fighting over one move.
    let fallback_text_value = props.value.clone();
    let order = use_item_index(menu_ctx.focus, props.index);
    let index = order.index;
    let item = use_item(
        collection_item(menu_ctx.focus, index)
            .disabled(disabled)
            .text_value(move || {
                Some((props.text_value)().unwrap_or_else(|| fallback_text_value.clone()))
            }),
    );
    let focused = move || item.focused() && (menu_ctx.is_open)();

    let onmounted = item.onmounted();

    // Owned by this component -- see `MenubarTrigger`'s identical
    // construction above (`dev-docs/backlog.md` row 93).
    let attributes = merge_attributes(vec![
        order.marker_attribute(),
        props.attributes,
        attributes!(div {
            role: crate::menu_semantics::MENU_ITEM_ROLE,
            // The disabled state of this pattern class (dev-docs/backlog.md
            // row 39): see `dropdown_menu.rs`'s identical `DropdownMenuItem`
            // wiring for the full account -- `data-disabled` alone never
            // tells assistive tech this item is disabled; `ContextMenuItem`
            // (`context_menu.rs`) sets this too.
            aria_disabled: disabled(),
            ..state_attrs::disabled(disabled()),
            ..state_attrs::presence(state_attrs::Flag::Highlighted, focused()),
            tabindex: if focused() { "0" } else { "-1" },
        }),
        menu_item::item_style_attributes(props.variant, props.inset),
    ]);

    rsx! {
        div {
            onpointerdown: {
                let value = props.value.clone();
                move |_| {
                    if !disabled() {
                        props.on_select.call(value.clone());
                        ctx.set_open_menu.call(None);
                        // APG menubar close-focus rule, via the collection's
                        // own focus mechanism -- see dev-docs/plan.md Phase 3.1
                        // and the matching Escape case in `MenubarMenu`.
                        // `ctx.focus` never actually left this trigger's
                        // index while item-level keyboard focus roamed the
                        // (separate) submenu collection, so `set_focus`
                        // alone would be a same-value no-op -- clear first.
                        ctx.focus.clear_focus();
                        ctx.focus.set_focus(Some(menu_ctx.index.cloned()));
                    }
                }
            },

            onkeydown: {
                let value = props.value.clone();
                move |event: Event<KeyboardData>| {
                    if event.key() == Key::Enter || event.key() == Key::Character(" ".to_string()) {
                        if !disabled() {
                            props.on_select.call(value.clone());
                            ctx.set_open_menu.call(None);
                            // See the matching comment on the `onpointerdown`
                            // handler above.
                            ctx.focus.clear_focus();
                            ctx.focus.set_focus(Some(menu_ctx.index.cloned()));
                        }
                        event.prevent_default();
                        event.stop_propagation();
                    }
                }
            },

            onmounted,

            // No `onblur` here (`dev-docs/backlog.md` row 85): closing this
            // menu whenever `focused()` (this item holds `menu_ctx.focus`'s
            // roving focus at the instant it blurs) is the most directly
            // reproducible instance of the class this row reports:
            // arrow-down onto this item (so it genuinely holds DOM focus),
            // then a raw `.focus()` call on `MenubarTrigger`. See
            // `MenubarTrigger`'s own note above in this file, and
            // `DropdownMenuTrigger`'s comment (`dropdown_menu.rs`) for the
            // root cause they share. `MenubarContentRendered`'s own
            // `use_outside_dismiss` call owns this job, DOM-truth-based
            // rather than signal-based.
            ..attributes,
            {props.children}
        }
    }
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
    menubar_checkable_item(state, props.into())
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
    menubar_checkable_item(state, props.into())
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

/// The body shared by [`MenubarCheckboxItem`] and [`MenubarRadioItem`]:
/// everything that is the same for both roles lives in `crate::menu_item`;
/// this is the host-specific half -- the item registers in its own
/// `MenubarMenu`'s roving-focus collection, commits on pointerdown like
/// [`MenubarItem`], and "close" is closing this menu and handing focus back
/// to its trigger (APG menubar close-focus rule). Called from a component
/// body; it calls hooks.
fn menubar_checkable_item(state: CheckableState, common: CheckableCommon) -> Element {
    let mut ctx: MenubarContext = use_context();
    let menu_ctx: MenubarMenuContext = use_context();

    let CheckableCommon {
        index,
        disabled: item_disabled,
        text_value,
        close_on_select,
        inset,
        attributes,
        children,
    } = common;
    let disabled = move || (ctx.disabled)() || item_disabled();
    let order = use_item_index(menu_ctx.focus, index);
    let index = order.index;
    let item = use_item(
        collection_item(menu_ctx.focus, index)
            .disabled(disabled)
            .text_value(move || state.text_value(text_value)),
    );
    let focused = move || item.focused() && (menu_ctx.is_open)();
    let onmounted = item.onmounted();

    let mut select = move || {
        if disabled() {
            return;
        }
        state.activate();
        if close_on_select() {
            ctx.set_open_menu.call(None);
            // See `MenubarItem`'s identical close: `ctx.focus` never left
            // this trigger's index while item-level keyboard focus roamed
            // the (separate) submenu collection, so clear first to force a
            // real transition.
            ctx.focus.clear_focus();
            ctx.focus.set_focus(Some(menu_ctx.index.cloned()));
        }
    };

    let attributes = merge_attributes(vec![
        order.marker_attribute(),
        attributes,
        state.owned_attributes(disabled(), inset, focused()),
    ]);

    rsx! {
        div {
            onpointerdown: move |_| select(),

            onkeydown: move |event: Event<KeyboardData>| {
                if menu_item::is_activation_key(&event.key()) {
                    select();
                    event.prevent_default();
                    event.stop_propagation();
                }
            },

            onmounted,

            ..attributes,
            {children}
        }
    }
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
