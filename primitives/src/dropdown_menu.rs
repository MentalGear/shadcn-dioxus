//! Defines the [`DropdownMenu`] component and its subcomponents.
//!
//! Also defines [`DropdownMenuSub`]/[`DropdownMenuSubTrigger`]/
//! [`DropdownMenuSubContent`]/[`DropdownMenuSubItem`], the nested-submenu
//! half of the APG "Menu and Menubar" pattern (`dev-docs/archive/rounds/component-backlog.md`
//! row 68).
//!
//! This file is the DropdownMenu *host* (backlog row 435): its trigger (a
//! toggle button), its root (the keys that open it and the focus/escape
//! plumbing) and its anchored panel. The items, the checkable items, the
//! submenu and the panel wiring are the menu family every host shares
//! (`crate::menu_item`, `crate::menu_sub`, `crate::menu_root`), reading the
//! `MenuHost` this root provides; its items select on `click`
//! (`SelectOn::Click`).

use crate::state_attrs;
use crate::AsChild;
use crate::{
    collection::{
        use_collection_provider, use_deferred_collection_focus, CollectionPlacement,
        CollectionState,
    },
    direction::{use_direction, use_direction_signal, Direction},
    menu_item::{
        self, MenuCheckboxItemProps, MenuGroupProps, MenuItemProps, MenuRadioGroupProps,
        MenuRadioItemProps, MenuSeparatorProps, MenuShortcutProps, MenuSubContentProps,
        MenuSubProps, MenuSubTriggerProps, SelectOn,
    },
    menu_root::{self, MenuHost, PanelConfig, PanelKind, PanelPlacement, PanelPopover},
    menu_sub, merge_attributes, use_controlled, use_id_or, use_outside_dismiss, use_unique_id,
    ContentAlign, ContentSide,
};
use dioxus::prelude::*;
use dioxus_attributes::attributes;

#[derive(Clone, Copy)]
struct DropdownMenuContext {
    // State
    open: Memo<bool>,
    set_open: Callback<bool>,
    disabled: ReadSignal<bool>,

    // This menu's own outer wrapping element id (set on the root `div`
    // `DropdownMenu` itself renders, the one enclosing both
    // `DropdownMenuTrigger` and `DropdownMenuContent`) -- mirrors
    // `ContextMenuCtx::root_id` (`context_menu.rs`) exactly, for the
    // same reason (`dev-docs/backlog.md` row 85). See
    // `DropdownMenuContentRendered`'s own `use_outside_dismiss` call for
    // why this id, not `trigger_id`/`content_id` individually, is what a
    // real "did focus/a click leave the whole widget" check needs to key
    // off (a `DropdownMenuTrigger` `onblur` would answer that question from
    // `focus.any_focused()`, a Rust-tracked signal a raw external
    // `.focus()` call never updates, instead of asking the DOM directly).
    root_id: Signal<String>,

    // Focus state
    focus: CollectionState,

    // Where focus should land once `DropdownMenuContent` next mounts. Every
    // key that opens the menu (Enter/Space/ArrowDown/ArrowUp) goes through
    // `open_with_focus` below, which sets this *before* flipping `open` --
    // never the reverse -- so opening never depends on the content already
    // being mounted or on the open<->focus sync effect below. See
    // dev-docs/reference/platform-rationale.md's "keyboard open contract" note.
    initial_focus: Signal<Option<CollectionPlacement>>,

    // Unique ID for the trigger button
    trigger_id: Signal<String>,

    // The trigger button by handle: where focus goes back to on close (`use_refocus_on_close_unless`)
    // and the last press the menu saw, which says whether that hand-back is skipped (touch, pen).
    trigger: crate::TriggerHandle,

    // The current `DropdownMenuContent`'s own element id, kept in sync by
    // that component -- mirrors `PopoverCtx::content_id` (`popover.rs`).
    // `DropdownMenuTrigger`'s `anchor-name` must key off *this* signal, not
    // `trigger_id` (this is the trigger's own id, unrelated), so that
    // `crate::top_layer::position_anchor_style` on the content side (built
    // from the content's own id) and `anchor_name_style` on the trigger
    // side name the same anchor. See `PopoverCtx::content_id`'s doc for the
    // exact bug this guards against if the two ever named different ids.
    content_id: Signal<String>,

    // Set just before a close caused by something *outside* the menu
    // (trigger/item blur), so `use_refocus_on_close_unless` (lib.rs) knows
    // not to yank focus back to the trigger for those closes. Reset to
    // `false` whenever the menu opens. See dev-docs/plan.md Phase 3.1.
    interacted_outside: Signal<bool>,

    // Text direction, rendered as the root's `dir` and `data-direction`. The
    // submenus read it from `MenuHost` (see `crate::menu_sub`).
    direction: ReadSignal<Direction>,
}

impl DropdownMenuContext {
    /// The single path every open key (Enter/Space/ArrowDown/ArrowUp) routes
    /// through: request `target` as the focus placement once the content
    /// mounts, then open. Never the other order -- setting `open` first
    /// would let a render happen (or a Playwright poll observe state)
    /// between the two with no focus request recorded yet. This is what
    /// the keyboard matrix's DropdownMenu-trigger rows rely on: see
    /// `oracle/tier1-apg/keyboard-matrix.spec.ts`.
    fn open_with_focus(&mut self, target: CollectionPlacement) {
        self.initial_focus.set(Some(target));
        self.set_open.call(true);
    }
}

/// The props for the [`DropdownMenu`] component
#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuProps {
    /// Whether the dropdown menu is open. If not provided, the component will be uncontrolled and use `default_open`.
    pub open: ReadSignal<Option<bool>>,

    /// Default open state if the component is not controlled.
    #[props(default)]
    pub default_open: bool,

    /// Callback when the open state changes. This is called when the dropdown menu is opened or closed.
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// Whether the dropdown menu is disabled. If true, the menu will not open and items will not be selectable.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether focus should loop around when reaching the end.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// The text direction for a [`DropdownMenuSub`]'s open/close arrow key
    /// and which side it opens on. Defaults to the nearest
    /// [`crate::direction::DirectionProvider`], or LTR if there is none.
    #[props(default)]
    pub dir: Option<Direction>,

    /// Additional attributes to apply to the dropdown menu element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the dropdown menu, which should include a [`DropdownMenuTrigger`] and a [`DropdownMenuContent`].
    pub children: Element,
}

