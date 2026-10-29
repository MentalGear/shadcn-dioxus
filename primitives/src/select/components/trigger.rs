//! SelectTrigger component implementation.

use crate::selectable::ListboxFocus;
use crate::{has_own_accessible_name, merge_attributes, use_id_or};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use dioxus_attributes::attributes;

use super::super::context::SelectContext;

/// The press that precedes the next `click` on the trigger: a `click` carries no pointer type, so the
/// `pointerdown` records it.
#[derive(Clone, Copy, PartialEq)]
enum TriggerPress {
    /// No press seen (a click made by script or a keyboard activation).
    None,
    /// A mouse press that already opened or closed the list: its `click` does nothing more.
    Mouse,
    /// A touch press: the `click` opens the list placed as popper.
    Touch,
    /// A pen press, or a mouse button other than the primary one: the `click` toggles the list.
    Other,
}

/// The props for the [`SelectTrigger`] component
#[derive(Props, Clone, PartialEq)]
pub struct SelectTriggerProps {
    /// The ID of the trigger button. If not provided, an internally
    /// generated ID is used -- see `SelectTrigger`'s use of `use_id_or`
    /// below for why this is a typed field rather than left to
    /// `GlobalAttributes` alone (dev-docs/backlog.md row 36).
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes for the trigger button
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children to render inside the trigger
    pub children: Element,
}

