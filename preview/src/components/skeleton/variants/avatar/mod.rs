use super::super::component::*;
use dioxus::prelude::*;

/// shadcn's avatar loading row: a 48px circle (the avatar) beside two text
/// lines (the name and the handle).
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; align-items: center; gap: 1rem;",
            Skeleton { style: "width: 3rem; height: 3rem; border-radius: 50%;" }
            div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                Skeleton { style: "width: 15.625rem; height: 1rem;" }
                Skeleton { style: "width: 12.5rem; height: 1rem;" }
            }
        }
    }
}
