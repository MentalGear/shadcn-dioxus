use super::*;

/// The `/docs` "Overview" page's own section headings, in the order
/// `Docs()` renders them. Single-sourced here so `Docs()`'s `h2`s and
/// `DocsLayout`'s "Start" sidebar sub-items (rendered off the
/// `page_sections` prop `Docs()` passes it, see `DocsLayout`) can never
/// drift out of sync with each other -- each `h2`'s visible text and the
/// matching sub-link's label both read from this one array, and
/// `docs_section_slug` derives both the `h2`'s `id` and the sub-link's
/// `href` from that same text rather than a hand-typed anchor.
///
/// `Home()`'s own `HOME_SECTIONS` (near `Home()`, below) is the same
/// pattern applied to the homepage's own sections -- both are plain
/// `&'static [&'static str]` so `DocsLayout`'s `page_sections` prop can
/// carry either one without caring which page it came from.
pub(crate) const DOCS_SECTIONS: &[&str] = &[
    "How it works",
    "New project",
    "Add a component",
    "Recommended workflow",
    "Motion",
    "Utilities",
    "Touch sizing",
    "Contrast mode",
];

/// Turns a heading's visible text into a same-page anchor id: lowercased,
/// with every run of non-alphanumeric characters (spaces included)
/// collapsed to a single hyphen and none left dangling at either end.
/// "How it works" -> "how-it-works". Generic over any page's headings --
/// `Home()` reuses this as-is for its own sections.
pub(crate) fn docs_section_slug(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());
    let mut pending_hyphen = false;
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_hyphen && !slug.is_empty() {
                slug.push('-');
            }
            slug.push(ch.to_ascii_lowercase());
            pending_hyphen = false;
        } else {
            pending_hyphen = true;
        }
    }
    slug
}

