use super::super::component::*;
use dioxus::prelude::*;

// shadcn's skeleton-demo (avatar + two lines) and skeleton-card (image +
// two lines): sizes are shadcn's `w-[250px]` / `w-[200px]` / `h-[125px]`, each capped by the viewport so the
// demo fits a 320px phone's card (row 280): its preview is the viewport less 74px of padding and border, and
// `100vw` also counts a desktop browser's 15px scrollbar gutter. Not `100%`: the demo is shrink-wrapped, so a
// percentage would resolve against its own intrinsic width (see the Resizable demo).
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
                // 10rem: the card's 6rem plus the 3rem avatar and the 1rem gap.
                Skeleton { style: "width: min(250px, 100vw - 10rem); height: 1rem;" }
                Skeleton { style: "width: min(200px, 100vw - 13rem); height: 1rem;" }
            }
        }
    }
}

#[component]
fn SkeletonCardDemo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.75rem;",
            Skeleton { style: "width: min(250px, 100vw - 6rem); height: 125px; border-radius: 0.75rem;" }
            div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                Skeleton { style: "width: min(250px, 100vw - 6rem); height: 1rem;" }
                Skeleton { style: "width: min(200px, 100vw - 9rem); height: 1rem;" }
            }
        }
    }
}
