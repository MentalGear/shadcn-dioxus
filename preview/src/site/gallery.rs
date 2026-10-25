use super::*;

#[component]
pub(crate) fn ComponentGallery() -> Element {
    rsx! {
        div { class: "dx-component-gallery",
            // `top_layer` is excluded here, deliberately: it is not a real,
            // installable component (there is no `dx components add
            // top_layer`) but an oracle fixture --
            // `preview/src/components/top_layer/component.rs`'s
            // `TopLayerFixture`, a large probe surface for
            // `playwright/oracle/tier2-html/top-layer.spec.ts` and
            // `native-dialog.spec.ts` (clipping-escape triggers, background-
            // inertness probes, native `<dialog>`/`<div popover>` references,
            // ...). Embedding it inline on `/` would put dozens of raw
            // `dioxus_primitives::` elements and duplicate landmark-shaped
            // markup into the one page every visitor and every gallery-wide
            // oracle (`preview.spec.ts`, hydration-parity Rule 4b) scans,
            // for a fixture no one browsing the catalog is here to see. It
            // stays fully reachable at its own page,
            // `/component/top_layer/` (still listed under Overlays in
            // `DocsSidebar`, since `components::DEMOS` itself lists it
            // and this filter only narrows the *gallery grid's* iteration)
            // -- see dev-docs/conformance-harness.md's hydration-parity
            // section for Rule 4b's subject (the fixture's toast region does
            // not render on `/`).
            for component in components::DEMOS.iter().filter(|c| c.name != "top_layer").cloned() {
                ComponentGalleryPreview { component }
            }
        }
    }
}

#[component]
pub(crate) fn ComponentGalleryPreview(component: ComponentDemoData) -> Element {
    let ComponentDemoData {
        name,
        r#type,
        description,
        variants,
        ..
    } = component;

    let first_variant = &variants[0];
    let Comp = first_variant.component;
    let display_name = name.replace("_", " ");
    let install_command = cli_add_command(name);

    // Only the `sidebar` card, and only when rendered under `Home()` (which
    // is the sole provider of this context -- see `HomeSidebarControls`'s
    // own doc comment), gets live controls instead of the plain
    // "Open full preview" link every other `ComponentType::Block` entry
    // falls back to. `try_consume_context` (not `consume_context`)
    // is what makes that fallback safe rather than a panic: this same
    // component would otherwise need every future caller to remember to
    // provide the context, for a card that doesn't use it.
    let home_sidebar_controls = (name == "sidebar")
        .then(try_consume_context::<HomeSidebarControls>)
        .flatten();

    let preview = match (r#type, home_sidebar_controls) {
        (ComponentType::Normal, _) => rsx! {
            Comp {}
        },
        (ComponentType::Block, Some(controls)) => rsx! {
            SidebarGalleryCardControls { controls }
        },
        (ComponentType::Block, None) => rsx! {
            // link-ok: home-gallery card link, never its own component page
            Link {
                to: Route::component(name),
                class: "dx-component-card-block-link",
                "Open full preview"
                ArrowUpRight { size: "18", stroke_width: "1.6" }
            }
        },
    };

    rsx! {
        // The `article` stays the layout/perf wrapper (`content-visibility`, the row spacing); the
        // card itself is the library's `Card` (header: title + description, content: the live demo,
        // footer: the install command).
        article { class: "dx-component-card",
            Card { class: "dx-component-card-body",
                CardHeader {
                    div { class: "dx-component-card-title-row",
                        CardTitle {
                            h3 { class: "dx-component-card-title",
                                // link-ok: home-gallery card link, never its own component page
                                Link {
                                    to: Route::component(name),
                                    class: "dx-component-card-title-link",
                                    "{display_name}"
                                    ArrowUpRight { size: "18", stroke_width: "1.6" }
                                }
                            }
                        }
                        ExtraBadge { name, tip: ExtraBadgeTip::Title }
                    }
                    CardDescription { "{description}" }
                }
                CardContent { class: "dx-component-card-preview", {preview} }
                CardFooter { class: "dx-component-card-footer",
                    div { class: "dx-component-card-command",
                        code { "{install_command}" }
                        CopyCommandButton { command: install_command.clone() }
                    }
                }
            }
        }
    }
}

