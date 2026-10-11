use super::super::component::*;
use dioxus::prelude::*;

/// A form's loading state: two label-and-input pairs, then the submit button.
/// Plain blocks, not a `<form>`, so there is nothing to submit.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1.5rem; width: 20rem;",
            FieldSkeleton {}
            FieldSkeleton {}
            Skeleton { style: "width: 6rem; height: 2rem; border-radius: 0.5rem;" }
        }
    }
}

#[component]
fn FieldSkeleton() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
            Skeleton { style: "width: 5rem; height: 1rem;" }
            Skeleton { style: "width: 100%; height: 2rem; border-radius: 0.5rem;" }
        }
    }
}
