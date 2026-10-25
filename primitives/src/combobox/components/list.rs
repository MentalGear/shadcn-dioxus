//! ComboboxList component.

use dioxus::prelude::*;
use dioxus_attributes::attributes;

use super::super::context::ComboboxContext;
use crate::listbox::use_listbox_container;
use crate::merge_attributes;

/// Props for [`ComboboxList`].
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxListProps {
    /// Optional id for the list element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Children, typically [`ComboboxOption`](super::option::ComboboxOption)s
    /// and an optional [`ComboboxEmpty`](super::empty::ComboboxEmpty).
    pub children: Element,
}

/// Listbox that contains the visible options.
#[component]
pub fn ComboboxList(props: ComboboxListProps) -> Element {
    let ctx = use_context::<ComboboxContext>();
    let listbox = use_listbox_container(props.id, ctx.selectable);
    let render = listbox.render;

    // The editing session ends once the popup has finished closing -- not
    // when `open` goes false: the closing list keeps its filter (and so its
    // options) through its exit animation. See `ComboboxContext::edited`.
    let mut edited = ctx.edited;
    use_effect(move || {
        if !render() && *edited.peek() {
            edited.set(false);
        }
        // A multi-select's text is only ever a query: it does not outlive the popup.
        if !render() && ctx.is_multi() && !ctx.query.peek().is_empty() {
            ctx.set_query.call(String::new());
        }
    });

    rsx! {
        if render() {
            ComboboxListRendered {
                id: listbox.id.cloned(),
                attributes: props.attributes,
                children: props.children,
            }
        } else {
            {props.children}
        }
    }
}

