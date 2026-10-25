//! `NavLink`: the one in-app link for navigation (navbar, footer, docs
//! sidebar, chart tabs), with a current-page marker that is the same on the
//! server and in the browser. A thin `Route`-typed wrapper over the library's
//! `dioxus_primitives::nav_link::NavLink` (row 126: the library's `NavbarItem`
//! renders the same construction, so the fix lives in one place).
//!
//! **Why not `dioxus_router::Link` (backlog row 126).** `Link` computes its
//! own `aria-current` as `href == router.full_route_string()`, and writes that
//! literal *beside* `..attributes` (so it cannot be overridden, and passing
//! `aria_current` yourself emits the attribute twice: the HTML parser keeps
//! the first, the component's own, the `check-attr-spread-collision.sh` class
//! inside the router crate). Every route here carries `?:dark_mode`, so on the
//! server both sides of that comparison read `/charts/?` and the prerendered
//! HTML says `aria-current="page"`; the browser's route string has no `?`, the
//! comparison is false on hydration, and the attribute vanishes (site-wide:
//! navbar, footer and sidebar links on every page). `NavLink` renders its own
//! anchor with exactly one `aria-current`, decided by [`is_current`](dioxus_primitives::nav_link::is_current), which
//! compares *paths* (the route rendered, query stripped), never the
//! `?`-suffixed string, so server and client agree by construction.
//!
//! Click handling is `Link`'s (dioxus-router 0.7.9, `components/link.rs`):
//! only a primary-button click with no modifier is taken over and sent to the
//! router (`navigator().push`); Ctrl/Cmd/Shift/Alt and middle clicks fall
//! through to the browser (new tab, ...). The `href` carries the router's
//! history prefix (`/shadcn-dioxus` on Pages). If a dioxus-router upgrade changes how `Link`
//! handles clicks, re-check this component.
//!
//! **Rule:** a router `Link {` in `preview/src` must be a `NavLink` instead,
//! unless it is an external target or a content link that can never point at
//! the page it is on; those carry a `link-ok: <reason>` comment.
//! `scripts/check-internal-hrefs.sh` enforces both that and "no `aria_current`
//! on a `Link`".

use crate::Route;
use dioxus::prelude::*;
use dioxus_primitives::nav_link::NavLink as PrimitiveNavLink;

#[derive(Props, Clone, PartialEq)]
pub struct NavLinkProps {
    /// The route to navigate to.
    pub to: Route,
    /// Overrides "`to` is the current page" for a link whose active state is
    /// wider than its own target (the Area chart tab is also current on
    /// `/charts/`). The caller must derive it from route state alone so the
    /// server and the browser agree.
    pub active: Option<bool>,
    /// The class attribute for the anchor.
    pub class: Option<String>,
    /// Runs after the router navigation (plain primary clicks only).
    pub onclick: Option<EventHandler<MouseEvent>>,
    /// Additional attributes for the anchor.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the anchor.
    pub children: Element,
}

#[component]
pub fn NavLink(props: NavLinkProps) -> Element {
    let NavLinkProps {
        to,
        active,
        class,
        onclick,
        attributes,
        children,
    } = props;
    rsx! {
        PrimitiveNavLink {
            to: NavigationTarget::Internal(to.to_string()),
            active,
            class,
            onclick,
            attributes,
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus_primitives::nav_link::is_current;

    fn target(route: &Route) -> NavigationTarget {
        NavigationTarget::Internal(route.to_string())
    }

    #[test]
    fn current_ignores_the_query() {
        let with = Route::Docs {
            dark_mode: Some(true),
        };
        let without = Route::Docs { dark_mode: None };
        assert!(is_current(&target(&with), &without.to_string()));
        assert!(!is_current(
            &target(&with),
            &Route::Demos { dark_mode: None }.to_string()
        ));
    }

    #[test]
    fn charts_tabs_are_distinct_pages() {
        let area = Route::ChartsKind {
            kind: "area".into(),
            dark_mode: None,
        };
        let bar = Route::ChartsKind {
            kind: "bar".into(),
            dark_mode: None,
        };
        assert!(!is_current(&target(&area), &bar.to_string()));
        assert!(!is_current(
            &target(&area),
            &Route::Charts { dark_mode: None }.to_string()
        ));
    }
}
