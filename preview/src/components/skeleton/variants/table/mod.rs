use super::super::component::*;
use dioxus::prelude::*;

/// A table's loading state: four rows of three cells, the cells sized
/// 1 : 2 : 1 the way a name, description and status column usually are.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.75rem; width: 24rem;",
            for _ in 0..4 {
                TableRowSkeleton {}
            }
        }
    }
}

#[component]
fn TableRowSkeleton() -> Element {
    rsx! {
        div { style: "display: flex; align-items: center; gap: 1rem;",
            Skeleton { style: "flex: 1; height: 1rem;" }
            Skeleton { style: "flex: 2; height: 1rem;" }
            Skeleton { style: "flex: 1; height: 1rem;" }
        }
    }
}
