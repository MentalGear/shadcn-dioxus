use super::*;

/// `date picker` -> `Date Picker`, for a page title.
pub(crate) fn title_case(text: &str) -> String {
    text.split(' ')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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
    // so old bookmarks, external links and the Playwright specs that still use
    // this URL form keep working -- it is NOT SSG-enumerable by
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
    /// The document title of this route: `<page> – shadcn-dioxus`, the bare site name for the home page.
    /// `AppLayout` is the only caller, so every route's title is decided here and nowhere else.
    pub fn page_title(&self) -> String {
        let page = match self {
            Route::Home { .. } => return SITE_NAME.to_string(),
            Route::Docs { .. } => "Docs".to_string(),
            Route::Demos { .. } => "Demos".to_string(),
            Route::Charts { .. } => charts_gallery::ChartKind::Area.tab_label().to_string(),
            Route::ChartsKind { kind, .. } => match charts_gallery::ChartKind::from_slug(kind) {
                Some(kind) => kind.tab_label().to_string(),
                None => "Chart type not found".to_string(),
            },
            Route::ComponentDemo { .. } => "Components".to_string(),
            Route::ComponentDemoPath { name, .. } => {
                if components::DEMOS.iter().any(|demo| demo.name == name) {
                    title_case(&name.replace('_', " "))
                } else {
                    "Component not found".to_string()
                }
            }
            Route::NotFound { .. } => "Page not found".to_string(),
            Route::ComponentBlockDemo { .. } => "Block demo".to_string(),
            Route::ComponentBlockDemoPath { name, variant, .. } => format!(
                "{} \u{2013} {} block demo",
                title_case(&name.replace('_', " ")),
                variant.replace('_', " ")
            ),
            Route::EmailClientDashboard { .. } => "Email client dashboard".to_string(),
        };
        format!("{page} \u{2013} {SITE_NAME}")
    }

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
