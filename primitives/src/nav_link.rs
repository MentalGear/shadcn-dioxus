//! [`NavLink`]: a router link with exactly one `aria-current`, identical on the
//! server and in the browser (backlog row 126).
//!
//! **Why not `dioxus_router::Link`.** In dioxus-router 0.7.9 `Link` computes its
//! own `aria-current` as `href == router.full_route_string()` and writes that
//! literal *beside* `..attributes`, so (1) it cannot be overridden and an
//! `aria_current` passed by the caller is emitted a second time (the HTML parser
//! keeps the first), and (2) for an app whose routes carry a query segment
//! (`/docs?:dark_mode`) the server's route string reads `/docs?` and the
//! browser's reads `/docs`, so the prerendered `aria-current="page"` is lost on
//! hydration. `NavLink` renders its own anchor with a single `aria-current`
//! decided by [`is_current`], which compares *paths* (query and fragment
//! stripped) and so agrees on both sides by construction. A caller-supplied
//! `aria-current` in the spread is dropped (the active state is `active` or the
//! route comparison, never a second writer).
//!
//! Click handling is `Link`'s: only an unmodified primary click on an internal,
//! non-`new_tab` target is taken over and sent to the router; modifier and
//! middle clicks, external targets and `new_tab` fall to the browser. The
//! `href` of an internal target carries the router's history prefix.

use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use dioxus_attributes::attributes;

/// The path of a route string: everything before the first `?` or `#`, without
/// a trailing slash (a static host redirects `/docs` to `/docs/`, so the
/// browser's location reads `/docs/` while the route renders `/docs`). The root
/// stays `/`.
fn path_of(route: &str) -> &str {
    let end = route.find(['?', '#']).unwrap_or(route.len());
    let path = route[..end].trim_end_matches('/');
    if path.is_empty() {
        "/"
    } else {
        path
    }
}

/// Whether the internal target `to` is the page the router is on: the same
/// *path*, ignoring the query, the fragment and a trailing slash.
/// An external target is never current.
pub fn is_current(to: &NavigationTarget, current_route: &str) -> bool {
    match to {
        NavigationTarget::Internal(url) => path_of(url) == path_of(current_route),
        NavigationTarget::External(_) => false,
    }
}

/// The props for the [`NavLink`] component: a superset of the router `Link`'s.
#[derive(Props, Clone, PartialEq)]
pub struct NavLinkProps {
    /// The navigation target. Roughly equivalent to the `href` of an anchor.
    #[props(into)]
    pub to: NavigationTarget,

    /// Overrides "`to` is the current page" for a link whose active state is
    /// wider than its own target. Derive it from route state alone so the server
    /// and the browser agree.
    pub active: Option<bool>,

    /// The class attribute for the `a` tag.
    pub class: Option<String>,

    /// A class added to the anchor while it is the current page.
    pub active_class: Option<String>,

    /// When `true`, the target opens in a new tab (the router is not involved).
    #[props(default)]
    pub new_tab: bool,

    /// The onclick event handler.
    pub onclick: Option<EventHandler<MouseEvent>>,

    /// Fired when the `<a>` element is mounted.
    pub onmounted: Option<EventHandler<MountedEvent>>,

    /// When `true` and `onclick` is set, only `onclick` runs (no router
    /// navigation), as on `Link`.
    #[props(default)]
    pub onclick_only: bool,

    /// The `rel` attribute; external targets default to `noopener noreferrer`.
    pub rel: Option<String>,

    /// Additional attributes for the anchor. An `aria-current` here is dropped.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the anchor.
    pub children: Element,
}

/// # NavLink
///
/// A router link with one `aria-current="page"` that is the same in the
/// prerendered HTML and after hydration. See the module docs for why it is not
/// `dioxus_router::Link`.
#[component]
pub fn NavLink(props: NavLinkProps) -> Element {
    let NavLinkProps {
        to,
        active,
        class,
        active_class,
        new_tab,
        onclick,
        onmounted,
        onclick_only,
        rel,
        mut attributes,
        children,
    } = props;
    attributes.retain(|attribute| attribute.name != "aria-current");

    let router = router();
    let is_active = active.unwrap_or_else(|| is_current(&to, &router.full_route_string()));
    let is_external = matches!(to, NavigationTarget::External(_));
    let href = match &to {
        NavigationTarget::Internal(url) => router.prefix().unwrap_or_default() + url,
        NavigationTarget::External(url) => url.clone(),
    };
    let class = match (class, active_class.filter(|_| is_active)) {
        (Some(class), Some(active)) => Some(format!("{class} {active}")),
        (class, active) => class.or(active),
    };
    let rel = rel.or_else(|| is_external.then(|| "noopener noreferrer".to_string()));
    let do_default = onclick.is_none() || !onclick_only;
    let target = to.clone();
    // One list, so a caller's `href`/`rel`/`target` replaces the anchor's own
    // instead of becoming a second attribute; `aria-current` is merged last
    // (and was dropped from the caller's list above), so it is always ours.
    let new_tab_target = new_tab.then_some("_blank");
    let aria_current = is_active.then_some("page");
    let attributes = crate::merge_attributes(vec![
        attributes!(a {
            href: href,
            rel: rel,
            target: new_tab_target
        }),
        attributes,
        attributes!(a {
            aria_current: aria_current
        }),
    ]);

    rsx! {
        a {
            class,
            onmounted: move |event| {
                if let Some(handler) = onmounted {
                    handler.call(event);
                }
            },
            onclick: move |event: MouseEvent| {
                if !event.modifiers().is_empty()
                    || event.trigger_button() != Some(MouseButton::Primary)
                    || new_tab
                    || (do_default && is_external)
                {
                    return;
                }
                event.prevent_default();
                if do_default && !is_external {
                    navigator().push(target.clone());
                }
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

    fn internal(url: &str) -> NavigationTarget {
        NavigationTarget::Internal(url.to_string())
    }

    #[test]
    fn current_ignores_query_and_fragment() {
        assert!(is_current(&internal("/charts/?"), "/charts/"));
        assert!(is_current(&internal("/charts/"), "/charts/?"));
        assert!(is_current(&internal("/docs?dark_mode=true"), "/docs#top"));
        assert!(!is_current(&internal("/charts/bar/"), "/charts/"));
        assert!(is_current(&internal("/docs"), "/docs/"));
        assert!(is_current(&internal("/"), "/?"));
        assert!(!is_current(&internal("/"), "/docs/"));
    }

    #[test]
    fn external_is_never_current() {
        let ext = NavigationTarget::External("https://example.com/".to_string());
        assert!(!is_current(&ext, "https://example.com/"));
    }
}