/// # DropdownMenu
///
/// The `DropdownMenu` component is a container for a [`DropdownMenuContent`] component activated by a [`DropdownMenuTrigger`] component.
///
/// It implements the APG **menu button** pattern (`aria-haspopup="menu"` on
/// the trigger, `role="menu"` on the content, `role="menuitem"` on each
/// item -- see `crate::menu_semantics` for the shared role definitions and
/// their APG citations). It does **not** use the APG **listbox** pattern's
/// roles (`aria-haspopup="listbox"` / `role="listbox"` / `role="option"`)
/// that [`crate::select`], a genuine listbox, uses. `DropdownMenu` has no
/// selection model (no `value`/`selected` state, no `aria-selected` on any
/// item -- activating an item is an action, via `on_select`, not a
/// selection), so those roles would be wrong for it; see
/// `dev-docs/backlog.md` row 24 and `oracle/tier1-apg/menu-roles.spec.ts`.
/// Code that queries `role="option"` (Playwright's
/// `getByRole('option', ...)` or an accessibility-tree assertion) against
/// this component's items should query `role="menuitem"`.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::dropdown_menu::{
///     DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         DropdownMenu { default_open: false,
///             DropdownMenuTrigger { "Open Menu" }
///             DropdownMenuContent {
///                 DropdownMenuItem::<String> {
///                     value: "edit".to_string(),
///                     on_select: move |value| {
///                         tracing::info!("Selected: {}", value);
///                     },
///                     "Edit"
///                 }
///                 DropdownMenuItem::<String> {
///                     value: "undo".to_string(),
///                     disabled: true,
///                     on_select: move |value| {
///                         tracing::info!("Selected: {}", value);
///                     },
///                     "Undo"
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`DropdownMenu`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates the current state of the dropdown menu. values are `open` or `closed`.
/// - `data-open` / `data-closed`: Present (empty) while open / closed, absent otherwise.
/// - `data-disabled`: Indicates if the dropdown menu is disabled. values are `true` or `false`.
#[component]
pub fn DropdownMenu(props: DropdownMenuProps) -> Element {
    let (open, set_open) = use_controlled(props.open, props.default_open, props.on_open_change);
    let direction = use_direction(props.dir);
    let direction_signal = use_direction_signal(direction);

    let disabled = props.disabled;
    let root_id = use_unique_id();
    let trigger_id = use_unique_id();
    let trigger = crate::use_trigger_handle();
    // Placeholder value until `DropdownMenuContent` mounts and syncs its
    // own id in -- see `DropdownMenuContext::content_id`'s doc.
    let content_id = use_unique_id();
    let interacted_outside = use_signal(|| false);
    let focus = use_collection_provider(props.roving_loop);
    let initial_focus = use_signal(|| None);
    let submenu_open_count = use_signal(|| 0usize);
    let mut ctx = use_context_provider(|| DropdownMenuContext {
        open,
        set_open,
        disabled,
        root_id,
        focus,
        initial_focus,
        trigger_id,
        trigger,
        content_id,
        interacted_outside,
        direction: direction_signal,
    });

    // What the shared menu family (`menu_item`, `menu_sub`) reads from this host. A selection clears the roving
    // focus and closes; a root item that blurs marks the close as caused outside the menu, so focus is not
    // yanked back to the trigger.
    let close = use_callback(move |()| {
        let mut focus = focus;
        focus.clear_focus();
        set_open.call(false);
    });
    let blur = use_callback(move |()| {
        let (mut focus, mut interacted_outside) = (focus, interacted_outside);
        interacted_outside.set(true);
        focus.clear_focus();
    });
    use_context_provider(|| MenuHost {
        open,
        disabled,
        focus,
        direction: direction_signal,
        submenu_open_count,
        select_on: SelectOn::Click,
        close,
        blur,
    });

    // dev-docs/backlog.md row 53 (the "open-index container" band): this effect
    // -- keep `open` in sync with whether `focus` currently holds DOM focus,
    // skipped while a descendant submenu is open -- is
    // `ContextMenu`'s own root effect (`context_menu.rs`), so both share
    // `menu_root::use_open_focus_sync` rather than carrying two copies. See
    // that function's own doc for why `Menubar`'s analogous (index-keyed)
    // effect stays out of this shared construction.
    crate::menu_root::use_open_focus_sync(focus, open, set_open, submenu_open_count);

    // dev-docs/backlog.md row 11 (typeahead): one buffer for this
    // menu's own root-level items (including any `DropdownMenuSubTrigger`
    // rows, which register in this same `focus` collection -- see that
    // component's own `text_value` prop). A submenu's own open content gets
    // its own, independent `TypeaheadState` -- see
    // `menu_sub`.
    let mut typeahead = crate::typeahead::use_typeahead_state();

    // A fresh open shouldn't inherit an `interacted_outside` flag set by a
    // previous close -- otherwise an internal close (Escape, item select)
    // right after an outside-dismiss would wrongly skip refocusing the
    // trigger.
    use_effect(move || {
        if (ctx.open)() {
            ctx.interacted_outside.set(false);
        }
    });

    // APG menu-button: "Escape: Closes the menu and sets focus to the menu
    // button." Radix's `onCloseAutoFocus` skips the refocus when the close
    // was caused by interacting outside the menu (see `interacted_outside`
    // wiring on the trigger/item `onblur` handlers below).
    crate::use_refocus_on_close_unless(
        ctx.open,
        ctx.trigger,
        ReadSignal::new(ctx.interacted_outside),
    );

    // Handle escape key to close the menu
    let handle_keydown = move |event: Event<KeyboardData>| {
        ctx.trigger.press().key_pressed(&event);
        if disabled() {
            return;
        }
        match event.key() {
            Key::Enter => {
                if open() {
                    ctx.set_open.call(false);
                } else {
                    ctx.open_with_focus(CollectionPlacement::First);
                }
            }
            Key::Character(c) if c == " " => {
                // APG menu-button (Optional): "Space: Opens the menu and
                // places focus on the first menu item." Mirrors the Enter
                // arm above -- both route through the same open-with-focus
                // path so neither can drift from the other (opening via the
                // native button click instead would refocus the trigger).
                if !open() {
                    ctx.open_with_focus(CollectionPlacement::First);
                }
            }
            Key::Escape => {
                // Web arm: `DropdownMenuContentRendered`'s `popover="auto"`
                // owns Escape dismissal natively (WHATWG HTML's light-dismiss
                // algorithm) -- returning here before `event.prevent_default()`
                // below leaves the key's default action alone so the browser's
                // own dismissal still runs. Calling both `set_open` *and*
                // `prevent_default` unconditionally here would race that native algorithm, and the
                // `prevent_default` specifically would suppress the
                // browser's default action for the key outright -- the same
                // "unconditional keydown-suppressor must never run on the
                // web arm" lesson `PopoverContentRendered`'s doc documents
                // for `use_global_escape_listener` (dev-docs/plan.md Phase
                // 4.4/4.2). This is a plain `cfg!()` compile-time branch on
                // ordinary code, not a conditionally-called hook, so it
                // carries none of that lesson's hook-order hazard -- no
                // component-boundary split needed for this one arm. Native
                // (Blitz) arm: closes here directly (Blitz
                // has no popover-API light dismiss to defer to).
                if cfg!(feature = "web") {
                    return;
                }
                ctx.set_open.call(false);
            }
            Key::ArrowDown => {
                // APG (Optional): "Down Arrow: opens the menu and moves
                // focus to the first menu item" from a closed trigger; once
                // open, plain roving-focus navigation.
                if open() {
                    menu_root::nav_key(&event, ctx.focus, &mut typeahead);
                } else {
                    ctx.open_with_focus(CollectionPlacement::First);
                }
            }
            Key::ArrowUp => {
                // APG (Optional): "Up Arrow: opens the menu and moves focus
                // to the last menu item" from a closed trigger.
                if open() {
                    menu_root::nav_key(&event, ctx.focus, &mut typeahead);
                } else {
                    ctx.open_with_focus(CollectionPlacement::Last);
                }
            }
            // APG Menu and Menubar pattern, "Keyboard Interaction" (h2), same pinned commit as
            // `DropdownMenuSubTrigger`'s doc: Home/End jump to the first/last item and "Any key that corresponds
            // to a printable character (Optional): Move focus to the next item in the current menu whose label
            // begins with that printable character." (`menu_root::nav_key`). Typeahead only while actually open:
            // while closed, `ctx.focus` has no registered items to search. `TypeaheadState::handle_key` itself
            // declines (so this `return`s before `prevent_default()`) any Ctrl/Meta/Alt-held key and Space, which
            // has its own arm above.
            Key::Character(_) if !open() => return,
            Key::Home | Key::End | Key::Character(_) => {
                if !menu_root::nav_key(&event, ctx.focus, &mut typeahead) {
                    return;
                }
            }
            _ => return,
        }
        event.prevent_default();
    };

    // Merged (caller-wins, deduped), not set-then-spread-over: `dev-docs/
    // backlog.md` row 85 needs a stable `id` on *this* element (the one
    // wrapping both `DropdownMenuTrigger` and `DropdownMenuContent`) so
    // `DropdownMenuContentRendered`'s `use_outside_dismiss(ctx.root_id,
    // ...)` can ask the DOM "is the thing that just received focus/a click
    // actually inside the whole widget". A caller-supplied `id` in a
    // trailing `..props.attributes` spread does NOT "win by spread order":
    // an explicit `id:` literal followed by `..props.attributes` on the
    // same element emits *both* into the SSR'd HTML (e.g. `id="dxc-1091"
    // ... id="clip-dropdown-menu-root"` on one tag), a WHATWG
    // duplicate-attribute parse error (`dev-docs/conformance-
    // harness.md` hydration-parity Rule 4). The browser's HTML parser keeps
    // the FIRST (this component's own generated id), silently dropping the
    // caller's override. `ContextMenu`'s identical root `div`
    // (`context_menu.rs`) merges this same shape via `merge_attributes`;
    // `id` is caller-wins like everything else here, and
    // `use_outside_dismiss`'s own `getElementById` lookup silently
    // no-ops if a caller's override id makes it miss.
    let attributes = merge_attributes(vec![
        attributes!(div {
            id: root_id.cloned(),
            dir: (ctx.direction)().as_str(),
            ..state_attrs::open(open()),
            ..state_attrs::disabled((props.disabled)()),
            "data-direction": (ctx.direction)().as_str(),
        }),
        props.attributes,
    ]);

    rsx! {
        div {
            onkeydown: handle_keydown,
            onpointerdown: move |event: PointerEvent| ctx.trigger.press().pressed(&event),
            ..attributes,
            {props.children}
        }
    }
}

