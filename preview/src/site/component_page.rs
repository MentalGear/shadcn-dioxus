use super::*;

/// The `h2` headings the component page renders itself (`ComponentHighlight`) or that `build.rs` adds
/// around a component's `docs.md` ("Usage notes" wraps the text before its first `##`). A `docs.md` must not add an `h2` with one of these
/// texts (the page would show it twice); `docs_md_has_no_template_h2` below
/// enforces that, so the list is the single source for both.
pub(crate) const PAGE_HEADING_INSTALLATION: &str = "Installation";
#[cfg(test)]
pub(crate) const PAGE_HEADING_USAGE: &str = "Usage notes";
pub(crate) const PAGE_HEADING_VARIANTS: &str = "Examples";
/// "Usage" is reserved too: the text before a docs.md's first `##` is rendered under "Usage notes", so
/// a docs h2 called "Usage" would read as a nested duplicate of it.
#[cfg(test)]
pub(crate) const TEMPLATE_H2_HEADINGS: [&str; 4] = [
    PAGE_HEADING_INSTALLATION,
    PAGE_HEADING_USAGE,
    "Usage",
    PAGE_HEADING_VARIANTS,
];

/// Prefixes the app base path (`dx --base-path`, e.g. `/shadcn-dioxus`)
/// onto root-absolute `href="/..."` links in the build-time rendered
/// `docs.md` HTML. `build.rs` bakes that HTML before the base path is
/// known, and a root-absolute href otherwise escapes the base path when
/// opened in a new tab or without JS (it resolves to `host/component/...`, a 404,
/// on the Pages deploy). Uses the router's own prefix -- the same value
/// every `Link` prepends -- so server and client render identically.
pub(crate) fn prefix_internal_hrefs(docs: &'static str) -> String {
    prefix_root_absolute_hrefs(docs, &router().prefix().unwrap_or_default())
}

pub(crate) fn prefix_root_absolute_hrefs(html: &str, prefix: &str) -> String {
    const ATTR: &str = "href=\"";
    let prefix = prefix.trim_end_matches('/');
    if prefix.is_empty() {
        return html.to_string();
    }
    let mut out = String::with_capacity(html.len() + 64);
    let mut rest = html;
    while let Some(at) = rest.find(ATTR) {
        let (head, value) = rest.split_at(at + ATTR.len());
        out.push_str(head);
        // Only a genuinely root-absolute path gets the prefix: not a
        // protocol-relative `//host/...` link, and not one that already
        // carries it (`<prefix>/...`), so running this twice -- or over
        // pre-prefixed markup -- can never double-prefix.
        let root_absolute = value.starts_with('/') && !value.starts_with("//");
        let already_prefixed = value
            .strip_prefix(prefix)
            .is_some_and(|after| after.starts_with(['/', '?', '#', '"']));
        if root_absolute && !already_prefixed {
            out.push_str(prefix);
        }
        rest = value;
    }
    out.push_str(rest);
    out
}

/// Opens the closed Accordion item or Collapsible (docs.md sections, Installation, "Manual installation
/// files", a demo's notes) that holds the URL fragment's target, on load and on every `hashchange`, then
/// scrolls to the target.
///
/// - A fragment naming a section's `h2` (`#installation`, `#docs-<slug>`: the h2 wraps the trigger, so it
///   is always visible) clicks that item's trigger: an ordinary, animated open.
/// - A fragment naming something inside closed content: every closed panel around it is mounted as
///   `hidden="until-found"` (`hidden_until_found`), and the browser reveals it itself for a fresh load or
///   a fragment navigation, then fires `beforematch`, which the primitive turns into an open without the
///   open animation. A hydrating page can miss that event, and a `hashchange` the router handles may never
///   cause it, so this dispatches the same `beforematch` on each closed panel around the target: one path,
///   the primitive's own (`use_beforematch`), whichever came first.
///
/// Runs two frames after install, so the panels' own `beforematch` listeners (effects of the child
/// components, installed in the same flush) exist when it dispatches.
pub(crate) fn use_open_section_for_hash() {
    use dioxus_primitives::js_listener::{use_js_listeners, JsListeners};
    use_js_listeners(move || {
        Some(JsListeners::install(
            "const open = () => {
                 let id = '';
                 try { id = decodeURIComponent(location.hash.slice(1)); } catch (_) {}
                 const target = id && document.getElementById(id);
                 if (!target) return;
                 const trigger = target.matches('h2') && target.querySelector('button[aria-expanded=\"false\"]');
                 if (trigger) trigger.click();
                 const closed = [];
                 for (let p = target.parentElement; p; p = p.parentElement) {
                     if (p.id && (p.getAttribute('hidden') === 'until-found' || p.getAttribute('data-open') === 'false')) closed.push(p);
                 }
                 closed.forEach((p) => p.dispatchEvent(new Event('beforematch')));
                 let frames = 0;
                 const settle = () => {
                     const waiting = closed.some((p) => p.hasAttribute('hidden'));
                     if (waiting && frames++ < 60) { requestAnimationFrame(settle); return; }
                     target.scrollIntoView();
                 };
                 settle();
             };
             requestAnimationFrame(() => requestAnimationFrame(open));
             listen(window, 'hashchange', open);",
        ))
    });
}