/// Web arm: promote the listbox to the top
/// layer via `popover="manual"`, anchored to `ComboboxInput` below/
/// start-aligned -- the same top-layer construction as
/// `Tooltip`/`HoverCard`/`Popover`/`DropdownMenu`/`Menubar`/`Select`
/// (dev-docs/plan.md Phase 4.4), matching the themed
/// `top: calc(100% + 0.25rem); left: 0` placement
/// (`../../../../preview/src/components/combobox/style.css`).
///
/// ## `manual`, not `auto`
///
/// Combobox has no `Select`-style DOM-focus/blur delicacy to worry about
/// (`ComboboxInput` keeps focus throughout via an `aria-activedescendant`
/// model, never blurring into this listbox), so `auto`'s light dismiss
/// might look like free, strictly additive insurance on top of
/// `ComboboxInput`'s own `onblur`/`onkeydown` dismissal. It is not:
/// `combobox.spec.ts`'s "dynamic option removal updates filtering and
/// keyboard selection" test clicks an *external* "Toggle SvelteKit" button
/// while the list is open (a supported pattern: that button's own
/// `onpointerdown` calls `prevent_default()` specifically so it does not
/// blur/close the combobox), and under `auto` the list closes anyway. WHATWG
/// light dismiss classifies *any* pointerdown outside the popover's own DOM
/// subtree as "outside," full stop -- neither `prevent_default()` on that
/// pointerdown nor a `popovertarget` association from `ComboboxInput` to
/// this listbox stops it (`auto` has no notion of "this button is allowed
/// to interact with me without closing me" beyond "is it a descendant").
/// That is fatal specifically for Combobox's own supported use case -- an
/// external control manipulating state while the list stays open -- in a
/// way it is not for `Select` (whose own suite has no such case, and for
/// which "any outside click closes it" is the same behavior a plain HTML
/// `<select>` has). `manual` sidesteps the whole question: WHATWG HTML never
/// light-dismisses a manual popover, so `ComboboxInput`'s own
/// `onblur`/`onkeydown` are the *only* dismissal path, immune to this class
/// of external-click interference -- exactly `ContextMenuContentRendered`'s
/// identical `manual` reasoning (`context_menu.rs`).
///
/// The one behavior `manual` gives up: `auto`'s light dismiss would
/// additionally close the list on a click over a *non-focusable*
/// area (one `ComboboxInput`'s `onblur` alone cannot see, since nothing
/// there ever takes focus). No `combobox.spec.ts` case depends on that gap
/// being closed, so trading it away for the external-control case above is
/// a deliberate net win.
///
/// **Animation**: does *not* use [`crate::top_layer::use_popover_sync`]
/// bound to the raw `open` signal -- see
/// [`crate::top_layer::use_popover_shown_while_mounted`]'s doc for the
/// animate-out race that binding would otherwise cause with
/// [`crate::use_animated_open`] (`ComboboxList`, above). It matters here
/// specifically: `combobox.spec.ts`'s "keeps
/// filtered options during keyboard close animation" test (an Enter-key,
/// script-driven close) requires the `data-state="closed"` element to stay
/// visible through its exit animation, which binding
/// `showPopover()`/`hidePopover()` to the raw `open` signal breaks (see the
/// linked doc for the exact mechanism) -- this reasoning is orthogonal to
/// the `auto`-vs-`manual` choice above and holds either way.
#[cfg(feature = "web")]
#[component]
fn ComboboxListRendered(
    /// The DOM id of the list element.
    id: String,
    /// Extra HTML attributes merged onto the list element.
    attributes: Vec<Attribute>,
    /// The options and empty state inside the list.
    children: Element,
) -> Element {
    let ctx: ComboboxContext = use_context();
    let mut selectable = ctx.selectable;
    let open = selectable.open;

    crate::top_layer::use_popover_shown_while_mounted(
        id.clone(),
        open,
        Callback::new(move |is_open: bool| {
            selectable.set_open(is_open);
        }),
    );
    // JS-measured static positioning fallback for engines without CSS
    // Anchor Positioning -- see `top_layer::use_anchor_position_fallback`'s
    // doc. `side`/`align`/gap match this component's themed CSS
    // (`top: calc(100% + 0.25rem); left: 0` == a 4px gap past the input's
    // own bottom edge): anchored below the input, left-aligned with it,
    // the same bottom/start convention every other listbox here uses.
    crate::top_layer::use_anchor_position_fallback(
        id.clone(),
        ctx.input_id.cloned(),
        open,
        crate::ContentSide::Bottom,
        crate::ContentAlign::Start,
        4,
    );
    // Closes when the INPUT leaves the viewport, not on any scroll: the list
    // holds a typed query, and a phone keyboard scrolls and resizes the page
    // while it is used -- "any scroll" would close it the moment it is
    // tapped. A focus-driven scroll reveals the input, it never hides it
    // (`top_layer::use_close_on_scroll`'s doc).
    crate::top_layer::use_close_on_scroll(
        id.clone(),
        ctx.input_id.cloned(),
        open,
        crate::top_layer::ScrollClose::AnchorLeavesViewport,
        Callback::new(move |is_open: bool| {
            selectable.set_open(is_open);
        }),
    );

    // See `dropdown_menu.rs`'s `DropdownMenuContentRendered` for why this
    // hand-written, never-`Styles::`-routed marker class exists: it is what
    // the shared, engine-injected anchor-positioning stylesheet
    // (`top_layer::ensure_anchor_positioning_styles`) selects on,
    // sidestepping `manganis-core`'s `css_module_parser` not scoping
    // classes inside `@supports` bodies.
    let attributes = merge_attributes(vec![
        attributes,
        attributes!(div {
            class: "dx-anchor-combobox"
        }),
    ]);
    // Folds the caller's own `style` together with the anchor binding into
    // one `style` attribute -- see `top_layer::anchored_content_attributes`'s
    // doc for why a bare `style: position_anchor_style(&id)` literal
    // alongside `..attributes` is the duplicate-`style` hazard
    // (`dev-docs/conformance-harness.md` hydration-parity Rule 4).
    let attributes =
        crate::top_layer::anchored_content_attributes(&ctx.input_id.cloned(), attributes);
    let owned = attributes!(div {
        role: "listbox",
        popover: crate::top_layer::PopoverKind::Manual.as_str(),
        aria_multiselectable: ctx.is_multi().then_some("true"),
        "data-state": if open() { "open" } else { "closed" },
    });
    let attributes = merge_attributes(vec![attributes, owned]);

    rsx! {
        div {
            id: id.clone(),
            onpointerdown: move |event| {
                event.prevent_default();
            },
            ..attributes,
            {children}
        }
    }
}

/// Native (Blitz) arm: Blitz has no popover-API support at all, so this is
/// the functional floor, a plain, always-in-flow `div`.
#[cfg(not(feature = "web"))]
#[component]
fn ComboboxListRendered(
    /// The DOM id of the list element.
    id: String,
    /// Extra HTML attributes merged onto the list element.
    attributes: Vec<Attribute>,
    /// The options and empty state inside the list.
    children: Element,
) -> Element {
    let ctx: ComboboxContext = use_context();
    let open = ctx.selectable.open;

    let owned = attributes!(div {
        role: "listbox",
        aria_multiselectable: ctx.is_multi().then_some("true"),
        "data-state": if open() { "open" } else { "closed" },
    });
    let attributes = merge_attributes(vec![attributes, owned]);

    rsx! {
        div {
            id,
            onpointerdown: move |event| {
                event.prevent_default();
            },
            ..attributes,
            {children}
        }
    }
}
