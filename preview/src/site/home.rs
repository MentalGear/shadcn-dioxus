use super::*;

/// Lets the homepage's own `ComponentGalleryPreview` (deep inside
/// `DocsLayout`'s `children`, via `ComponentGallery`) reach the same
/// `side`/`collapsible` signals `Home()` hands to `DocsLayout` itself, so
/// the Sidebar gallery card's Side/Collapse buttons reconfigure the real,
/// on-page nav live rather than a separate demo instance. Provided once
/// here and read with `try_consume_context` (not `consume_context`) by
/// `ComponentGalleryPreview`, since that component has no other reason to
/// require a `Home()` ancestor -- reading it defensively keeps it reusable
/// on a future page that doesn't provide this context, where its Sidebar
/// card then just falls back to the plain "Open full preview" link.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct HomeSidebarControls {
    pub(crate) side: Signal<SidebarSide>,
    pub(crate) collapsible: Signal<SidebarCollapsible>,
}

/// The homepage's own section headings, in the order `Home()` renders
/// them -- the same single-source-of-truth pattern as `DOCS_SECTIONS`
/// (see its doc comment): `docs_section_slug` derives both each `h2`'s
/// `id` and `DocsLayout`'s "Start" > "Home" sub-link `href`s from this
/// same text, so they can't drift apart. `HOME_SECTIONS[0]` ("Sample
/// interfaces") is `WidgetMasonry()`'s own heading, rendered by that
/// separate `#[component]` fn rather than here, so `Home()` passes its
/// slug down as a prop instead of duplicating the `h2` here.
pub(crate) const HOME_SECTIONS: &[&str] = &["Sample interfaces", "All components"];

#[component]
pub(crate) fn Home(iframe: Option<bool>, dark_mode: Option<bool>) -> Element {
    let side = use_signal(|| SidebarSide::Left);
    let collapsible = use_signal(|| SidebarCollapsible::Offcanvas);
    use_context_provider(|| HomeSidebarControls { side, collapsible });

    rsx! {
        DocsLayout {
            active: DocsNavActive::Home,
            side: Some(side),
            collapsible: Some(collapsible),
            default_open: Some(false),
            page_sections: Some(HOME_SECTIONS),
            // No `role: "main"` here -- `DocsLayout`'s `SidebarInset` already
            // renders the page's one `<main>` landmark (axe
            // `landmark-no-duplicate-main`, see `DocsLayout`'s own comment).
            div { class: "dx-home-page",
                div { id: "hero",
                    div { class: "dx-hero-shell",
                        h1 { class: "dx-hero-heading",
                            span { class: "dx-hero-title", "{SITE_NAME}" }
                            span { class: "dx-hero-subtitle",
                                "beautiful, accessible, responsive components for dioxus apps"
                            }
                        }
                        p { class: "dx-hero-summary",
                            "Accessible Dioxus components in the shadcn/ui style. Browse the catalog, install the CLI, and pull only what you need into your project."
                        }
                        div { class: "dx-hero-cta",
                            // link-ok: home-page hero button, never `/docs`
                            Link { to: Route::docs(), class: "dx-hero-cta-primary",
                                "get started"
                                ArrowRight { size: "18", stroke_width: "1.8" }
                            }
                            div { class: "dx-hero-command",
                                span { class: "dx-hero-prompt", "$" }
                                code { "{cli_install_command()}" }
                                CopyCommandButton { command: cli_install_command() }
                            }
                        }
                    }
                }
                WidgetMasonry { heading_id: docs_section_slug(HOME_SECTIONS[0]) }
                section { class: "dx-home-section dx-catalog-section",
                    header { class: "dx-section-header",
                        span { class: "dx-section-eyebrow", "Catalog" }
                        h2 { id: "{docs_section_slug(HOME_SECTIONS[1])}", class: "dx-section-title", "{HOME_SECTIONS[1]}" }
                        p { class: "dx-section-summary",
                            "Every primitive in the library, with live previews and a copy-paste install command for each one."
                        }
                    }
                    ComponentGallery {}
                }
            }
        }
    }
}

/// Which fixed column group of the masonry a card belongs to. A card's column is decided HERE, at
/// compile time, and is the same in the prerendered HTML, in a half-parsed document and after hydration:
/// it does not depend on the card's height or on how many of its neighbours have arrived (backlog row
/// 214; a CSS multi-column masonry re-assigns cards to columns on every parsed card).
///
/// Three columns above 1100px: `Left`, `Middle`, and `RightUpper` stacked on `RightLower`. Two columns
/// from 641 to 1100px: `Left` over `RightUpper`, and `Middle` over `RightLower` (a grid with two rows,
/// so the row-one tops align and only the shorter of `Left`/`Middle` leaves a gap, kept under ~35px by
/// the assignment below). One column below. See `.dx-widget-masonry` in `main.css`.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum MasonrySlot {
    Left,
    Middle,
    RightUpper,
    RightLower,
}

pub(crate) struct MasonryEntry {
    pub(crate) component: fn() -> Element,
    pub(crate) popout: bool,
    pub(crate) slot: MasonrySlot,
}

