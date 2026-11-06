//! Defines the [`ContextMenu`] component and its subcomponents, which provide a context menu interface.
//!
//! Also defines [`ContextMenuSub`]/[`ContextMenuSubTrigger`]/
//! [`ContextMenuSubContent`]/[`ContextMenuSubItem`], the nested-submenu
//! half of the APG "Menu and Menubar" pattern (`dev-docs/archive/rounds/component-backlog.md`
//! row 68).
//!
//! This file is the ContextMenu *host* (backlog row 435): its trigger (a
//! right-click or long-press target), its root (the dismiss layer and the focus/refocus
//! plumbing) and its point-anchored panel. The items, the checkable items,
//! the submenu and the panel wiring are the menu family every host shares
//! (`crate::menu_item`, `crate::menu_sub`, `crate::menu_root`), reading the
//! `MenuHost` this root provides; its items select on release
//! (`SelectOn::PressRelease`), so the long press that opened the menu and
//! lifts over an item selects nothing.

use crate::sleep::sleep;
use crate::state_attrs;
use crate::{
    collection::{use_collection_provider, CollectionState},
    direction::{use_direction, use_direction_signal, Direction},
    dismiss::{self, Cause, Policy, Scope},
    fold_style_attributes,
    menu_item::{
        self, MenuCheckboxItemProps, MenuGroupProps, MenuItemProps, MenuRadioGroupProps,
        MenuRadioItemProps, MenuSeparatorProps, MenuShortcutProps, MenuSubContentProps,
        MenuSubProps, MenuSubTriggerProps, SelectOn,
    },
    menu_root::{self, MenuHost, PanelConfig, PanelPlacement, PanelPopover},
    menu_sub, merge_attributes, use_controlled, use_id_or, use_unique_id,
};
use dioxus::prelude::*;
use dioxus_attributes::attributes;
use dioxus_core::Task;
use std::time::Duration;

/// How long a touch must be held before the context menu opens.
const LONG_PRESS_DURATION: Duration = Duration::from_millis(500);
/// Pointer drift (in CSS pixels, squared) that cancels an in-flight long press.
const LONG_PRESS_MOVE_TOLERANCE_SQ: f64 = 100.0;

#[derive(Clone, Copy)]
struct ContextMenuCtx {
    // State
    open: Memo<bool>,
    set_open: Callback<bool>,
    disabled: ReadSignal<bool>,

    // Whether the open menu should lock page scrolling. See
    // dev-docs/plan.md Phase 3.2.
    modal: ReadSignal<bool>,

    // Where the menu opened, as the press's `pageX`/`pageY` (document coordinates, the frame the
    // placement script works in) and, for the native arm, as `clientX`/`clientY`.
    position: Signal<(i32, i32)>,
    client_position: Signal<(i32, i32)>,

    // Focus state
    focus: CollectionState,

    // Id of the trigger element: `ContextMenuContent`'s `aria-labelledby`.
    trigger_id: Signal<String>,

    // The trigger element by handle, for `use_refocus_on_close_unless` (lib.rs) to return focus to
    // on close (see dev-docs/plan.md Phase 3.1), and the last press the menu saw, which says
    // whether that hand-back is skipped (touch, pen).
    trigger: crate::TriggerHandle,

    // Set just before a close caused by interacting outside the menu (see
    // `ContextMenuCtx::dismiss`), so the refocus hook knows to leave focus
    // where the user put it instead of yanking it back to the trigger.
    // Reset to `false` whenever the menu opens.
    interacted_outside: Signal<bool>,

    // Set briefly after a touch long-press opens the menu. It (a) swallows
    // Android Chrome's spurious `contextmenu` ~500ms later, and (b) ignores the
    // pointer re-dispatch Android sends to whatever element is now under the
    // still-held finger — that would otherwise look like an instant tap on the
    // menu item the menu just rendered over.
    long_press_just_fired: Signal<bool>,
}

impl ContextMenuCtx {
    /// Close the menu for `cause`: the roving focus is cleared, and `interacted_outside` says whether focus goes
    /// back to the trigger (it does not when the user acted somewhere else).
    fn dismiss(&mut self, cause: Cause) {
        self.interacted_outside.set(cause.interacted_outside());
        self.focus.clear_focus();
        self.set_open.call(false);
    }
}

/// The props for the [`ContextMenu`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuProps {
    /// Whether the context menu is disabled
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub disabled: ReadSignal<bool>,

    /// Whether the open menu should lock page scrolling, matching Radix's
    /// default. See dev-docs/plan.md Phase 3.2.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub modal: ReadSignal<bool>,

    /// Whether the context menu is open
    pub open: ReadSignal<Option<bool>>,

    /// Default open state
    #[props(default)]
    pub default_open: bool,

    /// Callback when open state changes
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// Whether focus should loop around when reaching the end.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// The text direction for a [`ContextMenuSub`]'s open/close arrow key
    /// and which side it opens on. Defaults to the nearest
    /// [`crate::direction::DirectionProvider`], or LTR if there is none.
    #[props(default)]
    pub dir: Option<Direction>,

    /// Additional attributes for the context menu element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the context menu component.
    pub children: Element,
}

