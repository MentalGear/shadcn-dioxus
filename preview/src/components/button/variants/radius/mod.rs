use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::ArrowUpRight;

/// The typed `radius`: `Full` is shadcn's `rounded-full`, a pill on a text size and a circle on an icon size.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.75rem;",
            Button { variant: ButtonVariant::Outline, "Default radius" }
            Button { variant: ButtonVariant::Outline, radius: ButtonRadius::Full, "Rounded" }
            Button { radius: ButtonRadius::Full, size: ButtonSize::Lg, "Large pill" }
            Button {
                variant: ButtonVariant::Secondary,
                size: ButtonSize::Icon,
                radius: ButtonRadius::Full,
                "aria-label": "Open",
                ArrowUpRight { size: "16px" }
            }
        }
    }
}
