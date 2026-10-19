//! The docs sidebar's component list, in two orders: A-Z (the default) and by category.
//!
//! **Both lists are in the DOM, one is hidden by CSS.** The SSG emits one static page for every visitor, so a
//! remembered choice cannot be rendered by Rust before the first paint: a client-side swap would show the
//! default order and then flip (a flash). Instead the page carries `data-nav-sort="az" | "category"` on `<html>`,
//! set by [`PREPAINT_NAV_SORT_JS`] while the parser is still inside the sidebar (before the lists after it are
//! laid out, so before first paint), and `main.css` shows exactly one of `.dx-nav-az` / `.dx-nav-category` from
//! it. The pressed look of the two toggle buttons is the same CSS, for the same reason. The hidden list is
//! `display: none`, so it is out of the accessibility tree and the tab order; the A-Z list comes first in the
//! source, so with no script at all the page is the default.
//!
//! `aria-pressed` is the one piece Rust owns: it starts as the SSG default and is synced from the attribute
//! once after mount, and on each click (which also writes `localStorage`, wrapped in try/catch, and moves the
//! attribute). The current page's link is scrolled into view inside the sidebar's own scroller after mount and
//! after a toggle, by arithmetic on `scrollTop`: `Element.scrollIntoView` would also scroll the page and every
//! other ancestor (the carousel's row-91 incident).

use crate::components::sidebar::{
    SidebarGroup, SidebarGroupLabel, SidebarMenu, SidebarMenuButton, SidebarMenuItem,
};
use crate::components::{self, ComponentCategory};
use crate::nav_link::NavLink;
use crate::{DocsNavActive, ExtraBadge, ExtraBadgeTip, Route};
use dioxus::prelude::*;
use dioxus_icons::lucide::{
    Bell, ChartColumn, ChevronsUpDown, Compass, Layers, LayoutGrid, MessageSquare, SquareCheck,
};

/// Runs inline, inside the sidebar, before the lists. Copies the remembered order onto `<html>` (an unknown or
/// missing value leaves the attribute off, which is A-Z). Idempotent; `localStorage` can throw (blocked site
/// data, private windows), hence the try/catch, and the page is correct without it.
pub(crate) const PREPAINT_NAV_SORT_JS: &str = "try{var s=localStorage.getItem('dx-nav-sort');if(s==='category'||s==='az')document.documentElement.setAttribute('data-nav-sort',s)}catch(e){}";

/// The click handlers as inline attributes, so the toggle works the moment the page is parsed (before the wasm has
/// booted), like `CopyButton`'s. Rust's own `onclick` only keeps `aria-pressed` and the scroll position in step.
const CHOOSE_AZ_JS: &str = "document.documentElement.setAttribute('data-nav-sort','az');try{localStorage.setItem('dx-nav-sort','az')}catch(e){}";
const CHOOSE_CATEGORY_JS: &str = "document.documentElement.setAttribute('data-nav-sort','category');try{localStorage.setItem('dx-nav-sort','category')}catch(e){}";

/// Reads the order the prepaint script (or a click) put on `<html>`; the SSG default when it is absent.
const READ_NAV_SORT_JS: &str =
    "dioxus.send(document.documentElement.getAttribute('data-nav-sort') === 'category' ? 'category' : 'az')";

/// Scrolls the visible list's current link into the sidebar's scroller, centred, only when it is not already
/// fully visible. Never touches the window or any other ancestor.
const SCROLL_CURRENT_JS: &str = r#"
const links = document.querySelectorAll('[data-slot="sidebar-content"] a[aria-current="page"]');
for (const a of links) {
  if (!a.offsetParent) continue; // inside the hidden list
  let box = a.parentElement;
  while (box && !(box.scrollHeight > box.clientHeight && /(auto|scroll)/.test(getComputedStyle(box).overflowY))) box = box.parentElement;
  if (!box) break;
  const b = box.getBoundingClientRect(), r = a.getBoundingClientRect();
  if (r.top >= b.top && r.bottom <= b.bottom) break;
  box.scrollTop += (r.top - b.top) - (b.height - r.height) / 2;
  break;
}
"#;

#[derive(Clone, Copy, PartialEq, Eq)]
enum NavSort {
    Az,
    Category,
}

/// Every component page, A-Z by its displayed name (underscores are spaces). The category order floats the
/// `chart` overview first (`demos_in_category`); a flat A-Z list has no group to lead, so it is plain A-Z.
fn demos_alphabetical() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = components::DEMOS.iter().map(|d| d.name).collect();
    names.sort_by_key(|name| name.replace('_', " ").to_lowercase());
    names
}