/// The props for the [`DropdownMenuTrigger`] component
#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuTriggerProps {
    /// Render the trigger element as a custom component/element.
    /// The callback gets the merged attributes (listeners included) and the children, see
    /// [`AsChild`].
    #[props(default)]
    pub r#as: Option<Callback<AsChild, Element>>,

    /// The ID of the trigger button. If not provided, an internally
    /// generated ID is used -- see `DropdownMenuTrigger`'s use of
    /// `use_id_or` for why this is a typed field rather than left to
    /// `GlobalAttributes` alone (dev-docs/backlog.md row 36).
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes to apply to the trigger element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the trigger
    pub children: Element,
}

/// # DropdownMenuTrigger
///
/// The trigger button for the parent [`DropdownMenu`]. This button toggles the visibility of the [`DropdownMenuContent`].
///
/// This must be used inside a [`DropdownMenu`] component.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::dropdown_menu::{
///     DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         DropdownMenu { default_open: false,
///             DropdownMenuTrigger { "Open Menu" }
///             DropdownMenuContent {
///                 DropdownMenuItem::<String> {
///                     value: "edit".to_string(),
///                     on_select: move |value| {
///                         tracing::info!("Selected: {}", value);
///                     },
///                     "Edit"
///                 }
///                 DropdownMenuItem::<String> {
///                     value: "undo".to_string(),
///                     disabled: true,
///                     on_select: move |value| {
///                         tracing::info!("Selected: {}", value);
///                     },
///                     "Undo"
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`DropdownMenuTrigger`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates the current state of the dropdown menu. values are `open` or `closed`.
/// - `data-open` / `data-closed`: Present (empty) while open / closed, absent otherwise.
/// - `data-popup-open`: Present (empty) while the popup this trigger controls is open.
/// - `data-disabled`: Indicates if the dropdown menu is disabled. values are `true` or `false`.
#[component]
pub fn DropdownMenuTrigger(props: DropdownMenuTriggerProps) -> Element {
    let ctx: DropdownMenuContext = use_context();
    // The pointer type of the press before a click is recorded once for the whole menu, by the
    // root's `onpointerdown` (`DropdownMenu`), which this button's press bubbles to.
    let press = ctx.trigger.press();

    let open = ctx.open;
    let disabled = ctx.disabled;

    // dev-docs/backlog.md row 36: `ctx.trigger_id` is the same signal
    // `DropdownMenuContent`'s `aria-labelledby` (below) and
    // `use_refocus_on_close_unless` both read back -- a caller-supplied
    // `id` here must resolve into it, not just win the render. Feeding it
    // straight into `use_id_or` as the generated-id signal does that:
    // `use_id_or`'s own effect writes a caller override back into its
    // `gen_id` argument, which *is* `ctx.trigger_id` here -- mirrors
    // `DialogTitle`'s identical `use_id_or(ctx.dialog_labelledby,
    // props.id)` (`dialog.rs`).
    let id = use_id_or(ctx.trigger_id, props.id);

    let base = attributes!(button {
        id: id.cloned(),
        r#type: "button",
        ..state_attrs::trigger_open(open),
        ..state_attrs::disabled(disabled),
        disabled: disabled,
        aria_expanded: open,
        aria_haspopup: crate::menu_semantics::MENU_TRIGGER_HASPOPUP,
        onmounted: ctx.trigger.onmounted(),
        onclick: move |_| {
            if disabled() {
                return;
            }

            let new_open = !open();
            ctx.set_open.call(new_open);

            // Focus the element on click. Safari does not do this automatically.
            // https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/button#clicking_and_focus
            // A mouse press only: after a touch or pen press the script focus would make the
            // trigger `:focus-visible` (it rings) and dismiss a field's software keyboard.
            if !press.was_touch() {
                if let Some(data) = ctx.trigger.element() {
                    spawn(async move {
                        // focus-ok: Safari click-focus remedy, mouse presses only (modality.rs)
                        _ = data.set_focus(true).await;
                    });
                }
            }
        },
        // No `onblur` here (`dev-docs/backlog.md` row 85): whether focus really
        // left the menu is answered by the DOM, not by a signal. A blur
        // handler would read `ctx.focus` (a Rust-tracked roving-focus signal)
        // synchronously, which is current for every focus move this crate's
        // own click/keyboard code drives (all update the signal *before*
        // imperatively moving DOM focus, see `open_with_focus`'s doc) but
        // stale for one arriving some other way -- no item in this file has
        // an `onfocus` reconciling it (unlike e.g. `tabs.rs`'s `onfocus:
        // move |_| ctx.focus.set_focus(...)`). A raw external `.focus()` call
        // landing directly on a menu item or a `DropdownMenuSubTrigger` (a
        // Playwright test driving focus programmatically, an assistive-
        // technology focus-restoration path) would blur this button
        // *before* anything updated `ctx.focus`, and the stale "nothing
        // focused" state would close the whole menu and (since it never
        // sets `interacted_outside`) `use_refocus_on_close_unless` would
        // refocus this same button. `ContextMenu` goes through
        // `use_outside_dismiss` (`primitives/src/lib.rs`), which asks the DOM
        // directly (a native `focusin` listener plus
        // `root.contains(event.target)`), so it is immune to the same-tick
        // blur-before-focus ordering hazard by construction.
        // `DropdownMenuContentRendered`'s own `use_outside_dismiss(ctx.root_id,
        // ...)` call below gives `DropdownMenu` that same construction.
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    // Ties this trigger to the web-arm content's `position-anchor`
    // (`DropdownMenuContentRendered`) so its anchor-positioned placement
    // resolves relative to this trigger once promoted to the top layer --
    // keyed on `ctx.content_id`, not `ctx.trigger_id` above (this trigger's
    // own id), for the same reason `PopoverTrigger` does (see
    // `PopoverCtx::content_id`'s doc in `popover.rs`). Folded with any
    // caller-supplied `style` by `top_layer::anchored_trigger_attributes`
    // rather than left to plain `merge_attributes` (which folds a plain
    // `style` attribute but not shorthand style props -- `padding: ".."`
    // etc., namespace `Some("style")` -- which would still render a second
    // `style` beside it); see that function's own doc.
    let merged = crate::top_layer::anchored_trigger_attributes(&ctx.content_id.cloned(), merged);

    if let Some(dynamic) = props.r#as {
        dynamic.call(AsChild::new(merged, props.children))
    } else {
        rsx! {
            button {
                ..merged,
                {props.children}
            }
        }
    }
}

/// The props for the [`DropdownMenuContent`] component
#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuContentProps {
    /// The ID of the dropdown menu content element. If not provided, a unique ID will be generated.
    pub id: ReadSignal<Option<String>>,
    /// The side of the trigger the menu opens on. It flips to the opposite side when that side has
    /// no room; the side it was actually placed on is published as `data-placed-side`.
    #[props(default = ContentSide::Bottom)]
    pub side: ContentSide,
    /// How the menu lines up with the trigger along that side: `Start` puts the two leading edges
    /// together.
    #[props(default = ContentAlign::Start)]
    pub align: ContentAlign,
    /// The gap between the trigger and the menu, in CSS pixels.
    #[props(default = 4)]
    pub side_offset: u32,
    /// Additional attributes to apply to the dropdown menu content element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the dropdown menu content, which should include one or more [`DropdownMenuItem`] components.
    pub children: Element,
}

