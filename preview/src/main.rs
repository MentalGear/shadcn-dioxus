use crate::components::{
    avatar::{AvatarImageSize, ImageAvatar},
    badge::{Badge, BadgeVariant, VerifiedIcon},
    button::{Button, ButtonSize, ButtonVariant},
    card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle},
    checkbox::Checkbox,
    color_picker::ColorPicker,
    combobox::{Combobox, ComboboxEmpty, ComboboxOption},
    drag_and_drop_list::DragAndDropList,
    input::Input,
    item::{
        Item, ItemContent, ItemDescription, ItemMedia, ItemMediaVariant, ItemTitle, ItemVariant,
    },
    label::Label,
    progress::Progress,
    radio_group::{RadioGroup, RadioItem},
    sidebar::{
        Sidebar, SidebarCollapsible, SidebarContent, SidebarCtx, SidebarGroup, SidebarGroupLabel,
        SidebarInset, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarMenuSub,
        SidebarMenuSubButton, SidebarMenuSubItem, SidebarProvider, SidebarSide, SidebarTrigger,
    },
    slider::Slider,
    switch::Switch,
    tabs::{TabContent, TabList, TabTrigger, Tabs, TabsVariant},
    textarea::{Textarea, TextareaVariant},
    toggle_group::{ToggleGroup, ToggleItem},
    tooltip::{Tooltip, TooltipContent, TooltipTrigger},
};
use charts_gallery::{Charts, ChartsKind};
use core::panic;
use dioxus::prelude::{dioxus_router::LinkProps, *};
use dioxus_code::{advanced::HighlightedSource, Code, CodeTheme, Theme};
use dioxus_i18n::prelude::{i18n, use_init_i18n, I18nConfig};
use dioxus_icons::lucide::{
    ArrowRight, ArrowUpRight, BookOpen, ChartColumn, Check, ChevronDown, ChevronLeft, Copy,
    ExternalLink, FileText, Hash, House, Mail, Menu, Monitor, Pause, Play, SkipBack, SkipForward,
};
use dioxus_primitives::direction::{Direction, DirectionProvider};
use dioxus_primitives::{ContentAlign, ContentSide};
use nav_link::NavLink;
use unic_langid::{langid, LanguageIdentifier};

#[cfg(test)]
mod chart_parity;
#[cfg(test)]
mod chart_tooltip_parity;
mod charts_gallery;
mod components;
mod dashboard;
mod docs_nav;
// Client builds only: the server keeps `ServerDocument` for the SSR `<head>` (see the module's docs).
#[cfg(not(feature = "server"))]
mod eager_head;
mod installed_source;
mod motion_docs;
mod nav_link;
#[cfg(test)]
mod polar_parity;
mod theme;
mod utilities_docs;
mod wasm_libc_shim;

/// The site's display name: the header brand, the footer, the home hero, the docs intro, every
/// `document::Title` suffix (`charts_gallery`) and the Open Graph `og:site_name`. The static shell's
/// `<title>` in `index.html` is the one place Rust cannot reach; `site_name_tests` pins it to this.
/// The crates keep their own names (`dioxus-primitives`), and so does the Git repository, which is
/// also the GitHub Pages path (`--base-path shadcn-dioxus`, renamed from `dioxus-components` on 2026-10-10): this is the SITE's name only.
pub(crate) const SITE_NAME: &str = "shadcn-dioxus";

/// This repository as a `dx components` registry: the root `component.json` plus every
/// `preview/src/components/<name>/component.json`. A bare `dx components add <name>` reads dx's
/// DEFAULT registry (upstream `DioxusLabs/components`), so every command we show carries
/// `--git {REGISTRY_GIT_URL}`. The same string is written into each `component.json` (its
/// `componentDependencies` entries and the `dioxus-primitives` cargo dependency) because dx resolves
/// a bare-name dependency in the default registry, never in the one the user named, and it keys its
/// clone cache on the exact URL text; `scripts/check-registry-url.sh` fails when the two drift.
pub(crate) const REGISTRY_GIT_URL: &str = "https://github.com/MentalGear/shadcn-dioxus";

/// The command that lists the registry's components.
fn dx_list_command() -> String {
    format!("dx components list --git {REGISTRY_GIT_URL}")
}

/// The command that installs `name` (a component directory name) from this registry.
fn dx_add_command(name: &str) -> String {
    format!("dx components add {name} --git {REGISTRY_GIT_URL}")
}

/// The command that scaffolds a new app from this repository's starter template
/// (`templates/starter`). The template's `Dioxus.toml` names `REGISTRY_GIT_URL` as the app's default
/// component registry, so afterwards a plain `dx components add <name>` reads this registry.
fn dx_new_command() -> String {
    format!("dx new myapp --template {REGISTRY_GIT_URL} --subtemplate templates/starter")
}

/// The command that installs the `shadcn-dioxus` installer CLI (`cli/`), the primary way to use this
/// registry: it reads this registry by default, so none of the commands below needs a `--git` flag.
/// `scripts/check-registry-url.sh` pins the same string in `cli/README.md` and the CLI's
/// `DEFAULT_REGISTRY_GIT_URL`.
fn cli_install_command() -> String {
    format!("cargo install --git {REGISTRY_GIT_URL} shadcn-dioxus")
}

/// The CLI command that installs `name` (a component directory name), its component dependencies and
/// its crates. The command on every component page's Installation section; `dx_add_command` is the same
/// install through dx, documented once on the Docs page.
fn cli_add_command(name: &str) -> String {
    format!("shadcn-dioxus add {name}")
}

/// The one-sentence description behind `<meta name="description">`, `og:description` and
/// `twitter:description`.
const SITE_DESCRIPTION: &str = "Accessible, themeable Dioxus components in the shadcn/ui style, built on unstyled primitives and copied into your project.";

/// Where the site is served from, without the path. A Pages repository rename moves the path, never
/// this; `og:image` needs an absolute URL because link-preview scrapers do not resolve relative ones.
const SITE_ORIGIN: &str = "https://mentalgear.github.io";

#[derive(Copy, Clone, PartialEq)]
enum ComponentType {
    /// Normal component as default.
    Normal,
    /// Component that render the preview inside an iframe for isolation.
    Block,
}

#[derive(Clone, PartialEq)]
struct ComponentDemoData {
    name: &'static str,
    r#type: ComponentType,
    description: &'static str,
    /// The page's docs sections (docs.md `##` sections, "Usage notes", "Props reference"), generated
    /// by `build.rs` (`render_docs_sections`) and rendered by `ComponentHighlight` as Accordion items.
    docs: &'static [DocsSection],
    component: HighlightedCode,
    style: CssHighlight,
    variants: &'static [ComponentVariantDemoData],
}

/// One section of a component page's docs, as `build.rs` generates it (its `DocsSection`).
#[derive(Clone, Copy, PartialEq)]
struct DocsSection {
    /// The `h2` id, the anchor target (`docs-<slug>`).
    id: &'static str,
    /// The heading's inline HTML.
    title_html: &'static str,
    /// The section body's HTML (root-absolute hrefs not yet prefixed).
    html: &'static str,
}

#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Clone, PartialEq)]
struct ComponentVariantDemoData {
    name: &'static str,
    rs_highlighted: HighlightedCode,
    css_highlighted: Option<CssHighlight>,
    component: fn() -> Element,
}

#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(App);
}

#[cfg(feature = "server")]
fn main() {
    use dioxus::server::axum::{routing::post, Json, Router};
    use dioxus::server::{DioxusRouterExt, IncrementalRendererConfig, ServeConfig};

    dioxus::server::serve(|| async {
        let cfg = ServeConfig::builder()
            // Enable incremental rendering
            .incremental(
                IncrementalRendererConfig::new()
                    // Store static files in the public directory where other static assets like wasm are stored
                    .static_dir(
                        std::env::current_exe()
                            .unwrap()
                            .parent()
                            .unwrap()
                            .join("public"),
                    )
                    // Don't clear the public folder on every build. The public folder has other files including the wasm
                    // binary and static assets required for the app to run
                    .clear_cache(false),
            )
            .enable_out_of_order_streaming();

        // Workaround for dioxus-cli 0.7.6: with `--base-path`, the `static_routes`
        // server function ends up under `/<base>/api/static_routes`, but the SSG
        // step POSTs to the unprefixed `/api/static_routes` and fails to parse
        // the empty body. Expose a shim at the root that returns the route list.
        //
        // This shim is ALSO the fix for dev-docs/backlog.md row 46 -- see
        // `server_static_routes`'s own doc comment for why.
        let router = Router::new()
            .route(
                "/api/static_routes",
                post(|| async { Json(server_static_routes()) }),
            )
            .serve_dioxus_application(cfg, App);

        Ok(router)
    })
}

/// The full list of routes `dx build --ssg` should prerender, as route
/// strings (`Route::to_string()`). Served by the `/api/static_routes` shim
/// above, which is the only caller.
///
/// `Route::static_routes()` (dioxus-router-0.7.9's own default,
/// `routable.rs`) only ever enumerates route variants whose path is made
/// ENTIRELY of literal segments -- it `filter_map`s away any variant
/// containing a `Dynamic`/`CatchAll` segment, so it can never expand
/// `ComponentDemoPath`'s `:name` (or `ComponentBlockDemoPath`'s
/// `:name`/`:variant`) into one concrete entry per `components::DEMOS` item;
/// it would just silently omit those routes entirely (dev-docs/backlog.md
/// row 46). Appending one resolved route string per demo (and per
/// block-demo variant) below is what actually makes every component page
/// SSG-enumerable. `Route::static_routes()` still supplies every genuinely
/// all-static route: `/`, `/docs`, `/demos`, `/dashboard/email-client`, and
/// the legacy bare `/component/`/`/component/block/` query-form shells (see
/// `ComponentDemo`/`ComponentBlockDemo`'s own doc comments for why those two
/// stay deliberately name-agnostic rather than being enumerated here too).
#[cfg(feature = "server")]
fn server_static_routes() -> Vec<String> {
    let mut routes: Vec<String> = Route::static_routes()
        .iter()
        .map(ToString::to_string)
        .collect();
    // The gallery's tabs: `Route::static_routes()` already lists the bare
    // `/charts/` (all-literal), but `/charts/:kind/` has a dynamic segment,
    // so every tab is enumerated here from `ChartKind::ALL` -- the same list
    // the tab row renders, so a tab can never lack its prerendered page.
    for kind in charts_gallery::ChartKind::ALL {
        let route = Route::ChartsKind {
            kind: kind.slug().to_string(),
            dark_mode: None,
        }
        .to_string();
        if !routes.contains(&route) {
            routes.push(route);
        }
    }
    for demo in components::DEMOS {
        routes.push(
            Route::ComponentDemoPath {
                name: demo.name.to_string(),
                iframe: None,
                dark_mode: None,
            }
            .to_string(),
        );
        if demo.r#type == ComponentType::Block {
            for variant in demo.variants {
                routes.push(
                    Route::ComponentBlockDemoPath {
                        name: demo.name.to_string(),
                        variant: variant.name.to_string(),
                        dark_mode: None,
                    }
                    .to_string(),
                );
            }
        }
    }
    routes
}

#[component]
pub fn App() -> Element {
    // Insert every `document::Link`/`Stylesheet`/`Meta` into `<head>` as it renders, not from an effect
    // that a same-turn teardown drops while Dioxus's de-duplication still counts the link as present
    // (the legacy `/component/?name=X` shell lost the header's `LanguageSelect`/`Popover` CSS that way).
    // First, so it is in place before any component can render a link. Client builds only: the server's
    // `ServerDocument` must keep collecting the SSR head. See `eager_head`.
    #[cfg(not(feature = "server"))]
    eager_head::use_eager_head_document();

    let locale = use_init_i18n(|| {
        I18nConfig::new(langid!("en-US"))
            .with_locale((langid!("en-US"), include_str!("i18n/en-US.ftl")))
            .with_locale((langid!("fr-FR"), include_str!("i18n/fr-FR.ftl")))
            .with_locale((langid!("es-ES"), include_str!("i18n/es-ES.ftl")))
            .with_locale((langid!("de-DE"), include_str!("i18n/de-DE.ftl")))
    });

    // `<html lang>` follows the ACTIVE locale, not whichever control last changed it (WCAG 3.1.1: a
    // German page read with an English voice). The static document says `lang="en"` and the app
    // starts on `en-US`, so SSR is right as it is; this effect re-reads `I18n::language()` (a signal)
    // and so also runs on every later `set_language`, from the header's `LanguageSelect` or any
    // future control. `<html>` is never vdom-owned, so writing it cannot desync hydration. The tag is
    // a `LanguageIdentifier` (letters and `-` only), so `{:?}` yields a valid JS string literal.
    use_effect(move || {
        let tag = locale.language().to_string();
        document::eval(&format!("document.documentElement.lang = {tag:?}"));
    });

    // Hydration-ready signal, by construction (dev-docs/backlog.md: SSG
    // interaction tests race hydration). `document::eval` only ever runs
    // client-side (a no-op string during SSR, per `dioxus-document`), and
    // `use_effect` only fires after a render has actually committed to the
    // DOM -- under the SSG lane that means after wasm has booted and
    // hydration has walked the tree and attached event listeners, not
    // merely after the prerendered HTML has loaded. `<html>` (`document
    // .documentElement`) is never vdom-owned by this app, so writing to its
    // `dataset` here cannot desync hydration or touch the SSR'd markup.
    // This effect reads no signal, so it runs exactly once, right after
    // that first commit, and never again.
    use_effect(|| {
        document::eval("document.documentElement.dataset.hydrated = 'true'");
    });

    rsx! {
        Router::<Route> {}
    }
}

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[layout(AppLayout)]
    #[layout(NavigationLayout)]
    #[route("/?:iframe&:dark_mode")]
    Home {
        iframe: Option<bool>,
        dark_mode: Option<bool>,
    },
    #[route("/docs?:dark_mode")]
    Docs { dark_mode: Option<bool> },
    #[route("/demos?:dark_mode")]
    Demos { dark_mode: Option<bool> },
    // The charts gallery (`charts_gallery.rs`): `/charts/` is the Area tab,
    // `/charts/<slug>/` every tab by name. The slug is a PATH segment, like
    // `ComponentDemoPath`'s name, so each tab is its own prerendered file.
    #[route("/charts/?:dark_mode")]
    Charts { dark_mode: Option<bool> },
    #[route("/charts/:kind/?:dark_mode")]
    ChartsKind {
        kind: String,
        dark_mode: Option<bool>,
    },
    // Legacy query-string deep link (dev-docs/backlog.md row 46). Kept ONLY
    // so old bookmarks/external links/the hundreds of existing Playwright
    // specs using this URL form keep working -- it is NOT SSG-enumerable by
    // construction (a query string can never become a distinct static FILE:
    // `dioxus-server-0.7.9`'s `FileSystemCache::map_path` strips everything
    // from `?` onward before mapping a route to a file path, so every
    // `name=X` value would collide onto the same `component/index.html`
    // even if this app's `/api/static_routes` enumerated every X). Renders a
    // name-agnostic loading shell (see `ComponentDemo`'s doc comment) and
    // redirects client-side to `ComponentDemoPath` once mounted.
    #[route("/component/?:name&:iframe&:dark_mode")]
    ComponentDemo {
        name: String,
        iframe: Option<bool>,
        dark_mode: Option<bool>,
    },
    // Canonical, SSG-enumerable component page (row 46's construction):
    // `name` lives in the PATH, so every `components::DEMOS` entry maps to
    // its own static file (`component/<name>/index.html`) instead of every
    // one collapsing onto a single query-keyed file. Every internal link
    // (`Route::component`) points here; `ComponentDemo` above only exists to
    // catch old links and hand them off to this route.
    #[route("/component/:name/?:iframe&:dark_mode")]
    ComponentDemoPath {
        name: String,
        iframe: Option<bool>,
        dark_mode: Option<bool>,
    },
    // Catch-all: a URL that matches no other route (a typo, a stale link, anything
    // GitHub Pages answers with `404.html`, see `scripts/emit-404.sh`). Without it
    // the router prints its own "Failed to parse route" debug output. Inside both
    // layouts so it has the same header and footer as the other not-found pages.
    // NEVER prerendered: `Route::static_routes()` drops every variant with a
    // catch-all segment, and `server_static_routes` enumerates only real pages.
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
    #[end_layout]
    // Legacy query-string block-demo deep link -- same reasoning and same
    // redirect-shell construction as `ComponentDemo` above;
    // `ComponentBlockDemoPath` is the canonical form.
    #[route("/component/block/?:name&:variant&:dark_mode")]
    ComponentBlockDemo {
        name: String,
        variant: Option<String>,
        dark_mode: Option<bool>,
    },
    // Canonical, SSG-enumerable block-demo page: both `name` AND `variant`
    // are path segments (`variant` required, unlike the legacy query form's
    // `Option` -- the canonical route has no "default variant" concept of
    // its own; `ComponentBlockDemo`'s redirect resolves that default before
    // handing off here), so every (demo, variant) pair -- not just every
    // demo -- gets its own static file.
    #[route("/component/block/:name/:variant/?:dark_mode")]
    ComponentBlockDemoPath {
        name: String,
        variant: String,
        dark_mode: Option<bool>,
    },
    #[route("/dashboard/email-client?:dark_mode")]
    EmailClientDashboard { dark_mode: Option<bool> },
}

