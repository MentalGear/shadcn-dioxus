use super::*;

#[component]
pub(crate) fn EmailClientDashboard(dark_mode: Option<bool>) -> Element {
    // `GlobalHead` renders once, in `AppLayout` (this route's own ancestor
    // layout) -- see that component's doc comment (row 46) for why it lives
    // there rather than per-route.
    rsx! {
        dashboard::views::email_client::EmailClient {}
    }
}

// Visually-hidden clip-rect, for `ComponentBlockDemo`'s heading below. The
// same construction exists, duplicated per component sheet, in this repo
// (e.g. `preview/src/components/sidebar/style.css`'s `.dx-sidebar-sr-only`,
// with its own TODO to abstract it as a utility class) -- inlined here as a
// style string instead of a shared class because `main.rs` has no component
// sheet of its own (it links `preview/assets/main.css` as a plain global
// stylesheet instead).
pub(crate) const SR_ONLY_STYLE: &str = "position: absolute; overflow: hidden; width: 1px; height: 1px; padding: 0; border: 0; margin: -1px; clip-path: inset(50%); white-space: nowrap;";

/// Legacy query-string block-demo deep link (`/component/block/?name=<x>&
/// variant=<y>&`, dev-docs/backlog.md row 46) -- same reasoning as
/// `ComponentDemo`'s doc comment: a name-agnostic loading shell, identical
/// on server and client, that redirects to the canonical
/// `ComponentBlockDemoPath` once mounted in a real browser. Resolves the
/// same "no variant specified -> use the demo's first (\"main\") variant"
/// default the direct-render path applies, so a link that
/// never names a variant still redirects to the content the direct
/// render shows.
#[component]
pub(crate) fn ComponentBlockDemo(
    name: String,
    variant: Option<String>,
    dark_mode: Option<bool>,
) -> Element {
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
    // `ComponentBlockDemoPath` is exactly the transition that doc
    // comment describes: calling `GlobalHead` independently in both of two routes a
    // client-side navigation can move between silently drops its `<link>`s
    // on the destination once the source unmounts, which a per-route call
    // here would cause.
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
pub(crate) fn ComponentBlockDemoPath(
    name: String,
    variant: String,
    dark_mode: Option<bool>,
) -> Element {
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

    // axe `page-has-heading-one`/`region` (dev-docs/backlog.md row 38): visited
    // directly -- rather than embedded in the main preview page's
    // `<iframe>`, where a separate document root makes the absence a
    // non-issue, see `sidebar.spec.ts`'s "preview page renders block" test
    // -- this route has no page-level heading or landmark of its own
    // at all. A visually-hidden `<h1>` below names the block for that
    // direct-visit case; it's wrapped in its own `<header>` (an implicit
    // "banner" landmark here, since it isn't nested inside any sectioning
    // element) rather than added as an outer `<main>` around the whole
    // route: every current block demo (`sidebar`'s `main`/`floating`/
    // `inset` variants) already renders its own `<main>` via
    // `SidebarInset`, so a second, outer `<main>` here would nest one
    // `<main>` inside another (`landmark-no-duplicate-main`/
    // `landmark-main-is-top-level` -- the same rule row 34 covers for
    // the dashboard). This way the block's own landmarks stay
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
