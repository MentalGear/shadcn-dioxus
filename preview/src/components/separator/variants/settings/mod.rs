use super::super::component::*;
use dioxus::prelude::*;

fn setting(title: &'static str, description: &'static str) -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.25rem; padding: 0.75rem 0;",
            span { style: "font-weight: 500;", "{title}" }
            span { style: "font-size: var(--dx-text-sm); color: var(--dx-muted-foreground);", "{description}" }
        }
    }
}

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; width: 100%; max-width: 24rem;",
            {setting("Marketing emails", "Receive emails about new products, features, and more.")}
            Separator { decorative: true }
            {setting("Security emails", "Receive emails about your account security.")}
            Separator { decorative: true }
            {setting("Product updates", "Get notified when a feature you asked for ships.")}
        }
    }
}