impl Route {
    pub fn iframe(&self) -> Option<bool> {
        match self {
            Route::Home { iframe, .. } => *iframe,
            Route::Docs { .. } => None,
            Route::Demos { .. } => None,
            Route::Charts { .. } => None,
            Route::ChartsKind { .. } => None,
            Route::ComponentDemo { iframe, .. } => *iframe,
            Route::ComponentDemoPath { iframe, .. } => *iframe,
            Route::ComponentBlockDemo { .. } => None,
            Route::ComponentBlockDemoPath { .. } => None,
            Route::EmailClientDashboard { .. } => None,
            Route::NotFound { .. } => None,
        }
    }

    pub fn in_iframe() -> Option<bool> {
        let route: Self = router().current();
        route.iframe()
    }

    pub fn dark_mode(&self) -> Option<bool> {
        match self {
            Route::Home { dark_mode, .. } => *dark_mode,
            Route::Docs { dark_mode, .. } => *dark_mode,
            Route::Demos { dark_mode, .. } => *dark_mode,
            Route::Charts { dark_mode, .. } => *dark_mode,
            Route::ChartsKind { dark_mode, .. } => *dark_mode,
            Route::ComponentDemo { dark_mode, .. } => *dark_mode,
            Route::ComponentDemoPath { dark_mode, .. } => *dark_mode,
            Route::ComponentBlockDemo { dark_mode, .. } => *dark_mode,
            Route::ComponentBlockDemoPath { dark_mode, .. } => *dark_mode,
            Route::EmailClientDashboard { dark_mode, .. } => *dark_mode,
            Route::NotFound { .. } => None,
        }
    }

    pub fn in_dark_mode() -> Option<bool> {
        let route: Self = router().current();
        route.dark_mode()
    }

    pub fn home() -> Self {
        let iframe = Self::in_iframe();
        let dark_mode = Self::in_dark_mode();
        Self::Home { iframe, dark_mode }
    }

    pub fn docs() -> Self {
        let dark_mode = Self::in_dark_mode();
        Self::Docs { dark_mode }
    }

    pub fn demos() -> Self {
        let dark_mode = Self::in_dark_mode();
        Self::Demos { dark_mode }
    }

    /// The charts gallery's index (`/charts/`, which shows the Area tab).
    pub fn charts_index() -> Self {
        let dark_mode = Self::in_dark_mode();
        Self::Charts { dark_mode }
    }

    /// One chart type's tab of the gallery (`/charts/<slug>/`).
    pub fn charts(kind: charts_gallery::ChartKind) -> Self {
        let dark_mode = Self::in_dark_mode();
        Self::ChartsKind {
            kind: kind.slug().to_string(),
            dark_mode,
        }
    }

    /// The canonical component-page link every internal caller (sidebar,
    /// home gallery cards, the navbar demo fixture) should use --
    /// `ComponentDemoPath` (row 46), never the legacy query-string
    /// `ComponentDemo`, so every link this app renders itself is already
    /// SSG-enumerable.
    pub fn component(name: impl ToString) -> Self {
        let iframe = Self::in_iframe();
        let dark_mode = Self::in_dark_mode();
        Self::ComponentDemoPath {
            name: name.to_string(),
            iframe,
            dark_mode,
        }
    }

    /// The canonical block-demo iframe-content link (mirrors `component`
    /// above) -- `ComponentBlockDemoPath`, used by
    /// `BlockComponentVariantHighlight` to build the `<iframe src>` for a
    /// given demo's variant.
    pub fn component_block(name: impl ToString, variant: impl ToString) -> Self {
        let dark_mode = Self::in_dark_mode();
        Self::ComponentBlockDemoPath {
            name: name.to_string(),
            variant: variant.to_string(),
            dark_mode,
        }
    }
}

/// The outermost layout, applied to EVERY route (never popped by an
/// `#[end_layout]` anywhere in `Route`) -- this is deliberately where
/// `GlobalHead` renders (dev-docs/backlog.md row 46 finding, below), not
/// `NavigationLayout`, precisely because it stays mounted across a
/// client-side transition between ANY two routes in the app, including two
/// routes that both sit outside `NavigationLayout`
/// (`ComponentBlockDemo`/`ComponentBlockDemoPath`/`EmailClientDashboard`).
///
/// **Row 46 finding:** before this construction, those three routes each
/// called `GlobalHead {}` themselves (the same shape, three separate call
/// sites -- CLAUDE.md's "two or more occurrences is a class" case). That
/// was harmless as long as the ONLY way to reach any of them was a hard
/// page load, but `ComponentBlockDemo`'s new client-side redirect (this
/// same row) to `ComponentBlockDemoPath` made it the first construction in
/// this app to ever SPA-navigate between two routes that each mount their
/// own independent `GlobalHead` instance -- and doing so silently dropped
/// `main.css`/`dx-components-theme.css`/the Google Fonts `<link>`s
/// entirely (confirmed by reading `document.styleSheets` before/after:
/// present on a hard load of the destination route directly, absent after
/// the redirect transition) rather than erroring, a hydration-adjacent
/// silent-CSS-loss defect of the exact same *class* `oracle/tier2-html/
/// global-stylesheet.spec.ts` already exists to catch for the `@import`
/// case. Root cause (corrected 2026-10-04, see `eager_head`'s module docs):
/// `document::Link` records its href in a de-duplication set while it
/// RENDERS but only appends the `<link>` from an effect queued on its own
/// scope, and a removed scope's queued effects are dropped -- so the FIRST
/// `GlobalHead` instance, torn down by the redirect in the turn it first
/// rendered, never inserted anything, and the SECOND instance's identical
/// `document::Link`s were skipped as duplicates of links that were never
/// there. (Unmounting does not remove head elements, so the sheets could
/// only have been absent because they were never inserted.) This layout's
/// once-only mount fixes that instance for these four sheets; the class --
/// every other component's link on any torn-down subtree -- is fixed by
/// `eager_head`, installed in `App`. `GlobalHead` now mounts exactly once,
/// here, and simply never unmounts for the lifetime of the app.
/// Subsumes all three prior call sites
/// (`NavigationLayout`, `ComponentBlockDemo`/`ComponentBlockDemoPath`,
/// `EmailClientDashboard`); does not need a matching fix for
/// `NavigationLayout`'s own `hero.css` link, which stays where it is --
/// no evidence of the same defect there (nothing outside `NavigationLayout`
/// ever needed `hero.css`, so no cross-layout transition has ever unmounted
/// it under a sibling that also renders it).
#[component]
fn AppLayout() -> Element {
    use_effect(move || {
        theme::theme_seed();
        if let Some(dark_mode) = Route::in_dark_mode() {
            theme::set_theme(dark_mode);
        }
    });

    rsx! {
        GlobalHead {}
        Outlet::<Route> {}
    }
}