/// # DropdownMenuTrigger
///
/// The contents of a [`DropdownMenu`]. The component will only be rendered when the parent [`DropdownMenu`] is open (as control by the [`DropdownMenuTrigger`]).
///
/// This must be used inside a [`DropdownMenu`] component.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::dropdown_menu::{
///     DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         DropdownMenu { default_open: false,
///             DropdownMenuTrigger { "Open Menu" }
///             DropdownMenuContent {
///                 DropdownMenuItem::<String> {
///                     value: "edit".to_string(),
///                     on_select: move |value| {
///                         tracing::info!("Selected: {}", value);
///                     },
///                     "Edit"
///                 }
///                 DropdownMenuItem::<String> {
///                     value: "undo".to_string(),
///                     disabled: true,
///                     on_select: move |value| {
///                         tracing::info!("Selected: {}", value);
///                     },
///                     "Undo"
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`DropdownMenuContent`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates the current state of the dropdown menu. values are `open` or `closed`.
/// - `data-open` / `data-closed`: Present (empty) while open / closed, absent otherwise.
#[component]
pub fn DropdownMenuContent(props: DropdownMenuContentProps) -> Element {
    let mut ctx: DropdownMenuContext = use_context();

    // dev-docs/backlog.md row 53 (the "content lifecycle/positioning pipeline"
    // band): id resolution + `use_animated_open`'s render gate, identical
    // across all three menu-family hosts' `*Content` components -- see
    // `menu_root::use_menu_content_lifecycle`'s own doc.
    let (id, render) = crate::menu_root::use_menu_content_lifecycle(props.id, ctx.open);

    // Keep `ctx.content_id` in sync with this content's actual id -- see
    // `DropdownMenuContext::content_id`'s doc. Mirrors `PopoverContent`'s
    // identical `ctx.content_id.set(id())` in `popover.rs`.
    use_effect(move || ctx.content_id.set(id()));

    // Apply the focus placement `open_with_focus` (`DropdownMenuContext`)
    // recorded when the menu was opened by keyboard, as soon as the
    // collection has items to focus -- which may be later than `render()`
    // first turning true, since items register via their own effects.
    // Mirrors `MenubarContent`'s identical call (`menubar.rs`).
    use_deferred_collection_focus(ctx.focus, ctx.initial_focus, render);

    rsx! {
        if render() {
            DropdownMenuContentRendered {
                id: id.cloned(),
                side: props.side,
                align: props.align,
                side_offset: props.side_offset,
                attributes: props.attributes,
                children: props.children,
            }
        }
    }
}

