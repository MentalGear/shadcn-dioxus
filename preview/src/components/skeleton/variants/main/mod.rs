use super::super::component::*;
use dioxus::prelude::*;

// shadcn's skeleton-demo (avatar + two lines) and skeleton-card (image +
// two lines): sizes are shadcn's `w-[250px]` / `w-[200px]` / `h-[125px]`.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; align-items: center; gap: 2rem;",
            SkeletonInfoDemo {}
            SkeletonCardDemo {}
        }
    }
}

#[component]
fn SkeletonInfoDemo() -> Element {
    rsx! {
        div { style: "display: flex; align-items: center; gap: 1rem;",
            Skeleton { style: "width: 3rem; height: 3rem; border-radius: 50%;" }
            div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                Skeleton { style: "width: 250px; height: 1rem;" }
                Skeleton { style: "width: 200px; height: 1rem;" }
            }
        }
    }
}

#[component]
fn SkeletonCardDemo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.75rem;",
            Skeleton { style: "width: 250px; height: 125px; border-radius: 0.75rem;" }
            div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                Skeleton { style: "width: 250px; height: 1rem;" }
                Skeleton { style: "width: 200px; height: 1rem;" }
            }
        }
    }
}