#[component]
fn NavigationLayout() -> Element {
    // Send the route to the parent window if in an iframe
    let mut initial_route = use_hook(|| CopyValue::new(true));
    use_effect(move || {
        let route: Route = router().current();

        // Only send route changes, not the initial route
        if initial_route() || !Route::in_iframe().unwrap_or_default() {
            initial_route.set(false);
            return;
        }

        let eval = document::eval(
            "let route = await dioxus.recv();
            window.top.postMessage({ 'route': route }, '*');",
        );
        let _ = eval.send(route.to_string());
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/hero.css") }
        Outlet::<Route> {}
        Footer {}
    }
}

/// The site's persistent top nav. Rendered as a descendant of `DocsLayout`'s
/// `SidebarProvider` on every route that has a sidebar (`Docs`, `Home`,
/// `ComponentDemo`/`ComponentHighlight`) so its leading `SidebarTrigger` can
/// sit inside this same row instead of a separate strip below it (the
/// previous layout: `NavigationLayout` rendered `Navbar {}` as a sibling of,
/// and before, `Outlet::<Route> {}`, so it was never a descendant of
/// `DocsLayout`'s `SidebarProvider` -- `DocsLayout` instead rendered its own
/// separate `header { class: "dx-docs-inset-topbar", SidebarTrigger {} }`
/// strip). `Demos` has no sidebar and renders `Navbar {}` directly (as does
/// `ComponentDemo`'s own "not found" branch, which also bypasses
/// `DocsLayout`) -- `try_consume_context` (not `consume_context`, same
/// defensive-read pattern as `HomeSidebarControls` above) is what makes
/// omitting the trigger there safe rather than a panic, since neither page
/// has a `SidebarProvider` ancestor.
/// The `id` of every page's `<main>`: the target of the "Skip to content" link `Navbar` renders. A page
/// that renders a `<main>` passes `id: MAIN_ID, tabindex: "-1"` (the `tabindex` lets the fragment
/// navigation move focus into it, not only scroll), so the link has the same target everywhere;
/// `playwright/all-routes.spec.ts` checks it on every route.
pub(crate) const MAIN_ID: &str = "main-content";

#[component]
fn Navbar() -> Element {
    let in_iframe = Route::in_iframe().unwrap_or_default();
    let in_component = matches!(
        router().current(),
        Route::ComponentDemo { .. } | Route::ComponentDemoPath { .. }
    );
    let has_sidebar = try_consume_context::<SidebarCtx>().is_some();
    if in_iframe {
        return rsx! {
            nav {
                class: "dx-preview-navbar",
                aria_label: "Primary",
                border: "none",
                padding: "1rem",
                justify_content: "flex-start",
                if has_sidebar {
                    SidebarTrigger {
                        class: "dx-navbar-sidebar-trigger",
                        icon: rsx! {
                            Menu { class: "dx-sidebar-trigger-icon", size: "1rem" }
                        },
                    }
                }
                if in_component {
                    NavLink {
                        to: Route::home(),
                        class: "dx-navbar-brand",
                        aria_label: "Back",
                        ChevronLeft {
                            size: "2rem",
                            stroke: "var(--dx-foreground)",
                        }
                    }
                }
            }
        };
    }

    rsx! {
        // WCAG 2.4.1: the first focusable element on the page. Without it a keyboard user tabs through
        // the whole docs sidebar (88 stops on `/component/button/`) before reaching the content. It is
        // visually hidden until focused (`main.css`, `.dx-skip-link`). The block-demo iframe branch
        // above omits it: that page is one demo, embedded, with no nav to skip.
        a { class: "dx-skip-link", href: "#{MAIN_ID}", "Skip to content" }
        nav { class: "dx-preview-navbar", aria_label: "Primary",
            div { class: "dx-navbar-inner",
                div { class: "dx-navbar-primary",
                    if has_sidebar {
                        SidebarTrigger {
                        class: "dx-navbar-sidebar-trigger",
                        icon: rsx! {
                            Menu { class: "dx-sidebar-trigger-icon", size: "1rem" }
                        },
                    }
                    }
                    NavLink { to: Route::home(), class: "dx-navbar-brand",
                        img {
                            src: asset!("/assets/dioxus_color.svg"),
                            alt: "Dioxus Logo",
                            width: "18",
                            height: "18",
                        }
                        span { "{SITE_NAME}" }
                    }
                    NavLink { to: Route::docs(), class: "dx-navbar-link", "Docs" }
                    NavLink { to: Route::demos(), class: "dx-navbar-link", "Demos" }
                    NavLink { to: Route::charts_index(), class: "dx-navbar-link", "Charts" }
                }
                div { class: "dx-navbar-utilities",
                    // TODO: restore once the primitives crate is published
                    // Link {
                    //     to: "https://crates.io/crates/dioxus-components",
                    //     class: "dx-navbar-link",
                    //     aria_label: "Dioxus-Components crates.io",
                    //     Icon {
                    //         width: "24px",
                    //         height: "24px",
                    //         viewBox: ViewBox::new(0, 0, 576, 512),
                    //         path {
                    //             d: "M290.8 48.6l78.4 29.7L288 109.5 206.8 78.3l78.4-29.7c1.8-.7 3.8-.7 5.7 0zM136 92.5l0 112.2c-1.3 .4-2.6 .8-3.9 1.3l-96 36.4C14.4 250.6 0 271.5 0 294.7L0 413.9c0 22.2 13.1 42.3 33.5 51.3l96 42.2c14.4 6.3 30.7 6.3 45.1 0L288 457.5l113.5 49.9c14.4 6.3 30.7 6.3 45.1 0l96-42.2c20.3-8.9 33.5-29.1 33.5-51.3l0-119.1c0-23.3-14.4-44.1-36.1-52.4l-96-36.4c-1.3-.5-2.6-.9-3.9-1.3l0-112.2c0-23.3-14.4-44.1-36.1-52.4l-96-36.4c-12.8-4.8-26.9-4.8-39.7 0l-96 36.4C150.4 48.4 136 69.3 136 92.5zM392 210.6l-82.4 31.2 0-89.2L392 121l0 89.6zM154.8 250.9l78.4 29.7L152 311.7 70.8 280.6l78.4-29.7c1.8-.7 3.8-.7 5.7 0zm18.8 204.4l0-100.5L256 323.2l0 95.9-82.4 36.2zM421.2 250.9c1.8-.7 3.8-.7 5.7 0l78.4 29.7L424 311.7l-81.2-31.1 78.4-29.7zM523.2 421.2l-77.6 34.1 0-100.5L528 323.2l0 90.7c0 3.2-1.9 6-4.8 7.3z",
                    //             fill: "currentColor",
                    //             fill_rule: "nonzero",
                    //         }
                    //     }
                    // }
                    // link-ok: external target, never the current page
                    Link {
                        to: REGISTRY_GIT_URL,
                        class: "dx-navbar-link",
                        img {
                            class: "dx-light-mode-only",
                            src: asset!("/assets/github-mark/github-mark.svg"),
                            alt: "GitHub",
                            width: "22",
                            height: "22",
                        }
                        img {
                            class: "dx-dark-mode-only",
                            src: asset!("/assets/github-mark/github-mark-white.svg"),
                            alt: "GitHub",
                            width: "22",
                            height: "22",
                        }
                    }
                    theme::ThemePicker {}
                    theme::DarkModeToggle {}
                    LanguageSelect {}
                }
            }
        }
    }
}

#[component]
fn Footer() -> Element {
    if Route::in_iframe().unwrap_or_default() {
        return rsx! {};
    }

    rsx! {
        footer { class: "dx-preview-footer",
            div { class: "dx-footer-inner",
                div { class: "dx-footer-brand",
                    NavLink { to: Route::home(), class: "dx-footer-brand-link",
                        img {
                            src: asset!("/assets/dioxus_color.svg"),
                            alt: "Dioxus Logo",
                            width: "22",
                            height: "22",
                        }
                        span { "{SITE_NAME}" }
                    }
                    p { class: "dx-footer-tagline",
                        "Accessible, themeable interface pieces for Dioxus apps."
                    }
                }
                nav { class: "dx-footer-nav", aria_label: "Footer",
                    div { class: "dx-footer-nav-group",
                        span { class: "dx-footer-nav-heading", "Library" }
                        NavLink { to: Route::home(), class: "dx-footer-link", "Components" }
                        NavLink { to: Route::docs(), class: "dx-footer-link", "Docs" }
                        NavLink { to: Route::demos(), class: "dx-footer-link", "Demos" }
                        NavLink { to: Route::charts_index(), class: "dx-footer-link", "Charts" }
                    }
                    div { class: "dx-footer-nav-group",
                        span { class: "dx-footer-nav-heading", "Project" }
                        // link-ok: external target, never the current page
                        Link {
                            to: REGISTRY_GIT_URL,
                            class: "dx-footer-link",
                            "GitHub"
                        }
                        // link-ok: external target, never the current page
                        Link {
                            to: "https://dioxuslabs.com",
                            class: "dx-footer-link",
                            "Dioxus"
                        }
                    }
                }
            }
            div { class: "dx-footer-base",
                span { class: "dx-footer-copy", "Built with Dioxus." }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct HighlightedCode {
    pub source: HighlightedSource,
}

/// A code listing in the docs prose. `.dx-docs-prose pre` scrolls sideways, and a scrollable region must
/// be reachable from the keyboard (axe `scrollable-region-focusable`), so every listing in the docs
/// goes through this one component instead of a bare `pre` (same reason `CodeBlock` is focusable).
#[component]
pub(crate) fn DocsPre(children: Element) -> Element {
    rsx! {
        pre { tabindex: "0", {children} }
    }
}

#[component]
fn CodeBlock(source: HighlightedCode) -> Element {
    rsx! {
        div {
            class: "dx-code-block",
            tabindex: "0",
            PreviewCode { source: source.source }
        }
        CopyButton { position: "absolute", top: "0.5em", right: "0.5em" }
    }
}

/// The line numbers shown beside a source listing: "1\n2\n...\nN", N being the number of lines
/// `Code` renders (it drops trailing newlines, so a file's final newline is not a line).
fn line_numbers(source: &str) -> String {
    let lines = source.trim_end_matches('\n').split('\n').count();
    (1..=lines)
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// A highlighted listing with its line numbers in a gutter. The gutter is a separate, `aria-hidden`,
/// `user-select: none` column (see `.dx-code-gutter`), so selecting and copying code never picks the
/// numbers up, and the Copy buttons hand over the source text untouched. It uses the code's own font
/// metrics and does not wrap, so each number stays level with its line.
#[component]
fn PreviewCode(source: HighlightedSource) -> Element {
    let numbers = line_numbers(source.source());
    rsx! {
        div {
            class: "dx-preview-code-theme",
            tabindex: "0",
            pre { class: "dx-code-gutter", "aria-hidden": "true", "data-slot": "code-gutter", "{numbers}" }
            Code {
                src: source,
                theme: CodeTheme::system(Theme::GITHUB_LIGHT, Theme::GITHUB_DARK),
            }
        }
    }
}

/// The Style tab's CSS source for one component file (`style.css`, or a
/// Block-kind variant's `demo.css`).
///
/// In release/SSG builds this carries the text highlighted (and embedded)
/// at compile time via `dioxus_code::code!()`, exactly like
/// `HighlightedCode` -- unchanged from before this type existed. In debug
/// builds (`dx serve`'s own dev loop) `code!()` is deliberately NOT used:
/// it expands to `include_str!(path)`, which makes rustc track the `.css`
/// file as a source dependency of this crate. `dx serve`'s file watcher
/// checks that dependency list (dioxus-cli's
/// `serve/runner.rs::handle_file_change`, via the compiled artifact's own
/// rustc dep-info `.d` file) and, finding the edited file listed there,
/// classifies the edit as `needs_full_rebuild` -- a full ~30-60s rebuild,
/// instead of the sub-second asset hot-reload every *other* `style.css`
/// edit already gets from its own separate `asset!()` stylesheet link
/// (`hotreload_bundled_assets`, checked first, but overridden once
/// `needs_full_rebuild` is also true for that same file). Measured
/// before/after and the full mechanism: `dev-docs/dev-loop.md`'s CSS
/// section. Debug builds instead carry only the file's own bundled
/// `asset!()` URL -- `asset!()` does not embed the file's text at compile
/// time, so referencing it costs nothing extra; `CssCodeBlock` below
/// fetches and highlights that URL's text lazily, client-side, the first
/// time the Style tab is rendered.
#[derive(Clone, PartialEq)]
struct CssHighlight {
    /// Pre-highlighted content -- release/SSG builds only.
    #[cfg(not(debug_assertions))]
    embedded: HighlightedCode,
    /// This file's own bundled asset URL -- debug builds only.
    #[cfg(debug_assertions)]
    asset: Asset,
}

/// Renders a CSS Style tab from a [`CssHighlight`]. Release/SSG builds
/// show the compile-time-highlighted text directly, identical to
/// `CodeBlock`.
#[cfg(not(debug_assertions))]
#[component]
fn CssCodeBlock(source: CssHighlight) -> Element {
    rsx! {
        CodeBlock { source: source.embedded }
    }
}

/// Debug builds' half of [`CssCodeBlock`]: fetch the CSS from its own
/// asset URL and highlight it at runtime instead of at compile time --
/// see `CssHighlight`'s doc comment for why.
#[cfg(debug_assertions)]
#[component]
fn CssCodeBlock(source: CssHighlight) -> Element {
    rsx! {
        LazyCssCodeBlock { asset: source.asset }
    }
}

/// Fetches `asset`'s own text over HTTP -- the same URL its
/// `document::Link`/`document::Stylesheet` sibling already loads as a live
/// stylesheet -- and highlights it client-side once it arrives, so a
/// `style.css` edit is visible here exactly like the live stylesheet
/// already is: via `dx serve`'s asset hot-reload, not a crate rebuild.
#[cfg(debug_assertions)]
#[component]
fn LazyCssCodeBlock(asset: Asset) -> Element {
    let url = asset.to_string();
    let css = use_resource(move || {
        let url = url.clone();
        async move { fetch_asset_text(url).await }
    });

    // Fully qualified rather than imported: this file already has its own,
    // unrelated local `enum Language` (the i18n language switcher above),
    // and importing `dioxus_code::Language` under that same bare name
    // shadows it -- confirmed live: it breaks the i18n enum's own inherent
    // `impl` block ("cannot define inherent `impl` for a type outside of
    // the crate") the moment both are in scope together.
    let placeholder = |text: &'static str| HighlightedCode {
        source: HighlightedSource::from_static_parts(text, dioxus_code::Language::Css, &[]),
    };

    // Cloned out of the resource's read guard into an owned value up front
    // (rather than matching on `&*css.read()` directly) so the guard drops
    // immediately, before any of the `rsx!` arms below run -- avoids tying
    // the returned `Element` to that guard's borrow.
    let state: Option<Option<String>> = css.read().clone();

    match state {
        Some(Some(text)) => rsx! {
            CodeBlock {
                source: HighlightedCode {
                    source: dioxus_code::SourceCode::new(dioxus_code::Language::Css, text).into(),
                },
            }
        },
        Some(None) => rsx! { CodeBlock { source: placeholder("/* failed to load style.css */") } },
        None => rsx! { CodeBlock { source: placeholder("/* loading style.css... */") } },
    }
}

/// One-shot fetch of a same-origin asset's raw text, used only by
/// [`LazyCssCodeBlock`] (debug builds). Mirrors this codebase's other
/// one-shot `document::eval` helpers (e.g. `primitives/src/input_otp.rs`'s
/// `snap_caret_to_slot`) -- a scoped async JS snippet, not a persistent
/// listener.
#[cfg(debug_assertions)]
async fn fetch_asset_text(url: String) -> Option<String> {
    let mut eval = document::eval(
        r#"
        const url = await dioxus.recv();
        try {
            const res = await fetch(url);
            dioxus.send(res.ok ? await res.text() : null);
        } catch (e) {
            dioxus.send(null);
        }
        "#,
    );
    let _ = eval.send(url);
    eval.recv::<Option<String>>().await.ok().flatten()
}

/// The docs' copy-to-clipboard button for a code block: the library's own ghost icon `Button`.
///
/// The copy runs from an inline `onclick` attribute (not a Rust handler) so it works before the
/// wasm hydrates, as the `<button>` this replaces did. The same script flips `data-copied`, and
/// `main.css` swaps the two icons on it (a Rust-side signal would need its own `onclick`, which
/// `merge_attributes` would let the inline string replace). Placement over the code block comes
/// from the caller's `attributes` (`position`/`top`/`right`).
#[component]
fn CopyButton(#[props(extends=GlobalAttributes)] attributes: Vec<Attribute>) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Ghost,
            size: ButtonSize::IconSm,
            attributes,
            class: "dx-copy-button",
            r#type: "button",
            aria_label: "Copy code",
            "onclick": "const b = this; const visiblePre = Array.from(b.parentNode.querySelectorAll('pre:not(.dx-code-gutter)')).find((pre) => pre.offsetParent !== null); navigator.clipboard.writeText(visiblePre ? visiblePre.innerText : Array.from(b.parentNode.childNodes).filter((node) => node !== b).map((node) => node.textContent).join('').trim()); b.dataset.copied = 'true';",
            Copy { class: "dx-copy-icon" }
            Check { class: "dx-check-icon" }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Language {
    English,
    French,
    Spanish,
    German,
}

impl Language {
    /// Every entry, in the order the dropdown lists them.
    const ALL: [Language; 4] = [
        Language::English,
        Language::French,
        Language::Spanish,
        Language::German,
    ];

    /// The variant name, also the `<option>` value (the old strum `Display`).
    const fn as_str(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::French => "French",
            Language::Spanish => "Spanish",
            Language::German => "German",
        }
    }

    /// The inverse of `as_str` (the old strum `EnumString`).
    fn from_name(name: &str) -> Option<Self> {
        Language::ALL
            .into_iter()
            .find(|language| language.as_str() == name)
    }

    /// The entry for the active locale, so a `LanguageSelect` that remounts on a route change (the
    /// whole docs layout does) shows the language the page is really in, not always English.
    fn from_id(id: &LanguageIdentifier) -> Self {
        Language::ALL
            .into_iter()
            .find(|language| language.id() == *id)
            .unwrap_or(Language::English)
    }

    const fn id(&self) -> LanguageIdentifier {
        match self {
            Language::English => langid!("en-US"),
            Language::French => langid!("fr-FR"),
            Language::Spanish => langid!("es-ES"),
            Language::German => langid!("de-DE"),
        }
    }

    const fn flag(&self) -> &'static str {
        match self {
            Language::English => "🇬🇧",
            Language::French => "🇫🇷",
            Language::Spanish => "🇪🇸",
            Language::German => "🇩🇪",
        }
    }

    fn display_name(&self) -> String {
        format!("{} {}", self.flag(), self.localize_name())
    }

    const fn localize_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::French => "Français",
            Language::Spanish => "Español",
            Language::German => "Deutsch",
        }
    }
}

#[component]
fn LanguageSelect() -> Element {
    let mut current_lang = use_signal(|| Language::from_id(&i18n().language()));

    rsx! {
        document::Stylesheet { href: asset!("/assets/language-select.css") }
        div { class: "dx-language-container",
            span { class: "dx-language-select-container",
                select {
                    class: "dx-language-select",
                    aria_label: "Language",
                    onchange: move |e| {
                        let name = e.value().parse().unwrap_or(current_lang.read().as_str().to_string());
                        if let Some(lang) = Language::from_name(&name) {
                            current_lang.set(lang);
                        }
                        let id = current_lang.read().id();
                        tracing::info!("Current lang: {id}");
                        // backlog row 84 finding 5: this call was commented
                        // out, so the dropdown changed its own displayed
                        // selection but never touched the app's actual
                        // locale -- every `tid!`/`t!` call site (e.g. the
                        // date_picker/calendar internationalized demos'
                        // `on_format_month`/`on_format_*_placeholder`
                        // callbacks) stayed on `en-US` regardless. `i18n()`
                        // (`dioxus_i18n::prelude`) reads the same `I18n`
                        // context `use_init_i18n` provided in `App` above;
                        // `set_language` writes its `active_bundle` signal,
                        // which every `tid!` call reads reactively
                        // (`I18n::try_translate_with_args`'s own
                        // `self.active_bundle.read()`), so this now
                        // propagates live to every already-mounted
                        // translated string, not just future ones.
                        i18n().set_language(id);
                    },
                    for lang in Language::ALL {
                        option {
                            value: lang.as_str(),
                            selected: lang == *current_lang.read(),
                            {lang.display_name()}
                        }
                    }
                }
                span { class: "dx-language-select-value",
                    {current_lang.read().flag()}
                    ChevronDown {
                        class: "dx-select-expand-icon",
                        size: "24px",
                        stroke: "var(--dx-foreground)",
                    }
                }
            }
        }
    }
}

#[component]
fn ComponentCode(
    rs_highlighted: HighlightedCode,
    css_highlighted: CssHighlight,
    #[props(default = ComponentType::Normal)] component_type: ComponentType,
) -> Element {
    rsx! {
        Tabs {
            default_value: "main.rs",
            border_bottom_left_radius: "0.5rem",
            border_bottom_right_radius: "0.5rem",
            horizontal: true,
            width: "100%",
            TabList {
                TabTrigger { value: "main.rs", index: 0usize, "main.rs" }
                TabTrigger { value: "style.css", index: 1usize, "style.css" }
                if component_type != ComponentType::Block {
                    TabTrigger { value: "dx-components-theme.css", index: 2usize, "dx-components-theme.css" }
                }
            }
            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                justify_content: "center",
                align_items: "center",
                TabContent {
                    index: 0usize,
                    padding: 0,
                    value: "main.rs",
                    width: "100%",
                    position: "relative",
                    CodeBlock { source: rs_highlighted }
                }
                TabContent {
                    index: 1usize,
                    padding: 0,
                    value: "style.css",
                    width: "100%",
                    position: "relative",
                    CssCodeBlock { source: css_highlighted }
                }
                if component_type != ComponentType::Block {
                    TabContent {
                        index: 2usize,
                        padding: 0,
                        value: "dx-components-theme.css",
                        width: "100%",
                        position: "relative",
                        CssCodeBlock { source: THEME_CSS }
                    }
                }
            }
        }
    }
}

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
const DOCS_SECTIONS: &[&str] = &[
    "How it works",
    "New project",
    "Add a component",
    "Recommended workflow",
    "Motion",
    "Utilities",
];

/// Turns a heading's visible text into a same-page anchor id: lowercased,
/// with every run of non-alphanumeric characters (spaces included)
/// collapsed to a single hyphen and none left dangling at either end.
/// "How it works" -> "how-it-works". Generic over any page's headings --
/// `Home()` reuses this as-is for its own sections.
fn docs_section_slug(title: &str) -> String {
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
fn Docs(dark_mode: Option<bool>) -> Element {
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
                }
                section { class: "dx-docs-section",
                    h2 { id: "{docs_section_slug(DOCS_SECTIONS[4])}", "{DOCS_SECTIONS[4]}" }
                    crate::motion_docs::MotionDocs {}
                }
                section { class: "dx-docs-section",
                    h2 { id: "{docs_section_slug(DOCS_SECTIONS[5])}", "{DOCS_SECTIONS[5]}" }
                    crate::utilities_docs::UtilitiesDocs {}
                }
            }
        }
    }
}

