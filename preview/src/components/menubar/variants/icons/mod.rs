use super::super::component::{Menubar, MenubarContent, MenubarItem, MenubarMenu, MenubarTrigger};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Download, ListPlus, Play, Repeat, Share2, Shuffle};

/// A media-player menubar whose items each lead with a lucide icon (shadcn's
/// menubar "icons" shape). The item is a flex row with a `gap`, so the icon is
/// just the first child.
///
/// The trigger names ("Library", "Playback") and the item names are chosen not
/// to collide with what `menubar.spec.ts` and the shared `oracle/tier1-apg`
/// specs query by name on this page ("File", "Edit", "View", "Accounts", ...);
/// trigger indices are per-menubar, so this menubar's `0`/`1` do not clash with
/// `main`'s.
///
/// Shortcut hints, a destructive item and separators between groups are the
/// `shortcuts` variant.
#[component]
pub fn Demo() -> Element {
    let mut chosen = use_signal(|| None::<String>);

    rsx! {
        div {
            Menubar { aria_label: "Media player menu",
                MenubarMenu { index: 0usize,
                    MenubarTrigger { "Library" }
                    MenubarContent {
                        MenubarItem {
                            index: 0usize,
                            value: "Add to playlist".to_string(),
                            text_value: "Add to playlist",
                            on_select: move |value| chosen.set(Some(value)),
                            ListPlus { size: "16px" }
                            "Add to playlist"
                        }
                        MenubarItem {
                            index: 1usize,
                            value: "Download".to_string(),
                            text_value: "Download",
                            on_select: move |value| chosen.set(Some(value)),
                            Download { size: "16px" }
                            "Download"
                        }
                        MenubarItem {
                            index: 2usize,
                            value: "Share".to_string(),
                            text_value: "Share",
                            disabled: true,
                            on_select: move |value| chosen.set(Some(value)),
                            Share2 { size: "16px" }
                            "Share"
                        }
                    }
                }
                MenubarMenu { index: 1usize,
                    MenubarTrigger { "Playback" }
                    MenubarContent {
                        MenubarItem {
                            index: 0usize,
                            value: "Play".to_string(),
                            text_value: "Play",
                            on_select: move |value| chosen.set(Some(value)),
                            Play { size: "16px" }
                            "Play"
                        }
                        MenubarItem {
                            index: 1usize,
                            value: "Shuffle".to_string(),
                            text_value: "Shuffle",
                            on_select: move |value| chosen.set(Some(value)),
                            Shuffle { size: "16px" }
                            "Shuffle"
                        }
                        MenubarItem {
                            index: 2usize,
                            value: "Repeat".to_string(),
                            text_value: "Repeat",
                            on_select: move |value| chosen.set(Some(value)),
                            Repeat { size: "16px" }
                            "Repeat"
                        }
                    }
                }
            }
        }
        if let Some(item) = chosen() {
            p { style: "margin-top: 0.75rem;", "Chose: {item}" }
        }
    }
}