/// The menu panel. On the web it is promoted to the top layer via `popover="auto"` so it escapes clipping and
/// transformed ancestors, the same construction `Tooltip`/`HoverCard`/non-modal `Popover` use
/// (dev-docs/plan.md Phase 4.4), and gets the platform's light dismiss (Escape and an outside pointerdown) beside
/// this crate's own DOM-truth dismissal. Without the popover API it is a plain, always-in-flow `div`, visible
/// exactly when `render()` (`DropdownMenuContent`) mounts it.
///
/// `menu_root::use_menu_panel` drives `showPopover()`/`hidePopover()` from `open` and mirrors the browser's own
/// `toggle` event (fired on light dismiss, Escape, or any other close) back into `set_open`, so the Rust signal
/// can never strand the way `dev-docs/reference/platform-rationale.md` Caveat 1 documents for `<dialog>`'s
/// one-way `showModal()`/`close()` binding. A native close also clears the focus collection and marks the close
/// as caused outside the menu.
///
/// The menu is placed at `side`/`align`, `side_offset` px from the trigger (`top_layer::use_anchor_position_
/// fallback`). The defaults match the Radix dropdown menu: below the trigger and start-aligned with it, because a
/// menu's items read from the trigger's own leading edge. The content carries no `data-side`: the engine's
/// `[data-side]` anchor rules use an 8px gap, so this measured placement is the one authority, and it publishes
/// the resolved `data-placed-side`.
///
/// "Did focus or a press leave the whole widget" is asked of the DOM (`use_outside_dismiss` on the root's id,
/// which covers the trigger *and* this content, submenus included, since a `DropdownMenuSub`'s content is a DOM
/// descendant of this one): a trigger `onblur` would answer it from `ctx.focus.any_focused()`, a Rust-tracked
/// signal a raw external `.focus()` call never updates.
#[component]
fn DropdownMenuContentRendered(
    /// The DOM id of the menu's content element.
    id: String,
    /// The side of the trigger the menu opens on ([`DropdownMenuContentProps::side`]).
    side: ContentSide,
    /// The menu's alignment along that side ([`DropdownMenuContentProps::align`]).
    align: ContentAlign,
    /// The gap to the trigger in px ([`DropdownMenuContentProps::side_offset`]).
    side_offset: u32,
    /// Extra HTML attributes for the menu's content element.
    attributes: Vec<Attribute>,
    /// The menu items.
    children: Element,
) -> Element {
    let mut ctx: DropdownMenuContext = use_context();

    // A native close (Escape or an outside pointerdown) clears the focus collection too, so a stale "something is
    // still focused" reading cannot disagree with `open` on the next interaction (the root's `open` <->
    // `focus.any_focused()` sync effect reacts to focus changes, not to this direct `set_open` call).
    let on_native_toggle = use_callback(move |is_open: bool| {
        if !is_open {
            ctx.focus.clear_focus();
            ctx.interacted_outside.set(true);
        }
        ctx.set_open.call(is_open);
    });
    let attributes = menu_root::use_menu_panel(
        PanelConfig {
            id: id.clone(),
            open: ctx.open,
            trigger_id: ctx.trigger_id.cloned(),
            on_native_toggle,
            popover: PanelPopover::Auto,
            placement: PanelPlacement::Anchored {
                kind: PanelKind::Dropdown,
                side,
                align,
                offset: side_offset,
            },
            owned: Vec::new(),
        },
        attributes,
    );
    use_outside_dismiss(ctx.root_id, move || {
        ctx.interacted_outside.set(true);
        ctx.focus.clear_focus();
        ctx.set_open.call(false);
    });

    rsx! {
        div {
            id,
            // The user is starting a click inside the dropdown menu. Prevent the blur event from occurring during
            // pointerdown, to keep the dropdown menu open until pointerup happens, thus enabling
            // onclick/onselect events to fire -- see `menu_root::keep_focus_on_pointerdown`.
            onpointerdown: menu_root::keep_focus_on_pointerdown,
            ..attributes,
            {children}
        }
    }
}