/// The homepage's Sidebar gallery card's own controls -- Side/Collapse
/// buttons in the same shape as `sidebar/variants/main/mod.rs`'s
/// `DemoSettingControls` (that file's own demo page), but writing to
/// `Home()`'s signals instead of a demo-local pair, so they reconfigure
/// the real, on-page site nav (`DocsLayout`'s `Sidebar`) live.
#[component]
pub(crate) fn SidebarGalleryCardControls(controls: HomeSidebarControls) -> Element {
    let HomeSidebarControls {
        mut side,
        mut collapsible,
    } = controls;

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.75rem; width: 100%;",
            div { style: "display: flex; align-items: center; justify-content: space-between; gap: 0.75rem; flex-wrap: wrap;",
                span { style: "font-size: 0.75rem; font-weight: 600; color: var(--dx-foreground);",
                    "Side"
                }
                div { style: "display: inline-flex; gap: 0.5rem;",
                    Button {
                        variant: if side() == SidebarSide::Left { ButtonVariant::Primary } else { ButtonVariant::Outline },
                        onclick: move |_| side.set(SidebarSide::Left),
                        style: "padding: 0.4rem 0.6rem; font-size: 0.75rem;",
                        "Left"
                    }
                    Button {
                        variant: if side() == SidebarSide::Right { ButtonVariant::Primary } else { ButtonVariant::Outline },
                        onclick: move |_| side.set(SidebarSide::Right),
                        style: "padding: 0.4rem 0.6rem; font-size: 0.75rem;",
                        "Right"
                    }
                }
            }
            div { style: "display: flex; align-items: center; justify-content: space-between; gap: 0.75rem; flex-wrap: wrap;",
                span { style: "font-size: 0.75rem; font-weight: 600; color: var(--dx-foreground);",
                    "Collapse"
                }
                div { style: "display: inline-flex; gap: 0.5rem; flex-wrap: wrap;",
                    Button {
                        variant: if collapsible() == SidebarCollapsible::Offcanvas { ButtonVariant::Primary } else { ButtonVariant::Outline },
                        onclick: move |_| collapsible.set(SidebarCollapsible::Offcanvas),
                        style: "padding: 0.4rem 0.6rem; font-size: 0.75rem;",
                        "Offcanvas"
                    }
                    Button {
                        variant: if collapsible() == SidebarCollapsible::Icon { ButtonVariant::Primary } else { ButtonVariant::Outline },
                        onclick: move |_| collapsible.set(SidebarCollapsible::Icon),
                        style: "padding: 0.4rem 0.6rem; font-size: 0.75rem;",
                        "Icon"
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn CopyCommandButton(
    command: String,
    /// The button's accessible name; two commands on one page (the CLI's and dx's) need different ones.
    #[props(default = "Copy install command".to_string())]
    label: String,
) -> Element {
    // Inline `onclick`, as in `CopyButton`: works before hydration, and `main.css` swaps the icons
    // on the `data-copied` it sets.
    rsx! {
        Button {
            variant: ButtonVariant::Ghost,
            size: ButtonSize::IconXs,
            class: "dx-copy-button",
            r#type: "button",
            aria_label: "{label}",
            "data-command": "{command}",
            "onclick": "navigator.clipboard.writeText(this.dataset.command); this.dataset.copied = 'true';",
            Copy { class: "dx-copy-icon" }
            Check { class: "dx-check-icon" }
        }
    }
}

#[component]
pub(crate) fn GotoIcon(mut props: LinkProps) -> Element {
    props.children = rsx! {
        ExternalLink {
            size: "20px",
            stroke: "var(--dx-foreground)",
        }
    };
    Link(props)
}

// Same class as `CssHighlight` (see its doc comment): this shared theme
// stylesheet lives under `preview/assets/`, not a component folder, but it
// is *also* embedded via `dioxus_code::code!()` for this same Style tab --
// so editing it hits the identical rustc-dep-info full-rebuild path a
// component's own `style.css` does, for the identical reason. It gets the
// identical fix.
#[cfg(not(debug_assertions))]
pub(crate) const THEME_CSS: CssHighlight = CssHighlight {
    embedded: HighlightedCode {
        source: dioxus_code::code!("/assets/dx-components-theme.css"),
    },
};

#[cfg(debug_assertions)]
pub(crate) const THEME_CSS: CssHighlight = CssHighlight {
    asset: asset!("/assets/dx-components-theme.css"),
};
