use crate::components::toggle::{ToggleSize, ToggleVariant};
use dioxus::prelude::*;
use dioxus_primitives::{
    dioxus_attributes::attributes,
    direction::Direction,
    merge_attributes,
    toggle_group::{self, ToggleItemProps},
};
use std::collections::HashSet;

// docs/backlog.md row 32: `#[css_module]` is gone -- see checkbox/component.rs's
// header comment for the full delivery-mechanism rationale (asset!() +
// document::Link, embedded in both exported entry points so a
// `shadcn-dioxus add toggle_group`-copied component needs no extra wiring).
//
// This component is also on the row-32 "not already namespaced" lane: its
// item class used to be the bare `dx-toggle-item`, not
// `dx-toggle-group-item`. `#[css_module]`'s hash kept it collision-safe
// regardless, but a plain `dx-toggle-item` isn't provably this component's
// own once the hash is gone. Renamed to `dx-toggle-group-item` in the same
// change that drops the macro, in both this file and `style.css` -- see
// `scripts/check-dx-class-prefix.sh`.

/// The props for the themed [`ToggleGroup`]: the primitive's, plus `variant`,
/// `size` and `spacing` (shadcn's `ToggleGroup` props, which every item inherits).
#[derive(Props, Clone, PartialEq)]
pub struct ToggleGroupProps {
    /// The default pressed items if the component is not controlled.
    #[props(default)]
    pub default_pressed: HashSet<usize>,

    /// The currently pressed items. This can be used to drive the component when controlled.
    pub pressed: ReadSignal<Option<HashSet<usize>>>,

    /// Callback to handle changes in pressed state
    #[props(default)]
    pub on_pressed_change: Callback<HashSet<usize>>,

    /// Whether the toggle group is disabled
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// If multiple items can be pressed at the same time. If this is false, only one item can be pressed at a time (radio-style).
    #[props(default)]
    pub allow_multiple_pressed: ReadSignal<bool>,

    /// Whether the toggle group is horizontal (the default, shadcn's `orientation="horizontal"`)
    /// or vertical (`false`).
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub horizontal: ReadSignal<bool>,

    /// The text direction for `ArrowLeft`/`ArrowRight` roving focus.
    #[props(default)]
    pub dir: Option<Direction>,

    /// Whether focus should loop around when reaching the end.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// The look of every item: no edge (`Default`) or a 1px edge (`Outline`).
    #[props(default)]
    pub variant: ToggleVariant,

    /// The size of every item: `Sm` (28px), `Default` (32px) or `Lg` (36px).
    #[props(default)]
    pub size: ToggleSize,

    /// The gap between items in 4px steps (Tailwind's spacing scale), shadcn's `spacing`.
    /// The default 2 is a row of separate toggles 8px apart; `0` joins the items into one
    /// segmented bar (with `Outline`, a shared 1px edge) whose end items carry the radius.
    #[props(default = 2)]
    pub spacing: u8,

    /// Additional attributes to apply to the toggle group element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the toggle group, which should include multiple [`ToggleItem`] components.
    pub children: Element,
}

#[component]
pub fn ToggleGroup(props: ToggleGroupProps) -> Element {
    // `toggle_group::ToggleGroup` has no named `class` prop of its own (see
    // `primitives/src/toggle_group.rs`), so unlike `Toggle`/`RadioItem` this
    // theme class and a caller-supplied class both have to travel through
    // the shared `attributes` bucket -- a literal `class: "dx-toggle-group"`
    // shorthand here alongside an explicit `attributes: props.attributes`
    // field would hit the exact same clobber-not-merge bug commit 724bfee
    // fixed for `Input`, so this merges them the same way
    // `../popover/component.rs`'s `PopoverRoot` does.
    //
    // `variant` / `size` / `spacing` are slot variables, not per-item props: the
    // root carries `data-variant` / `data-size` / `data-spacing` and the stylesheet
    // redeclares `--dx-toggle-*` on it, which every item inherits (shadcn's own
    // context does the same job, with `context.variant || variant` winning).
    // They are theme state, so they win over a caller's attribute of the same name.
    let base = attributes!(div { class: "dx-toggle-group", "data-dx-root": "" });
    let owned = attributes!(div {
        "data-variant": props.variant.class(),
        "data-size": props.size.class(),
        "data-spacing": props.spacing.to_string(),
        style: format!("--dx-toggle-group-spacing: {};", props.spacing),
    });
    let merged = merge_attributes(vec![base, props.attributes, owned]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/toggle_group/style.css") }
        toggle_group::ToggleGroup {
            default_pressed: props.default_pressed,
            pressed: props.pressed,
            on_pressed_change: props.on_pressed_change,
            disabled: props.disabled,
            allow_multiple_pressed: props.allow_multiple_pressed,
            horizontal: props.horizontal,
            dir: props.dir,
            roving_loop: props.roving_loop,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn ToggleItem(props: ToggleItemProps) -> Element {
    // Same reasoning as `ToggleGroup` above: `toggle_group::ToggleItem` has
    // no named `class` prop either, so the theme class and a caller class
    // are merged through `attributes` rather than risking one clobbering
    // the other.
    let base = attributes!(button { class: "dx-toggle-group-item" });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/toggle_group/style.css") }
        toggle_group::ToggleItem {
            index: props.index,
            disabled: props.disabled,
            attributes: merged,
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[component]
    fn GroupWithCallerClasses() -> Element {
        rsx! {
            ToggleGroup {
                pressed: None,
                class: "caller-group-class",
                ToggleItem {
                    index: 0usize,
                    class: "caller-item-class",
                    "B"
                }
            }
        }
    }

    #[test]
    fn caller_class_and_theme_class_both_render() {
        let mut dom = VirtualDom::new(GroupWithCallerClasses);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        // A caller's own class must extend, not replace, the theme's
        // "dx-toggle-group"/"dx-toggle-group-item" -- see commit 724bfee.
        assert!(html.contains(r#"class="dx-toggle-group caller-group-class""#));
        assert!(html.contains(r#"class="dx-toggle-group-item caller-item-class""#));
    }

    #[test]
    fn theme_classes_alone_render_when_caller_sets_no_class() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                ToggleGroup {
                    pressed: None,
                    ToggleItem { index: 0usize, "B" }
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains(r#"class="dx-toggle-group""#));
        assert!(html.contains(r#"class="dx-toggle-group-item""#));
    }

    #[test]
    fn variant_size_and_spacing_render_on_the_root() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                ToggleGroup {
                    pressed: None,
                    variant: ToggleVariant::Outline,
                    size: ToggleSize::Lg,
                    spacing: 0u8,
                    style: "margin: 1px;",
                    ToggleItem { index: 0usize, "B" }
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains(r#"data-variant="outline""#));
        assert!(html.contains(r#"data-size="lg""#));
        assert!(html.contains(r#"data-spacing="0""#));
        // The spacing variable joins the caller's own declarations instead of replacing them.
        assert!(html.contains("margin: 1px;"));
        assert!(html.contains("--dx-toggle-group-spacing: 0;"));
    }

    #[test]
    fn defaults_are_separate_default_toggles_two_steps_apart() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                ToggleGroup { pressed: None,
                    ToggleItem { index: 0usize, "B" }
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains(r#"data-variant="default""#));
        assert!(html.contains(r#"data-size="default""#));
        assert!(html.contains(r#"data-spacing="2""#));
        assert!(html.contains(r#"data-orientation="horizontal""#));
        assert!(html.contains(r#"role="group""#));
    }
}