/// The props for the [`DropdownMenuItem`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuItemProps`].
pub type DropdownMenuItemProps<T> = MenuItemProps<T>;

/// # DropdownMenuTrigger
///
/// An item within a [`DropdownMenuContent`]. This component represents an individual selectable item in the dropdown menu.
///
/// This must be used inside a [`DropdownMenu`] component.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::dropdown_menu::{
///     DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         DropdownMenu { default_open: false,
///             DropdownMenuTrigger { "Open Menu" }
///             DropdownMenuContent {
///                 DropdownMenuItem::<String> {
///                     value: "edit".to_string(),
///                     on_select: move |value| {
///                         tracing::info!("Selected: {}", value);
///                     },
///                     "Edit"
///                 }
///                 DropdownMenuItem::<String> {
///                     value: "undo".to_string(),
///                     disabled: true,
///                     on_select: move |value| {
///                         tracing::info!("Selected: {}", value);
///                     },
///                     "Undo"
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`DropdownMenuItem`] component defines the following data attributes you can use to control styling:
/// - `data-disabled`: Indicates whether the item is disabled. Values are `true` or `false`.
/// - `data-highlighted`: Present (empty) while this item holds the roving focus, absent otherwise.
#[component]
pub fn DropdownMenuItem<T: Clone + PartialEq + 'static>(
    props: DropdownMenuItemProps<T>,
) -> Element {
    menu_item::render_item(props, "DropdownMenuItem")
}

/// The props for the [`DropdownMenuCheckboxItem`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuCheckboxItemProps`].
pub type DropdownMenuCheckboxItemProps = MenuCheckboxItemProps;

/// The props for the [`DropdownMenuRadioGroup`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuRadioGroupProps`].
pub type DropdownMenuRadioGroupProps = MenuRadioGroupProps;

/// The props for the [`DropdownMenuRadioItem`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuRadioItemProps`].
pub type DropdownMenuRadioItemProps = MenuRadioItemProps;

/// # DropdownMenuCheckboxItem
///
/// An item within a [`DropdownMenuContent`] (or a [`DropdownMenuSubContent`])
/// that can be checked and unchecked independently of its siblings --
/// shadcn's `DropdownMenuCheckboxItem`. Renders `role="menuitemcheckbox"`
/// with an always-present `aria-checked` (see `crate::menu_semantics`).
///
/// Controlled with `checked` + `on_checked_change`, or uncontrolled with
/// `default_checked`. Clicking it, or pressing Enter or Space while it has
/// focus, toggles it. Like Base UI's `closeOnClick = false`, that **keeps the
/// menu open** so several items can be toggled in one visit (APG: "changes the
/// state without closing the menu"); set `close_on_select: true` for Radix's
/// `onSelect` behaviour, which closes it. Disabled items cannot be
/// toggled and are skipped by arrow keys and typeahead. A checkbox item has
/// no `value`, so give it a `text_value` to make it a typeahead target.
///
/// This must be used inside a [`DropdownMenuContent`] component.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::dropdown_menu::{
///     DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     let mut show_status_bar = use_signal(|| true);
///     rsx! {
///         DropdownMenu {
///             DropdownMenuTrigger { "View" }
///             DropdownMenuContent {
///                 DropdownMenuCheckboxItem {
///                     checked: Some(show_status_bar()),
///                     on_checked_change: move |checked| show_status_bar.set(checked),
///                     text_value: "Status Bar",
///                     "Status Bar"
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`DropdownMenuCheckboxItem`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Whether the item is checked. Values are `checked` or `unchecked`.
/// - `data-checked` / `data-unchecked`: Present (empty) while checked / unchecked, absent otherwise.
/// - `data-disabled`: Indicates whether the item is disabled. Values are `true` or `false`.
/// - `data-highlighted`: Present (empty) while this item holds the roving focus, absent otherwise.
#[component]
pub fn DropdownMenuCheckboxItem(props: DropdownMenuCheckboxItemProps) -> Element {
    let state = menu_item::use_checkbox_state(
        props.checked,
        props.default_checked,
        props.on_checked_change,
    );
    menu_item::render_checkable(state, props.into())
}

/// # DropdownMenuRadioGroup
///
/// A set of mutually exclusive [`DropdownMenuRadioItem`]s -- shadcn's
/// `DropdownMenuRadioGroup`. Renders `role="group"`; give it an accessible
/// name with `aria-label` or `aria-labelledby`. Controlled with `value` +
/// `on_value_change`, or uncontrolled with `default_value`.
///
/// This must be used inside a [`DropdownMenuContent`] component.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::dropdown_menu::{
///     DropdownMenu, DropdownMenuContent, DropdownMenuRadioGroup, DropdownMenuRadioItem,
///     DropdownMenuTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     let mut position = use_signal(|| "bottom".to_string());
///     rsx! {
///         DropdownMenu {
///             DropdownMenuTrigger { "Panel" }
///             DropdownMenuContent {
///                 DropdownMenuRadioGroup {
///                     aria_label: "Panel position",
///                     value: Some(position()),
///                     on_value_change: move |value| position.set(value),
///                     DropdownMenuRadioItem {
///                         value: "top".to_string(),
///                         "Top"
///                     }
///                     DropdownMenuRadioItem {
///                         value: "bottom".to_string(),
///                         "Bottom"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn DropdownMenuRadioGroup(props: DropdownMenuRadioGroupProps) -> Element {
    menu_item::render_radio_group(props)
}

