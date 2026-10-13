use super::super::component::*;
use dioxus::prelude::*;

/// Modal popover with the page-dimming overlay switched on (`overlay: true`).
///
/// A modal popover (`is_modal: true`: focus trap, inert page, scroll lock, click-outside
/// dismissal) does not dim the page by default, like shadcn's: `overlay` defaults to `false`.
/// Setting it paints the shared modal scrim (the same one Dialog and Sheet use) behind the
/// panel. The overlay is independent of modality but needs the modal arm: `overlay: true` on the
/// default non-modal popover has nothing to dim, because a non-modal popover has no `::backdrop`.
#[component]
pub fn Demo() -> Element {
    rsx! {
        PopoverRoot { is_modal: true, overlay: true,
            PopoverTrigger { "Open with overlay" }
            PopoverContent {
                PopoverHeader {
                    PopoverTitle { "Dimensions" }
                    PopoverDescription { "The page behind is dimmed while this is open." }
                }
            }
        }
    }
}