/// # ContextMenu
///
/// The [`ContextMenu`] component is a container that can be used to create a context menu. You can
/// use the [`ContextMenuTrigger`] to open the menu on a right-click, and the [`ContextMenuContent`] to define the menu item.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::context_menu::{
///     ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         ContextMenu {
///             ContextMenuTrigger {
///                 "right click here"
///             }
///             ContextMenuContent {
///                 ContextMenuItem {
///                     value: "edit".to_string(),
///                     on_select: move |value| {
///                         tracing::info!("Selected item: {}", value);
///                     },
///                     "Edit"
///                 }
///                 ContextMenuItem {
///                     value: "undo".to_string(),
///                     disabled: true,
///                     on_select: move |value| {
///                         tracing::info!("Selected item: {}", value);
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
/// The [`ContextMenu`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates if the state of the context menu. Values are `open` or `closed`.
/// - `data-open` / `data-closed`: Present (empty) while open / closed, absent otherwise.
/// - `data-disabled`: Indicates if the context menu is disabled. values are `true` or `false`.
/// - `data-direction`: The resolved text direction. Values are `ltr` or `rtl`.
#[component]
pub fn ContextMenu(props: ContextMenuProps) -> Element {
    // See `DialogRoot`'s identical call for why this must be at the root,
    // not only inside `use_scroll_lock` (reached via `ScrollLockGuard`,
    // which mounts lazily inside `ContextMenuContent`'s own open guard).
    use_effect(crate::scroll_lock::ensure_scrollbar_gutter_baseline);

    let (open, set_open) = use_controlled(props.open, props.default_open, props.on_open_change);
    let direction = use_direction(props.dir);
    let direction_signal = use_direction_signal(direction);
    let position = use_signal(|| (0, 0));
    let client_position = use_signal(|| (0, 0));
    // Id on the root wrapper: covers both trigger and content, so the dismiss layer treats them as "inside".
    let root_id = menu_root::use_root_id(&props.attributes);
    let trigger_id = use_unique_id();
    let trigger = crate::use_trigger_handle();
    let interacted_outside = use_signal(|| false);
    let long_press_just_fired = use_signal(|| false);

    let focus = use_collection_provider(props.roving_loop);
    let submenu_open_count = use_signal(|| 0usize);
    let mut ctx = use_context_provider(|| ContextMenuCtx {
        open,
        set_open,
        disabled: props.disabled,
        modal: props.modal,
        position,
        client_position,
        focus,
        trigger_id,
        trigger,
        interacted_outside,
        long_press_just_fired,
    });

    // What the shared menu family (`menu_item`, `menu_sub`) reads from this host. A selection, or a root item
    // that loses focus, clears the roving focus; a selection also closes.
    let close = use_callback(move |()| {
        let mut focus = focus;
        focus.clear_focus();
        set_open.call(false);
    });
    let blur = use_callback(move |()| {
        let mut focus = focus;
        focus.clear_focus();
    });
    use_context_provider(|| MenuHost {
        open,
        disabled: props.disabled,
        focus,
        direction: direction_signal,
        submenu_open_count,
        select_on: SelectOn::PressRelease,
        close,
        blur,
    });

    // dev-docs/backlog.md row 53 (the "open-index container" band): this effect
    // is `DropdownMenu`'s own root effect
    // (`dropdown_menu.rs`), shared -- see `menu_root::use_open_focus_sync`'s own doc
    // for the shared construction and for why `Menubar`'s analogous
    // (index-keyed) effect stays out of it.
    crate::menu_root::use_open_focus_sync(focus, open, set_open, submenu_open_count);

    // A fresh open shouldn't inherit an `interacted_outside` flag left over
    // from a previous close.
    use_effect(move || {
        if (ctx.open)() {
            ctx.interacted_outside.set(false);
        }
    });

    // Escape and item-select close from inside the menu; Radix's
    // `onCloseAutoFocus` semantics restore focus to the trigger for those,
    // but not for a dismiss caused by interacting outside (the dismiss
    // layer below sets `interacted_outside` from what caused the close).
    crate::use_refocus_on_close_unless(
        ctx.open,
        ctx.trigger,
        ReadSignal::new(ctx.interacted_outside),
    );

    // One layer for the open menu, on the root: Escape, a press outside and focus leaving all close it
    // (`Policy::MENU`), and what caused the close decides whether focus goes back to the trigger. The panel is a
    // `manual` popover, which the platform never closes on its own, so the layer is the only dismissal path and
    // the `toggle` echo of `use_menu_panel` only repeats a close this crate already made.
    dismiss::use_dismiss(
        root_id,
        Scope::Element,
        Policy::MENU,
        Some(open.into()),
        move |cause| {
            let mut ctx = ctx;
            ctx.dismiss(cause);
        },
    );

    let handle_keydown = move |event: Event<KeyboardData>| {
        ctx.trigger.press().key_pressed(&event);
    };

    // Merged (caller-wins, deduped), not set-then-spread-over: see
    // `toast.rs`'s "Attribute-override dedup" doc for why an explicit
    // attribute followed by `..props.attributes` on the same element emits
    // a duplicate in the SSR'd HTML whenever a caller overrides it (the
    // top-layer oracle fixture's `id: "clip-context-menu-root"`/
    // `id: "scroll-context-menu-root"` do exactly that) --
    // `dev-docs/conformance-harness.md` hydration-parity Rule 4. `id` is
    // caller-wins here like everything else, matching the client behavior
    // of a plain spread (`..props.attributes` wins by spread order)
    // rather than protecting it the way `toast.rs`
    // protects its always-addressed-by-id region: nothing here does its
    // own `document.getElementById(root_id)` lookup against a *fixed*
    // expectation of winning -- `menu_root::use_root_id` writes a caller's
    // override into `root_id`, so the dismiss layer's lookup finds the
    // element that really carries the id.
    let attributes = merge_attributes(vec![
        attributes!(div {
            id: root_id,
            tabindex: 0,
            dir: direction.as_str(),
            ..state_attrs::open(open()),
            ..state_attrs::disabled((props.disabled)()),
            "data-direction": direction.as_str(),
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

/// The props for the [`ContextMenuTrigger`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuTriggerProps {
    /// The ID of the trigger element. If not provided, an internally
    /// generated ID is used -- see `ContextMenuTrigger`'s use of
    /// `use_id_or` for why this is a typed field rather than left to
    /// `GlobalAttributes` alone (dev-docs/backlog.md row 36).
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes for the context menu trigger element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the context menu trigger.
    pub children: Element,
}

/// # ContextMenuTrigger
///
/// The [`ContextMenuTrigger`] component is used to define the element that will trigger the context menu when right-clicked.
///
/// This must be used inside a [`ContextMenu`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::context_menu::{
///     ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         ContextMenu {
///             ContextMenuTrigger {
///                 "right click here"
///             }
///             ContextMenuContent {
///                 ContextMenuItem {
///                     value: "edit".to_string(),
///                     on_select: move |value| {
///                         tracing::info!("Selected item: {}", value);
///                     },
///                     "Edit"
///                 }
///                 ContextMenuItem {
///                     value: "undo".to_string(),
///                     disabled: true,
///                     on_select: move |value| {
///                         tracing::info!("Selected item: {}", value);
///                     },
///                     "Undo"
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn ContextMenuTrigger(props: ContextMenuTriggerProps) -> Element {
    let ctx: ContextMenuCtx = use_context();
    // dev-docs/backlog.md row 36: `ctx.trigger_id` is the same signal
    // `ContextMenuContent`'s `aria-labelledby` (below) and
    // `use_refocus_on_close_unless` both read back -- a caller-supplied
    // `id` here must resolve into it, not just win the render. Feeding it
    // straight into `use_id_or` as the generated-id signal does that:
    // `use_id_or`'s own effect writes a caller override back into its
    // `gen_id` argument, which *is* `ctx.trigger_id` here -- mirrors
    // `DialogTitle`'s identical `use_id_or(ctx.dialog_labelledby,
    // props.id)` (`dialog.rs`).
    let id = use_id_or(ctx.trigger_id, props.id);
    // iOS Safari does not deliver `contextmenu` from a long-press on touch, so
    // we run a manual timer keyed on the initial touch position and fire it
    // ourselves once the finger has held still long enough.
    let mut long_press_task: Signal<Option<Task>> = use_signal(|| None);
    let mut long_press_start: Signal<Option<(f64, f64)>> = use_signal(|| None);
    let mut long_press_just_fired = ctx.long_press_just_fired;

    let cancel_long_press =
        move |mut task: Signal<Option<Task>>, mut start: Signal<Option<(f64, f64)>>| {
            if let Some(t) = task.write().take() {
                t.cancel();
            }
            start.set(None);
        };

    let handle_context_menu = move |event: Event<MouseData>| {
        if !(ctx.disabled)() {
            // Android Chrome dispatches `contextmenu` ~500ms after a touch long
            // press, which can race our own timer. Defuse the race so only one
            // open lands.
            cancel_long_press(long_press_task, long_press_start);
            if long_press_just_fired.cloned() {
                // Timer already opened the menu; suppress the browser context
                // menu but don't open a second time. Leave the flag set —
                // the timer task clears it ~700ms after the menu opened, and
                // the menu items rely on that window to ignore Android's
                // re-dispatched pointerdown/up on the element under the
                // still-held finger.
                event.prevent_default();
                return;
            }
            let data = event.data();
            let (page, client) = (data.page_coordinates(), data.client_coordinates());
            let mut position = ctx.position;
            let mut client_position = ctx.client_position;
            position.set((page.x as i32, page.y as i32));
            client_position.set((client.x as i32, client.y as i32));
            ctx.set_open.call(true);
            event.prevent_default();
        }
    };

    // The dismiss layer registers a moment after the menu opens; an Escape in that gap is the trigger's, which holds
    // focus after a right click (`dismiss::escape_in_gap`).
    let handle_key_down = move |event: Event<KeyboardData>| {
        let mut ctx = ctx;
        dismiss::escape_in_gap(&event, (ctx.open)(), || ctx.dismiss(Cause::EscapeWithin));
    };

    let handle_pointer_down = move |event: Event<PointerData>| {
        // Long-press fires for touch and pen (Apple Pencil etc.); mouse keeps
        // using the native `contextmenu` event.
        if event.pointer_type() == "mouse" || (ctx.disabled)() {
            return;
        }
        cancel_long_press(long_press_task, long_press_start);
        let p = event.client_coordinates();
        let page = event.page_coordinates();
        long_press_start.set(Some((p.x, p.y)));
        let set_open = ctx.set_open;
        let mut position = ctx.position;
        let mut client_position = ctx.client_position;
        let task = spawn(async move {
            sleep(LONG_PRESS_DURATION).await;
            long_press_task.set(None);
            position.set((page.x as i32, page.y as i32));
            client_position.set((p.x as i32, p.y as i32));
            set_open.call(true);
            // Stay armed long enough to catch Android's compat `contextmenu`,
            // then disarm so future mouse right-clicks aren't suppressed.
            long_press_just_fired.set(true);
            sleep(Duration::from_millis(700)).await;
            long_press_just_fired.set(false);
        });
        long_press_task.set(Some(task));
    };

    let handle_pointer_move = move |event: Event<PointerData>| {
        let Some((sx, sy)) = long_press_start.cloned() else {
            return;
        };
        let p = event.client_coordinates();
        let dx = p.x - sx;
        let dy = p.y - sy;
        if dx * dx + dy * dy > LONG_PRESS_MOVE_TOLERANCE_SQ {
            cancel_long_press(long_press_task, long_press_start);
        }
    };

    let handle_pointer_end = move |_event: Event<PointerData>| {
        cancel_long_press(long_press_task, long_press_start);
    };

    // Merged (caller-wins, deduped) for the same reason as `ContextMenuRoot`
    // above -- the fixture's `id: "clip-context-menu-trigger"`/
    // `id: "scroll-context-menu-trigger"` would duplicate `id` in the SSR'd
    // HTML without it. `style` needs the extra `fold_style_attributes` step
    // first: this element's own fixed touch-suppression CSS is one literal
    // `style: "..."` string, and the preview app's themed wrapper
    // (`preview/src/components/context_menu/component.rs`) sets its
    // background/padding/etc. the *shorthand* way -- those combine into a
    // SEPARATE served `style="..."` from this one (see that helper's doc),
    // so plain `merge_attributes` alone leaves two. Folding first
    // reduces it to the one case `merge_attributes` already knows how to
    // dedupe (a plain `style` on both sides).
    let (caller_style, attributes) = fold_style_attributes(props.attributes);
    let style = match caller_style {
        Some(caller) => format!(
            "-webkit-touch-callout: none; user-select: none; -webkit-user-select: none; -webkit-tap-highlight-color: transparent; touch-action: none; {caller}"
        ),
        None => "-webkit-touch-callout: none; user-select: none; -webkit-user-select: none; -webkit-tap-highlight-color: transparent; touch-action: none;".to_string(),
    };
    let attributes = merge_attributes(vec![
        attributes!(div {
            // Not otherwise part of the tab order (the context menu opens via
            // right-click/long-press, not Tab) -- but it must be
            // programmatically focusable so Escape/select can return focus
            // here per APG's menu-button-adjacent close-focus rule. See
            // dev-docs/plan.md Phase 3.1.
            id: id.cloned(),
            tabindex: "-1",
            role: "button",
            aria_haspopup: crate::menu_semantics::MENU_TRIGGER_HASPOPUP,
            aria_expanded: (ctx.open)(),
            onmounted: ctx.trigger.onmounted(),
            // Suppress iOS Safari's long-press behaviors (callout sheet, text
            // selection magnifier, gray tap-flash) and the system's own touch
            // gestures so our timer is the only thing that fires -- combined
            // above with any caller style so only one `style` is ever built.
            style,
        }),
        attributes,
    ]);

    rsx! {
        div {
            oncontextmenu: handle_context_menu,
            onkeydown: handle_key_down,
            onpointerdown: handle_pointer_down,
            onpointermove: handle_pointer_move,
            onpointerup: handle_pointer_end,
            onpointercancel: handle_pointer_end,
            ..attributes,
            {props.children}
        }
    }
}

/// The props for the [`ContextMenuContent`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuContentProps {
    /// The ID of the context menu content element.
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes for the context menu content element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the context menu content.
    pub children: Element,
}

/// # ContextMenuContent
///
/// The [`ContextMenuContent`] component is used to define the content of the context menu. It is only rendered
/// when the context menu is open.
///
/// This must be used inside a [`ContextMenu`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::context_menu::{
///     ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         ContextMenu {
///             ContextMenuTrigger {
///                 "right click here"
///             }
///             ContextMenuContent {
///                 ContextMenuItem {
///                     value: "edit".to_string(),
///                     on_select: move |value| {
///                         tracing::info!("Selected item: {}", value);
///                     },
///                     "Edit"
///                 }
///                 ContextMenuItem {
///                     value: "undo".to_string(),
///                     disabled: true,
///                     on_select: move |value| {
///                         tracing::info!("Selected item: {}", value);
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
/// The [`ContextMenuContent`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates if the state of the context menu. Values are `open` or `closed`.
/// - `data-open` / `data-closed`: Present (empty) while open / closed, absent otherwise.
#[component]
pub fn ContextMenuContent(props: ContextMenuContentProps) -> Element {
    let ctx: ContextMenuCtx = use_context();
    let open = ctx.open;

    // dev-docs/backlog.md row 53 (the "content lifecycle/positioning pipeline"
    // band): id resolution + `use_animated_open`'s render gate, identical
    // across all three menu-family hosts' `*Content` components -- see
    // `menu_root::use_menu_content_lifecycle`'s own doc.
    let (id, render) = crate::menu_root::use_menu_content_lifecycle(props.id, open);

    // Lock page scroll while the menu is open and modal, matching Radix's
    // default. See dev-docs/plan.md Phase 3.2. `ContextMenuContent` itself
    // never unmounts (only the rendered content below does, via `render()`),
    // so the lock is held by `ScrollLockGuard` -- a child mounted inside
    // that same conditional -- rather than by this component directly; see
    // that guard's doc comment.
    let modal = ctx.modal;
    let scroll_lock_active = use_memo(move || modal() && open());

    rsx! {
        if render() {
            ContextMenuContentRendered {
                id: id.cloned(),
                attributes: props.attributes,
                scroll_lock_active,
                children: props.children,
            }
        }
    }
}

/// The menu panel. On the web it is promoted to the top layer via `popover="manual"`, so it escapes clipping and
/// transformed ancestors the same way `DropdownMenuContentRendered` does (`dropdown_menu.rs`); without the popover
/// API it is a plain `position: fixed` div.
///
/// The menu opens at a document point (`ContextMenuCtx::position`, the press's `pageX`/`pageY`), with no trigger
/// element to anchor to (`anchor-name`/`position-anchor` are never wired up for it).
/// `top_layer::use_point_anchor_placement` places it with the same script as every anchored overlay (rule P,
/// backlog row 477): the point is a zero-size anchor, the menu is absolute in the top layer and moved by
/// `transform`, so it stays at that point of the document when the page scrolls, opens upward or leftward when the
/// room below or to the right is short, and is capped to the room left. Until the script takes the panel over it
/// is `position: fixed` at the press's client point: a focus move into a panel that is not placed yet must not
/// scroll the document to it.
///
/// `manual` (not `auto`) is also the dismissal decision: `auto`'s native light dismiss (Escape, outside
/// pointerdown) would race this component's own dismiss layer, which
/// implements APG's exact close-and-refocus semantics (Radix's `onCloseAutoFocus`, `interacted_outside`
/// distinguishing an outside dismiss from an internal one -- see `ContextMenuCtx::interacted_outside`'s doc). A
/// native light dismiss has no way to tell Rust *why* it closed, so marking every native close as an outside
/// interaction would silently break `use_refocus_on_close_unless`'s Escape/select refocus. A `manual` popover is
/// never closed by the platform on its own, so the dismiss layer stays the *only*
/// dismissal path, and the `toggle` echo of `use_menu_panel` only ever repeats a close this crate already made.
#[component]
fn ContextMenuContentRendered(
    /// The DOM id of the menu's content element.
    id: String,
    /// Extra HTML attributes for the menu's content element.
    attributes: Vec<Attribute>,
    /// Whether page scrolling is locked while the menu is open. True only for an open modal menu.
    scroll_lock_active: Memo<bool>,
    /// The menu items.
    children: Element,
) -> Element {
    let mut ctx: ContextMenuCtx = use_context();
    let open = ctx.open;
    let (x, y) = (ctx.position)();
    let (cx, cy) = (ctx.client_position)();

    // One typeahead buffer for this menu's own items (`menu_root::nav_key`).
    let mut typeahead = crate::typeahead::use_typeahead_state();

    let onkeydown = move |event: Event<KeyboardData>| {
        if !open() || !menu_root::nav_key(&event, ctx.focus, &mut typeahead) {
            // While closed (or mid close-animation) `ctx.focus` may still carry whatever it last held, so a
            // keystroke on an unrelated, no-longer-visible menu is ignored.
            return;
        }
        event.prevent_default();
    };

    let mut menu_ref: Signal<Option<std::rc::Rc<MountedData>>> = use_signal(|| None);
    let focused = move || open() && !ctx.focus.any_focused();
    // If the menu is open, but no item is focused, focus the div itself to capture events
    use_effect(move || {
        let Some(menu) = menu_ref() else {
            return;
        };
        if focused() {
            spawn(async move {
                // Focus the menu itself to capture keyboard events
                // focus-ok: the menu container takes focus for its keys (a container: no outline, no editable)
                _ = menu.set_focus(true).await;
            });
        }
    });

    // The `toggle` echo only repeats a close this crate already made (see the doc above).
    let on_native_toggle = use_callback(move |is_open: bool| ctx.set_open.call(is_open));
    let attributes = menu_root::use_menu_panel(
        PanelConfig {
            id: id.clone(),
            open,
            trigger_id: ctx.trigger_id.cloned(),
            on_native_toggle,
            popover: PanelPopover::Manual,
            placement: PanelPlacement::Point { x, y },
            owned: attributes!(div {
                aria_orientation: "vertical",
                tabindex: if focused() { "0" } else { "-1" },
            }),
        },
        attributes,
    );

    rsx! {
        div {
            id,
            position: "fixed",
            margin: "0",
            top: "{cy}px",
            left: "{cx}px",
            right: "auto",
            bottom: "auto",
            pointer_events: open().then_some("auto"),
            onkeydown,
            onblur: move |_| {
                if focused() {
                    ctx.focus.clear_focus();
                }
            },
            onmounted: move |evt| menu_ref.set(Some(evt.data())),
            ..attributes,

            crate::scroll_lock::ScrollLockGuard { active: scroll_lock_active }
            {children}
        }
    }
}

/// The props for the [`ContextMenuItem`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuItemProps`] (a `String`
/// value here).
pub type ContextMenuItemProps = MenuItemProps<String>;

/// # ContextMenuItem
///
/// The [`ContextMenuItem`] component defines an individual item in the context menu. Its `index` is optional: items are
/// ordered for keyboard navigation as they are created (tree order), so set it only when an item is
/// inserted between mounted items or a keyed list reorders them. It never changes where an item is drawn.
///
/// When an item is selected with either the pointer or the keyboard, the menu is closed and the `on_select` callback is called with the item's value.
///
/// This must be used inside a [`ContextMenuContent`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::context_menu::{
///     ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         ContextMenu {
///             ContextMenuTrigger {
///                 "right click here"
///             }
///             ContextMenuContent {
///                 ContextMenuItem {
///                     value: "edit".to_string(),
///                     on_select: move |value| {
///                         tracing::info!("Selected item: {}", value);
///                     },
///                     "Edit"
///                 }
///                 ContextMenuItem {
///                     value: "undo".to_string(),
///                     disabled: true,
///                     on_select: move |value| {
///                         tracing::info!("Selected item: {}", value);
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
/// The [`ContextMenuItem`] component defines the following data attributes you can use to control styling:
/// - `data-disabled`: Indicates if the item is disabled. Possible values are `true` or `false`.
/// - `data-highlighted`: Present (empty) while this item holds the roving focus, absent otherwise.
#[component]
pub fn ContextMenuItem(props: ContextMenuItemProps) -> Element {
    menu_item::render_item(props, "ContextMenuItem")
}

/// The props for the [`ContextMenuCheckboxItem`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuCheckboxItemProps`].
pub type ContextMenuCheckboxItemProps = MenuCheckboxItemProps;

/// The props for the [`ContextMenuRadioGroup`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuRadioGroupProps`].
pub type ContextMenuRadioGroupProps = MenuRadioGroupProps;

/// The props for the [`ContextMenuRadioItem`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuRadioItemProps`].
pub type ContextMenuRadioItemProps = MenuRadioItemProps;

/// # ContextMenuCheckboxItem
///
/// An item within a [`ContextMenuContent`] (or a [`ContextMenuSubContent`])
/// that can be checked and unchecked independently of its siblings --
/// shadcn's `ContextMenuCheckboxItem`. Renders `role="menuitemcheckbox"` with
/// an always-present `aria-checked` (see `crate::menu_semantics`).
///
/// Controlled with `checked` + `on_checked_change`, or uncontrolled with
/// `default_checked`. Pressing it with the pointer, or pressing Enter or
/// Space while it has focus, toggles it. Like Base UI's `closeOnClick = false`,
/// that **keeps the menu open** so several items can be toggled in one visit
/// (APG: "changes the state without closing the menu"); set
/// `close_on_select: true` for Radix's `onSelect` behaviour, which closes it.
/// Disabled items cannot be toggled and are skipped by arrow keys and
/// typeahead. A checkbox item has no `value`, so give it a `text_value` to
/// make it a typeahead target.
///
/// This must be used inside a [`ContextMenuContent`] component.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::context_menu::{
///     ContextMenu, ContextMenuCheckboxItem, ContextMenuContent, ContextMenuTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     let mut show_bookmarks = use_signal(|| true);
///     rsx! {
///         ContextMenu {
///             ContextMenuTrigger { "right click here" }
///             ContextMenuContent {
///                 ContextMenuCheckboxItem {
///                     checked: Some(show_bookmarks()),
///                     on_checked_change: move |checked| show_bookmarks.set(checked),
///                     text_value: "Show Bookmarks",
///                     "Show Bookmarks"
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`ContextMenuCheckboxItem`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Whether the item is checked. Values are `checked` or `unchecked`.
/// - `data-checked` / `data-unchecked`: Present (empty) while checked / unchecked, absent otherwise.
/// - `data-disabled`: Indicates if the item is disabled. Possible values are `true` or `false`.
/// - `data-highlighted`: Present (empty) while this item holds the roving focus, absent otherwise.
#[component]
pub fn ContextMenuCheckboxItem(props: ContextMenuCheckboxItemProps) -> Element {
    let state = menu_item::use_checkbox_state(
        props.checked,
        props.default_checked,
        props.on_checked_change,
    );
    menu_item::render_checkable(state, props.into())
}

/// # ContextMenuRadioGroup
///
/// A set of mutually exclusive [`ContextMenuRadioItem`]s -- shadcn's
/// `ContextMenuRadioGroup`. Renders `role="group"`; give it an accessible
/// name with `aria-label` or `aria-labelledby`. Controlled with `value` +
/// `on_value_change`, or uncontrolled with `default_value`.
///
/// This must be used inside a [`ContextMenuContent`] component.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::context_menu::{
///     ContextMenu, ContextMenuContent, ContextMenuRadioGroup, ContextMenuRadioItem,
///     ContextMenuTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     let mut person = use_signal(|| "pedro".to_string());
///     rsx! {
///         ContextMenu {
///             ContextMenuTrigger { "right click here" }
///             ContextMenuContent {
///                 ContextMenuRadioGroup {
///                     aria_label: "People",
///                     value: Some(person()),
///                     on_value_change: move |value| person.set(value),
///                     ContextMenuRadioItem {
///                         value: "pedro".to_string(),
///                         "Pedro Duarte"
///                     }
///                     ContextMenuRadioItem {
///                         value: "colm".to_string(),
///                         "Colm Tuite"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn ContextMenuRadioGroup(props: ContextMenuRadioGroupProps) -> Element {
    menu_item::render_radio_group(props)
}

/// # ContextMenuRadioItem
///
/// One choice in a [`ContextMenuRadioGroup`] -- shadcn's
/// `ContextMenuRadioItem`. Renders `role="menuitemradio"` with an
/// always-present `aria-checked`; it is checked while the group's value
/// equals its own `value`, and choosing it (pointer, Enter or Space) sets
/// the group's value and, like Base UI's `closeOnClick = false`, **keeps the
/// menu open** unless `close_on_select` is `true`. Falls back to `value` as its
/// typeahead label. Disabled items cannot be chosen and are skipped by arrow
/// keys and typeahead.
///
/// This must be used inside a [`ContextMenuRadioGroup`] component.
///
/// ## Styling
///
/// The [`ContextMenuRadioItem`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Whether the item is checked. Values are `checked` or `unchecked`.
/// - `data-checked` / `data-unchecked`: Present (empty) while checked / unchecked, absent otherwise.
/// - `data-disabled`: Indicates if the item is disabled. Possible values are `true` or `false`.
/// - `data-highlighted`: Present (empty) while this item holds the roving focus, absent otherwise.
#[component]
pub fn ContextMenuRadioItem(props: ContextMenuRadioItemProps) -> Element {
    let state = menu_item::use_radio_state(props.value);
    menu_item::render_checkable(state, props.into())
}

/// The props for the [`ContextMenuSeparator`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuSeparatorProps`].
pub type ContextMenuSeparatorProps = MenuSeparatorProps;

/// The props for the [`ContextMenuGroup`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuGroupProps`].
pub type ContextMenuGroupProps = MenuGroupProps;

/// The props for the [`ContextMenuShortcut`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuShortcutProps`].
pub type ContextMenuShortcutProps = MenuShortcutProps;

/// # ContextMenuSeparator
///
/// A divider between groups of items -- shadcn's `ContextMenuSeparator`. Renders
/// `role="separator"` with `aria-orientation="horizontal"`. It is not
/// focusable and takes no `index`: arrow keys, Home/End and typeahead skip
/// it.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::context_menu::{ContextMenuSeparator};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         ContextMenuSeparator {}
///     }
/// }
/// ```
#[component]
pub fn ContextMenuSeparator(props: ContextMenuSeparatorProps) -> Element {
    menu_item::render_separator(props)
}

/// # ContextMenuGroup
///
/// A `role="group"` around related items -- shadcn's `ContextMenuGroup`. It adds
/// no behaviour: its items still take part in the enclosing menu's keyboard
/// navigation and typeahead. Name it with `aria-label` when it has a visible
/// heading (a label).
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::context_menu::{ContextMenuGroup};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         ContextMenuGroup { aria_label: "File", "..." }
///     }
/// }
/// ```
#[component]
pub fn ContextMenuGroup(props: ContextMenuGroupProps) -> Element {
    menu_item::render_group(props)
}

/// # ContextMenuShortcut
///
/// A keyboard-shortcut hint at the inline end of an item -- shadcn's
/// `ContextMenuShortcut`. Text only: it renders a `span` and does not bind any
/// key.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::context_menu::{ContextMenuShortcut};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         ContextMenuShortcut { "⌘T" }
///     }
/// }
/// ```
#[component]
pub fn ContextMenuShortcut(props: ContextMenuShortcutProps) -> Element {
    menu_item::render_shortcut(props)
}

/// The props for the [`ContextMenuSub`] component. Shared by every
/// menu-family host with submenus -- see [`crate::menu_item::MenuSubProps`].
pub type ContextMenuSubProps = MenuSubProps;

/// # ContextMenuSub
///
/// A submenu nested inside a [`ContextMenu`], implementing the APG "Menu
/// and Menubar" pattern's nested-submenu contract -- what that page calls
/// a "parent menuitem" opening a submenu
/// (`content/patterns/menubar/menu-and-menubar-pattern.html`, "Keyboard
/// Interaction" and "WAI-ARIA Roles, States, and Properties" (both h2),
/// pinned commit `7e4034b262bc0d25332e330d8a582aaf34113829` of
/// `w3c/aria-practices` -- see `playwright/oracle/reference/README.md`;
/// [`crate::dropdown_menu::DropdownMenuSubTrigger`]'s own doc carries the
/// exact quotes). Renders no element of its
/// own -- purely a context boundary around a [`ContextMenuSubTrigger`] and
/// a [`ContextMenuSubContent`], the same shape
/// [`crate::dropdown_menu::DropdownMenuSub`] has.
///
/// The state machine is shared with `DropdownMenuSub` via
/// `crate::menu_sub` -- see that module's doc for exactly what is shared
/// (the open/close state, this submenu's own roving-focus collection, the
/// hover-intent timers) and what stays file-local (which enclosing
/// collection a trigger registers in -- here, `ContextMenuCtx`'s
/// `focus`). **Scope: one level of nesting**, same as `DropdownMenuSub` --
/// see `crate::menu_sub`'s module doc for why.
///
/// Unlike [`ContextMenuContent`] (pinned to a raw click point, never
/// anchored -- see that component's own doc), this submenu's own content
/// *is* anchored, to its [`ContextMenuSubTrigger`] -- an ordinary menu
/// item sitting inside the already-positioned root content, the same as
/// `DropdownMenuSub`'s content anchors to its own trigger.
///
/// This must be used inside a [`ContextMenuContent`].
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::context_menu::{
///     ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuSub,
///     ContextMenuSubContent, ContextMenuSubItem, ContextMenuSubTrigger,
///     ContextMenuTrigger,
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         ContextMenu {
///             ContextMenuTrigger { "right click here" }
///             ContextMenuContent {
///                 ContextMenuItem {
///                     value: "edit".to_string(),
///                     on_select: move |value| {
///                         tracing::info!("Selected item: {}", value);
///                     },
///                     "Edit"
///                 }
///                 ContextMenuSub {
///                     ContextMenuSubTrigger { "More tools" }
///                     ContextMenuSubContent {
///                         ContextMenuSubItem {
///                             value: "duplicate".to_string(),
///                             on_select: move |value| {
///                                 tracing::info!("Selected item: {}", value);
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
pub fn ContextMenuSub(props: ContextMenuSubProps) -> Element {
    menu_sub::render_sub(props)
}

/// The props for the [`ContextMenuSubTrigger`] component. Shared by every
/// menu-family host with submenus -- see [`crate::menu_item::MenuSubTriggerProps`].
pub type ContextMenuSubTriggerProps = MenuSubTriggerProps;

/// # ContextMenuSubTrigger
///
/// The trigger for a [`ContextMenuSub`]'s submenu -- see
/// [`crate::dropdown_menu::DropdownMenuSubTrigger`]'s doc for the full APG
/// citation (`aria-haspopup`/`aria-expanded`/`aria-controls`, the
/// ArrowRight/Enter/Space keyboard contract, and the hover-intent/close-
/// grace timing), which this component mirrors exactly against
/// `ContextMenuCtx`'s collection instead of `DropdownMenuContext`'s. Hover
/// opening is on by default; pass `open_on_hover: false` to
/// [`ContextMenuSub`] for a click/keyboard-activated submenu.
///
/// This must be used inside a [`ContextMenuSub`].
///
/// ## Styling
///
/// The [`ContextMenuSubTrigger`] component defines the following data
/// attributes you can use to control styling:
/// - `data-state`: Indicates whether the submenu is open. Values are
///   `open` or `closed`.
/// - `data-open` / `data-closed`: Present (empty) while open / closed, absent otherwise.
/// - `data-popup-open`: Present (empty) while the popup this trigger controls is open.
/// - `data-disabled`: Indicates if the sub-trigger is disabled. Values are
///   `true` or `false`.
#[component]
pub fn ContextMenuSubTrigger(props: ContextMenuSubTriggerProps) -> Element {
    menu_sub::render_sub_trigger(props)
}

/// The props for the [`ContextMenuSubContent`] component. Shared by every
/// menu-family host with submenus -- see [`crate::menu_item::MenuSubContentProps`].
pub type ContextMenuSubContentProps = MenuSubContentProps;

/// # ContextMenuSubContent
///
/// The contents of a [`ContextMenuSub`]'s submenu -- a `role="menu"`, only
/// rendered while its own [`ContextMenuSubTrigger`] reports it open. This
/// must be used inside a [`ContextMenuSub`].
///
/// ## Styling
///
/// The [`ContextMenuSubContent`] component defines the following data
/// attributes you can use to control styling:
/// - `data-state`: Indicates the current state of the submenu. Values are
///   `open` or `closed`.
/// - `data-open` / `data-closed`: Present (empty) while open / closed, absent otherwise.
#[component]
pub fn ContextMenuSubContent(props: ContextMenuSubContentProps) -> Element {
    menu_sub::render_sub_content(props)
}

/// The props for the [`ContextMenuSubItem`] component. Shared by every
/// menu-family host -- see [`crate::menu_item::MenuItemProps`] (a `String`
/// value here).
pub type ContextMenuSubItemProps = MenuItemProps<String>;

/// # ContextMenuSubItem
///
/// An item within a [`ContextMenuSubContent`]. Behaves like
/// [`ContextMenuItem`], except selecting it closes the **entire** menu
/// tree (this submenu and every ancestor, including the root
/// [`ContextMenu`]), not just this submenu -- matching
/// [`crate::dropdown_menu::DropdownMenuSubItem`]'s identical default, and
/// reached the same way: by calling the *root* menu's own `set_open`
/// (found via `ContextMenuCtx`, not this submenu's own
/// `crate::menu_sub::SubMenuState`).
///
/// This must be used inside a [`ContextMenuSubContent`].
///
/// ## Styling
///
/// The [`ContextMenuSubItem`] component defines the following data
/// attributes you can use to control styling:
/// - `data-disabled`: Indicates if the item is disabled. Possible values
///   are `true` or `false`.
/// - `data-highlighted`: Present (empty) while this item holds the roving focus, absent otherwise.
#[component]
pub fn ContextMenuSubItem(props: ContextMenuSubItemProps) -> Element {
    menu_item::render_item(props, "ContextMenuSubItem")
}