/// # DropdownMenuRadioItem
///
/// One choice in a [`DropdownMenuRadioGroup`] -- shadcn's
/// `DropdownMenuRadioItem`. Renders `role="menuitemradio"` with an
/// always-present `aria-checked`; it is checked while the group's value
/// equals its own `value`, and choosing it (click, Enter or Space) sets the
/// group's value and, like Base UI's `closeOnClick = false`, **keeps the menu
/// open** unless `close_on_select` is `true`. Falls back to `value` as its
/// typeahead label. Disabled items cannot be chosen and are skipped by arrow
/// keys and typeahead.
///
/// This must be used inside a [`DropdownMenuRadioGroup`] component.
///
/// ## Styling
///
/// The [`DropdownMenuRadioItem`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Whether the item is checked. Values are `checked` or `unchecked`.
/// - `data-checked` / `data-unchecked`: Present (empty) while checked / unchecked, absent otherwise.
/// - `data-disabled`: Indicates whether the item is disabled. Values are `true` or `false`.
/// - `data-highlighted`: Present (empty) while this item holds the roving focus, absent otherwise.
#[component]
pub fn DropdownMenuRadioItem(props: DropdownMenuRadioItemProps) -> Element {
    let state = menu_item::use_radio_state(props.value);
    menu_item::render_checkable(state, props.into())
}

/// The props for the [`DropdownMenuSeparator`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuSeparatorProps`].
pub type DropdownMenuSeparatorProps = MenuSeparatorProps;

/// The props for the [`DropdownMenuGroup`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuGroupProps`].
pub type DropdownMenuGroupProps = MenuGroupProps;

/// The props for the [`DropdownMenuShortcut`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuShortcutProps`].
pub type DropdownMenuShortcutProps = MenuShortcutProps;

/// # DropdownMenuSeparator
///
/// A divider between groups of items -- shadcn's `DropdownMenuSeparator`. Renders
/// `role="separator"` with `aria-orientation="horizontal"`. It is not
/// focusable and takes no `index`: arrow keys, Home/End and typeahead skip
/// it.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::dropdown_menu::{DropdownMenuSeparator};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         DropdownMenuSeparator {}
///     }
/// }
/// ```
#[component]
pub fn DropdownMenuSeparator(props: DropdownMenuSeparatorProps) -> Element {
    menu_item::render_separator(props)
}

/// # DropdownMenuGroup
///
/// A `role="group"` around related items -- shadcn's `DropdownMenuGroup`. It adds
/// no behaviour: its items still take part in the enclosing menu's keyboard
/// navigation and typeahead. Name it with `aria-label` when it has a visible
/// heading (a label).
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::dropdown_menu::{DropdownMenuGroup};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         DropdownMenuGroup { aria_label: "File", "..." }
///     }
/// }
/// ```
#[component]
pub fn DropdownMenuGroup(props: DropdownMenuGroupProps) -> Element {
    menu_item::render_group(props)
}

/// # DropdownMenuShortcut
///
/// A keyboard-shortcut hint at the inline end of an item -- shadcn's
/// `DropdownMenuShortcut`. Text only: it renders a `span` and does not bind any
/// key.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::dropdown_menu::{DropdownMenuShortcut};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         DropdownMenuShortcut { "⌘T" }
///     }
/// }
/// ```
#[component]
pub fn DropdownMenuShortcut(props: DropdownMenuShortcutProps) -> Element {
    menu_item::render_shortcut(props)
}

/// The props for the [`DropdownMenuSub`] component. Shared by every
/// menu-family host with submenus -- see [`crate::menu_item::MenuSubProps`].
pub type DropdownMenuSubProps = MenuSubProps;