#[component]
pub(crate) fn ComponentHighlight(demo: ComponentDemoData) -> Element {
    use components::accordion::{
        Accordion, AccordionChevron, AccordionContent, AccordionItem, AccordionTrigger,
    };
    use components::collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};
    use_open_section_for_hash();
    let ComponentDemoData {
        name: raw_name,
        r#type,
        docs,
        description,
        variants,
        component,
        style,
    } = demo;
    let name = raw_name.replace("_", " ");
    let [main, variants @ ..] = variants else {
        unreachable!("Expected at least one variant for component: {}", name);
    };

    rsx! {
        DocsLayout { active: DocsNavActive::Component(raw_name),
            article { class: "dx-component-page",
                header { class: "dx-component-page-header",
                    p { class: "dx-docs-eyebrow", "Component" }
                    div { class: "dx-component-page-title-row",
                        // The badge is a sibling of the `h1`, not inside it, so the
                        // heading's accessible name stays the component's name.
                        div { class: "dx-component-page-title",
                            h1 { "{name}" }
                            ExtraBadge { name: raw_name, tip: ExtraBadgeTip::Tooltip }
                        }
                        // The commands live in the Installation section below (one place); the
                        // header only points at it, and the hash handler opens that disclosure.
                        div { class: "dx-component-page-actions",
                            if matches!(r#type, ComponentType::Normal) {
                                DirectionToggle {}
                            }
                            a { class: "dx-component-install-link", href: "#installation", "Install" }
                        }
                    }
                    ExtraExplainer { name: raw_name }
                    p { "{description}" }
                    if charts_gallery::offers_gallery_link(raw_name) {
                        // link-ok: content link on a component page, never `/charts/`
                        Link {
                            to: Route::charts_index(),
                            class: "dx-charts-gallery-link",
                            "Browse all charts"
                            ArrowRight { size: "1rem", "aria-hidden": "true" }
                        }
                    }
                }
                section { class: "dx-component-section",
                    match r#type {
                        ComponentType::Normal => rsx! {
                            ComponentVariantHighlight { variant: main.clone(), main_variant: true, owner: raw_name }
                        },
                        ComponentType::Block => rsx! {
                            BlockComponentVariantHighlight { variant: main.clone(), main_variant: true, component_name: raw_name }
                        },
                    }
                }
                // Installation and the docs.md sections are ONE `Accordion` of the library's own (dev-docs/
                // decisions.md: the docs are built from its components, so they look and move like them):
                // several may be open, every item starts closed, and closed content stays in the server
                // markup as `hidden="until-found"`, so find-in-page and `#fragment` links still reach it
                // and hydration changes nothing the reader sees. Each `h2` wraps its trigger (shadcn's
                // heading-around-trigger shape), so the outline and the anchors stay. The docs.md items
                // are `build.rs` data (`render_docs_sections`), Installation is written here.
                section { class: "dx-component-section dx-component-accordion dx-docs-prose",
                    // The Table component's sheet, for the docs.md and props-reference tables
                    // `build.rs` emits with its classes (they are HTML, not `Table` elements).
                    document::Link { rel: "stylesheet", href: asset!("/src/components/table/style.css") }
                    Accordion {
                        class: "dx-docs-accordion",
                        width: "100%",
                        allow_multiple_open: true,
                        collapsible: true,
                        hidden_until_found: true,
                        chevron: AccordionChevron::Start,
                        AccordionItem {
                            h2 { id: "installation", class: "dx-docs-accordion-heading",
                                AccordionTrigger { class: "dx-docs-accordion-trigger",
                                    "{PAGE_HEADING_INSTALLATION}"
                                }
                            }
                            AccordionContent {
                                ComponentInstallCommand { name: raw_name }
                                p { class: "dx-docs-muted",
                                    "Also adds the components and crates this one needs. First time in this app? "
                                    // link-ok: content link to the Docs page, never to a component page
                                    Link { to: Route::docs(), "Set up the CLI and your app" }
                                    " first."
                                }
                                Collapsible { class: "dx-component-manual-install", hidden_until_found: true,
                                    // Same leading chevron as the accordion rows around it (row 299): the
                                    // trigger wears the accordion's start-chevron recipe (`data-style` opts
                                    // out of the Collapsible's own), and `as` puts the icon first instead of
                                    // the Collapsible's trailing one.
                                    CollapsibleTrigger {
                                        class: "dx-docs-collapsible-trigger dx-accordion-trigger",
                                        "data-style": "accordion",
                                        "data-chevron": "start",
                                        r#as: move |el: AsChild| rsx! {
                                            button { ..el.attributes,
                                                ChevronDown {
                                                    class: "dx-accordion-expand-icon",
                                                    size: "16px",
                                                    stroke: "var(--dx-muted-foreground)",
                                                }
                                                "Manual installation files"
                                            }
                                        },
                                    }
                                    CollapsibleContent {
                                        ManualComponentInstallation { component, style }
                                    }
                                }
                            }
                        }
                        for section in docs.iter() {
                            AccordionItem { key: "{section.id}",
                                h2 { id: section.id, class: "dx-docs-accordion-heading",
                                    AccordionTrigger { class: "dx-docs-accordion-trigger",
                                        span { dangerous_inner_html: section.title_html }
                                    }
                                }
                                AccordionContent {
                                    div {
                                        class: "dx-component-description",
                                        dangerous_inner_html: prefix_internal_hrefs(section.html),
                                    }
                                }
                            }
                        }
                    }
                }
                if !variants.is_empty() {
                    // A new section after the docs accordion: `.dx-component-examples` adds the space above.
                    section { class: "dx-component-section dx-component-examples",
                        div { class: "dx-component-section-heading",
                            h2 { "{PAGE_HEADING_VARIANTS}" }
                            p { "Alternative examples for common configurations." }
                        }
                        for variant in variants {
                            div { class: "dx-component-variant",
                                match r#type {
                                    ComponentType::Normal => rsx! {
                                        ComponentVariantHighlight { variant: variant.clone(), main_variant: false, owner: raw_name }
                                    },
                                    ComponentType::Block => rsx! {
                                        BlockComponentVariantHighlight { variant: variant.clone(), main_variant: false, component_name: raw_name }
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The reading direction every demo on a component page is rendered in (backlog row 137): the one
/// switch that gives every component an RTL example. `ComponentVariantHighlight` wraps each demo in
/// a `DirectionProvider` reading this and sets `dir` on the demo frame, so primitives that read
/// `use_direction` and the logical-property CSS both follow it live. A global (not page state) so
/// the choice survives navigating to another component. Defaults to LTR, which is what the
/// prerendered pages contain.
pub(crate) static PREVIEW_DIRECTION: GlobalSignal<Direction> = Signal::global(|| Direction::Ltr);

/// The LTR/RTL toggle in a component page's header. A pressed button, because it is a two-state
/// switch of the whole page's demos, not a navigation.
#[component]
pub(crate) fn DirectionToggle() -> Element {
    let rtl = PREVIEW_DIRECTION() == Direction::Rtl;
    rsx! {
        Button {
            id: "dx-direction-toggle",
            variant: ButtonVariant::Outline,
            r#type: "button",
            "aria-pressed": "{rtl}",
            "data-direction": if rtl { "rtl" } else { "ltr" },
            onclick: move |_| {
                let next = if PREVIEW_DIRECTION() == Direction::Rtl {
                    Direction::Ltr
                } else {
                    Direction::Rtl
                };
                *PREVIEW_DIRECTION.write() = next;
            },
            "RTL"
        }
    }
}

/// The component's install command: the CLI line with its copy button. It is rendered once per page,
/// inside the Installation disclosure. The `dx components add --git` alternative is documented once, on
/// the Docs page ("Without the CLI"), not repeated on every component page.
#[component]
pub(crate) fn ComponentInstallCommand(name: &'static str) -> Element {
    let command = cli_add_command(name);

    rsx! {
        div { class: "dx-component-install-commands",
            div { class: "dx-component-inline-command",
                code { "{command}" }
                CopyCommandButton { command: command.clone() }
            }
        }
    }
}

/// How an [`ExtraBadge`] carries the reason it is an extra.
#[derive(Clone, Copy, PartialEq, Default)]
pub(crate) enum ExtraBadgeTip {
    /// The native `title` attribute: for a badge that sits inside another
    /// interactive element (the sidebar link) or beside a link (the homepage
    /// card), where a second focusable tooltip trigger would nest or crowd
    /// the control. Mouse-only, which is why the component page also prints
    /// the reason ([`ExtraExplainer`]).
    #[default]
    Title,
    /// The project's `Tooltip`: the badge becomes a focusable trigger, so the
    /// reason opens on hover and on keyboard focus. Only for a badge that is
    /// not inside another interactive element (the component page header).
    Tooltip,
}

/// The "Extra" badge: shown next to the name of a component that is not in
/// shadcn/ui's catalog (`components::extra_reason`, backed by `components::
/// CATALOG`, the one list). Renders nothing for a shadcn component or one
/// nobody has classified yet. The text is real ("Extra", so it is read out and
/// survives copy/paste); the reason it is an extra (`components::
/// extra_reason`, carried by `Origin::Extra`, so it cannot be missing) is the
/// tooltip, delivered per [`ExtraBadgeTip`]. It stays out of the heading
/// elements (the page `h1`, the card `h3`) so those keep the component's name
/// as their accessible name, and sits inside the sidebar link, where it
/// extends the link's name to `"<name> Extra"`.
#[component]
pub(crate) fn ExtraBadge(name: &'static str, #[props(default)] tip: ExtraBadgeTip) -> Element {
    let Some(reason) = components::extra_reason(name) else {
        return rsx! {};
    };
    match tip {
        ExtraBadgeTip::Title => rsx! {
            Badge {
                variant: BadgeVariant::Outline,
                class: "dx-extra-badge",
                title: reason,
                "Extra"
            }
        },
        ExtraBadgeTip::Tooltip => rsx! {
            Tooltip {
                TooltipTrigger {
                    Badge {
                        variant: BadgeVariant::Outline,
                        class: "dx-extra-badge",
                        "Extra"
                    }
                }
                TooltipContent { side: ContentSide::Bottom, align: ContentAlign::Start, "{reason}" }
            }
        },
    }
}

/// The visible, always-on form of the "Extra" reason, under the component
/// page's heading: `"Extra: not in shadcn/ui. <reason>"`. Unlike a tooltip it
/// works on touch and is read in order by screen readers. Renders nothing for
/// a shadcn component or an unclassified one.
#[component]
pub(crate) fn ExtraExplainer(name: &'static str) -> Element {
    let Some(reason) = components::extra_reason(name) else {
        return rsx! {};
    };
    rsx! {
        p { class: "dx-extra-explainer", "Extra: not in shadcn/ui. {reason}" }
    }
}

#[component]
pub(crate) fn ManualComponentInstallation(
    component: HighlightedCode,
    style: CssHighlight,
) -> Element {
    rsx! {
        div { class: "dx-component-manual-copy",
            p { class: "dx-docs-muted",
                "Copy the component source and CSS into your app. Import the shared theme CSS once near your app root."
            }
        }
        div { class: "dx-component-manual-code",
            ComponentCode {
                rs_highlighted: component,
                css_highlighted: style,
                component_type: ComponentType::Normal,
            }
        }
    }
}
