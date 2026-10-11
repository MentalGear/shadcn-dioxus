use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use dioxus_icons::lucide::CloudUpload;

// `Empty` has no `outline` prop; like shadcn's own example (a `border-dashed`
// class on the root) the dashed outline is a plain border on the root.
#[component]
pub fn Demo() -> Element {
    rsx! {
        Empty { style: "border: 1px dashed var(--dx-border); width: 100%; max-width: 28rem;",
            EmptyHeader {
                EmptyMedia { variant: EmptyMediaVariant::Icon,
                    CloudUpload {}
                }
                EmptyTitle { "Cloud storage is empty" }
                EmptyDescription { "Upload files to your cloud storage to access them anywhere." }
            }
            EmptyContent {
                Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm, "Upload files" }
            }
        }
    }
}
