use dioxus::prelude::*;
use dioxus_icons::lucide::X;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
use dioxus_primitives::tag_group::{
    self, TagGroupEmptyProps, TagGroupLabelProps, TagGroupMultiProps, TagGroupProps, TagListProps,
};

// dev-docs/backlog.md row 32: see checkbox/component.rs's
// header comment for the full delivery-mechanism rationale (asset!() +
// document::Link, embedded in every exported entry point of this file so a
// `shadcn-dioxus add tag_group`-copied component needs no extra wiring).
//
// Every class lives under the full `dx-tag-group-` namespace
// (`dx-tag-group-tag`, `dx-tag-group-list`, `dx-tag-group-remove-button`), in
// both this file and `style.css`. A short `.dx-remove-button` would collide
// with `drag_and_drop_list`'s differently-sized one (26x26px +
// `margin-left: 10px` vs this one's unsized, `margin-left: 0.25rem`) and
// silently restyle whichever sheet lost the CSS load order (backlog row 32);
// `dx-tag` and `dx-tag-list` would be out of namespace even without an active
// collision -- see `scripts/check-dx-class-prefix.sh`.
#[component]
pub fn TagGroup(props: TagGroupProps<String>) -> Element {
    let base = attributes!(div { class: "dx-tag-group", "data-dx-root": "" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/tag_group/style.css") }
        tag_group::TagGroup {
            value: props.value,
            default_value: props.default_value,
            on_value_change: props.on_value_change,
            on_remove: props.on_remove,
            disabled: props.disabled,
            selectable: props.selectable,
            allow_empty_selection: props.allow_empty_selection,
            escape_clears_selection: props.escape_clears_selection,
            roving_loop: props.roving_loop,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn TagGroupMulti(props: TagGroupMultiProps<String>) -> Element {
    let base = attributes!(div { class: "dx-tag-group", "data-dx-root": "" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/tag_group/style.css") }
        tag_group::TagGroupMulti {
            values: props.values,
            default_values: props.default_values,
            on_values_change: props.on_values_change,
            on_remove: props.on_remove,
            disabled: props.disabled,
            selectable: props.selectable,
            allow_empty_selection: props.allow_empty_selection,
            escape_clears_selection: props.escape_clears_selection,
            roving_loop: props.roving_loop,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn TagGroupLabel(props: TagGroupLabelProps) -> Element {
    let base = attributes!(div { class: "dx-tag-group-label" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/tag_group/style.css") }
        tag_group::TagGroupLabel { id: props.id, attributes: merged, {props.children} }
    }
}

#[component]
pub fn TagGroupEmpty(props: TagGroupEmptyProps) -> Element {
    let base = attributes!(div { class: "dx-tag-group-empty" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/tag_group/style.css") }
        tag_group::TagGroupEmpty { attributes: merged, {props.children} }
    }
}

#[component]
pub fn TagList(props: TagListProps) -> Element {
    let base = attributes!(div { class: "dx-tag-group-list" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/tag_group/style.css") }
        tag_group::TagList { attributes: merged, {props.children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TagProps {
    /// The tag's value, reported when it is selected or removed.
    pub value: ReadSignal<String>,
    /// Plain text used for typeahead when the tag's content is not text.
    #[props(default)]
    pub text_value: ReadSignal<Option<String>>,
    /// The tag's position in the list; keeps keyboard order stable.
    pub index: ReadSignal<usize>,
    /// The `id` of the tag element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,
    /// Whether the tag is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The tag's content.
    pub children: Element,
}

#[component]
pub fn Tag(props: TagProps) -> Element {
    let base = attributes!(div { class: "dx-tag-group-tag" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/tag_group/style.css") }
        tag_group::TagOption::<String> {
            value: props.value,
            text_value: props.text_value,
            disabled: props.disabled,
            id: props.id,
            index: props.index,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn RemoveButton(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// Extra content inside the button, before the remove icon. Usually left empty.
    children: Element,
) -> Element {
    let base = attributes!(button { class: "dx-tag-group-remove-button" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/tag_group/style.css") }
        tag_group::TagRemoveButton {
            attributes: merged,
            {children}
            X { size: "12px" }
        }
    }
}
