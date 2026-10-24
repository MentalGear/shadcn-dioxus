use super::super::component::*;
use dioxus::prelude::*;

/// Non-modal variant with `is_modal: false` spelled out -- shadcn's default, and
/// the primitive's (dev-docs/backlog.md row 169), so this is the
/// same behaviour as `main` in a smaller panel. A non-modal popover does not trap
/// focus or block the rest of the page and paints no overlay at all (shadcn's
/// `Popover` has none); it is light-dismissed by the browser (`popover="auto"`).
/// The `modal` variant is the opt-in `is_modal: true` arm: it traps focus, makes
/// the page inert and locks the page scroll, but dims nothing unless `overlay:
/// true` (see the `overlay` variant). Content is the
/// header of shadcn's own canonical Popover example ("Dimensions" / "Set the
/// dimensions for the layer."), without the form rows, to keep this demo small.
///
/// This variant also gives `playwright/popover.spec.ts`'s close-fade check
/// (dev-docs/backlog.md rows 19, 7) a themed, non-modal subject to sample: the
/// `top_layer` oracle fixture's `#stack-popover-*` instance composes the raw
/// `dioxus_primitives::popover` primitive directly
/// (`scripts/check-preview-composition.sh`'s documented, deliberate exemption
/// for that fixture), so it never loads `../../style.css` and has no close
/// animation defined at all -- `use_animated_open` sees zero running animations
/// and unmounts immediately, which is a fixture-composition gap, not a defect in
/// rows 19/7's fix. This variant, routed through the themed `PopoverRoot`/
/// `PopoverTrigger`/`PopoverContent` wrappers (`use super::super::component::*`,
/// same as `main`), loads the real stylesheet via each wrapper's own
/// `document::Link` (`../../component.rs`) the same way every other component
/// page does.
#[component]
pub fn Demo() -> Element {
    rsx! {
        PopoverRoot { is_modal: false,
            PopoverTrigger { "Open popover" }
            PopoverContent {
                PopoverHeader {
                    PopoverTitle { "Dimensions" }
                    PopoverDescription { "Set the dimensions for the layer." }
                }
            }
        }
    }
}
