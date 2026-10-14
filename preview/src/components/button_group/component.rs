use crate::components::separator::Separator;
use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

/// The enclosing [`ButtonGroup`]'s orientation (`true` = vertical column),
/// published by context so a [`ButtonGroupSeparator`] derives its own
/// orientation from the group instead of being told it.
#[derive(Clone, Copy)]
struct GroupVertical(Signal<bool>);

/// Groups a row (or column) of [`Button`](crate::components::button::Button)s
/// so their borders/radii visually merge into one control, matching the
/// shadcn/ui `ButtonGroup`. Plain layout, `role="group"` -- no ARIA widget
/// pattern and no primitive underneath.
#[component]
pub fn ButtonGroup(
    #[props(default)] orientation_vertical: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div { class: "dx-button-group" });
    let mut group_vertical = use_context_provider(|| GroupVertical(Signal::new(orientation_vertical)));
    if *group_vertical.0.peek() != orientation_vertical {
        group_vertical.0.set(orientation_vertical);
    }
    // `role="group"`/`data-orientation` are this wrapper's own required
    // semantics/typed-prop state, not a caller default -- owned-wins.
    let owned = attributes!(div {
        role: "group",
        "data-orientation": if orientation_vertical { "vertical" } else { "horizontal" },
    });
    let merged = merge_attributes(vec![base, attributes, owned]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/button_group/style.css") }
        div { ..merged, {children} }
    }
}

/// A visual divider between two clusters of buttons inside a
/// [`ButtonGroup`] -- e.g. separating a primary action from an overflow
/// menu trigger. Composes the themed [`Separator`]; Nova's
/// `ButtonGroupSeparator` is vertical by default, so here the orientation is
/// not a prop at all but always the group's cross axis (a vertical rule in a
/// row, a horizontal one in a column). A separator therefore can never be the
/// 100%-wide horizontal kind inside a horizontal flex row. Outside any
/// `ButtonGroup` it behaves as in a row group.
#[component]
pub fn ButtonGroupSeparator(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let base = attributes!(div {
        class: "dx-button-group-separator",
    });
    let merged = merge_attributes(vec![base, attributes]);
    let group_vertical = try_use_context::<GroupVertical>().is_some_and(|g| *g.0.read());

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/button_group/style.css") }
        Separator {
            // Cross axis of the group: column group -> horizontal rule.
            horizontal: group_vertical,
            decorative: true,
            attributes: merged,
        }
    }
}

/// A static, non-interactive label chip inside a [`ButtonGroup`] -- e.g. a
/// unit ("px", "$") flanking an input-like control.
#[component]
pub fn ButtonGroupText(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div { class: "dx-button-group-text" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/button_group/style.css") }
        div { ..merged, {children} }
    }
}
