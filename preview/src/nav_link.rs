//! `NavLink`: the one in-app link for navigation (navbar, footer, docs
//! sidebar, chart tabs), with a current-page marker that is the same on the
//! server and in the browser.
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
//! anchor with exactly one `aria-current`, decided by [`is_current`], which
//! compares *paths* (the route rendered, query stripped), never the
//! `?`-suffixed string, so server and client agree by construction.
//!
//! Click handling is `Link`'s (dioxus-router 0.7.9, `components/link.rs`):
//! only a primary-button click with no modifier is taken over and sent to the
//! router (`navigator().push`); Ctrl/Cmd/Shift/Alt and middle clicks fall
//! through to the browser (new tab, ...). The `href` carries the router's
//! history prefix (`/shadcn-dioxus` on Pages). If a dioxus-router upgrade
//! changes `Link`'s click rules, mirror them here once.
//!
//! **Rule:** a router `Link {` in `preview/src` must be a `NavLink` instead,
//! unless it is an external target or a content link that can never point at
//! the page it is on; those carry a `link-ok: <reason>` comment.
//! `scripts/check-internal-hrefs.sh` enforces both that and "no `aria_current`
//! on a `Link`".

use crate::Route;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

/// Whether `to` is the page the router is on: the same *path*, ignoring the
/// query (`?:dark_mode`, `?:iframe` are display state, not a different page).
/// Both strings come from `Route`'s own `Display`, so a browser location and a
/// server request that parse to the same route compare equal.
pub(crate) fn is_current(to: &Route, current: &Route) -> bool {
    path_of(to) == path_of(current)
}

fn path_of(route: &Route) -> String {
    let rendered = route.to_string();
    match rendered.split_once('?') {
        Some((path, _)) => path.to_string(),
        None => rendered,
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct NavLinkProps {
    /// The route to navigate to.
    pub to: Route,
    /// Overrides "`to` is the current page" for a link whose active state is
    /// wider than its own target (the Area chart tab is also current on
    /// `/charts/`). The caller must derive it from route state alone so the
    /// server and the browser agree.
    pub active: Option<bool>,
    pub class: Option<String>,
    /// Runs after the router navigation (plain primary clicks only).
    pub onclick: Option<EventHandler<MouseEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn NavLink(props: NavLinkProps) -> Element {
    let NavLinkProps {
        to,
        active,
        class,
        onclick,
        mut attributes,
        children,
    } = props;
    // The single `aria-current` is ours: a caller-supplied one in the spread
    // would be a second attribute (the duplicate-attribute class).
    attributes.retain(|attribute| attribute.name != "aria-current");
    let router = router();
    let current: Route = router.current();
    let is_active = active.unwrap_or_else(|| is_current(&to, &current));
    let href = format!("{}{}", router.prefix().unwrap_or_default(), to);
    let target = to.clone();
    rsx! {
        a {
            class,
            href,
            aria_current: is_active.then_some("page"),
            onclick: move |event: MouseEvent| {
                // Modified and non-primary clicks keep the browser's own
                // behavior, like `Link`.
                if !event.modifiers().is_empty()
                    || event.trigger_button() != Some(MouseButton::Primary)
                {
                    return;
                }
                event.prevent_default();
                navigator().push(target.clone());
                if let Some(handler) = onclick {
                    handler.call(event);
                }
            },
            ..attributes,
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_ignores_the_query() {
        let with = Route::Docs {
            dark_mode: Some(true),
        };
        let without = Route::Docs { dark_mode: None };
        assert!(is_current(&with, &without));
        assert!(!is_current(&with, &Route::Demos { dark_mode: None }));
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
        assert!(!is_current(&area, &bar));
        assert!(!is_current(&area, &Route::Charts { dark_mode: None }));
    }
}
