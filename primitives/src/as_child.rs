//! The `as` render prop: render a component as the caller's own element (Radix `asChild`, row 395).
//!
//! Every component with an `as` prop takes `Option<Callback<AsChild, Element>>`. The callback
//! gets everything the component would have put on and in its own element, so nothing given to
//! the component is lost when the caller picks the element:
//!
//! ```rust
//! use dioxus::prelude::*;
//! use dioxus_primitives::collapsible::{Collapsible, CollapsibleTrigger};
//! use dioxus_primitives::AsChild;
//!
//! #[component]
//! fn Demo() -> Element {
//!     rsx! {
//!         Collapsible {
//!             CollapsibleTrigger {
//!                 r#as: |el: AsChild| rsx! {
//!                     a { href: "#", ..el.attributes, {el.children} }
//!                 },
//!                 "Recent activity"
//!             }
//!         }
//!     }
//! }
//! ```

use dioxus::prelude::*;

/// What an `as` callback receives (row 395).
///
/// Spread [`attributes`](Self::attributes) on the element and render
/// [`children`](Self::children) inside it: `a { href: "#", ..el.attributes, {el.children} }`.
#[derive(Clone, PartialEq)]
pub struct AsChild {
    /// The component's merged attributes, its own event listeners and the handler props given
    /// to it (`onclick`, `onkeydown`, ...) included, so a click on the rendered element runs
    /// them. Give a further handler to the component, or chain it in with
    /// [`merge_attributes`](crate::merge_attributes) (`merge_attributes(vec![el.attributes,
    /// mine])` runs both).
    pub attributes: Vec<Attribute>,
    /// The children given to the component.
    pub children: Element,
}

impl AsChild {
    /// The value an `as` callback is called with.
    pub fn new(attributes: Vec<Attribute>, children: Element) -> Self {
        Self {
            attributes,
            children,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collapsible::{Collapsible, CollapsibleTrigger};
    use crate::test_support::find_element;
    use std::cell::RefCell;

    thread_local! {
        static NAMES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    }

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    /// The children given to the component reach the `as` element, and the component's own
    /// attributes and listener ride in `attributes` (row 395).
    #[test]
    fn as_keeps_children_and_listeners() {
        fn app() -> Element {
            rsx! {
                Collapsible {
                    CollapsibleTrigger {
                        r#as: |el: AsChild| {
                            NAMES.with(|n| {
                                n.borrow_mut().extend(el.attributes.iter().map(|a| a.name.to_string()))
                            });
                            rsx! {
                                a { href: "#t", ..el.attributes, {el.children} }
                            }
                        },
                        span { "data-probe": "child", "Recent activity" }
                    }
                }
            }
        }
        NAMES.with(|n| n.borrow_mut().clear());
        let html = render(app);
        let link = find_element(&html, |a| a.get("href").map(String::as_str) == Some("#t"))
            .expect("the `as` element renders");
        assert_eq!(
            link.attrs.get("aria-expanded").map(String::as_str),
            Some("false")
        );
        assert!(
            html.contains(r#"data-probe="child""#) && html.contains("Recent activity"),
            "the trigger's children render inside the `as` element: {html}"
        );
        assert!(
            html.find(r##"href="#t""##) < html.find(r#"data-probe="child""#),
            "the children are inside the link, after its start tag: {html}"
        );
        let names = NAMES.with(|n| n.borrow().clone());
        assert!(
            names.iter().any(|n| n == "onclick"),
            "the trigger's toggle listener is in `attributes`: {names:?}"
        );
    }
}