#[component]
pub(crate) fn Docs(dark_mode: Option<bool>) -> Element {
    let cli_install = cli_install_command();
    let cli_new = "shadcn-dioxus new myapp";
    let cli_init = "shadcn-dioxus init";
    let cli_list = "shadcn-dioxus list";
    let cli_add_button = cli_add_command("button");
    let cli_update = "shadcn-dioxus update";
    let list_command = dx_list_command();
    let add_button = dx_add_command("button");
    let registry_toml = format!("[components.registry]\ngit = \"{REGISTRY_GIT_URL}\"");
    let new_command = dx_new_command();
    let add_button_plain = "dx components add button";
    rsx! {
        DocsLayout { active: DocsNavActive::Overview, page_sections: Some(DOCS_SECTIONS),
            article { class: "dx-docs-page dx-docs-prose",
                header { class: "dx-docs-page-header",
                    p { class: "dx-docs-eyebrow", "Docs" }
                    h1 { "Build with {SITE_NAME}" }
                    p {
                        "{SITE_NAME} is a collection of styled, accessible Dioxus components designed to be copied into your app. Use the CLI when you want the fastest path, or copy the source when you want complete ownership."
                    }
                }
                section { class: "dx-docs-section",
                    h2 { id: "{docs_section_slug(DOCS_SECTIONS[0])}", "{DOCS_SECTIONS[0]}" }
                    p {
                        "{SITE_NAME} is not yet on crates.io. Components ship from this Git repository, which is a registry: each styled component is a folder (its Rust source, its stylesheet and a manifest) that gets copied into your source tree, next to the unstyled "
                        code { "dioxus-primitives" }
                        " library from the same repository. The installer is the "
                        code { "shadcn-dioxus" }
                        " command-line tool. Install it once:"
                    }
                    div { class: "dx-docs-command",
                        code { "{cli_install}" }
                        CopyCommandButton { command: cli_install.clone() }
                    }
                    p {
                        "It reads this registry by default, so none of the commands below needs a "
                        code { "--git" }
                        " flag. To see everything that's available:"
                    }
                    div { class: "dx-docs-command",
                        code { "{cli_list}" }
                        CopyCommandButton { command: cli_list.to_string() }
                    }
                    p {
                        "Each "
                        code { "shadcn-dioxus add" }
                        " copies the component's Rust source and stylesheet into your project, installs the components it depends on, and adds the crates it needs (including "
                        code { "dioxus-primitives" }
                        ") to your "
                        code { "Cargo.toml" }
                        ". Once it's in your tree, the code is yours: keep the included CSS as-is, replace the class names with Tailwind utilities, or rewrite the styles from scratch. There is no runtime dependency on this registry after the copy."
                    }
                    p {
                        "If you manage dependencies by hand, the primitives library is this line in "
                        code { "Cargo.toml" }
                        ":"
                    }
                    DocsPre {
                        code { r#"dioxus-primitives = {{ git = "{REGISTRY_GIT_URL}" }}"# }
                    }
                }
                section { class: "dx-docs-section",
                    h2 { id: "{docs_section_slug(DOCS_SECTIONS[1])}", "{DOCS_SECTIONS[1]}" }
                    p { "Starting a new app? One command creates it from this repository's starter template, with no Dioxus CLI needed:" }
                    div { class: "dx-docs-command",
                        code { "{cli_new}" }
                        CopyCommandButton { command: cli_new.to_string() }
                    }
                    p {
                        "The starter already declares "
                        code { "mod components;" }
                        ", depends on "
                        code { "dioxus-primitives" }
                        " from this repository and names this registry in its "
                        code { "Dioxus.toml" }
                        ". Then, from inside the new app:"
                    }
                    div { class: "dx-docs-command",
                        code { "cd myapp && {cli_add_button}" }
                        CopyCommandButton { command: format!("cd myapp && {cli_add_button}") }
                    }
                    p {
                        "After your first add, link the shared theme once in your root component (the snippet under "
                        em { "Add a component" }
                        ", below)."
                    }
                    h3 { id: "new-project-with-dx", "The same with dx" }
                    p {
                        "The starter is a cargo-generate template, so the Dioxus CLI can create the same project. Inside it a plain "
                        code { "dx components add" }
                        " reads this registry too, because the starter's "
                        code { "Dioxus.toml" }
                        " names it:"
                    }
                    div { class: "dx-docs-command",
                        code { "{new_command}" }
                        CopyCommandButton { command: new_command.clone() }
                    }
                    div { class: "dx-docs-command",
                        code { "{add_button_plain}" }
                        CopyCommandButton { command: add_button_plain.to_string() }
                    }
                }
                section { class: "dx-docs-section",
                    h2 { id: "{docs_section_slug(DOCS_SECTIONS[2])}", "{DOCS_SECTIONS[2]}" }
                    p {
                        "In an existing Dioxus app, run this once from the project root:"
                    }
                    div { class: "dx-docs-command",
                        code { "{cli_init}" }
                        CopyCommandButton { command: cli_init.to_string() }
                    }
                    p {
                        code { "init" }
                        " writes this registry into your "
                        code { "Dioxus.toml" }
                        " (keeping your comments), creates "
                        code { "src/components" }
                        " with its "
                        code { "mod.rs" }
                        ", declares "
                        code { "mod components;" }
                        " in "
                        code { "src/main.rs" }
                        ", copies the shared theme to "
                        code { "assets/dx-components-theme.css" }
                        " and links it from your root component. It is safe to run again. Then add components by name, swapping "
                        code { "button" }
                        " for any component in the sidebar:"
                    }
                    div { class: "dx-docs-command",
                        code { "{cli_add_button}" }
                        CopyCommandButton { command: cli_add_button.clone() }
                    }
                    p {
                        "To bring installed components up to the registry's latest:"
                    }
                    div { class: "dx-docs-command",
                        code { "{cli_update}" }
                        CopyCommandButton { command: cli_update.to_string() }
                    }
                    p {
                        code { "update" }
                        " never overwrites a file you edited: a component with an edited file is left unchanged and listed, and the command exits non-zero. Pass "
                        code { "--force" }
                        " to replace it anyway. "
                        code { "shadcn-dioxus remove <name>" }
                        " uninstalls a component."
                    }
                    h3 { id: "without-the-cli", "Without the CLI" }
                    p {
                        "The registry is also a "
                        code { "dx components" }
                        " registry. Name it once in your app's "
                        code { "Dioxus.toml" }
                        " and plain "
                        code { "dx components add button" }
                        " reads it:"
                    }
                    DocsPre {
                        code { "{registry_toml}" }
                    }
                    p {
                        "Or pass the registry on every command, keeping the URL exactly as written (the components that depend on each other, and on "
                        code { "dioxus-primitives" }
                        ", name it too):"
                    }
                    div { class: "dx-docs-command",
                        code { "{add_button}" }
                        CopyCommandButton { command: add_button.clone() }
                    }
                    p {
                        code { "{list_command}" }
                        " lists what's available."
                    }
                    p {
                        strong { "Heads up: " }
                        "plain "
                        code { "dx components add" }
                        " without that setup (no "
                        code { "Dioxus.toml" }
                        " registry, no "
                        code { "--git" }
                        ") installs upstream's version from "
                        code { "DioxusLabs/components" }
                        ", not this one. The Dioxus CLI reads its own default registry unless you tell it otherwise."
                    }
                    p {
                        "With dx, the first add creates "
                        code { "src/components" }
                        " and copies the shared theme to "
                        code { "assets/dx-components-theme.css" }
                        ". Declare the module with "
                        code { "mod components;" }
                        " in your "
                        code { "main.rs" }
                        " and link the theme once in your root component (the CLI's "
                        code { "init" }
                        " does both for you):"
                    }
                    DocsPre {
                        code { r#"document::Link {{ rel: "stylesheet", href: asset!("/assets/dx-components-theme.css") }}"# }
                    }
                    p { class: "dx-docs-muted",
                        "If you do not have the Dioxus CLI yet, install it once with cargo install dioxus-cli."
                    }
                }
                section { class: "dx-docs-section",
                    h2 { id: "{docs_section_slug(DOCS_SECTIONS[3])}", "{DOCS_SECTIONS[3]}" }
                    ol {
                        li { "Pick a component from the sidebar or catalog." }
                        li { "Preview the default example and variants." }
                        li { "Open Installation on the component page and run its CLI command." }
                        li { "Customize the generated Rust and CSS to fit your app." }
                        li { "Run the update command now and then; it keeps your edits." }
                    }
                    // The header and the first four sections fill the first screens at every width: the top group.
                    // Inside the section, so `.dx-docs-section + .dx-docs-section` still matches the next one.
                    FirstPaintGate {}
                }
                section { class: "dx-docs-section",
                    h2 { id: "{docs_section_slug(DOCS_SECTIONS[4])}", "{DOCS_SECTIONS[4]}" }
                    crate::motion_docs::MotionDocs {}
                }
                section { class: "dx-docs-section",
                    h2 { id: "{docs_section_slug(DOCS_SECTIONS[5])}", "{DOCS_SECTIONS[5]}" }
                    crate::utilities_docs::UtilitiesDocs {}
                }
                section { class: "dx-docs-section",
                    h2 { id: "{docs_section_slug(DOCS_SECTIONS[6])}", "{DOCS_SECTIONS[6]}" }
                    crate::coarse_docs::CoarseDocs {}
                }
                section { class: "dx-docs-section",
                    h2 { id: "{docs_section_slug(DOCS_SECTIONS[7])}", "{DOCS_SECTIONS[7]}" }
                    crate::contrast_docs::ContrastDocs {}
                }
            }
        }
    }
}

/// Which sidebar nav entry (if any) corresponds to the page currently being
/// rendered inside `DocsLayout`, for active-link highlighting.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum DocsNavActive {
    /// The `/docs` written-guide page ("Start" > "Overview").
    Overview,
    /// A component page, keyed by the component's raw name.
    Component(&'static str),
    /// The homepage (`/`) -- part of this same nav, but distinct from
    /// `Overview`: a bare `Option<&'static str>` with `None` for "on
    /// Docs" would wrongly light up "Overview" on the gallery page.
    Home,
    /// The demo-apps index (`/demos`): "Start" > "Demos" (backlog row 303).
    Demos,
    /// The chart gallery (`/charts/...`, every chart type): "Start" > "Charts".
    Charts,
}

/// The heading-derived sub-item list nested under whichever "Start" nav
/// item corresponds to the page currently being viewed (see `DocsLayout`'s
/// `page_sections` prop and its "Start" group markup, below). Written once
/// and called from both the "Home" and "Overview" `SidebarMenuItem`s so
/// their sub-items can never diverge in markup shape -- same `SidebarMenuSub`/
/// `SidebarMenuSubItem`/`SidebarMenuSubButton` structure, same plain same-
/// page `a` anchor (not the router's `Link`) with a `Hash` icon, regardless
/// of which page's sections are being listed.
pub(crate) fn page_sections_submenu(sections: &'static [&'static str]) -> Element {
    rsx! {
        SidebarMenuSub {
            for title in sections.iter().copied() {
                SidebarMenuSubItem { key: "{title}",
                    SidebarMenuSubButton {
                        as: move |el: AsChild| rsx! {
                            a {
                                href: "#{docs_section_slug(title)}",
                                ..el.attributes,
                                Hash { size: "0.875rem", "aria-hidden": "true" }
                                span { "{title}" }
                            }
                        },
                    }
                }
            }
        }
    }
}

/// The site's real navigation shell: `SidebarProvider`/`Sidebar`/
/// `SidebarInset` composed the same way `sidebar/variants/main/mod.rs`'s own
/// demo (`Demo()`) composes them -- a `SidebarContent` of `SidebarGroup`s
/// holding `SidebarMenu`/`SidebarMenuItem`/`SidebarMenuButton`s, and a
/// `SidebarInset` whose own leading `header` holds just the `SidebarTrigger`
/// (mirrors both that demo's `Demo()` and the email-client dashboard's
/// `EmailClient()`, `dashboard/views/email_client/mod.rs`). `page` content
/// renders as the inset's children -- callers do not wrap it in a `<main>`
/// themselves, since `SidebarInset` already renders the page's one `<main>`
/// landmark (see `EmailClient()`'s own comment on that same point).
#[component]
pub(crate) fn DocsLayout(
    active: DocsNavActive,
    // Only the homepage passes these (`Home()`, via its own `Signal`s that
    // its `ComponentGalleryPreview`'s Sidebar card also writes to) so its
    // card's Side/Collapse buttons can reconfigure this same, real nav
    // in place. Every other call site (`/docs`, component pages) leaves
    // both `None` and gets its own fresh, fixed-default `Sidebar` state
    // here -- Dioxus already unmounts/remounts this whole layout on route
    // change (a new component instance per route), so that default can't
    // leak in from whatever the homepage's own sidebar was last set to.
    #[props(default)] side: Option<Signal<SidebarSide>>,
    #[props(default)] collapsible: Option<Signal<SidebarCollapsible>>,
    // Only `Home()` passes `Some(false)`, so the homepage's real site nav
    // starts closed/out-of-the-way behind its hero + gallery content (that
    // page already has its own primary draw, and the same `SidebarTrigger`
    // in the topbar still opens it). Every other call site (`/docs`,
    // component pages) leaves this `None` and keeps the open-by-
    // default behavior -- those pages need the nav immediately usable.
    // `SidebarProvider`'s own `default_open` (`components/sidebar/
    // component.rs`) carries it, threaded through here.
    #[props(default)] default_open: Option<bool>,
    // The CURRENT page's own section headings, if it has any -- `Docs()`
    // passes its `DOCS_SECTIONS`, `Home()` passes its `HOME_SECTIONS`.
    // Page-agnostic on purpose: which top-level "Start" nav item this
    // grows a heading-derived sub-list under is decided below purely from
    // `active` (`DocsNavActive::Home` nests it under "Home",
    // `DocsNavActive::Overview` under "Overview"), so this same prop
    // shape carries either page's list without `DocsLayout` needing a
    // separate prop -- or a separate `SidebarMenuSub` rendering block --
    // per page. `ComponentHighlight()` leaves this `None` and keeps
    // rendering both "Home" and "Overview" as plain, sub-item-less
    // entries.
    #[props(default)] page_sections: Option<&'static [&'static str]>,
    children: Element,
) -> Element {
    // Always call both hooks (rather than only inside an `unwrap_or_else`
    // closure) so hook order never depends on whether the caller passed
    // `side`/`collapsible` -- these locals are simply unused/discarded
    // when a real signal was supplied.
    let default_side = use_signal(|| SidebarSide::Left);
    let default_collapsible = use_signal(|| SidebarCollapsible::Offcanvas);
    let side = side.unwrap_or(default_side);
    let collapsible = collapsible.unwrap_or(default_collapsible);

    rsx! {
        SidebarProvider { class: "dx-docs-shell", default_open: default_open.unwrap_or(true),
            // `Navbar` renders first, as a direct child of this
            // `SidebarProvider` (whose own render is just `div { class:
            // "dx-sidebar-wrapper", {children} }` -- see `component.rs`) so
            // its `SidebarTrigger` finds this provider's `SidebarCtx` via
            // `use_context`. `Sidebar`/`SidebarInset` are grouped under
            // their own `.dx-docs-shell-body` row beneath it rather than
            // sitting as siblings of `Navbar` directly, because the base
            // `.dx-sidebar-wrapper` rule (`sidebar/style.css`) is a flex ROW
            // sized for exactly that pair -- a third flex child would sit
            // beside them instead of stacking above (see `main.css`'s
            // `.dx-docs-shell-body` comment for the full rationale).
            Navbar {}
            div { class: "dx-docs-shell-body",
                Sidebar { side: side(), collapsible: collapsible(),
                    SidebarContent {
                        SidebarGroup {
                            SidebarGroupLabel {
                                BookOpen { size: "1rem", "aria-hidden": "true" }
                                span { "Start" }
                            }
                            SidebarMenu {
                                SidebarMenuItem {
                                    SidebarMenuButton {
                                        is_active: active == DocsNavActive::Home,
                                        as: move |el: AsChild| rsx! {
                                            NavLink { to: Route::home(), attributes: el.attributes,
                                                House { size: "1rem", "aria-hidden": "true" }
                                                span { "Home" }
                                            }
                                        },
                                    }
                                    // Only rendered while `Home()` is the page being viewed
                                    // AND it passed its own `page_sections` (`HOME_SECTIONS`)
                                    // -- see `DocsLayout`'s prop doc comment and
                                    // `page_sections_submenu`. On `/docs` or a component page
                                    // this is `None`/doesn't match `active`, so "Home" stays
                                    // the plain, sub-item-less link.
                                    if active == DocsNavActive::Home {
                                        if let Some(sections) = page_sections {
                                            {page_sections_submenu(sections)}
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    SidebarMenuButton {
                                        is_active: active == DocsNavActive::Overview,
                                        as: move |el: AsChild| rsx! {
                                            NavLink { to: Route::docs(), attributes: el.attributes,
                                                FileText { size: "1rem", "aria-hidden": "true" }
                                                span { "Overview" }
                                            }
                                        },
                                    }
                                    // Mirrors the "Home" item above, but scoped to `Docs()`
                                    // being the page being viewed instead -- same shared
                                    // `page_sections_submenu`, so `/docs`'s 3 sub-items
                                    // (`DOCS_SECTIONS`) render the same way as under
                                    // "Home".
                                    if active == DocsNavActive::Overview {
                                        if let Some(sections) = page_sections {
                                            {page_sections_submenu(sections)}
                                        }
                                    }
                                }
                                // The two standalone galleries the top nav also links to
                                // (backlog row 303): they render inside this same shell, so
                                // the sidebar and its trigger are the same on every page.
                                SidebarMenuItem {
                                    SidebarMenuButton {
                                        is_active: active == DocsNavActive::Demos,
                                        as: move |el: AsChild| rsx! {
                                            NavLink { to: Route::demos(), attributes: el.attributes,
                                                Monitor { size: "1rem", "aria-hidden": "true" }
                                                span { "Demos" }
                                            }
                                        },
                                    }
                                }
                                SidebarMenuItem {
                                    SidebarMenuButton {
                                        is_active: active == DocsNavActive::Charts,
                                        as: move |el: AsChild| rsx! {
                                            // Current on every chart type (`/charts/bar/` ...), not only on `/charts/`.
                                            NavLink {
                                                to: Route::charts_index(),
                                                active: active == DocsNavActive::Charts,
                                                attributes: el.attributes,
                                                ChartColumn { size: "1rem", "aria-hidden": "true" }
                                                span { "Charts" }
                                            }
                                        },
                                    }
                                }
                            }
                        }
                        // A-Z by default or by category: see `docs_nav`.
                        docs_nav::DocsComponentNav { active }
                    }
                }
                SidebarInset { id: MAIN_ID, tabindex: "-1", {children} }
            }
        }
    }
}
