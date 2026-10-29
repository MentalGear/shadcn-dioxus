//! SelectList component implementation.

use crate::{
    has_own_accessible_name, listbox::use_listbox_container, merge_attributes, use_effect,
};
use dioxus::prelude::*;
use dioxus_attributes::attributes;

use super::super::context::SelectContext;
#[cfg(feature = "web")]
use super::super::context::SelectPosition;
use super::scroll_button::ListScroll;

/// The props for the [`SelectList`] component
#[derive(Props, Clone, PartialEq)]
pub struct SelectListProps {
    /// The ID of the list for ARIA attributes
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes for the list
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children to render inside the list
    pub children: Element,
}

/// # SelectList
///
/// The dropdown list container for the [`Select`](super::select::Select) component that contains the
/// [`SelectOption`](super::option::SelectOption)s. The list will only be rendered when the select is open.
///
/// This must be used inside a [`Select`](super::select::Select) component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::select::{
///     Select, SelectGroup, SelectGroupLabel, SelectItemIndicator, SelectList, SelectOption,
///     SelectTrigger, SelectValue,
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Select::<String> {
///             item_to_label: |value: String| value,
///             SelectTrigger {
///                 aria_label: "Select Trigger",
///                 width: "12rem",
///                 SelectValue { placeholder: "Select a fruit..." }
///             }
///             SelectList {
///                 aria_label: "Select Demo",
///                 SelectGroup {
///                     SelectGroupLabel { "Fruits" }
///                     SelectOption::<String> {
///                         value: "apple",
///                         "Apple"
///                         SelectItemIndicator { "✔️" }
///                     }
///                     SelectOption::<String> {
///                         value: "banana",
///                         "Banana"
///                         SelectItemIndicator { "✔️" }
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn SelectList(props: SelectListProps) -> Element {
    let ctx = use_context::<SelectContext>();

    let listbox = use_listbox_container(props.id, ctx.selectable);
    let render = listbox.render;

    // A touch open's popper placement lasts until the list unmounts (the exit animation keeps the
    // placement it opened with); the next open decides afresh.
    let mut touch_open = ctx.touch_open;
    use_effect(move || {
        if !render() {
            touch_open.set(false);
        }
    });

    rsx! {
        if render() {
            SelectListRendered {
                id: listbox.id.cloned(),
                attributes: props.attributes,
                children: props.children,
            }
        } else {
            // If not rendering, return children directly so we can populate the selected list, but they should choose to not render themselves
            {props.children}
        }
    }
}

/// The item-aligned placement script, in document coordinates (`PLACEMENT_HELPERS_JS` is spliced in
/// front of it). The list is absolute in the top layer and moved by `transform` alone, so the page
/// carries it when it scrolls. It reads the geometry it needs in untransformed layout units
/// (`offsetTop`/`offsetHeight`, never a rect of the list: the pop-in animates `scale`), puts the
/// selected option's centre on the trigger's centre, then keeps the list inside the visible region
/// by moving it and, past that, scrolling the option away from the trigger line. With no selected
/// option there is nothing to align, so the whole list sits below the trigger and flips above it on
/// collision, the popper's rule (`data-side` names the side for the CSS).
#[cfg(feature = "web")]
const ITEM_ALIGNED_JS: &str = r#"
const [id, triggerId] = await dioxus.recv();
const list = document.getElementById(id);
const trigger = document.querySelector('[style*="anchor-name: --dxa-' + triggerId + ';"]');
if (!list || !trigger) return;
const MARGIN = 8;
const GAP = 4;
const put = writer(list);
const origin = originOf(list);
const clamp = (n, lo, hi) => Math.min(Math.max(n, lo), Math.max(lo, hi));
const offsetIn = (el) => {
    let y = 0;
    for (let n = el; n && n !== list; n = n.offsetParent) y += n.offsetTop;
    return y;
};
const place = () => {
    const v = visible();
    const a = docRect(trigger, v);
    const selected = list.querySelector('[role="option"][aria-selected="true"]');
    for (const [prop, value] of [['position', 'absolute'], ['inset', 'auto'], ['margin', '0'], ['top', '0'], ['left', '0']]) {
        list.style.setProperty(prop, value);
    }
    list.style.setProperty('--dx-anchor-width', a.r - a.l + 'px');
    list.style.setProperty('--dx-available-height', Math.max(0, v.b - v.t - 2 * MARGIN) + 'px');
    let ch = list.offsetHeight;
    let side = null;
    if (!selected) {
        const below = v.b - MARGIN - (a.b + GAP);
        const above = a.t - GAP - v.t - MARGIN;
        side = ch <= below || below >= above ? 'bottom' : 'top';
        list.style.setProperty('--dx-available-height', Math.max(0, side === 'bottom' ? below : above) + 'px');
        ch = list.offsetHeight;
        list.dataset.side = side;
    } else {
        delete list.dataset.side;
    }
    const cw = list.offsetWidth;
    const maxScroll = list.scrollHeight - list.clientHeight;
    const optTop = selected ? offsetIn(selected) + list.clientTop : 0;
    const optH = selected ? selected.offsetHeight : 0;
    const want = (a.t + a.b) / 2 - optH / 2;
    let top = selected ? want - optTop : side === 'bottom' ? a.b + GAP : a.t - GAP - ch;
    let scroll = 0;
    const lo = v.t + MARGIN;
    const hi = v.b - MARGIN - ch;
    if (selected && (top < lo || top > hi)) {
        top = clamp(top, lo, hi);
        // The panel cannot go further: scroll the list so the option still lands as close as it can.
        scroll = clamp(optTop - (want - top) - list.clientTop, 0, maxScroll);
    }
    top = clamp(top, lo, hi);
    const left = clamp(getComputedStyle(list).direction === 'rtl' ? a.r - cw : a.l, v.l + MARGIN, v.r - MARGIN - cw);
    setPlacement(put, snap(left), snap(top), false, false, origin);
    list.scrollTop = scroll;
    list.dataset.placed = 'true';
};
let tries = 0;
const start = () => {
    if (!list.isConnected) return;
    if (!list.matches(':popover-open') && tries++ < 20) { requestAnimationFrame(start); return; }
    place();
};
start();
listen(window, 'resize', place, { passive: true });
listen(window.visualViewport, 'resize', place, { passive: true });
"#;

/// Places the listbox for `position="item-aligned"`; does nothing for the popper placement.
#[cfg(feature = "web")]
fn use_item_aligned(id: String, ctx: SelectContext) {
    use crate::js_listener::{use_js_listeners, JsListeners};
    let open = ctx.selectable.open;
    use_js_listeners(move || {
        if !open() || ctx.placement() != SelectPosition::ItemAligned {
            return None;
        }
        Some(
            JsListeners::install(&format!(
                "{}{}",
                crate::top_layer::PLACEMENT_HELPERS_JS,
                ITEM_ALIGNED_JS
            ))
            .arg((id.clone(), ctx.selectable.trigger_id.cloned())),
        )
    });
}

/// Shared listbox keydown handling for both render arms below -- factored
/// out so it isn't duplicated across the web/native split.
fn select_list_onkeydown(mut ctx: SelectContext) -> impl FnMut(KeyboardEvent) {
    move |event: KeyboardEvent| {
        let key = event.key();
        let code = event.code();

        // Learn from keyboard events for adaptive matching
        if let Key::Character(actual_char) = &key {
            if let Some(actual_char) = actual_char.chars().next() {
                ctx.learn_from_keyboard_event(&code.to_string(), actual_char);
            }
        }

        let mut arrow_key_navigation = |event: &KeyboardEvent| {
            // Clear the typeahead buffer
            ctx.typeahead_buffer.take();
            event.prevent_default();
            event.stop_propagation();
        };

        match key {
            Key::Character(new_text) => {
                if new_text == " " {
                    ctx.select_current_item();
                    event.prevent_default();
                    event.stop_propagation();
                    return;
                }

                ctx.add_to_typeahead_buffer(&new_text);
            }
            Key::ArrowUp => {
                arrow_key_navigation(&event);
                ctx.selectable.collection.focus_prev();
            }
            Key::End => {
                arrow_key_navigation(&event);
                ctx.selectable.collection.focus_last();
            }
            Key::ArrowDown => {
                arrow_key_navigation(&event);
                ctx.selectable.collection.focus_next();
            }
            Key::Home => {
                arrow_key_navigation(&event);
                ctx.selectable.collection.focus_first();
            }
            Key::Enter => {
                ctx.select_current_item();
                event.prevent_default();
                event.stop_propagation();
            }
            Key::Escape => {
                // Both arms close here: the web arm's popover is `manual` (see
                // `SelectListRendered`), so the browser's Escape light dismiss does not run. The
                // default is cancelled so the key does not also close an enclosing dialog.
                ctx.set_open(false);
                event.prevent_default();
                event.stop_propagation();
            }
            _ => {}
        }
    }
}

/// Web arm: promote the listbox to the top layer via `popover="manual"` so it escapes
/// clipping/transformed ancestors, the same construction as `Tooltip`/`HoverCard`/`Combobox`
/// (dev-docs/plan.md Phase 4.4). The placement is the engine's anchored popper, or the item-aligned
/// script (`use_item_aligned`), per [`SelectContext::placement`].
///
/// ## `manual`, not `auto`
///
/// The trigger opens the list on `pointerdown` for a mouse, so the list becomes `:popover-open`
/// between a gesture's `pointerdown` and `pointerup`. An `auto` popover opened there is hidden by
/// that same gesture's `pointerup` (the opening-gesture contract in
/// [`crate::top_layer::PopoverKind::Auto`]; with the popper placement the release lands outside the
/// list). A `manual` popover is never light-dismissed, so this component owns every dismissal:
///
/// - **Escape**: `select_list_onkeydown` (focus on the list or an option) and `SelectTrigger`'s
///   `onkeydown` (focus on the trigger after an Alt+ArrowDown open) close it; the default is
///   cancelled so an enclosing dialog stays open.
/// - **An outside press or focus**: [`crate::use_outside_dismiss`] on the list (a mouse press at
///   once, a touch or pen press as a tap, so a page scroll that starts outside leaves the list
///   open), plus this listbox's own `onblur` and an individual option's (`option.rs`) when focus
///   leaves the whole widget. All land on `interacted_outside = true`, `open = false`, which is
///   idempotent when more than one fires.
/// - **Animation**: does *not* use [`crate::top_layer::use_popover_sync`] bound to the raw `open`
///   signal, unlike the other anchored consumers -- see
///   [`crate::top_layer::use_popover_shown_while_mounted`]'s doc for the animate-out race that
///   binding would otherwise cause with [`crate::use_animated_open`] (`SelectList`, above), and why
///   this mount/unmount-scoped variant avoids it.
///
/// Page scroll is not locked while the list is open and the panel stays attached to the trigger as
/// the page scrolls (row 477's placement); the list scrolls inside its own `overflow` (row 478).
#[cfg(feature = "web")]
#[component]
fn SelectListRendered(
    /// The DOM id of the list element.
    id: String,
    /// Extra HTML attributes for the list element.
    attributes: Vec<Attribute>,
    /// The list's items.
    children: Element,
) -> Element {
    let mut ctx: SelectContext = use_context();
    let open = ctx.selectable.open;
    // axe `aria-input-field-name` (dev-docs/backlog.md row 34): an
    // ARIA `listbox` is an input field and needs an accessible name.
    // Default it from the trigger (whose own visible text is the
    // placeholder/selected value) -- but ONLY if the caller hasn't already
    // supplied one, since this component's own doc example demonstrates
    // `SelectList { aria_label: "Select Demo", ... }` as the intended
    // override, and `aria-labelledby` would otherwise take ARIA precedence
    // over that caller-supplied `aria-label` and silently shadow it.
    let mut listbox_ref: Signal<Option<std::rc::Rc<MountedData>>> = use_signal(|| None);
    // Scroll buttons render only inside this subtree and read the list's scroll state
    // (`scroll_button.rs`).
    let list_scroll = use_context_provider(|| ListScroll::new(listbox_ref));
    // See `SelectContext::keep_trigger_focus`'s doc: an Alt+ArrowDown open
    // must leave DOM focus on the trigger, so this "nothing focused yet --
    // focus the listbox container" fallback must not fire for that open.
    let focused =
        move || open() && !ctx.selectable.collection.any_focused() && !(ctx.keep_trigger_focus)();

    use_effect(move || {
        let Some(listbox_ref) = listbox_ref() else {
            return;
        };
        if focused() {
            spawn(async move {
                // focus-ok: the listbox container takes focus for its keys (a container: no outline, no editable)
                _ = listbox_ref.set_focus(true);
            });
        }
    });

    let onkeydown = select_list_onkeydown(ctx);

    crate::top_layer::use_popover_shown_while_mounted(
        id.clone(),
        open,
        Callback::new(move |is_open: bool| {
            if !is_open {
                // The popover hid by other means than `open` going false (the browser's own
                // hide, e.g. a removed anchor): clear the roving focus like a blur-close does.
                ctx.selectable.interacted_outside.set(true);
                ctx.selectable.collection.clear_focus();
            }
            ctx.set_open(is_open);
        }),
    );
    // The popover is `manual`: an outside press or focus dismisses through this hook, a touch
    // press as a tap (see the component doc).
    let list_id = use_memo(use_reactive!(|id| id));
    crate::use_outside_dismiss(list_id, move || {
        ctx.selectable.interacted_outside.set(true);
        ctx.set_open(false);
    });
    // JS-measured static positioning fallback for engines without CSS
    // Anchor Positioning -- see `top_layer::use_anchor_position_fallback`'s
    // doc. `side`/`align`/gap match this component's themed CSS
    // (`top: 100%; left: 0; margin-top: 0.25rem` == 4px at the default root
    // font size): anchored below the trigger, left-aligned with it, the
    // same bottom/start convention `DropdownMenu`/`Menubar` already use.
    // Item-aligned places the list itself (`use_item_aligned`), so the popper engine sits out.
    let popper_open = use_memo(move || open() && ctx.placement() == SelectPosition::Popper);
    crate::top_layer::use_anchor_position_fallback(
        id.clone(),
        ctx.selectable.trigger_id.cloned(),
        popper_open,
        crate::ContentSide::Bottom,
        crate::ContentAlign::Start,
        4,
    );
    // `position="item-aligned"`: the selected option over the trigger, inside the viewport.
    use_item_aligned(id.clone(), ctx);
    // axe `aria-input-field-name` (dev-docs/backlog.md row 34): an
    // ARIA `listbox` is an input field and needs an accessible name.
    // Default it from the trigger (whose own visible text is the
    // placeholder/selected value) -- but ONLY if the caller hasn't already
    // supplied one, since this component's own doc example demonstrates
    // `SelectList { aria_label: "Select Demo", ... }` as the intended
    // override, and `aria-labelledby` would otherwise take ARIA precedence
    // over that caller-supplied `aria-label` and silently shadow it.
    // Contributed as its own merge input -- present only when applicable --
    // rather than a bare `aria_labelledby: ...` literal alongside the
    // `..attributes` spread below: a caller attribute list can carry a
    // same-named `aria-labelledby`/`aria-label` with an
    // empty/`AttributeValue::None` value (`has_own_accessible_name`'s own
    // doc explains why), and two entries for one attribute name is exactly
    // the duplicate-attribute hazard `merge_attributes` exists to prevent
    // (`dev-docs/conformance-harness.md` hydration-parity Rule 4) -- a literal
    // plus a raw spread cannot dedupe that; only routing every contributor
    // through one `merge_attributes` call can. Ordered after `attributes`
    // so it wins over exactly that spurious caller-side empty value, while
    // a caller's own *real* name (which makes `has_own_accessible_name`
    // true) leaves this contributing nothing at all to collide with.
    let labelledby: Vec<Attribute> = if has_own_accessible_name(&attributes) {
        Vec::new()
    } else {
        attributes!(div {
            aria_labelledby: "{ctx.selectable.trigger_id}"
        })
    };
    // Only the popper placement is engine-anchored. An item-aligned list is placed by its own script
    // (`use_item_aligned`) in document px and writes `data-side` for its own CSS; the engine's
    // `[data-dx-anchored][popover][data-side=...]` rules (`bottom: anchor(top)`, `margin`,
    // `transform: translateX(-50%)`, `position-try-fallbacks`) would apply on top of that
    // placement, so the list carries no engine attribute.
    let anchor_kind = if ctx.placement() == SelectPosition::Popper {
        crate::top_layer::AnchorKind::Select
    } else {
        crate::top_layer::AnchorKind::SelectItemAligned
    };
    let attributes = merge_attributes(vec![attributes, labelledby]);
    // Folds the caller's own `style` together with the anchor binding into
    // one `style` attribute -- see `top_layer::anchored_content_attributes`'s
    // doc for why a bare `style: position_anchor_style(&id)` literal
    // alongside `..attributes` is the duplicate-`style` hazard
    // (`dev-docs/conformance-harness.md` hydration-parity Rule 4).
    let attributes = crate::top_layer::anchored_content_attributes(
        anchor_kind,
        &ctx.selectable.trigger_id.cloned(),
        attributes,
    );

    // All owned: role/popover/dir define this listbox's widget semantics,
    // `tabindex`/`aria_multiselectable` are managed focus/selection-mode
    // state, and the `data-*` pair mirrors its open/direction state.
    let owned = attributes!(div {
        role: "listbox",
        tabindex: if focused() { "0" } else { "-1" },
        aria_multiselectable: ctx.multi(),
        popover: crate::top_layer::PopoverKind::Manual.as_str(),
        dir: (ctx.direction)().as_str(),
        "data-state": if open() { "open" } else { "closed" },
        "data-direction": (ctx.direction)().as_str(),
        "data-position": ctx.placement().as_str(),
    });
    let attributes = merge_attributes(vec![attributes, owned]);

    rsx! {
        div {
            id: id.clone(),

            onmounted: move |evt| listbox_ref.set(Some(evt.data())),
            onscroll: move |evt| list_scroll.on_scroll(&evt.data()),
            onresize: move |evt| list_scroll.on_resize(&evt.data()),
            onkeydown,
            onblur: move |_| {
                if focused() {
                    ctx.selectable.interacted_outside.set(true);
                    ctx.set_open(false);
                }
            },

            ..attributes,
            {children}
        }
    }
}

/// Native (Blitz) arm: Blitz has no popover-API support at all
/// (`dev-docs/reference/platform-rationale.md` Caveat 2), so this stays the
/// functional floor, a plain, always-in-flow `div`.
#[cfg(not(feature = "web"))]
#[component]
fn SelectListRendered(
    /// The DOM id of the list element.
    id: String,
    /// Extra HTML attributes for the list element.
    attributes: Vec<Attribute>,
    /// The list's items.
    children: Element,
) -> Element {
    let mut ctx: SelectContext = use_context();
    let open = ctx.selectable.open;
    let mut listbox_ref: Signal<Option<std::rc::Rc<MountedData>>> = use_signal(|| None);
    // Scroll buttons render only inside this subtree and read the list's scroll state
    // (`scroll_button.rs`).
    let list_scroll = use_context_provider(|| ListScroll::new(listbox_ref));
    // See `SelectContext::keep_trigger_focus`'s doc: an Alt+ArrowDown open
    // must leave DOM focus on the trigger, so this "nothing focused yet --
    // focus the listbox container" fallback must not fire for that open.
    let focused =
        move || open() && !ctx.selectable.collection.any_focused() && !(ctx.keep_trigger_focus)();

    use_effect(move || {
        let Some(listbox_ref) = listbox_ref() else {
            return;
        };
        if focused() {
            spawn(async move {
                // focus-ok: the listbox container takes focus for its keys (a container: no outline, no editable)
                _ = listbox_ref.set_focus(true);
            });
        }
    });

    let onkeydown = select_list_onkeydown(ctx);
    // See the web arm's identical construction (dev-docs/backlog.md row 34) for
    // why this is conditional and routed through `merge_attributes` rather
    // than a bare literal alongside `..attributes`.
    let labelledby: Vec<Attribute> = if has_own_accessible_name(&attributes) {
        Vec::new()
    } else {
        attributes!(div {
            aria_labelledby: "{ctx.selectable.trigger_id}"
        })
    };
    // Same reasoning as the web arm above.
    let owned = attributes!(div {
        role: "listbox",
        tabindex: if focused() { "0" } else { "-1" },
        aria_multiselectable: ctx.multi(),
        dir: (ctx.direction)().as_str(),
        "data-state": if open() { "open" } else { "closed" },
        "data-direction": (ctx.direction)().as_str(),
        "data-position": ctx.placement().as_str(),
    });
    let attributes = merge_attributes(vec![attributes, labelledby, owned]);

    rsx! {
        div {
            id,

            onmounted: move |evt| listbox_ref.set(Some(evt.data())),
            onscroll: move |evt| list_scroll.on_scroll(&evt.data()),
            onresize: move |evt| list_scroll.on_resize(&evt.data()),
            onkeydown,
            onblur: move |_| {
                if focused() {
                    ctx.selectable.interacted_outside.set(true);
                    ctx.set_open(false);
                }
            },

            ..attributes,
            {children}
        }
    }
}