/// # DropdownMenuSub
///
/// A submenu nested inside a [`DropdownMenu`] (or another
/// [`DropdownMenuContent`]/[`DropdownMenuSubContent`]), implementing the
/// APG "Menu and Menubar" pattern's nested-submenu contract -- what that
/// page calls a "parent menuitem" opening a submenu
/// (`content/patterns/menubar/menu-and-menubar-pattern.html`, "Keyboard
/// Interaction" and "WAI-ARIA Roles, States, and Properties" (both h2),
/// pinned commit `7e4034b262bc0d25332e330d8a582aaf34113829` of
/// `w3c/aria-practices` -- see `playwright/oracle/reference/README.md`;
/// [`DropdownMenuSubTrigger`]'s own doc carries the exact quotes). Renders
/// no element of its
/// own -- purely a context boundary around a [`DropdownMenuSubTrigger`]
/// and a [`DropdownMenuSubContent`], the same shape Radix's
/// `DropdownMenu.Sub` has.
///
/// The state machine (open/close, this submenu's own roving-focus
/// collection, the hover-intent timers its trigger/content use) is shared
/// with [`crate::context_menu`]'s identical `ContextMenuSub` via
/// `crate::menu_sub` -- see that module's doc for exactly what is shared
/// and why the rest (which enclosing collection a trigger registers in,
/// how the content is positioned) has to stay file-local. **Scope: one
/// level of nesting** -- a `DropdownMenuSub`'s own content may hold plain
/// items but not a further nested `DropdownMenuSub`; see `crate::menu_sub`'s
/// module doc for why.
///
/// This must be used inside a [`DropdownMenuContent`] (or another
/// [`DropdownMenuSubContent`]).
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::dropdown_menu::{
///     DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSub,
///     DropdownMenuSubContent, DropdownMenuSubItem, DropdownMenuSubTrigger,
///     DropdownMenuTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         DropdownMenu { default_open: false,
///             DropdownMenuTrigger { "Open Menu" }
///             DropdownMenuContent {
///                 DropdownMenuItem::<String> {
///                     value: "edit".to_string(),
///                     on_select: move |value| {
///                         tracing::info!("Selected: {}", value);
///                     },
///                     "Edit"
///                 }
///                 DropdownMenuSub {
///                     DropdownMenuSubTrigger { "More tools" }
///                     DropdownMenuSubContent {
///                         DropdownMenuSubItem::<String> {
///                             value: "duplicate".to_string(),
///                             on_select: move |value| {
///                                 tracing::info!("Selected: {}", value);
///                             },
///                             "Duplicate"
///                         }
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn DropdownMenuSub(props: DropdownMenuSubProps) -> Element {
    menu_sub::render_sub(props)
}

/// The props for the [`DropdownMenuSubTrigger`] component. Shared by every
/// menu-family host with submenus -- see [`crate::menu_item::MenuSubTriggerProps`].
pub type DropdownMenuSubTriggerProps = MenuSubTriggerProps;

/// # DropdownMenuSubTrigger
///
/// The trigger for a [`DropdownMenuSub`]'s submenu. Renders as a
/// `menuitem` within the *enclosing* menu -- APG calls this a "parent
/// menuitem": "If activating a menuitem opens a submenu, the menuitem is
/// known as a parent menuitem" ... "A parent menuitem has aria-haspopup
/// set to either menu or true" ... "A parent menuitem has aria-expanded
/// set to false when its child menu is not visible and set to true when
/// the child menu is visible" (`content/patterns/menubar/
/// menu-and-menubar-pattern.html`, "WAI-ARIA Roles, States, and
/// Properties" (h2), pinned commit
/// `7e4034b262bc0d25332e330d8a582aaf34113829` of `w3c/aria-practices` --
/// see `playwright/oracle/reference/README.md`). Also sets
/// `aria-controls` to the submenu's own content id, so the sub-trigger
/// unambiguously owns the popup it opens -- this crate's own choice, not
/// an APG-required property: the pattern page above never mentions
/// `aria-controls`, so this is added precision, not a cited requirement.
///
/// ## Keyboard
/// - `ArrowRight`/`Enter`/`Space`: same page, "Keyboard Interaction" (h2).
///   Right Arrow: "When focus is in a menu and on a menuitem that has a
///   submenu, opens the submenu and places focus on its first item."
///   Enter: "When focus is on a menuitem that has a submenu, opens the
///   submenu and places focus on its first item." Space (Optional): the
///   same as Enter.
///
/// ## Hover
/// A pointer hovering this trigger opens its submenu after a short
/// hover-intent delay (not an APG requirement -- see `crate::menu_sub`'s
/// doc for the citation-free rationale); hover-opening does **not** move
/// keyboard focus. Leaving the trigger without entering its submenu closes
/// it after a short grace delay, so a diagonal mouse move from the trigger
/// toward its own submenu content is not misread as "left." Pass
/// `open_on_hover: false` to [`DropdownMenuSub`] to turn all of this off and
/// make the submenu click/keyboard-activated only.
///
/// This must be used inside a [`DropdownMenuSub`].
///
/// ## Styling
///
/// The [`DropdownMenuSubTrigger`] component defines the following data
/// attributes you can use to control styling:
/// - `data-state`: Indicates whether the submenu is open. Values are
///   `open` or `closed`.
/// - `data-open` / `data-closed`: Present (empty) while open / closed, absent otherwise.
/// - `data-popup-open`: Present (empty) while the popup this trigger controls is open.
/// - `data-disabled`: Indicates if the sub-trigger is disabled. Values are
///   `true` or `false`.
#[component]
pub fn DropdownMenuSubTrigger(props: DropdownMenuSubTriggerProps) -> Element {
    menu_sub::render_sub_trigger(props)
}

/// The props for the [`DropdownMenuSubContent`] component. Shared by every
/// menu-family host with submenus -- see [`crate::menu_item::MenuSubContentProps`].
pub type DropdownMenuSubContentProps = MenuSubContentProps;

/// # DropdownMenuSubContent
///
/// The contents of a [`DropdownMenuSub`]'s submenu -- a `role="menu"`,
/// only rendered while its own [`DropdownMenuSubTrigger`] reports it open.
/// This must be used inside a [`DropdownMenuSub`].
///
/// ## Styling
///
/// The [`DropdownMenuSubContent`] component defines the following data
/// attributes you can use to control styling:
/// - `data-state`: Indicates the current state of the submenu. Values are
///   `open` or `closed`.
/// - `data-open` / `data-closed`: Present (empty) while open / closed, absent otherwise.
#[component]
pub fn DropdownMenuSubContent(props: DropdownMenuSubContentProps) -> Element {
    menu_sub::render_sub_content(props)
}

/// The props for the [`DropdownMenuSubItem`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuItemProps`].
pub type DropdownMenuSubItemProps<T> = MenuItemProps<T>;

/// # DropdownMenuSubItem
///
/// An item within a [`DropdownMenuSubContent`]. Behaves like
/// [`DropdownMenuItem`], except selecting it closes the **entire** menu
/// tree (this submenu and every ancestor, including the root
/// [`DropdownMenu`]), not just this submenu -- matching Radix's default
/// `DropdownMenu.Sub` behavior, and reached by calling the *root* menu's
/// own `set_open` (found via `DropdownMenuContext`, not this submenu's own
/// `crate::menu_sub::SubMenuState`).
///
/// This must be used inside a [`DropdownMenuSubContent`].
///
/// ## Styling
///
/// The [`DropdownMenuSubItem`] component defines the following data
/// attributes you can use to control styling:
/// - `data-disabled`: Indicates whether the item is disabled. Values are
///   `true` or `false`.
/// - `data-highlighted`: Present (empty) while this item holds the roving focus, absent otherwise.
#[component]
pub fn DropdownMenuSubItem<T: Clone + PartialEq + 'static>(
    props: DropdownMenuSubItemProps<T>,
) -> Element {
    menu_item::render_item(props, "DropdownMenuSubItem")
}

/// `dev-docs/backlog.md` row 85 / hydration-parity Rule 4 coverage:
/// proves `DropdownMenu`'s root `div` renders a caller-supplied `id`
/// exactly once, with no second, internally-generated `dxc-N` id also
/// present. Without routing the root's attributes through `merge_attributes`
/// both `id="dxc-0"` and `id="my-menu-root"` would be present (2 total
/// `id="` occurrences); this is the same construction `context_menu.rs`'s
/// identical fixture proves.
#[cfg(test)]
mod ssr_tests {
    use super::*;

    #[component]
    fn DropdownMenuWithOwnId() -> Element {
        rsx! {
            DropdownMenu { id: "my-menu-root", "content" }
        }
    }

    fn render(component: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(component);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn callers_own_id_on_the_root_is_not_duplicated() {
        let html = render(DropdownMenuWithOwnId);
        assert_eq!(
            html.matches(" id=\"").count(),
            1,
            "expected exactly one `id` attribute on the root, got: {html}"
        );
        assert!(html.contains(r#"id="my-menu-root""#), "html: {html}");
        assert!(
            !html.contains("dxc-"),
            "the internally-generated id must not also render: {html}"
        );
    }
}