/// # SelectTrigger
///
/// The trigger button for the [`Select`](super::select::Select) component which controls if the [`SelectList`](super::list::SelectList) is rendered.
///
/// This must be used inside a [`Select`](super::select::Select) component.
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
///
///
/// ## Opening
///
/// A mouse opens the list on `pointerdown` (Base UI opens on `mousedown`, Radix on `pointerdown`), so
/// a press-drag-release picks an option; the `click` that follows is ignored. A touch or pen opens on
/// the tap (`click`), so a scroll that starts on the trigger (the browser cancels the pointer and
/// sends no click) leaves the list closed. A touch open places the list as
/// [`SelectPosition::Popper`](super::super::SelectPosition::Popper).
///
/// ## Styling
///
/// The [`SelectTrigger`] component defines a span with a `data-placeholder` attribute if a placeholder is set.
#[component]
pub fn SelectTrigger(props: SelectTriggerProps) -> Element {
    let mut ctx = use_context::<SelectContext>();
    let open = ctx.selectable.open;

    // dev-docs/backlog.md row 36: `ctx.selectable.trigger_id` is the same
    // signal `SelectList`'s `aria-labelledby` (`list.rs`) and the anchor-
    // position fallback (`ctx.selectable.trigger_id.cloned()` below and in
    // `list.rs`) both read back -- so a caller-supplied `id` here must
    // resolve into it, not just win the render. Feeding it straight into
    // `use_id_or` as the generated-id signal does that: `use_id_or`'s own
    // effect writes a caller override back into its `gen_id` argument,
    // which *is* `ctx.selectable.trigger_id` here (no separate local
    // placeholder to keep in sync, unlike `DropdownMenuContent`'s
    // `content_id`) -- mirrors `DialogTitle`'s identical
    // `use_id_or(ctx.dialog_labelledby, props.id)` (`dialog.rs`).
    let id = use_id_or(ctx.selectable.trigger_id, props.id);
    let mut press = use_signal(|| TriggerPress::None);
    // A mouse press whose `click` never reached the trigger (the button was released over the list)
    // must not swallow the next click once the list is closed.
    use_effect(move || {
        if !open() {
            press.set(TriggerPress::None);
        }
    });

    // Merged (caller-wins, deduped) rather than set here and then spread
    // over by `..props.attributes` below: a caller `id`/`aria_label`/etc.
    // override (e.g. the top-layer oracle fixture's
    // `id: "clip-select-trigger"`) would duplicate the attribute in the
    // SSR'd HTML instead of replacing it -- WHATWG HTML's duplicate-
    // attribute parse error keeps the *first* (this component's own
    // default) while the CSR/hydrated DOM path keeps the *last* (the
    // caller's), so server and client disagree
    // (`dev-docs/conformance-harness.md` hydration-parity Rule 4; see
    // `toast.rs`'s `ToastRegionRendered` doc for this construction).
    //
    // axe `button-name` (backlog row 8): a plain `<button>`'s accessible
    // name computes from its own text content ("name from content"), but
    // that computation step does not apply to `role="combobox"` below -- so
    // every caller that relies on this trigger's rendered content (its
    // `SelectValue`'s placeholder or selected-option text, or fully custom
    // `children`) for a "free" accessible name would lose it silently.
    // `select.spec.ts`'s own demo (no `aria_label`), the `top_layer` oracle
    // fixture's `#clip-select-trigger`, and the homepage gallery preview
    // all fail axe's `button-name` (critical) without it -- three
    // independent call sites, not one,
    // which is why this is a primitive-level default rather than a
    // per-call-site `aria_label` patch (the defect is a class, fixed by
    // construction). Checked
    // before `props.attributes` is moved into the merge below.
    let has_caller_name = has_own_accessible_name(&props.attributes);

    let attributes = merge_attributes(vec![
        attributes!(button {
            id: id.cloned(),
            dir: (ctx.direction)().as_str(),
            "data-direction": (ctx.direction)().as_str(),
            // See `crate::top_layer::anchor_name_style`: ties this trigger
            // to the web-arm listbox's `position-anchor`
            // (`SelectListRendered`, `list.rs`) so its anchor-positioned
            // placement resolves relative to this trigger once promoted to
            // the top layer. Keyed on `ctx.selectable.trigger_id` -- kept in
            // sync with this button's own (possibly caller-overridden) `id`
            // above, so the two sides still agree once promoted to the top
            // layer. Inert (empty) off the web arm.
            style: crate::top_layer::anchor_name_style(&ctx.selectable.trigger_id.cloned()),
            // Standard HTML attributes
            disabled: (ctx.selectable.disabled)(),
            type: "button",
            // ARIA attributes
            //
            // dev-docs/backlog.md row 8: role="combobox", matching the APG
            // select-only combobox pattern (vendored reference:
            // playwright/oracle/reference/7e4034b/content/patterns/combobox/
            // examples/combobox-select-only.html's #combo1 -- "role=combobox
            // ... aria-controls=listbox1 aria-expanded=false
            // aria-haspopup=listbox") -- and Radix's Select trigger, which
            // does the same. This is not the implicit-button role:
            // a plain button cannot carry aria-required at all (axe flags
            // it), but
            // a combobox can -- the pattern's own prose says so directly:
            // "Comboboxes and listboxes can be marked as required with
            // aria-required=true" (combobox-pattern.html, "About This
            // Pattern"). `aria-controls`/`aria-expanded` are what
            // combobox requires; the role, `aria-autocomplete`, and
            // `aria-required` below complete it.
            role: "combobox",
            aria_haspopup: "listbox",
            aria_expanded: open(),
            aria_controls: ctx.selectable.list_id,
            // "The combobox element has aria-autocomplete set to a value
            // that corresponds to its autocomplete behavior" -- "none" is
            // correct for a select-only combobox, whose popup contents never
            // change based on typed characters (combobox-pattern.html,
            // "WAI-ARIA Roles, States, and Properties" /
            // "aria-autocomplete"). NOTE: the vendored reference example
            // above does not itself set this attribute (neither its static
            // markup nor select-only.js does) -- it is not part
            // of the tier-1 dual-subject calibration in
            // select-only-combobox.spec.ts's R1 for exactly that reason.
            // Matches Radix's trigger regardless, which does set it.
            aria_autocomplete: "none",
            // "Comboboxes and listboxes can be marked as required with
            // aria-required=true" (combobox-pattern.html, "About This
            // Pattern") -- valid because of the role="combobox" above.
            // Requiredness is enforced functionally by the hidden
            // native <select required> mirror (select.rs); this exposes it
            // to assistive technology too. `SelectMulti` has no `required`
            // prop (see `SelectContext::required`'s doc) -- its context
            // always carries a constant `false` here, so this reads
            // "aria-required=false" for it unconditionally, same as
            // `Checkbox`/`Switch`/`RadioGroup`'s identical
            // `aria_required: props.required` shape.
            aria_required: (ctx.required)(),
        }),
        props.attributes,
        // A self-referencing `aria-labelledby` (pointing this button's
        // `aria-labelledby` at its own `id`, so the accname computation
        // would recurse into this element's own rendered content) does not
        // work, though it looks plausible: the idea is that "name from
        // content" restrictions apply only to a role's *direct*
        // computation, not to a node reached via `aria-labelledby`
        // traversal. Real Chromium/axe-core does not special-case that
        // self-reference for `role="combobox"`: the referenced node's own
        // role still suppresses name-from-content, so it resolves to an
        // empty name and axe keeps reporting the identical `button-name`
        // violation (`select.spec.ts`'s Axe scan). Recorded as a negative
        // result so a future reader does not re-attempt the same idea
        // -- an `aria-label` with a concrete string is the only mechanism
        // that unconditionally works, since it needs no further
        // computation at all.
        //
        // `ctx.selected_text()` (`context.rs`, already used
        // for this exact "what should the trigger show" question by
        // `SelectValue`, `value.rs`) is the best available fallback text:
        // the selected option's own text when something is selected,
        // falling back to `SelectValue`'s own default placeholder string
        // when nothing is (this crate cannot see a *caller-customized*
        // placeholder from here -- that is local to `SelectValue`'s own
        // props, not threaded through `SelectContext` -- so a caller who
        // both customizes the placeholder and never sets their own
        // `aria_label` gets a fallback name that is generic rather than
        // exactly its visible text; still strictly better than no name at
        // all, and every concrete instance of the missing-name defect
        // (`select.spec.ts`'s demo, the homepage gallery,
        // `select-only-combobox.spec.ts`'s R3, the `top_layer` fixture's
        // `#clip-select-trigger`) is in the unselected/default-placeholder
        // state, where this fallback matches the visible text exactly).
        // Same `has_caller_name` gate and merge-order reasoning as the
        // `aria-labelledby` approach above and as `list.rs`'s identical
        // pattern: only when the caller has not already supplied a real
        // `aria-label`/`aria-labelledby` (this component's own doc example
        // demonstrates `aria_label: "Select Trigger"` as the intended
        // override), ordered after `props.attributes` so it also wins over
        // a spurious empty-valued caller attribute
        // (`has_own_accessible_name`'s own doc) without colliding with a
        // real one.
        if has_caller_name {
            Vec::new()
        } else {
            let fallback_label = ctx
                .selected_text()
                .unwrap_or_else(|| "Select an option".to_string());
            attributes!(button {
                aria_label: fallback_label
            })
        },
    ]);

    rsx! {
        button {
            onpointerdown: move |event| {
                let kind = event.pointer_type();
                if kind == "touch" {
                    press.set(TriggerPress::Touch);
                } else if kind == "mouse" && event.trigger_button() == Some(MouseButton::Primary) {
                    if (ctx.selectable.disabled)() {
                        return;
                    }
                    press.set(TriggerPress::Mouse);
                    if open() {
                        ctx.set_open(false);
                    } else {
                        // One task later, after the press's `mousedown` has moved focus onto the
                        // trigger: the list takes focus from the trigger then, and nothing hands
                        // it back. The press has also marked the page as pointer-driven by then,
                        // so the option the list focuses paints no keyboard ring.
                        spawn(async move {
                            crate::sleep::sleep(std::time::Duration::ZERO).await;
                            ctx.open_for_pointer(false);
                        });
                    }
                } else {
                    press.set(TriggerPress::Other);
                }
            },
            onclick: move |_| {
                let pressed = press();
                press.set(TriggerPress::None);
                if pressed == TriggerPress::Mouse {
                    return;
                }
                if open() {
                    ctx.set_open(false);
                } else {
                    ctx.open_for_pointer(pressed == TriggerPress::Touch);
                }
            },
            onkeydown: move |event| {
                match event.key() {
                    Key::ArrowUp => {
                        ctx.set_open(true);
                        ctx.selectable.initial_focus.set(Some(ListboxFocus::LAST));
                        event.prevent_default();
                        event.stop_propagation();
                    }
                    Key::ArrowDown => {
                        ctx.set_open(true);
                        // APG select-only combobox (Optional): "Alt + Down
                        // Arrow: ... displays the popup without moving
                        // focus." Plain ArrowDown (no Alt) still moves focus
                        // to the first option, same as Up Arrow above.
                        if event.modifiers().alt() {
                            ctx.keep_trigger_focus.set(true);
                        } else {
                            ctx.selectable.initial_focus.set(Some(ListboxFocus::FIRST));
                        }
                        event.prevent_default();
                        event.stop_propagation();
                    }
                    // The list is a `manual` popover: no browser Escape dismissal, so a trigger that
                    // keeps focus (the Alt+ArrowDown open) closes it itself.
                    Key::Escape if open() => {
                        ctx.set_open(false);
                        event.prevent_default();
                        event.stop_propagation();
                    }
                    Key::Enter => {
                        ctx.open_with_selected_or_first_focus();
                        event.prevent_default();
                        event.stop_propagation();
                    }
                    Key::Character(c) if c == " " => {
                        ctx.open_with_selected_or_first_focus();
                        event.prevent_default();
                        event.stop_propagation();
                    }
                    _ => {}
                }
            },

            // Pass through the merged, deduped attributes
            ..attributes,

            // Render children (options)
            {props.children}
        }
    }
}