/// Which sidebar nav entry (if any) corresponds to the page currently being
/// rendered inside `DocsLayout`, for active-link highlighting.
#[derive(Clone, Copy, PartialEq)]
enum DocsNavActive {
    /// The `/docs` written-guide page ("Start" > "Overview").
    Overview,
    /// A `/component/?name=...` page, keyed by the component's raw name.
    Component(&'static str),
    /// The homepage (`/`) -- part of this same nav now, but distinct from
    /// `Overview`: before the homepage had a sidebar at all, `None` here
    /// meant "on Docs", so a bare `Option<&'static str>` would have wrongly
    /// lit up "Overview" while browsing the gallery too.
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
fn page_sections_submenu(sections: &'static [&'static str]) -> Element {
    rsx! {
        SidebarMenuSub {
            for title in sections.iter().copied() {
                SidebarMenuSubItem { key: "{title}",
                    SidebarMenuSubButton {
                        as: move |attributes: Vec<Attribute>| rsx! {
                            a {
                                href: "#{docs_section_slug(title)}",
                                ..attributes,
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
/// renders as the inset's children -- callers no longer wrap it in a `<main>`
/// themselves, since `SidebarInset` already renders the page's one `<main>`
/// landmark (see `EmailClient()`'s own comment on that same point).
#[component]
fn DocsLayout(
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
    // component pages) leaves this `None` and keeps the previous, open-by-
    // default behavior -- those pages need the nav immediately usable.
    // `SidebarProvider`'s own `default_open` (`components/sidebar/
    // component.rs`) already exists for exactly this and just needed
    // threading through here.
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
    // entries, same as before this page had sections of its own.
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
                                        as: move |attributes: Vec<Attribute>| rsx! {
                                            NavLink { to: Route::home(), attributes,
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
                                    // the plain, sub-item-less link it always was.
                                    if active == DocsNavActive::Home {
                                        if let Some(sections) = page_sections {
                                            {page_sections_submenu(sections)}
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    SidebarMenuButton {
                                        is_active: active == DocsNavActive::Overview,
                                        as: move |attributes: Vec<Attribute>| rsx! {
                                            NavLink { to: Route::docs(), attributes,
                                                FileText { size: "1rem", "aria-hidden": "true" }
                                                span { "Overview" }
                                            }
                                        },
                                    }
                                    // Mirrors the "Home" item above, but scoped to `Docs()`
                                    // being the page being viewed instead -- same shared
                                    // `page_sections_submenu`, so `/docs`'s 3 sub-items
                                    // (`DOCS_SECTIONS`) render exactly as they did before
                                    // "Home" gained this same treatment.
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
                                        as: move |attributes: Vec<Attribute>| rsx! {
                                            NavLink { to: Route::demos(), attributes,
                                                Monitor { size: "1rem", "aria-hidden": "true" }
                                                span { "Demos" }
                                            }
                                        },
                                    }
                                }
                                SidebarMenuItem {
                                    SidebarMenuButton {
                                        is_active: active == DocsNavActive::Charts,
                                        as: move |attributes: Vec<Attribute>| rsx! {
                                            // Current on every chart type (`/charts/bar/` ...), not only on `/charts/`.
                                            NavLink {
                                                to: Route::charts_index(),
                                                active: active == DocsNavActive::Charts,
                                                attributes,
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
                    // The theme picker's phone entry: a "Theme" button pinned under the
                    // scrolling nav that opens the picker as a bottom `Drawer`. Always
                    // rendered; `main.css` shows it only at the header's phone breakpoint
                    // (the header's own popover trigger is hidden there) -- see
                    // `theme::ThemePicker`'s doc for why that is CSS and not a Rust check.
                    theme::ThemePickerSidebarFooter {}
                }
                SidebarInset { id: MAIN_ID, tabindex: "-1", {children} }
            }
        }
    }
}

struct DemoEntry {
    tag: &'static str,
    title: &'static str,
    description: &'static str,
    route: fn() -> Route,
    thumb: fn() -> Element,
}

fn email_client_thumb() -> Element {
    rsx! {
        Mail { size: "56", stroke_width: "1.4" }
    }
}

const DEMO_ENTRIES: &[DemoEntry] = &[DemoEntry {
    tag: "Dashboard",
    title: "Email client",
    description:
        "Multi-pane mail app composed from the sidebar, item list, reading pane, and compose modal.",
    route: || Route::EmailClientDashboard {
        dark_mode: Route::in_dark_mode(),
    },
    thumb: email_client_thumb,
}];

#[component]
fn Demos(dark_mode: Option<bool>) -> Element {
    rsx! {
        // The same docs shell as every other page (backlog row 303): its `SidebarInset`
        // renders the page's one `<main>`, so the content is a plain `div` here.
        DocsLayout { active: DocsNavActive::Demos,
            div { class: "dx-home-page",
                section { class: "dx-home-section",
                    header { class: "dx-section-header",
                        span { class: "dx-section-eyebrow", "Demos" }
                        h1 { class: "dx-section-title", "Demo apps" }
                        p { class: "dx-section-summary",
                            "End-to-end app demos assembled from these primitives. Open one to explore the layout and try it live."
                        }
                    }
                    ul { class: "dx-demos-grid",
                        for entry in DEMO_ENTRIES {
                            li { class: "dx-demos-item",
                                // link-ok: a demo-app card on `/demos`, never `/demos` itself
                                Link {
                                    to: (entry.route)(),
                                    class: "dx-demos-card",
                                    div { class: "dx-demos-card-thumb", {(entry.thumb)()} }
                                    div { class: "dx-demos-card-meta",
                                        span { class: "dx-demos-card-tag", "{entry.tag}" }
                                        h2 { class: "dx-demos-card-title", "{entry.title}" }
                                        p { class: "dx-demos-card-description", "{entry.description}" }
                                        span { class: "dx-demos-card-cta",
                                            "Open demo"
                                            ArrowRight { size: "16", stroke_width: "1.6" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Legacy query-string deep link (`/component/?name=<x>&`, dev-docs/
/// backlog.md row 46). Renders the SAME markup regardless of `name` -- a
/// generic, name-agnostic loading shell -- on every platform, so there is
/// nothing for a server prerender and a client hydration to ever disagree
/// on structurally (unlike the old behavior this replaces, which tried to
/// resolve `name` against `components::DEMOS` here and rendered the
/// "Component not found" shell server-side for every X, since `dx build
/// --ssg` only ever prerenders this route with an EMPTY query -- see
/// `Route::ComponentDemo`'s own doc comment). Once mounted client-side (in
/// a real browser, where `name` IS available from the real URL), redirects
/// to the canonical `ComponentDemoPath` route so old links/bookmarks/specs
/// keep landing on the right component.
#[component]
fn ComponentDemo(iframe: Option<bool>, dark_mode: Option<bool>, name: String) -> Element {
    let nav = navigator();
    use_effect(move || {
        nav.replace(Route::ComponentDemoPath {
            name: name.clone(),
            iframe,
            dark_mode,
        });
    });

    rsx! {
        Navbar {}
        main {
            id: MAIN_ID,
            tabindex: "-1",
            class: "dx-component-demo-redirect",
            role: "main",
            p { "Loading component…" }
        }
    }
}

/// Canonical, SSG-enumerable component page (`/component/<name>/`, row 46).
/// `name` is a PATH segment here, so the server prerender for a given `name`
/// and the client's initial render for that same URL always resolve to the
/// same branch below -- unlike the legacy `ComponentDemo` query route, a
/// genuinely nonexistent `name` shows "Component not found" identically on
/// both sides rather than on every name unconditionally.
#[component]
fn ComponentDemoPath(iframe: Option<bool>, dark_mode: Option<bool>, name: String) -> Element {
    let route = router().current::<Route>();
    tracing::info!("route: {route}");
    let Some(demo) = components::DEMOS
        .iter()
        .find(|demo| demo.name == name)
        .cloned()
    else {
        // Bypasses `ComponentHighlight`/`DocsLayout` entirely, so -- like
        // `Demos` -- there is no `SidebarProvider` ancestor here either;
        // `Navbar {}` renders its trigger-less fallback (see its own doc
        // comment) rather than being left off this page altogether.
        return rsx! {
            Navbar {}
            main { id: MAIN_ID, tabindex: "-1", class: "dx-component-demo-not-found",
                h3 { "Component not found" }
                p { "The requested component does not exist." }
            }
        };
    };
    rsx! {
        ComponentHighlight { demo }
    }
}

/// The catch-all (`Route::NotFound`): a URL no route matches. Same markup and
/// class as `ComponentDemoPath`'s "Component not found" shell (no sidebar, so
/// `Navbar {}` renders its trigger-less form), plus a way home. Client-only in
/// practice: it is never prerendered (see the route's comment).
#[component]
fn NotFound(segments: Vec<String>) -> Element {
    let path = segments.join("/");
    rsx! {
        document::Title { "Page not found \u{2013} {SITE_NAME}" }
        Navbar {}
        main { id: MAIN_ID, tabindex: "-1", class: "dx-component-demo-not-found",
            h3 { "Page not found" }
            p { "There is no page at \"/{path}\"." }
            // link-ok: content link; the catch-all page is never `/`
            Link { to: Route::home(), "Back to the home page" }
        }
    }
}

/// The `h2` headings the component page renders itself (`ComponentHighlight`) or that `build.rs` adds
/// around a component's `docs.md` ("Usage notes" wraps the text before its first `##`). A `docs.md` must not add an `h2` with one of these
/// texts (the page would show it twice); `docs_md_has_no_template_h2` below
/// enforces that, so the list is the single source for both.
const PAGE_HEADING_INSTALLATION: &str = "Installation";
#[cfg(test)]
const PAGE_HEADING_USAGE: &str = "Usage notes";
const PAGE_HEADING_VARIANTS: &str = "Examples";
/// "Usage" is reserved too: the text before a docs.md's first `##` is rendered under "Usage notes", so
/// a docs h2 called "Usage" would read as a nested duplicate of it.
#[cfg(test)]
const TEMPLATE_H2_HEADINGS: [&str; 4] = [
    PAGE_HEADING_INSTALLATION,
    PAGE_HEADING_USAGE,
    "Usage",
    PAGE_HEADING_VARIANTS,
];

// `variant_notes(component, variant)`: generated by `build.rs` from `variants/<variant>/notes.md`.
include!(concat!(env!("OUT_DIR"), "/variant_notes.rs"));

/// Prefixes the app base path (`dx --base-path`, e.g. `/shadcn-dioxus`)
/// onto root-absolute `href="/..."` links in the build-time rendered
/// `docs.md` HTML. `build.rs` bakes that HTML before the base path is
/// known, and a root-absolute href otherwise escapes the base path when
/// opened in a new tab or without JS (it hit `host/component/...`, a 404,
/// on the Pages deploy). Uses the router's own prefix -- the same value
/// every `Link` prepends -- so server and client render identically.
fn prefix_internal_hrefs(docs: &'static str) -> String {
    prefix_root_absolute_hrefs(docs, &router().prefix().unwrap_or_default())
}

fn prefix_root_absolute_hrefs(html: &str, prefix: &str) -> String {
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
fn use_open_section_for_hash() {
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
fn ComponentHighlight(demo: ComponentDemoData) -> Element {
    use components::accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};
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
                // Installation and the docs.md sections are ONE `Accordion` of the library's own (owner
                // 2026-10-19: the docs are built from its components, so they look and move like them):
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
                        AccordionItem { index: 0usize,
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
                                    CollapsibleTrigger { class: "dx-docs-collapsible-trigger",
                                        "Manual installation files"
                                    }
                                    CollapsibleContent {
                                        ManualComponentInstallation { component, style }
                                    }
                                }
                            }
                        }
                        for (i , section) in docs.iter().enumerate() {
                            AccordionItem { key: "{section.id}", index: i + 1,
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
static PREVIEW_DIRECTION: GlobalSignal<Direction> = Signal::global(|| Direction::Ltr);

/// The LTR/RTL toggle in a component page's header. A pressed button, because it is a two-state
/// switch of the whole page's demos, not a navigation.
#[component]
fn DirectionToggle() -> Element {
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
fn ComponentInstallCommand(name: &'static str) -> Element {
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
enum ExtraBadgeTip {
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
/// extends the link's name to "<name> Extra".
#[component]
fn ExtraBadge(name: &'static str, #[props(default)] tip: ExtraBadgeTip) -> Element {
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
/// page's heading: "Extra: not in shadcn/ui. <reason>". Unlike a tooltip it
/// works on touch and is read in order by screen readers. Renders nothing for
/// a shadcn component or an unclassified one.
#[component]
fn ExtraExplainer(name: &'static str) -> Element {
    let Some(reason) = components::extra_reason(name) else {
        return rsx! {};
    };
    rsx! {
        p { class: "dx-extra-explainer", "Extra: not in shadcn/ui. {reason}" }
    }
}

#[component]
fn ManualComponentInstallation(component: HighlightedCode, style: CssHighlight) -> Element {
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

/// Display title for a variant's `h3` on a component page, derived from the
/// variant's identifier: underscores become spaces and a known acronym is
/// upper-cased, so `rtl` reads "RTL" rather than "Rtl". An acronym that
/// *qualifies* another variant name goes in parentheses, so
/// `virtual_loop_rtl` reads "virtual loop (RTL)". (`.dx-component-variant-title`
/// then title-cases the words via CSS `text-transform: capitalize`, which
/// leaves an already upper-case acronym alone.)
fn variant_title(name: &str) -> String {
    const ACRONYMS: &[&str] = &["api", "ltr", "otp", "rtl", "ssr", "ui", "url"];
    let words: Vec<&str> = name.split('_').filter(|w| !w.is_empty()).collect();
    let is_acronym = |w: &str| ACRONYMS.contains(&w);
    let render = |w: &str| {
        if is_acronym(w) {
            w.to_uppercase()
        } else {
            w.to_string()
        }
    };
    match words.as_slice() {
        [rest @ .., last] if !rest.is_empty() && is_acronym(last) => format!(
            "{} ({})",
            rest.iter().map(|w| render(w)).collect::<Vec<_>>().join(" "),
            render(last)
        ),
        _ => words
            .iter()
            .map(|w| render(w))
            .collect::<Vec<_>>()
            .join(" "),
    }
}

#[component]
fn ComponentVariantHighlight(
    variant: ComponentVariantDemoData,
    main_variant: bool,
    /// The component the variant belongs to (its folder name): the code tab
    /// shows the source as it reads once that component is installed.
    owner: &'static str,
) -> Element {
    let ComponentVariantDemoData {
        name,
        rs_highlighted,
        css_highlighted: _,
        component: Comp,
    } = variant;
    let highlighted = installed_source::installed(owner, name, &rs_highlighted);
    // Every variant of a "Normal" component renders on the SAME page (the
    // "Examples" section below loops over all of them, main included --
    // see `ComponentHighlight` above), so a literal `"component-preview-
    // frame"` id can't be shared across more than one of these `TabContent`s
    // without producing duplicate ids -- confirmed by execution: adding
    // `popover`'s `non_modal` variant (dev-docs/backlog.md rows 19/7) gave
    // the popover page two, and `popover.spec.ts`'s basic test (`page.
    // locator("#component-preview-frame")`, no `.first()`) failed with a
    // strict-mode "2 elements" violation. Fixed by construction, not
    // per-instance: `main_variant` (already the exact signal distinguishing
    // the one call site above from every entry in the loop below) picks
    // exactly `component-preview-frame` for `main` -- unchanged, so every
    // existing single-variant component page and every spec that already
    // targets that literal id keeps working untouched -- and
    // `component-preview-frame-<name>` for every other variant, unique per
    // variant name so two additional variants on the same page (e.g.
    // `calendar[simple, internationalized, range, multi_month,
    // unavailable_dates]`, already exposed to this same defect before this
    // fix, silently tolerated via `.first()` in `calendar.spec.ts`) can
    // never collide with each other either.
    let frame_id = if main_variant {
        "component-preview-frame".to_string()
    } else {
        format!("component-preview-frame-{name}")
    };
    rsx! {
        if !main_variant {
            h3 { class: "dx-component-variant-title", "{variant_title(name)}" }
        }
        Tabs {
            default_value: "Demo",
            border_bottom_left_radius: "0.5rem",
            border_bottom_right_radius: "0.5rem",
            horizontal: true,
            width: "100%",
            variant: TabsVariant::Ghost,
            div { class: "dx-component-tabs-header",
                TabList {
                    TabTrigger { value: "Demo", index: 0usize, "DEMO" }
                    TabTrigger { value: "Code", index: 1usize, "CODE" }
                }
            }
            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                justify_content: "center",
                align_items: "center",
                TabContent {
                    index: 0usize,
                    class: "dx-component-preview-frame",
                    id: "{frame_id}",
                    value: "Demo",
                    width: "100%",
                    position: "relative",
                    // The page's LTR/RTL toggle (`PREVIEW_DIRECTION`): `dir` on the frame mirrors
                    // layout and logical-property CSS, the provider the keyboard/positioning logic.
                    dir: if PREVIEW_DIRECTION() == Direction::Rtl { Some("rtl") } else { None },
                    DirectionProvider { direction: PREVIEW_DIRECTION(), Comp {} }
                }
                TabContent {
                    index: 1usize,
                    class: "dx-component-preview-frame",
                    value: "Code",
                    width: "100%",
                    position: "relative",
                    CodeBlock { source: highlighted }
                }
            }
        }
        VariantNotes { owner, name }
    }
}

/// The prose about one demo (`variants/<variant>/notes.md`, rendered by `build.rs`), in a closed
/// `Collapsible` under that demo, so a long explanation costs one line until the reader asks for it. It
/// replaces the text-only "Examples" lists that described the demos from a section away (row 233).
/// `hidden_until_found` keeps the closed note in the server markup, searchable and reachable by a
/// `#fragment` (`use_open_section_for_hash`).
#[component]
fn VariantNotes(owner: &'static str, name: &'static str) -> Element {
    use components::collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};
    let Some(notes) = variant_notes(owner, name) else {
        return VNode::empty();
    };
    rsx! {
        Collapsible { class: "dx-variant-notes dx-docs-prose", hidden_until_found: true,
            CollapsibleTrigger { class: "dx-docs-collapsible-trigger dx-variant-notes-trigger",
                "About this example"
            }
            CollapsibleContent {
                div {
                    class: "dx-component-description",
                    dangerous_inner_html: prefix_internal_hrefs(notes),
                }
            }
        }
    }
}

#[component]
fn BlockComponentVariantHighlight(
    component_name: &'static str,
    variant: ComponentVariantDemoData,
    main_variant: bool,
) -> Element {
    let ComponentVariantDemoData {
        name,
        rs_highlighted,
        css_highlighted,
        component: _,
    } = variant;
    let highlighted = installed_source::installed(component_name, name, &rs_highlighted);

    // The canonical, SSG-enumerable path route (row 46) -- NOT the legacy
    // `Route::ComponentBlockDemo` query form, so every block demo's iframe
    // content this app renders itself is already a page `dx build --ssg`
    // can prerender per (name, variant) pair.
    let route_path = Route::component_block(component_name, name).to_string();

    let iframe_src = match router().prefix() {
        Some(prefix) => format!("{prefix}{route_path}"),
        None => route_path,
    };

    // Same defect, same construction as `ComponentVariantHighlight`'s
    // identical `frame_id` above (see its doc comment for the full
    // rationale) -- this is the Block-kind sibling of that function, and
    // every "Block" component with more than one variant (e.g. `sidebar(
    // block)[floating, inset]`) renders all of them on this same
    // `/component/?name=<name>&` page (`ComponentHighlight` above doesn't
    // branch on kind for *that* -- only for which of these two functions
    // renders each variant), so it was exposed to the identical duplicate-
    // id defect, just not yet caught by a spec: `sidebar.spec.ts` only ever
    // drives the single-variant `/component/block/?name=sidebar&variant=
    // ...&` route (`ComponentBlockDemo`, a different function entirely,
    // one variant per page by construction), never this one.
    let frame_id = if main_variant {
        "component-preview-frame".to_string()
    } else {
        format!("component-preview-frame-{name}")
    };

    rsx! {
        if !main_variant {
            h3 { class: "dx-component-variant-title", "{variant_title(name)}" }
        }
        Tabs {
            default_value: "Preview",
            border_bottom_left_radius: "0.5rem",
            border_bottom_right_radius: "0.5rem",
            horizontal: true,
            width: "100%",
            variant: TabsVariant::Ghost,
            div { class: "dx-component-tabs-header",
                TabList {
                    TabTrigger { value: "Preview", index: 0usize, "PREVIEW" }
                    TabTrigger { value: "Code", index: 1usize, "CODE" }
                }
            }
            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                justify_content: "center",
                align_items: "center",
                TabContent {
                    index: 0usize,
                    id: "{frame_id}",
                    // Nova tabs content has no padding (row 132); the Block frame keeps its 2rem.
                    padding: "var(--dx-space-8)",
                    value: "Preview",
                    width: "100%",
                    position: "relative",
                    // `overflow-x: clip` is set globally on `html`/`body`
                    // (`assets/main.css`) to keep the page itself from ever
                    // growing a horizontal scrollbar -- so an iframe wider
                    // than the space this page happens to leave it would
                    // otherwise just get silently clipped, not pushed into
                    // a scrollbar. This local `auto` opts *this* frame back
                    // into scrolling on its own, so `.dx-block-demo-iframe`'s
                    // desktop-only `min-width` (`assets/main.css`) stays
                    // reachable instead of being cut off.
                    overflow_x: "auto",
                    iframe {
                        // axe `frame-title`: every block demo is built here, so one title names them all.
                        title: "{component_name} {variant_title(name)} demo",
                        src: "{iframe_src}",
                        width: "100%",
                        // Block demos embed here at whatever width this
                        // page's own layout leaves them -- since the site
                        // nav itself became a real `Sidebar` (`DocsLayout`,
                        // this file), that can now be under 768px on a
                        // normal desktop viewport (e.g. ~684px measured on
                        // the live `sidebar` demo). Any block component
                        // that (like `Sidebar` itself, `components/sidebar/
                        // component.rs`'s `MOBILE_BREAKPOINT`) switches to
                        // a different, closed-by-default layout below that
                        // breakpoint would silently render nothing in the
                        // preview: its own `window.innerWidth` is this
                        // iframe's content width, not the outer page's, so
                        // it has no way to know it's being squeezed rather
                        // than genuinely viewed on a narrow screen.
                        //
                        // A PREVIOUS fix forced a flat, always-on
                        // `min-width: 820px` here to keep the preview
                        // desktop-sized -- but that ALSO fired for a
                        // genuinely narrow real device/window, forcing an
                        // 820px-wide iframe into e.g. a 390px mobile
                        // viewport (confirmed live + locally: the iframe's
                        // own `getBoundingClientRect()` reported `width:
                        // 822` inside a 390px viewport, clipped/garbled
                        // rather than scrolled to, since `overflow-x: clip`
                        // on `html`/`body` hides the escape without giving
                        // a scrollbar). A real mobile visitor SHOULD see
                        // this block's own mobile rendering (e.g.
                        // `Sidebar`'s Sheet-trigger button) -- forcing
                        // desktop width there is wrong, not just ugly.
                        //
                        // The fix is `.dx-block-demo-iframe` in
                        // `assets/main.css`: the `min-width: 820px` rule
                        // lives behind `@media (min-width: 768px)` in THAT
                        // file, not this iframe's own document. A media
                        // query written inside the iframe's own stylesheet
                        // would evaluate against the iframe's own (possibly
                        // squeezed) viewport -- the exact ambiguous signal
                        // at the heart of this bug -- but `main.css` is
                        // loaded by the OUTER top-level page, so its media
                        // query evaluates against the outer page's real
                        // `window.innerWidth`, i.e. the actual visitor
                        // viewport/device width. That correctly tells apart
                        // "the outer desktop page's own sidebar nav is
                        // squeezing an otherwise-plenty-wide viewport"
                        // (outer viewport >= 768: force the comfortable
                        // desktop min-width; `overflow-x: auto` above makes
                        // the extra width reachable by scrolling within
                        // this card instead of overflowing the page) from
                        // "this actually is a narrow device/window" (outer
                        // viewport < 768: no forced min-width, so the
                        // iframe sizes to its real, narrow container and
                        // the embedded block component's own breakpoint
                        // check sees a genuinely narrow width and renders
                        // its real mobile layout, matching what a mobile
                        // visitor should see).
                        class: "dx-block-demo-iframe",
                        height: "600px",
                        border: "1px solid var(--dx-border)",
                        border_radius: "0.5em",
                    }
                }
                TabContent {
                    index: 1usize,
                    padding: "var(--dx-space-8)",
                    value: "Code",
                    width: "100%",
                    position: "relative",
                    if let Some(css) = css_highlighted {
                        ComponentCode {
                            rs_highlighted: highlighted,
                            css_highlighted: css,
                            component_type: ComponentType::Block,
                        }
                    } else {
                        CodeBlock { source: highlighted }
                    }
                }
            }
        }
        VariantNotes { owner: component_name, name }
    }
}

/// Self-hosted Latin-subset variable fonts (SIL OFL, licence beside them), Geist 29 KB + Geist Mono 23 KB.
const GEIST_WOFF2: Asset = asset!("/assets/fonts/Geist-Latin-Variable.woff2");
const GEIST_MONO_WOFF2: Asset = asset!("/assets/fonts/GeistMono-Latin-Variable.woff2");
const LATIN_RANGE: &str = "U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD";

/// The head links every route shares, in one place so the three route roots
/// cannot drift apart (they used to repeat this block by hand).
///
/// The Geist fonts are self-hosted (row 162): no request leaves the origin, and the preloads and the
/// `@font-face` rules are written here from the same `asset!` URLs (they cannot be in `main.css`, which
/// does not rewrite `url()` to the hashed names). Never `@import` a font from `main.css`: a stylesheet
/// whose `@import` is still loading is not applied at all (found 2026-09-02, see the top of
/// `assets/main.css` and `playwright/oracle/tier2-html/global-stylesheet.spec.ts`).
///
/// **No font-swap layout shift (measured 2026-10-05, hard load, CLS summed):** with `display=swap` the
/// page paints in the fallback face and re-wraps when Geist arrives (0.117 on /component/form/). Two
/// things, together: the two LATIN woff2 files are `preload`ed from the head so they are ready by first
/// paint, and the faces say `font-display: optional`, so a face that is not ready in time keeps the
/// fallback for that view instead of swapping late. `crossorigin` is required on a font preload or the
/// browser fetches the file twice. `check-css-delivery.sh` holds this pair.
#[component]
fn GlobalHead() -> Element {
    // The `og:image` URL: the 512 px icon, absolute (see `SITE_ORIGIN`). `asset!()` already carries the
    // build's `--base-path`, so a root-relative path gets only the origin put in front of it.
    let icon = asset!("/assets/icon-512.png").to_string();
    let og_image = if icon.starts_with('/') {
        format!("{SITE_ORIGIN}{icon}")
    } else {
        icon
    };
    // `@font-face` cannot live in `main.css`: `asset!` hashes the font file names and does not rewrite a
    // stylesheet's `url()`, so the rules are written here from the same `asset!` URLs as the preloads.
    // `font-display: optional` is the CLS fix (see above); the tokens that name the families are in main.css.
    let font_face_css = format!(
        "@font-face{{font-family:Geist;font-style:normal;font-weight:100 900;font-display:optional;src:url({GEIST_WOFF2}) format(\"woff2\");unicode-range:{LATIN_RANGE}}}\
         @font-face{{font-family:\"Geist Mono\";font-style:normal;font-weight:100 900;font-display:optional;src:url({GEIST_MONO_WOFF2}) format(\"woff2\");unicode-range:{LATIN_RANGE}}}"
    );
    rsx! {
        // Icons go through `asset!()` (a static `<link>` in `index.html` is not prefixed with the base
        // path). The PNG comes first and the SVG last, so an engine that understands both takes the SVG,
        // which follows the browser's light/dark scheme itself (`@media (prefers-color-scheme)` inside
        // the file; it does not follow the site's own `data-theme` toggle).
        document::Link {
            rel: "icon",
            r#type: "image/png",
            sizes: "32x32",
            href: asset!("/assets/favicon-32.png"),
        }
        document::Link {
            rel: "icon",
            r#type: "image/svg+xml",
            sizes: "any",
            href: asset!("/assets/favicon.svg"),
        }
        document::Link {
            rel: "apple-touch-icon",
            sizes: "180x180",
            href: asset!("/assets/apple-touch-icon.png"),
        }
        document::Meta { name: "description", content: SITE_DESCRIPTION }
        document::Meta { property: "og:site_name", content: SITE_NAME }
        document::Meta { property: "og:title", content: SITE_NAME }
        document::Meta { property: "og:type", content: "website" }
        document::Meta { property: "og:description", content: SITE_DESCRIPTION }
        document::Meta { property: "og:image", content: "{og_image}" }
        document::Meta { name: "twitter:card", content: "summary" }
        document::Meta { name: "twitter:title", content: SITE_NAME }
        document::Meta { name: "twitter:description", content: SITE_DESCRIPTION }
        document::Meta { name: "twitter:image", content: "{og_image}" }
        // Self-hosted fonts (row 162): the two preloads and the `@font-face` rules name the SAME hashed
        // `asset!` URLs, so the preload is always the file the face asks for. No request leaves the origin.
        document::Link {
            rel: "preload",
            r#as: "font",
            r#type: "font/woff2",
            crossorigin: "anonymous",
            href: GEIST_WOFF2,
        }
        document::Link {
            rel: "preload",
            r#as: "font",
            r#type: "font/woff2",
            crossorigin: "anonymous",
            href: GEIST_MONO_WOFF2,
        }
        document::Style { "{font_face_css}" }
        document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/dx-components-theme.css") }
        // Optional effects (`dx-scroll-fade`, `dx-shimmer`) the chat kit (attachment, marker) uses.
        document::Link { rel: "stylesheet", href: asset!("/assets/dx-effects.css") }
        // Docs-only: shadcn's theme customizer as `[data-theme-*]` blocks. Never part of the
        // installable theme (`component.json` ships only the file above); see the file's header.
        document::Link { rel: "stylesheet", href: asset!("/assets/theme-presets.css") }
    }
}

#[component]
fn EmailClientDashboard(dark_mode: Option<bool>) -> Element {
    // `GlobalHead` renders once, in `AppLayout` (this route's own ancestor
    // layout) -- see that component's doc comment (row 46) for why it moved
    // there rather than being called per-route as it used to be here.
    rsx! {
        dashboard::views::email_client::EmailClient {}
    }
}

// Visually-hidden clip-rect, for `ComponentBlockDemo`'s heading below. The
// same construction already exists, duplicated per css_module, in this repo
// (e.g. `preview/src/components/sidebar/style.css`'s `.dx-sr-only`, with its
// own "TODO: abstract as Utility class" note) -- inlined here as a style
// string instead of a shared class because `main.rs` has no css_module of
// its own (it links `preview/assets/main.css` as a plain global stylesheet
// instead), and this fix's own file lane does not include any stylesheet.
const SR_ONLY_STYLE: &str = "position: absolute; overflow: hidden; width: 1px; height: 1px; padding: 0; border: 0; margin: -1px; clip-path: inset(50%); white-space: nowrap;";

/// Legacy query-string block-demo deep link (`/component/block/?name=<x>&
/// variant=<y>&`, dev-docs/backlog.md row 46) -- same reasoning as
/// `ComponentDemo`'s doc comment: a name-agnostic loading shell, identical
/// on server and client, that redirects to the canonical
/// `ComponentBlockDemoPath` once mounted in a real browser. Resolves the
/// same "no variant specified -> use the demo's first (\"main\") variant"
/// default the old direct-render code used to apply, so an old link that
/// never named a variant still redirects to the content it used to render
/// directly.
#[component]
fn ComponentBlockDemo(name: String, variant: Option<String>, dark_mode: Option<bool>) -> Element {
    let nav = navigator();
    use_effect(move || {
        let name = name.clone();
        let resolved_variant = variant.clone().unwrap_or_else(|| {
            components::DEMOS
                .iter()
                .find(|d| d.name == name)
                .map(|d| d.variants[0].name.to_string())
                .unwrap_or_default()
        });
        nav.replace(Route::ComponentBlockDemoPath {
            name,
            variant: resolved_variant,
            dark_mode,
        });
    });

    // `GlobalHead` renders once, in `AppLayout` (see that component's doc
    // comment, row 46) -- NOT called here. This route's own redirect to
    // `ComponentBlockDemoPath` is exactly the transition that finding
    // documents: calling `GlobalHead` independently in both of two routes a
    // client-side navigation can move between silently drops its `<link>`s
    // on the destination once the source unmounts, which a per-route call
    // here would reintroduce.
    rsx! {
        main {
            style: "min-height: 100vh; display: flex; align-items: center; justify-content: center; padding: 2rem;",
            p { "Loading component…" }
        }
    }
}

/// Canonical, SSG-enumerable block-demo page
/// (`/component/block/<name>/<variant>/`, row 46): both `name` and `variant`
/// are PATH segments -- unlike the legacy `ComponentBlockDemo` query route's
/// `Option<String>` variant with an implicit "first variant" default, this
/// route always names a concrete variant, so every (demo, variant) pair gets
/// its own static file and its own server/client-agreeing render.
#[component]
fn ComponentBlockDemoPath(name: String, variant: String, dark_mode: Option<bool>) -> Element {
    // `GlobalHead` renders once, in `AppLayout` -- not per-route here; see
    // `AppLayout`'s doc comment (row 46) for why.
    let Some(demo) = components::DEMOS.iter().find(|d| d.name == name).cloned() else {
        return rsx! {
            main {
                h1 { "Block component not found" }
            }
        };
    };

    let Some(variant) = demo.variants.iter().find(|v| v.name == variant) else {
        return rsx! {
            main {
                style: "min-height: 100vh; display: flex; align-items: center; justify-content: center; padding: 2rem;",
                h1 { "Variant content not found: {variant}" }
            }
        };
    };

    let Comp = variant.component;

    // axe `page-has-heading-one`/`region` (docs/backlog.md row 38): visited
    // directly -- rather than embedded in the main preview page's
    // `<iframe>`, where a separate document root makes the absence a
    // non-issue, see `sidebar.spec.ts`'s "preview page renders block" test
    // -- this route rendered no page-level heading or landmark of its own
    // at all. A visually-hidden `<h1>` below names the block for that
    // direct-visit case; it's wrapped in its own `<header>` (an implicit
    // "banner" landmark here, since it isn't nested inside any sectioning
    // element) rather than added as an outer `<main>` around the whole
    // route: every current block demo (`sidebar`'s `main`/`floating`/
    // `inset` variants) already renders its own `<main>` via
    // `SidebarInset`, so a second, outer `<main>` here would nest one
    // `<main>` inside another (`landmark-no-duplicate-main`/
    // `landmark-main-is-top-level` -- the same defect row 34 already had
    // to fix in the dashboard). This way the block's own landmarks stay
    // untouched and nothing is duplicated; see `Sidebar`'s own component
    // (`components/sidebar/component.rs`) for the matching fix that gives
    // the block's *sidebar* content itself a landmark, so `region` has no
    // remaining orphan content between the two.
    let block_label = demo.name.replace('_', " ");
    let heading = if variant.name == "main" {
        format!("{block_label} block preview")
    } else {
        format!(
            "{block_label} block preview ({} variant)",
            variant.name.replace('_', " ")
        )
    };

    rsx! {
        header {
            h1 { style: "{SR_ONLY_STYLE}", "{heading}" }
        }
        div { style: "min-height: 100vh;", Comp {} }
    }
}

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
struct HomeSidebarControls {
    side: Signal<SidebarSide>,
    collapsible: Signal<SidebarCollapsible>,
}

/// The homepage's own section headings, in the order `Home()` renders
/// them -- the same single-source-of-truth pattern as `DOCS_SECTIONS`
/// (see its doc comment): `docs_section_slug` derives both each `h2`'s
/// `id` and `DocsLayout`'s "Start" > "Home" sub-link `href`s from this
/// same text, so they can't drift apart. `HOME_SECTIONS[0]` ("Sample
/// interfaces") is `WidgetMasonry()`'s own heading, rendered by that
/// separate `#[component]` fn rather than here, so `Home()` passes its
/// slug down as a prop instead of duplicating the `h2` here.
const HOME_SECTIONS: &[&str] = &["Sample interfaces", "All components"];

#[component]
fn Home(iframe: Option<bool>, dark_mode: Option<bool>) -> Element {
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
/// 214; the CSS multi-column masonry this replaces re-assigned cards to columns on every parsed card).
///
/// Three columns above 1100px: `Left`, `Middle`, and `RightUpper` stacked on `RightLower`. Two columns
/// from 641 to 1100px: `Left` over `RightUpper`, and `Middle` over `RightLower` (a grid with two rows,
/// so the row-one tops align and only the shorter of `Left`/`Middle` leaves a gap, kept under ~35px by
/// the assignment below). One column below. See `.dx-widget-masonry` in `main.css`.
#[derive(Clone, Copy, PartialEq)]
enum MasonrySlot {
    Left,
    Middle,
    RightUpper,
    RightLower,
}

struct MasonryEntry {
    component: fn() -> Element,
    popout: bool,
    slot: MasonrySlot,
}

const BLOCKS: &[MasonryEntry] = &[
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
fn WidgetMasonry(heading_id: String) -> Element {
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
        }
    }
}

/// One fixed column group of the masonry: the cards whose `BLOCKS` entry names `slot`, in `BLOCKS` order.
#[component]
fn MasonryColumn(slot: MasonrySlot) -> Element {
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
/// invokes (`component.call(())` around `(entry.component)()`, which is what
/// this used to do): `Callback::call` runs its closure with the scope it was
/// CREATED in on top of the stack (dioxus-core `events.rs`, `with_scope_on_stack`),
/// i.e. `WidgetMasonry`'s, not this card's. A block that calls hooks directly
/// (`BlockPlayer`, `BlockColorPalette`, `BlockCommand`, `BlockComposer`) then
/// pushed them onto `WidgetMasonry`'s hook list, whose `hook_index` is only
/// reset when `WidgetMasonry` itself re-renders (never). Every later re-render of
/// the card therefore ran those hooks at an index past the end of the list and
/// allocated a FRESH set (new signal at its initial value, new memo, new
/// effect) while the old ones lived on, unreachable and never dropped. For
/// `BlockPlayer` that was a new, never-cleared 100 ms `setInterval` per
/// re-render (one a second, since each fresh player re-renders when its label
/// ticks), and the readout never got past 1:24 because every fresh signal
/// restarted at 84 s (dev-docs/research/scroll-jank-2026-10-04.md, Cause 1).
#[component]
fn MasonryCard(#[props(default)] popout: bool, children: Element) -> Element {
    let class = if popout {
        "dx-widget-card dx-widget-card-popout"
    } else {
        "dx-widget-card"
    };
    rsx! {
        Card { class, CardContent { {children} } }
    }
}

#[component]
fn BlockSignIn() -> Element {
    rsx! {
        div { style: "display: grid; gap: 0.3rem; margin-bottom: 1.1rem;",
            h3 { style: "margin: 0; font-size: 1.05rem; font-weight: 660; color: var(--dx-foreground);", "Welcome back" }
            p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: 0.85rem;", "Sign in to your workspace." }
        }
        div { style: "display: grid; gap: 0.75rem; margin-bottom: 1rem;",
            div { style: "display: grid; gap: 0.35rem;",
                Label { html_for: "blk-signin-email", "Email" }
                Input { id: "blk-signin-email", r#type: "email", placeholder: "you@example.com" }
            }
            div { style: "display: grid; gap: 0.35rem;",
                div { style: "display: flex; align-items: center;",
                    Label { html_for: "blk-signin-pw", "Password" }
                    span { style: "margin-left: auto; font-size: 0.78rem; color: var(--dx-muted-foreground); text-decoration: underline; text-underline-offset: 3px;",
                        "Forgot?"
                    }
                }
                Input { id: "blk-signin-pw", r#type: "password", placeholder: "••••••••" }
            }
        }
        div { style: "display: grid; gap: 0.5rem;",
            Button { style: "width: 100%;", "Sign in" }
            Button { variant: ButtonVariant::Outline, style: "width: 100%;", "Continue with Google" }
        }
    }
}

/// The bundled demo avatar (abstract geometric SVGs generated by `scripts/gen-demo-avatars.mjs`; see
/// `preview/assets/avatars/README.md`) for an invented person, by full name or initials. Unknown names
/// get no image, so `ImageAvatar` shows their initials.
fn demo_avatar(who: &str) -> String {
    match who {
        "Avery Lin" | "AL" => asset!("/assets/avatars/avery-lin.svg"),
        "Casey Park" | "CP" => asset!("/assets/avatars/casey-park.svg"),
        "Robin Hayes" | "RH" => asset!("/assets/avatars/robin-hayes.svg"),
        "Sarah Chen" | "SC" => asset!("/assets/avatars/sarah-chen.svg"),
        "Marcus Wright" | "MW" => asset!("/assets/avatars/marcus-wright.svg"),
        "Nina Hart" | "NH" => asset!("/assets/avatars/nina-hart.svg"),
        _ => return String::new(),
    }
    .to_string()
}

#[component]
fn BlockProfile() -> Element {
    rsx! {
        div { style: "display: flex; align-items: center; gap: 0.75rem;",
            ImageAvatar {
                size: AvatarImageSize::Medium,
                src: demo_avatar("Avery Lin"),
                alt: "Avery Lin",
                aria_label: "Avatar",
                "AL"
            }
            div { style: "flex: 1; display: grid; gap: 0.1rem; min-width: 0;",
                div { style: "display: flex; align-items: center; gap: 0.4rem;",
                    span { style: "font-weight: 600; color: var(--dx-foreground);", "Avery Lin" }
                    Badge {
                        variant: BadgeVariant::Secondary,
                        style: "padding: 0.15rem 0.3rem; background-color: var(--dx-ring-color); color: white;",
                        VerifiedIcon {}
                    }
                }
                span { style: "color: var(--dx-muted-foreground); font-size: 0.85rem;", "@averylin" }
            }
            Button { variant: ButtonVariant::Outline, "Follow" }
        }
        p { style: "margin: 1.1rem 0 0; color: var(--dx-muted-foreground); font-size: 0.9rem; line-height: 1.55;",
            "Building UI primitives that ship to web, desktop, and mobile. Mostly Rust, mostly weekends."
        }
        div { style: "display: flex; gap: 0.35rem; margin-top: 0.85rem; flex-wrap: wrap;",
            Badge { variant: BadgeVariant::Outline, "Rust" }
            Badge { variant: BadgeVariant::Outline, "WebAssembly" }
            Badge { variant: BadgeVariant::Outline, "UI" }
        }
    }
}

#[component]
fn BlockStats() -> Element {
    rsx! {
        div { style: "display: grid; gap: 0.45rem;",
            p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: 0.74rem; text-transform: uppercase; letter-spacing: 0.1em; font-weight: 600;",
                "Active users · 30d"
            }
            div { style: "display: flex; align-items: baseline; gap: 0.6rem;",
                span { style: "font-size: 2rem; font-weight: 720; color: var(--dx-foreground); line-height: 1.1;",
                    "24,815"
                }
                Badge {
                    variant: BadgeVariant::Secondary,
                    // axe `color-contrast`, in BOTH themes (docs/backlog.md row 39, then the
                    // 2026-10-05 sweep): the first fix was a literal green pair measured on the light page
                    // only, and it was 2.3:1 in dark (#137337 on #173321). Tokens fix both: the
                    // `--dx-success-subtle` ground is pale in light and deep in dark; the ink is the
                    // success colour itself in dark (pale mint, high contrast) and, in light, that colour
                    // with its OKLCH lightness clamped to 0.45 (the `--dx-primary-ink` idea; #10b981
                    // unclamped is 2.5:1 on the pale ground).
                    style: "background-color: var(--dx-success-subtle); color: var(--light, oklch(from var(--dx-success) min(l, 0.45) c h)) var(--dark, var(--dx-success));",
                    "+12.4%"
                }
            }
        }
        div { style: "margin-top: 1rem;",
            Progress {
                value: 68.0,
                aria_label: "Toward Q2 target",
                style: "width: 100%;",
            }
        }
        p { style: "margin: 0.65rem 0 0; color: var(--dx-muted-foreground); font-size: 0.82rem;",
            "On track for the 36k Q2 target."
        }
    }
}

#[component]
fn BlockNotifications() -> Element {
    rsx! {
        div { style: "display: grid; gap: 0.3rem; margin-bottom: 1rem;",
            h3 { style: "margin: 0; font-size: 1rem; font-weight: 660; color: var(--dx-foreground);", "Notifications" }
            p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: 0.85rem;", "Pick what we ping you about." }
        }
        div { style: "display: grid; gap: 0.95rem;",
            NotificationRow { id: "blk-notif-comments", name: "Comments", description: "Replies on your posts", default_on: true }
            NotificationRow { id: "blk-notif-mentions", name: "Mentions", description: "When someone @'s you", default_on: true }
            NotificationRow { id: "blk-notif-weekly", name: "Weekly digest", description: "A Monday morning recap", default_on: false }
            NotificationRow { id: "blk-notif-updates", name: "Product updates", description: "New features and releases", default_on: false }
        }
    }
}

#[component]
fn NotificationRow(id: String, name: String, description: String, default_on: bool) -> Element {
    let mut checked = use_signal(|| default_on);
    rsx! {
        div { style: "display: flex; align-items: center; gap: 0.75rem;",
            div { style: "flex: 1; display: grid; gap: 0.1rem; min-width: 0;",
                span { style: "font-weight: 540; font-size: 0.92rem; color: var(--dx-foreground);", "{name}" }
                span { style: "color: var(--dx-muted-foreground); font-size: 0.8rem;", "{description}" }
            }
            Switch {
                id: "{id}",
                checked: checked(),
                aria_label: "{name}",
                on_checked_change: move |v| checked.set(v),
            }
        }
    }
}

#[component]
fn BlockPlayer() -> Element {
    const TRACK_DURATION_SECONDS: f64 = 212.0;
    const TRACK_START_SECONDS: f64 = 84.0;

    let mut playing = use_signal(|| true);
    let mut progress_seconds = use_signal(|| Some(TRACK_START_SECONDS));
    let current_time = use_memo(move || format_track_time(progress_seconds().unwrap_or(0.0)));
    let duration_time = format_track_time(TRACK_DURATION_SECONDS);

    // One tick per second: the slider has `step: 1.0` and the label shows whole seconds, so a
    // faster clock only adds main-thread frames (every tick moves the thumb inside the masonry's
    // multi-column container, which re-lays it out). The timer is owned by this component and
    // dies with it (`use_interval`); the tick reads through `.peek()` and writes with `.set()`.
    // Gated on visibility: while the card is off-screen, skipped or the tab is hidden the task is
    // dropped (zero wakeups) and the clock restarts with a full second on return.
    let motion = dioxus_primitives::activity::use_motion();
    dioxus_primitives::interval::use_interval_while(
        motion.active(),
        std::time::Duration::from_secs(1),
        move || {
            if !*playing.peek() {
                return;
            }
            let current = progress_seconds.peek().unwrap_or(0.0);
            let next = if current >= TRACK_DURATION_SECONDS {
                0.0
            } else {
                (current + 1.0).min(TRACK_DURATION_SECONDS)
            };
            progress_seconds.set(Some(next));
        },
    );

    rsx! {
        div {
            style: "display: flex; gap: 0.85rem; align-items: center;",
            ..motion.attributes(),
            // A CSS gradient "cover", so the demo needs no network (and no remote image).
            div {
                role: "img",
                aria_label: "Midnight City album art",
                style: "width: 64px; height: 64px; border-radius: 0.45rem; flex-shrink: 0; background: linear-gradient(135deg, #1e1b4b 0%, #6d28d9 55%, #f472b6 100%); box-shadow: 0 6px 18px -8px rgba(0,0,0,0.35);",
            }
            div { style: "flex: 1; min-width: 0;",
                p { style: "margin: 0; font-weight: 600; color: var(--dx-foreground); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "Midnight City"
                }
                p { style: "margin: 0.15rem 0 0; color: var(--dx-muted-foreground); font-size: 0.85rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "M83 · Hurry Up, We're Dreaming"
                }
            }
        }
        div { style: "margin-top: 1.1rem;",
            Slider {
                horizontal: true,
                min: 0.0,
                max: TRACK_DURATION_SECONDS,
                step: 1.0,
                value: progress_seconds,
                on_value_change: move |value| progress_seconds.set(Some(value)),
                label: "Track progress",
            }
            div { style: "display: flex; justify-content: space-between; margin-top: 0.45rem; color: var(--dx-muted-foreground); font-size: 0.78rem;",
                span { "{current_time}" }
                span { "{duration_time}" }
            }
        }
        div { style: "display: flex; align-items: center; justify-content: center; gap: 0.5rem; margin-top: 0.6rem;",
            Button {
                variant: ButtonVariant::Ghost,
                aria_label: "Previous",
                onclick: move |_| progress_seconds.set(Some(0.0)),
                SkipBack { size: "18", fill: "currentColor", stroke_width: "1.5" }
            }
            Button {
                aria_label: "Play or pause",
                onclick: move |_| { let v = !playing(); playing.set(v); },
                if playing() {
                    Pause { size: "18", fill: "currentColor", stroke_width: "1.5" }
                } else {
                    Play { size: "18", fill: "currentColor", stroke_width: "1.5" }
                }
            }
            Button {
                variant: ButtonVariant::Ghost,
                aria_label: "Next",
                onclick: move |_| progress_seconds.set(Some(0.0)),
                SkipForward { size: "18", fill: "currentColor", stroke_width: "1.5" }
            }
        }
    }
}

fn format_track_time(seconds: f64) -> String {
    let seconds = if seconds.is_finite() { seconds } else { 0.0 };
    let seconds = seconds.max(0.0).floor() as u64;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

#[component]
fn BlockPricing() -> Element {
    rsx! {
        div { style: "display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.6rem;",
            h3 { style: "margin: 0; font-size: 1rem; font-weight: 660; color: var(--dx-foreground);", "Team" }
            Badge { variant: BadgeVariant::Secondary, "Most popular" }
        }
        div { style: "display: flex; align-items: baseline; gap: 0.3rem; margin-bottom: 0.55rem;",
            span { style: "font-size: 2.4rem; font-weight: 720; color: var(--dx-foreground); line-height: 1;", "$12" }
            span { style: "color: var(--dx-muted-foreground);", "/ seat / mo" }
        }
        p { style: "margin: 0 0 1rem; color: var(--dx-muted-foreground); font-size: 0.86rem; line-height: 1.55;",
            "Everything in Pro, plus shared workspaces and audit logs."
        }
        ul { style: "list-style: none; padding: 0; margin: 0 0 1rem; display: grid; gap: 0.55rem; color: var(--dx-foreground); font-size: 0.88rem;",
            for feature in ["Unlimited projects", "Role-based access", "SSO + SAML", "Priority support"] {
                li { style: "display: flex; align-items: center; gap: 0.55rem;",
                    svg {
                        width: "16",
                        height: "16",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "var(--highlight-color-tertiary)",
                        stroke_width: "2.5",
                        "aria-hidden": "true",
                        polyline { points: "20 6 9 17 4 12" }
                    }
                    "{feature}"
                }
            }
        }
        Button { style: "width: 100%;", "Start free trial" }
    }
}

#[component]
fn BlockFilters() -> Element {
    rsx! {
        div { style: "display: grid; gap: 0.3rem; margin-bottom: 1rem;",
            h3 { style: "margin: 0; font-size: 1rem; font-weight: 660; color: var(--dx-foreground);", "Filter results" }
            p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: 0.85rem;", "Narrow down what's shown below." }
        }
        div { style: "display: grid; gap: 1.1rem;",
            div { style: "display: grid; gap: 0.45rem;",
                span { style: "color: var(--dx-muted-foreground); font-size: 0.78rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.08em;",
                    "Status"
                }
                RadioGroup { default_value: "active".to_string(),
                    RadioItem { value: "active".to_string(), index: 0usize, "Active" }
                    RadioItem { value: "draft".to_string(), index: 1usize, "Drafts" }
                    RadioItem { value: "archived".to_string(), index: 2usize, "Archived" }
                }
            }
            div { style: "display: grid; gap: 0.45rem;",
                span { style: "color: var(--dx-muted-foreground); font-size: 0.78rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.08em;",
                    "Tags"
                }
                div { style: "display: grid; gap: 0.4rem;",
                    for tag in [("ft-design", "Design", true), ("ft-eng", "Engineering", false), ("ft-research", "Research", false)] {
                        // `var(--dx-space-3)` (12px), not the `0.55rem`
                        // (8.8px) every other unrelated gap in this
                        // dashboard mockup still uses -- this is the one
                        // row that actually pairs a `Checkbox` with its
                        // label text, so it should match the same
                        // checkbox+label spacing convention every other
                        // composed checkbox/radio pairing in this codebase
                        // (Field's horizontal layout, RadioGroup) already
                        // uses.
                        div { style: "display: flex; align-items: center; gap: var(--dx-space-3);",
                            Checkbox {
                                id: tag.0,
                                name: tag.0,
                                default_checked: if tag.2 { dioxus_primitives::checkbox::CheckboxState::Checked } else { dioxus_primitives::checkbox::CheckboxState::Unchecked },
                                aria_label: tag.1,
                            }
                            Label { html_for: tag.0, "{tag.1}" }
                        }
                    }
                }
            }
            Button { style: "width: 100%; margin-top: 0.2rem;", "Apply filters" }
        }
    }
}

#[component]
fn BlockColorPalette() -> Element {
    use dioxus_primitives::color_picker::Color;
    use palette::{encoding, Hsv, IntoColor};

    let mut color = use_signal(|| -> Hsv<encoding::Srgb, f64> {
        Color::new(124, 58, 237).into_format::<f64>().into_color()
    });

    rsx! {
        div { style: "display: grid; gap: 0.3rem; margin-bottom: 1.1rem;",
            h3 { style: "margin: 0; font-size: 1rem; font-weight: 660; color: var(--dx-foreground);", "Theme accent" }
            p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: 0.85rem;", "Tune the accent that shows up across the workspace." }
        }
        ColorPicker {
            label: "Theme accent color",
            color: color(),
            on_color_change: move |c| color.set(c),
        }
    }
}

#[component]
fn BlockTabs() -> Element {
    let members: &[(&str, &str, &str, &str)] = &[
        ("Avery Lin", "Eng lead", "online", "AL"),
        ("Casey Park", "Design", "away", "CP"),
        ("Robin Hayes", "PM", "offline", "RH"),
    ];
    let activity: &[(&str, &str, &str)] = &[
        ("Casey", "shipped v2.4.1", "12m ago"),
        ("Avery", "opened PR #482", "1h ago"),
        ("Robin", "moved 4 tasks", "3h ago"),
    ];
    rsx! {
        div { style: "display: grid; gap: 0.3rem; margin-bottom: 1.1rem;",
            h3 { style: "margin: 0; font-size: 1rem; font-weight: 660; color: var(--dx-foreground);", "Workspace" }
            p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: 0.85rem;", "Team activity at a glance." }
        }
        Tabs {
            default_value: "members".to_string(),
            horizontal: true,
            width: "100%",
            TabList {
                TabTrigger { value: "members".to_string(), index: 0usize, "Members" }
                TabTrigger { value: "activity".to_string(), index: 1usize, "Activity" }
                TabTrigger { value: "files".to_string(), index: 2usize, "Files" }
            }
            TabContent { index: 0usize, value: "members".to_string(), padding: "var(--dx-space-8)",
                div { style: "display: grid; gap: 0.85rem;",
                    for member in members.iter() {
                        div { style: "display: flex; align-items: center; gap: 0.7rem;",
                            ImageAvatar {
                                size: AvatarImageSize::Small,
                                src: demo_avatar(member.0),
                                alt: "{member.0}",
                                aria_label: "{member.0}",
                                "{member.3}"
                            }
                            div { style: "flex: 1; min-width: 0;",
                                div { style: "font-weight: 540; color: var(--dx-foreground); font-size: 0.9rem;", "{member.0}" }
                                div { style: "color: var(--dx-muted-foreground); font-size: 0.78rem;", "{member.1}" }
                            }
                            span {
                                style: match member.2 {
                                    "online" => "width: 0.55rem; height: 0.55rem; border-radius: 999px; background-color: rgb(34,197,94);",
                                    "away" => "width: 0.55rem; height: 0.55rem; border-radius: 999px; background-color: rgb(234,179,8);",
                                    _ => "width: 0.55rem; height: 0.55rem; border-radius: 999px; background-color: var(--dx-border);",
                                },
                            }
                        }
                    }
                }
            }
            TabContent { index: 1usize, value: "activity".to_string(), padding: "var(--dx-space-8)",
                div { style: "display: grid; gap: 0.85rem;",
                    for entry in activity.iter() {
                        div { style: "display: flex; align-items: baseline; gap: 0.45rem; font-size: 0.88rem;",
                            span { style: "font-weight: 600; color: var(--dx-foreground);", "{entry.0}" }
                            span { style: "color: var(--dx-muted-foreground);", "{entry.1}" }
                            span { style: "margin-left: auto; color: var(--dx-muted-foreground); font-size: 0.78rem; white-space: nowrap;", "{entry.2}" }
                        }
                    }
                }
            }
            TabContent { index: 2usize, value: "files".to_string(), padding: "var(--dx-space-8)",
                div { style: "display: grid; gap: 0.6rem; color: var(--dx-foreground); font-size: 0.88rem;",
                    div { style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "font-family: var(--dx-font-mono); color: var(--dx-muted-foreground);", "/" }
                        span { "Roadmap Q2.md" }
                        Badge { variant: BadgeVariant::Outline, style: "margin-left: auto;", "Draft" }
                    }
                    div { style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "font-family: var(--dx-font-mono); color: var(--dx-muted-foreground);", "/" }
                        span { "Brand guidelines.pdf" }
                    }
                    div { style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "font-family: var(--dx-font-mono); color: var(--dx-muted-foreground);", "/" }
                        span { "Onboarding deck.key" }
                    }
                }
            }
        }
    }
}

#[component]
fn BlockSchedule() -> Element {
    rsx! {
        div { style: "display: flex; align-items: center; gap: 0.6rem; margin-bottom: 0.85rem;",
            div { style: "flex: 1;",
                h3 { style: "margin: 0; font-size: 1rem; font-weight: 660; color: var(--dx-foreground);", "Schedule" }
                p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: 0.85rem;", "Pick a day for the standup." }
            }
            Badge { variant: BadgeVariant::Outline, "Mar 2026" }
        }
        div { style: "display: grid; justify-items: center;",
            components::calendar::variants::main::Demo {}
        }
    }
}

#[component]
fn BlockCommand() -> Element {
    let mut query = use_signal(String::new);
    let workspaces: &[(&str, &str)] = &[
        ("acme", "Acme Inc."),
        ("orbit", "Orbit Studio"),
        ("nimbus", "Nimbus Labs"),
        ("strata", "Strata Health"),
        ("vela", "Vela Robotics"),
        ("riverstone", "Riverstone Capital"),
    ];
    rsx! {
        div { style: "display: grid; gap: 0.3rem; margin-bottom: 1rem;",
            h3 { style: "margin: 0; font-size: 1rem; font-weight: 660; color: var(--dx-foreground);", "Switch workspace" }
            p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: 0.85rem;", "Jump between projects your team owns." }
        }
        Combobox::<String> {
            query: Some(query()),
            on_query_change: move |next| query.set(next),
            placeholder: "Search workspaces...",
            aria_label: "Switch workspace",
            list_aria_label: "Workspaces",
            ComboboxEmpty { "No workspaces match." }
            for (i , (value , label)) in workspaces.iter().enumerate() {
                ComboboxOption::<String> {
                    index: i,
                    value: value.to_string(),
                    text_value: label.to_string(),
                    "{label}"
                }
            }
        }
    }
}

#[component]
fn BlockInbox() -> Element {
    let messages: &[(&str, &str, &str)] = &[
        ("Sarah Chen", "Left 3 comments on the auth flow", "2m"),
        ("Marcus Wright", "Roadmap sync notes attached", "1h"),
        ("Nina Hart", "Refactored the sidebar layout", "4h"),
    ];
    rsx! {
        div { style: "display: flex; align-items: center; gap: 0.55rem; margin-bottom: 0.85rem;",
            div { style: "flex: 1;",
                h3 { style: "margin: 0; font-size: 1rem; font-weight: 660; color: var(--dx-foreground);", "Inbox" }
                p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: 0.85rem;", "3 new conversations." }
            }
            Badge { variant: BadgeVariant::Secondary, "3" }
        }
        div { style: "display: grid; gap: 0.5rem;",
            for (sender , preview , time) in messages.iter() {
                Item { variant: ItemVariant::Outline,
                    ItemMedia { variant: ItemMediaVariant::Icon,
                        ImageAvatar {
                            size: AvatarImageSize::Small,
                            src: demo_avatar(sender),
                            alt: "{sender}",
                            aria_label: "{sender}",
                            "{sender.chars().next().unwrap_or('?')}"
                        }
                    }
                    ItemContent {
                        ItemTitle { "{sender}" }
                        ItemDescription { "{preview}" }
                    }
                    ItemContent { flex: "none",
                        ItemDescription { "{time}" }
                    }
                }
            }
        }
    }
}

#[component]
fn BlockTasks() -> Element {
    let tasks: &[(&str, &str, &str, &str)] = &[
        ("LNC-128", "Ship Q2 product roadmap", "Today", "AL"),
        ("LNC-142", "Redesign onboarding flow", "Apr 24", "CP"),
        ("LNC-147", "Audit payment webhook logs", "Apr 29", "RH"),
        ("LNC-151", "Draft changelog for v2.4", "May 02", "AL"),
    ];
    let items: Vec<Element> = tasks
        .iter()
        .map(|t| {
            rsx! {
                div { key: "{t.0}", style: "display: flex; align-items: center; gap: 0.75rem; min-width: 0;",
                    div { style: "flex: 1; min-width: 0; display: grid; gap: 0.2rem;",
                        div { style: "color: var(--dx-foreground); font-size: 0.9rem; font-weight: 540; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                            "{t.1}"
                        }
                        div { style: "display: flex; align-items: center; gap: 0.45rem; color: var(--dx-muted-foreground); font-size: 0.78rem;",
                            span { style: "font-family: var(--dx-font-mono);", "{t.0}" }
                            span { style: "width: 3px; height: 3px; border-radius: 999px; background-color: var(--dx-muted-foreground);" }
                            span { "{t.2}" }
                        }
                    }
                    ImageAvatar {
                        size: AvatarImageSize::Small,
                        src: demo_avatar(t.3),
                        alt: "{t.3}",
                        aria_label: "Assignee {t.3}",
                        "{t.3}"
                    }
                }
            }
        })
        .collect();

    rsx! {
        div { style: "display: flex; align-items: center; gap: 0.55rem; margin-bottom: 1.1rem;",
            div { style: "flex: 1;",
                h3 { style: "margin: 0; font-size: 1rem; font-weight: 660; color: var(--dx-foreground);", "Launch priorities" }
                p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: 0.85rem;", "Drag to reorder — top is highest priority." }
            }
            Badge { variant: BadgeVariant::Outline, "4 active" }
        }
        DragAndDropList { items }
    }
}

#[component]
fn BlockComposer() -> Element {
    let mut draft = use_signal(|| {
        "Big thanks to the team for landing the new roadmap view — looks great!".to_string()
    });
    rsx! {
        div { style: "display: flex; align-items: center; gap: 0.65rem; margin-bottom: 1rem;",
            ImageAvatar {
                size: AvatarImageSize::Small,
                src: demo_avatar("Avery Lin"),
                alt: "Avery Lin",
                aria_label: "Avery Lin",
                "AL"
            }
            div { style: "flex: 1; display: grid; gap: 0.1rem;",
                span { style: "font-weight: 600; color: var(--dx-foreground); font-size: 0.9rem;", "Reply to roadmap thread" }
                span { style: "color: var(--dx-muted-foreground); font-size: 0.78rem;", "Posting as @averylin · #product" }
            }
        }
        Textarea {
            variant: TextareaVariant::Default,
            value: draft,
            oninput: move |e: FormEvent| draft.set(e.value()),
            placeholder: "Share an update…",
            style: "width: 100%; min-height: 5.5rem; resize: vertical;",
        }
        div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.55rem; margin-top: 0.85rem;",
            ToggleGroup { horizontal: true, allow_multiple_pressed: true, aria_label: "Text formatting",
                ToggleItem { index: 0usize, aria_label: "Bold",
                    b { "B" }
                }
                ToggleItem { index: 1usize, aria_label: "Italic",
                    i { "I" }
                }
                ToggleItem { index: 2usize, aria_label: "Underline",
                    u { "U" }
                }
            }
            div { style: "margin-left: auto; display: flex; gap: 0.45rem;",
                Button { variant: ButtonVariant::Ghost, "Save draft" }
                Button { "Post" }
            }
        }
    }
}

#[component]
fn ComponentGallery() -> Element {
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
            // ...). Embedding it inline on `/` put dozens of raw
            // `dioxus_primitives::` elements and duplicate landmark-shaped
            // markup into the one page every visitor and every gallery-wide
            // oracle (`preview.spec.ts`, hydration-parity Rule 4b) scans,
            // for a fixture no one browsing the catalog is here to see. It
            // stays fully reachable at its own page,
            // `/component/?name=top_layer&` (still listed under Overlays in
            // `DocsSidebar`, since `components::DEMOS` itself is unchanged
            // and this filter only narrows the *gallery grid's* iteration)
            // -- see docs/backlog.md for the landed row and
            // docs/conformance-harness.md's hydration-parity section for
            // Rule 4b's replacement subject, now that its old one
            // (the fixture's toast region) no longer renders on `/`.
            for component in components::DEMOS.iter().filter(|c| c.name != "top_layer").cloned() {
                ComponentGalleryPreview { component }
            }
        }
    }
}

#[component]
fn ComponentGalleryPreview(component: ComponentDemoData) -> Element {
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
    // still falls back to. `try_consume_context` (not `consume_context`)
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
fn SidebarGalleryCardControls(controls: HomeSidebarControls) -> Element {
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
fn CopyCommandButton(
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
fn GotoIcon(mut props: LinkProps) -> Element {
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
const THEME_CSS: CssHighlight = CssHighlight {
    embedded: HighlightedCode {
        source: dioxus_code::code!("/assets/dx-components-theme.css"),
    },
};

#[cfg(debug_assertions)]
const THEME_CSS: CssHighlight = CssHighlight {
    asset: asset!("/assets/dx-components-theme.css"),
};

#[cfg(test)]
mod variant_title_tests {
    use super::variant_title;

    #[test]
    fn variant_titles_are_readable() {
        assert_eq!(variant_title("rtl"), "RTL");
        assert_eq!(variant_title("api"), "API");
        assert_eq!(variant_title("virtual_loop_rtl"), "virtual loop (RTL)");
        assert_eq!(variant_title("virtual_many"), "virtual many");
        assert_eq!(variant_title("multi_month"), "multi month");
        assert_eq!(variant_title("sizes"), "sizes");
    }
}

#[cfg(test)]
mod site_name_tests {
    const INDEX_HTML: &str = include_str!("../index.html");

    /// The static shell paints before any Rust runs, so its `<title>` is a second copy of the name.
    /// It must be the site name and nothing else, or the tab flashes a different name on load.
    #[test]
    fn index_html_title_is_the_site_name() {
        assert!(
            INDEX_HTML.contains(&format!("<title>{}</title>", super::SITE_NAME)),
            "preview/index.html's <title> must be exactly `{}` (main.rs SITE_NAME)",
            super::SITE_NAME
        );
    }
}

#[cfg(test)]
mod docs_tests {
    use super::TEMPLATE_H2_HEADINGS;

    /// Every h2 of a `docs.md`: ATX `## ` headings and raw `<h2>` lines,
    /// ignoring fenced code blocks (``` and ~~~; a fence only closes on the
    /// same marker, at least as long, so a ``` line inside a ~~~ block -- or
    /// the reverse -- does not toggle it).
    fn h2_headings(markdown: &str) -> Vec<String> {
        let mut fence: Option<(char, usize)> = None;
        let mut headings = Vec::new();
        for line in markdown.lines() {
            let trimmed = line.trim_start();
            let marker = trimmed
                .chars()
                .next()
                .filter(|c| *c == '`' || *c == '~')
                .map(|c| (c, trimmed.chars().take_while(|x| *x == c).count()))
                .filter(|(_, n)| *n >= 3);
            match (fence, marker) {
                (None, Some(open)) => fence = Some(open),
                (Some((c, n)), Some((mc, mn)))
                    if mc == c && mn >= n && trimmed[mn..].trim().is_empty() =>
                {
                    fence = None
                }
                (Some(_), _) => {}
                (None, None) => {
                    if let Some(text) = line.strip_prefix("## ") {
                        headings.push(text.trim().trim_end_matches(':').trim().to_string());
                    } else if trimmed.len() >= 3
                        && trimmed.is_char_boundary(3)
                        && trimmed[..3].eq_ignore_ascii_case("<h2")
                        && trimmed[3..].starts_with(['>', ' '])
                    {
                        let inner = trimmed.split_once('>').map_or("", |(_, rest)| rest);
                        let inner = inner.split("</").next().unwrap_or(inner);
                        headings.push(inner.trim().trim_end_matches(':').trim().to_string());
                    }
                }
            }
        }
        headings
    }

    #[test]
    fn h2_headings_sees_atx_raw_and_skips_fences() {
        let md = "## Real\n```rust\n## in backticks\n```\n~~~\n## in tildes\n```\n## still in tildes\n~~~\n<h2>Raw</h2>\n<H2 id=\"x\">Usage:</h2>\n## After\n";
        assert_eq!(h2_headings(md), vec!["Real", "Raw", "Usage", "After"],);
    }

    #[test]
    fn root_absolute_hrefs_gain_the_base_path() {
        let html = r##"<a href="/component/chart/">Chart</a> <a href="https://x.dev/">x</a> <a href="#a">a</a>"##;
        let prefixed = super::prefix_root_absolute_hrefs(html, "/shadcn-dioxus");
        assert_eq!(
            prefixed,
            r##"<a href="/shadcn-dioxus/component/chart/">Chart</a> <a href="https://x.dev/">x</a> <a href="#a">a</a>"##
        );
        assert_eq!(super::prefix_root_absolute_hrefs(html, ""), html);
        assert_eq!(super::prefix_root_absolute_hrefs(html, "/"), html);
        // Idempotent: already-prefixed hrefs are left alone.
        assert_eq!(
            super::prefix_root_absolute_hrefs(&prefixed, "/shadcn-dioxus"),
            prefixed
        );
        // Protocol-relative links are not root-absolute paths.
        let proto = r#"<a href="//cdn.example.com/x">x</a> <a href="/docs">d</a>"#;
        assert_eq!(
            super::prefix_root_absolute_hrefs(proto, "/shadcn-dioxus"),
            r#"<a href="//cdn.example.com/x">x</a> <a href="/shadcn-dioxus/docs">d</a>"#
        );
        // A sibling path that merely starts with the prefix text still gets it.
        assert_eq!(
            super::prefix_root_absolute_hrefs(
                r#"<a href="/shadcn-dioxus-x/">x</a>"#,
                "/shadcn-dioxus"
            ),
            r#"<a href="/shadcn-dioxus/shadcn-dioxus-x/">x</a>"#
        );
    }

    /// `build.rs` cannot import `main.rs`, so the "Usage notes" it writes into every docs page is
    /// pinned to this file's constant here.
    #[test]
    fn build_rs_usage_notes_heading_matches_the_reserved_one() {
        let build_rs = include_str!("../build.rs");
        let declaration = format!(
            "const USAGE_NOTES_HEADING: &str = \"{}\";",
            crate::PAGE_HEADING_USAGE
        );
        assert!(
            build_rs.contains(&declaration),
            "build.rs lost `{declaration}`"
        );
    }

    #[test]
    fn docs_md_has_no_template_h2() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/components");
        let mut checked = 0;
        let mut offenders = Vec::new();
        for entry in std::fs::read_dir(&root).unwrap().flatten() {
            let path = entry.path().join("docs.md");
            let Ok(markdown) = std::fs::read_to_string(&path) else {
                continue;
            };
            checked += 1;
            for heading in h2_headings(&markdown) {
                if TEMPLATE_H2_HEADINGS
                    .iter()
                    .any(|reserved| reserved.eq_ignore_ascii_case(&heading))
                {
                    offenders.push(format!("{}: `## {heading}`", path.display()));
                }
            }
        }
        assert!(checked > 0, "found no docs.md under {}", root.display());
        assert!(
            offenders.is_empty(),
            "docs.md repeats an h2 the component page template already renders \
             (rename it, e.g. `Demos` / `Quick start`):\n{}",
            offenders.join("\n")
        );
    }

    #[test]
    fn docs_md_has_no_root_absolute_query_links() {
        // `/component/?name=x` is the legacy JS-redirect route (client-side
        // redirect to `/component/x/`); nothing rewrites it at build time, so
        // keep the sources on the canonical `/component/<name>/` form.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/components");
        let mut offenders = Vec::new();
        for entry in std::fs::read_dir(&root).unwrap().flatten() {
            let path = entry.path().join("docs.md");
            if let Ok(markdown) = std::fs::read_to_string(&path) {
                if markdown.contains("](/component/?") {
                    offenders.push(path.display().to_string());
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "use `](/component/<name>/)`, not the legacy query form, in: {offenders:?}"
        );
    }
}

#[cfg(test)]
mod line_number_tests {
    use super::line_numbers;

    #[test]
    fn numbers_one_per_line_of_the_rendered_listing() {
        assert_eq!(line_numbers("a\nb\nc"), "1\n2\n3");
        assert_eq!(line_numbers("a"), "1");
        assert_eq!(line_numbers(""), "1");
    }

    #[test]
    fn trailing_newlines_are_not_lines() {
        // `Code` trims them before rendering, so the gutter must too, or it would outrun the code.
        assert_eq!(line_numbers("a\nb\n"), "1\n2");
        assert_eq!(line_numbers("a\nb\n\n\n"), "1\n2");
    }

    #[test]
    fn blank_lines_inside_the_listing_count() {
        assert_eq!(line_numbers("a\n\nb"), "1\n2\n3");
    }
}
