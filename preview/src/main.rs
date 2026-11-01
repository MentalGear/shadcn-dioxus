use dioxus::prelude::*;
mod site;
use site::*;

#[cfg(test)]
mod chart_parity;
#[cfg(test)]
mod chart_tooltip_parity;
mod charts_gallery;
mod coarse_docs;
mod components;
mod contrast_docs;
mod dashboard;
#[path = "../docs_groups.rs"]
mod docs_groups;
mod docs_nav;
// Client builds only: the server keeps `ServerDocument` for the SSR `<head>` (see the module's docs).
#[cfg(not(feature = "server"))]
mod eager_head;
mod i18n;
mod installed_source;
mod motion_docs;
mod nav_link;
#[cfg(test)]
mod polar_parity;
#[cfg(test)]
mod ssr_parity;
mod theme;
mod utilities_docs;
mod wasm_libc_shim;

/// The site's display name: the header brand, the footer, the home hero, the docs intro, every
/// `document::Title` suffix (`charts_gallery`) and the Open Graph `og:site_name`. The static shell's
/// `<title>` in `index.html` is the one place Rust cannot reach; `site_name_tests` pins it to this.
/// The crates keep their own names (`dioxus-primitives`), and so does the Git repository, which is
/// also the GitHub Pages path (`--base-path shadcn-dioxus`): this is the SITE's name only.
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
    format!("cargo install --git {REGISTRY_GIT_URL} shadcn-dioxus --locked")
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
    /// by `build.rs` (`render_docs_sections`) in page order (`docs_groups::page_position`) and rendered by
    /// `ComponentHighlight` as Accordion items in two groups.
    docs: &'static [DocsSection],
    component: HighlightedCode,
    style: CssHighlight,
    variants: &'static [ComponentVariantDemoData],
}

/// One section of a component page's docs, as `build.rs` generates it (its `DocsSection`).
#[derive(Clone, Copy, PartialEq)]
struct DocsSection {
    /// The `h3` id, the anchor target (`docs-<slug>`).
    id: &'static str,
    /// The heading's inline HTML.
    title_html: &'static str,
    /// The section body's HTML (root-absolute hrefs not yet prefixed).
    html: &'static str,
}

#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "the derived PartialEq compares a component fn pointer by address, as every Dioxus prop does; `#[expect]` is not seen by the derive"
)]
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
    // (the legacy `/component/?name=X` shell would lose the header's `Popover` CSS that way).
    // First, so it is in place before any component can render a link. Client builds only: the server's
    // `ServerDocument` must keep collecting the SSR head. See `eager_head`.
    #[cfg(not(feature = "server"))]
    eager_head::use_eager_head_document();

    // Hydration-ready signal, by construction (dev-docs/backlog.md: SSG
    // interaction tests race hydration). `document::eval` only ever runs
    // client-side (a no-op string during SSR, per `dioxus-document`), and
    // `use_effect` only fires after a render has actually committed to the
    // DOM -- in a prerendered page that means after wasm has booted and
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

    /// Every component page's generated sections, checked against the one definition of the page order
    /// (`docs_groups`, backlog row 422): the Basic sections are present once each and in `BASIC_ORDER`, the
    /// page's own sections keep their authored order, and the recurring topics close the group in `TAIL`
    /// order. The page renders the Basic group from `BASIC_ORDER` and the other group in this order, so this
    /// pins both groups' order for every `DEMOS` entry.
    #[test]
    fn every_page_orders_its_docs_sections_by_group() {
        use crate::docs_groups::{
            basic_of_id, page_position, plain_text, tail_rank, Basic, BASIC_ORDER,
        };
        assert!(!crate::components::DEMOS.is_empty());
        for demo in crate::components::DEMOS {
            let name = demo.name;
            let positions: Vec<(u8, usize)> = demo
                .docs
                .iter()
                .map(|s| page_position(s.id, &plain_text(s.title_html)))
                .collect();
            assert!(
                positions.is_sorted(),
                "`{name}`: docs sections are not in page order: {:?}",
                demo.docs.iter().map(|s| s.id).collect::<Vec<_>>()
            );

            let basic: Vec<Basic> = demo.docs.iter().filter_map(|s| basic_of_id(s.id)).collect();
            let expected: Vec<Basic> = BASIC_ORDER
                .into_iter()
                .filter(|b| *b != Basic::Installation)
                .collect();
            assert_eq!(
                basic, expected,
                "`{name}`: the Basic group must hold Usage notes, Component structure and Props reference once each, in that order (Installation is the page's own)"
            );
            for section in demo
                .docs
                .iter()
                .filter_map(|s| basic_of_id(s.id).map(|b| (s, b)))
            {
                assert_eq!(
                    plain_text(section.0.title_html),
                    section.1.heading(),
                    "`{name}`: {} must carry the Basic group's heading",
                    section.0.id
                );
            }

            let specific: Vec<_> = demo
                .docs
                .iter()
                .filter(|s| basic_of_id(s.id).is_none())
                .collect();
            let ranks: Vec<Option<usize>> = specific
                .iter()
                .map(|s| tail_rank(&plain_text(s.title_html)))
                .collect();
            let first_tail = ranks
                .iter()
                .position(Option::is_some)
                .unwrap_or(ranks.len());
            assert!(
                ranks[first_tail..].iter().all(Option::is_some),
                "`{name}`: a page-specific section follows a recurring topic"
            );
            let tail: Vec<usize> = ranks[first_tail..].iter().flatten().copied().collect();
            assert!(
                tail.is_sorted(),
                "`{name}`: the recurring topics are out of order: {tail:?}"
            );

            // The page's own sections keep the order the `docs.md` wrote them in.
            let key = |title: &str| {
                title
                    .chars()
                    .filter(char::is_ascii_alphanumeric)
                    .collect::<String>()
                    .to_ascii_lowercase()
            };
            let own_generated: Vec<String> = specific
                .iter()
                .filter(|s| tail_rank(&plain_text(s.title_html)).is_none())
                .map(|s| key(&plain_text(s.title_html)))
                .collect();
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/components")
                .join(name)
                .join("docs.md");
            let own_authored: Vec<String> = h2_headings(&std::fs::read_to_string(&path).unwrap())
                .iter()
                .filter(|title| {
                    tail_rank(title).is_none()
                        && !BASIC_ORDER.iter().any(|b| key(b.heading()) == key(title))
                })
                .map(|title| key(title))
                .collect();
            assert_eq!(
                own_generated, own_authored,
                "`{name}`: the page-specific sections are not in the docs.md order"
            );
        }
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
