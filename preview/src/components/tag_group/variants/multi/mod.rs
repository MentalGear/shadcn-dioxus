use dioxus::prelude::*;

use super::super::component::*;

const LABELS: [&str; 6] = ["bug", "feature", "core", "desktop", "example", "duplicate"];

#[component]
pub fn Demo() -> Element {
    // The app owns the list: `on_remove` drops the removed tags from it.
    let mut labels = use_signal(|| LABELS.to_vec());
    let mut last_removed = use_signal(Vec::<String>::new);
    let mut values = use_signal(|| vec!["bug".to_string()]);
    let values_signal = use_memo(move || Some(values()));

    let tags = labels.read().clone().into_iter().map(|t| {
        // `index` is the tag's place in the full list, so keyboard order stays stable as tags go.
        let index = LABELS.iter().position(|&l| l == t).unwrap_or_default();
        let disabled = matches!(t, "feature" | "example");
        rsx! {
            Tag {
                key: "{t}",
                index,
                value: t,
                disabled,
                "{t}"
                RemoveButton {}
            }
        }
    });

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: var(--dx-space-3)",
            TagGroupMulti {
                values: values_signal,
                on_values_change: move |v| values.set(v),
                on_remove: move |removed: Vec<String>| {
                    labels.retain(|l| !removed.iter().any(|r| r == l));
                    last_removed.set(removed);
                },
                allow_empty_selection: false,
                TagGroupLabel { "Labels" }
                TagList {
                    TagGroupEmpty { "No tags" }
                    {tags}
                }
            }
            p {
                role: "status",
                "data-testid": "tag-group-removed",
                style: "margin: 0; font-size: var(--dx-text-sm); color: var(--dx-muted-foreground)",
                if last_removed.read().is_empty() {
                    "{labels.read().len()} tags"
                } else {
                    "Removed {last_removed.read().join(\", \")}, {labels.read().len()} tags left"
                }
            }
        }
    }
}