fn category_icon(cat: ComponentCategory) -> Element {
    match cat {
        ComponentCategory::Forms => rsx! {
            SquareCheck { size: "1rem", "aria-hidden": "true" }
        },
        ComponentCategory::Navigation => rsx! {
            Compass { size: "1rem", "aria-hidden": "true" }
        },
        ComponentCategory::Overlays => rsx! {
            Layers { size: "1rem", "aria-hidden": "true" }
        },
        ComponentCategory::Feedback => rsx! {
            Bell { size: "1rem", "aria-hidden": "true" }
        },
        ComponentCategory::Disclosure => rsx! {
            ChevronsUpDown { size: "1rem", "aria-hidden": "true" }
        },
        ComponentCategory::DataDisplay => rsx! {
            LayoutGrid { size: "1rem", "aria-hidden": "true" }
        },
        ComponentCategory::Chat => rsx! {
            MessageSquare { size: "1rem", "aria-hidden": "true" }
        },
        ComponentCategory::Charts => rsx! {
            ChartColumn { size: "1rem", "aria-hidden": "true" }
        },
    }
}

fn nav_item(name: &'static str, active: DocsNavActive) -> Element {
    rsx! {
        SidebarMenuItem { key: "{name}",
            SidebarMenuButton {
                is_active: active == DocsNavActive::Component(name),
                as: move |attributes: Vec<Attribute>| rsx! {
                    NavLink { to: Route::component(name), attributes,
                        {name.replace("_", " ")}
                        ExtraBadge { name, tip: ExtraBadgeTip::Title }
                    }
                },
            }
        }
    }
}

/// The toggle and the two component lists, as `SidebarGroup`s inside the docs `SidebarContent`.
#[component]
pub(crate) fn DocsComponentNav(active: DocsNavActive) -> Element {
    let mut sort = use_signal(|| NavSort::Az);

    // After mount: adopt the order the prepaint script put on `<html>` (for `aria-pressed`; the visuals are
    // CSS), then bring the current link into view in whichever list is showing.
    use_effect(move || {
        spawn(async move {
            let mut read = document::eval(READ_NAV_SORT_JS);
            if let Ok(value) = read.recv::<String>().await {
                sort.set(if value == "category" {
                    NavSort::Category
                } else {
                    NavSort::Az
                });
            }
            document::eval(SCROLL_CURRENT_JS);
        });
    });

    let mut choose = move |next: NavSort| {
        sort.set(next);
        document::eval(SCROLL_CURRENT_JS);
    };

    let names = demos_alphabetical();

    rsx! {
        SidebarGroup { class: "dx-nav-sort-group",
            div { class: "dx-nav-sort", role: "group", aria_label: "Order components by",
                script { dangerous_inner_html: PREPAINT_NAV_SORT_JS }
                button {
                    r#type: "button",
                    class: "dx-nav-sort-option",
                    "data-nav-sort-option": "az",
                    "onclick": CHOOSE_AZ_JS,
                    aria_pressed: if sort() == NavSort::Az { "true" } else { "false" },
                    onclick: move |_| choose(NavSort::Az),
                    "A-Z"
                }
                button {
                    r#type: "button",
                    class: "dx-nav-sort-option",
                    "data-nav-sort-option": "category",
                    "onclick": CHOOSE_CATEGORY_JS,
                    aria_pressed: if sort() == NavSort::Category { "true" } else { "false" },
                    onclick: move |_| choose(NavSort::Category),
                    "Category"
                }
            }
        }
        SidebarGroup { class: "dx-nav-list dx-nav-az",
            SidebarGroupLabel {
                LayoutGrid { size: "1rem", "aria-hidden": "true" }
                span { "Components" }
            }
            SidebarMenu {
                for name in names {
                    {nav_item(name, active)}
                }
            }
        }
        for cat in ComponentCategory::ALL.iter().copied() {
            SidebarGroup { class: "dx-nav-list dx-nav-category", key: "{cat.label()}",
                SidebarGroupLabel {
                    {category_icon(cat)}
                    span { "{cat.label()}" }
                }
                SidebarMenu {
                    for component in components::demos_in_category(cat) {
                        {nav_item(component.name, active)}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_scripts_share_one_storage_key_and_one_attribute() {
        for js in [PREPAINT_NAV_SORT_JS, CHOOSE_AZ_JS, CHOOSE_CATEGORY_JS] {
            assert!(js.contains("'dx-nav-sort'"), "{js}");
            assert!(js.contains("data-nav-sort"), "{js}");
            assert!(
                js.contains("catch(e)"),
                "storage access must be guarded: {js}"
            );
            assert!(!js.contains("</script"), "inlined in a <script>: {js}");
        }
        assert!(READ_NAV_SORT_JS.contains("data-nav-sort"));
    }

    #[test]
    fn a_to_z_is_flat_sorted_and_complete() {
        let names = demos_alphabetical();
        assert_eq!(names.len(), components::DEMOS.len());
        let shown: Vec<String> = names
            .iter()
            .map(|n| n.replace('_', " ").to_lowercase())
            .collect();
        let mut sorted = shown.clone();
        sorted.sort();
        assert_eq!(shown, sorted);
    }
}