pub(crate) const BLOCKS: &[MasonryEntry] = &[
    MasonryEntry {
        component: BlockSignIn,
        popout: false,
        slot: MasonrySlot::Left,
    },
    MasonryEntry {
        component: BlockProfile,
        popout: false,
        slot: MasonrySlot::Middle,
    },
    MasonryEntry {
        component: BlockStats,
        popout: false,
        slot: MasonrySlot::RightUpper,
    },
    MasonryEntry {
        component: BlockInbox,
        popout: false,
        slot: MasonrySlot::Left,
    },
    MasonryEntry {
        component: BlockTasks,
        popout: false,
        slot: MasonrySlot::Middle,
    },
    MasonryEntry {
        component: BlockNotifications,
        popout: false,
        slot: MasonrySlot::Middle,
    },
    MasonryEntry {
        component: BlockPlayer,
        popout: false,
        slot: MasonrySlot::RightLower,
    },
    MasonryEntry {
        component: BlockCommand,
        popout: true,
        slot: MasonrySlot::RightUpper,
    },
    MasonryEntry {
        component: BlockComposer,
        popout: false,
        slot: MasonrySlot::Middle,
    },
    MasonryEntry {
        component: BlockPricing,
        popout: false,
        slot: MasonrySlot::Left,
    },
    MasonryEntry {
        component: BlockFilters,
        popout: false,
        slot: MasonrySlot::Left,
    },
    MasonryEntry {
        component: BlockColorPalette,
        popout: true,
        slot: MasonrySlot::Middle,
    },
    MasonryEntry {
        component: BlockTabs,
        popout: false,
        slot: MasonrySlot::RightUpper,
    },
    MasonryEntry {
        component: BlockSchedule,
        popout: false,
        slot: MasonrySlot::RightLower,
    },
];

// `heading_id` is `Home()`'s `docs_section_slug(HOME_SECTIONS[0])` --
// threaded in as a prop (rather than this fn reaching for `HOME_SECTIONS`
// itself) because the id belongs to whichever page mounts this section,
// and `Home()` is the only caller today. The heading TEXT below still
// reads from `HOME_SECTIONS[0]` directly (same module, no need to thread
// that too), so id and text can't drift apart from each other.
#[component]
pub(crate) fn WidgetMasonry(heading_id: String) -> Element {
    rsx! {
        section { class: "dx-home-section dx-masonry-section",
            header { class: "dx-section-header",
                span { class: "dx-section-eyebrow", "Showcase" }
                h2 { id: "{heading_id}", class: "dx-section-title", "{HOME_SECTIONS[0]}" }
                p { class: "dx-section-summary",
                    "Live, interactive UI blocks composed from the primitives below. Use your keyboard to test the accessibility interactions."
                }
            }
            div { class: "dx-widget-masonry",
                MasonryColumn { slot: MasonrySlot::Left }
                MasonryColumn { slot: MasonrySlot::Middle }
                div { class: "dx-masonry-right",
                    MasonryColumn { slot: MasonrySlot::RightUpper }
                    MasonryColumn { slot: MasonrySlot::RightLower }
                }
            }
            // The hero and the whole masonry are the home page's top group.
            FirstPaintGate {}
        }
    }
}

/// One fixed column group of the masonry: the cards whose `BLOCKS` entry names `slot`, in `BLOCKS` order.
#[component]
pub(crate) fn MasonryColumn(slot: MasonrySlot) -> Element {
    rsx! {
        div { class: "dx-masonry-col",
            for entry in BLOCKS.iter().filter(|entry| entry.slot == slot) {
                MasonryCard { popout: entry.popout,
                    // A real component, never `(entry.component)()`: see `MasonryCard`.
                    {
                        let Block = entry.component;
                        rsx! { Block {} }
                    }
                }
            }
        }
    }
}

/// Wraps one showcase block. The block arrives as `children`, built by
/// `WidgetMasonry` as a component element (`Block {}`), so every block owns a
/// scope and its hooks live in it.
///
/// It must NOT be passed as a `Callback<(), Element>` that this component then
/// invokes (`component.call(())` around `(entry.component)()`):
/// `Callback::call` runs its closure with the scope it was
/// CREATED in on top of the stack (dioxus-core `events.rs`, `with_scope_on_stack`),
/// i.e. `WidgetMasonry`'s, not this card's. A block that calls hooks directly
/// (`BlockPlayer`, `BlockColorPalette`, `BlockCommand`, `BlockComposer`) then
/// pushes them onto `WidgetMasonry`'s hook list, whose `hook_index` is only
/// reset when `WidgetMasonry` itself re-renders (never). Every later re-render of
/// the card would run those hooks at an index past the end of the list and
/// allocate a FRESH set (new signal at its initial value, new memo, new
/// effect) while the old ones live on, unreachable and never dropped. For
/// `BlockPlayer` that is a new, never-cleared 100 ms `setInterval` per
/// re-render (one a second, since each fresh player re-renders when its label
/// ticks), and the readout never gets past 1:24 because every fresh signal
/// restarts at 84 s (dev-docs/archive/research/scroll-jank-2026-10-04.md, Cause 1).
#[component]
pub(crate) fn MasonryCard(#[props(default)] popout: bool, children: Element) -> Element {
    let class = if popout {
        "dx-widget-card dx-widget-card-popout"
    } else {
        "dx-widget-card"
    };
    rsx! {
        Card { class, CardContent { {children} } }
    }
}
