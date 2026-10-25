//! Defines the [`Accordion`] component and its sub-components.

use crate::collapsible::{hidden_attribute, use_beforematch, PanelHidden};
use crate::collection::{
    collection_item, use_item, use_tree_item_index, CollectionOptions, CollectionState,
};
use crate::dioxus_elements::Key;
use crate::direction::use_direction;
use crate::{merge_attributes, use_animated_open, use_id_or, use_unique_id};
use dioxus::prelude::*;
use dioxus_attributes::attributes;

// TODO: controlled version
// TODO: rewrite this to use collapsible

/// Which end of an [`AccordionTrigger`] row the expand chevron sits on.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AccordionChevron {
    /// After the label (shadcn's layout).
    #[default]
    End,
    /// Before the label.
    Start,
}

impl AccordionChevron {
    /// The `data-chevron` value: `"start"` or `"end"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::End => "end",
            Self::Start => "start",
        }
    }
}

/// Internal accordion context.
#[derive(Clone, Copy)]
struct AccordionContext {
    /// Tracks the next runtime-generated id.
    next_id: Signal<usize>,

    /// The runtime generated ids of the open items.
    open_items: Signal<Vec<usize>>,

    /// Whether multiple items can be open at once.
    allow_multiple_open: ReadSignal<bool>,

    /// Whether the entire accordion is disabled.
    disabled: ReadSignal<bool>,

    /// Whether all accordion items can be collapsed.
    collapsible: ReadSignal<bool>,

    /// Whether the accordion is horizontal.
    horizontal: ReadSignal<bool>,

    /// Whether closed content stays mounted as `hidden="until-found"`.
    hidden_until_found: ReadSignal<bool>,

    /// Which end of each trigger the chevron sits on.
    chevron: ReadSignal<AccordionChevron>,

    /// Roving focus state, keyed by the per-item runtime id.
    focus: CollectionState,
}

impl AccordionContext {
    pub fn new(
        allow_multiple_open: ReadSignal<bool>,
        disabled: ReadSignal<bool>,
        collapsible: ReadSignal<bool>,
        horizontal: ReadSignal<bool>,
        hidden_until_found: ReadSignal<bool>,
        chevron: ReadSignal<AccordionChevron>,
        focus_id: Signal<String>,
    ) -> Self {
        Self {
            next_id: Signal::new(0),
            open_items: Signal::new(Vec::new()),
            allow_multiple_open,
            disabled,
            collapsible,
            horizontal,
            hidden_until_found,
            chevron,
            focus: CollectionState::new(
                ReadSignal::new(Signal::new(true)),
                CollectionOptions::default(),
                focus_id,
            ),
        }
    }

    pub fn register_item(&mut self) -> usize {
        let mut next_id = self.next_id.write();
        let id = *next_id;
        *next_id += 1;
        id
    }

    pub fn set_open(&mut self, id: usize) {
        if !*self.allow_multiple_open.peek() {
            self.open_items.clear();
        }
        self.open_items.push(id);
    }

    pub fn set_closed(&mut self, id: usize) {
        let mut open_items = self.open_items.write();

        // If the accordion is not collapsible, we can't close this one.
        if !*self.collapsible.peek() && open_items.len() == 1 {
            return;
        }

        *open_items = open_items
            .iter()
            .cloned()
            .filter(|item| *item != id)
            .collect();
    }

    pub fn is_open(&self, id: usize) -> bool {
        self.open_items.read().contains(&id)
    }

    pub fn is_disabled(&self) -> bool {
        (self.disabled)()
    }

    pub fn is_horizontal(&self) -> bool {
        (self.horizontal)()
    }
}

/// The props for the [`Accordion`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AccordionProps {
    /// The id of the accordion root element.
    pub id: Option<String>,

    /// Whether multiple accordion items are allowed to be open at once.
    ///
    /// Defaults to false.
    #[props(default)]
    pub allow_multiple_open: ReadSignal<bool>,

    /// Set whether the accordion is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether the accordion can be fully collapsed.
    ///
    /// Setting this to true will allow all accordion items to close. Defaults to true.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub collapsible: ReadSignal<bool>,

    /// Whether the accordion is horizontal.
    ///
    /// Settings this to true will use left/right keybinds for navigation instead of up/down. Defaults to false.
    #[props(default)]
    pub horizontal: ReadSignal<bool>,

    /// Keep every [`AccordionContent`] mounted while closed and mark it `hidden="until-found"`, so
    /// the browser's find-in-page (Ctrl+F) and `#fragment` navigation can reach text inside closed
    /// items. When they do, the browser reveals the content and the item opens: the item's
    /// `on_change` is called with `true`, and when [`allow_multiple_open`](Self::allow_multiple_open)
    /// is false the other open item closes, exactly as if its trigger had been clicked.
    ///
    /// Defaults to false, which unmounts closed content after its close animation, so it costs
    /// nothing until opted in; opting in renders every item's content into the server-rendered
    /// HTML as well. Opening because of a match is not animated (the browser has already scrolled
    /// to the match at full size); the close animation plays as usual, and the content is re-hidden
    /// once it finishes.
    ///
    /// Browser support: Chrome 102+, Safari 26.2+ (does not scroll to the match) and Firefox 139+
    /// (148+ for a correct scroll target). Engines that do not know `until-found` treat it as plain
    /// `hidden`: closed content stays unsearchable, as without the prop. Like `hidden`, closed
    /// content is not in the accessibility tree.
    ///
    /// Revealing is not blocked by `disabled` on the accordion or on an item.
    #[props(default)]
    pub hidden_until_found: ReadSignal<bool>,

    /// Which end of every trigger row the expand chevron sits on: [`AccordionChevron::End`]
    /// (default, after the label) or [`AccordionChevron::Start`] (before it). Emitted as
    /// `data-chevron` (`"start"` / `"end"`) on the root and on every [`AccordionTrigger`], so the
    /// theme places and rotates the chevron from one attribute. The primitive draws no chevron
    /// itself; the styled component reads this.
    #[props(default)]
    pub chevron: ReadSignal<AccordionChevron>,

    /// Attributes to extend the root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the accordion, which should contain [`AccordionItem`] components.
    pub children: Element,
}

/// # Accordion
///
/// The accordion component displays a list of collapsible items, allowing users to expand or collapse sections of content.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::accordion::{
///     Accordion, AccordionContent, AccordionItem, AccordionTrigger,
/// };
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Accordion {
///             allow_multiple_open: false,
///             horizontal: false,
///             for _ in 0..4 {
///                 AccordionItem {
///                     on_change: move |open| {
///                         tracing::info!("{open};");
///                     },
///                     on_trigger_click: move || {
///                         tracing::info!("trigger");
///                     },
///                     AccordionTrigger {
///                         "the quick brown fox"
///                     }
///                     AccordionContent {
///                         div { padding_bottom: "1rem",
///                             p {
///                                 padding: "0",
///                                 "Jumped over the lazy dog."
///                             }
///                         }
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`Accordion`] component defines the following data attributes you can use to control styling:
/// - `data-disabled`: Indicates if the accordion is disabled. values are `true` or `false`.
/// - `data-chevron`: Which end of each trigger the chevron sits on, `start` or `end`.
#[component]
pub fn Accordion(props: AccordionProps) -> Element {
    let focus_id = use_unique_id();
    let mut ctx = use_context_provider(|| {
        AccordionContext::new(
            props.allow_multiple_open,
            props.disabled,
            props.collapsible,
            props.horizontal,
            props.hidden_until_found,
            props.chevron,
            focus_id,
        )
    });

    let owned = attributes!(div {
        "data-disabled": (props.disabled)(),
        "data-chevron": (props.chevron)().as_str(),
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        div {
            id: props.id,

            onfocusout: move |_| {
                ctx.focus.clear_focus();
            },

            ..merged,

            {props.children}
        }
    }
}

/// The props for the [`AccordionItem`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AccordionItemProps {
    /// Whether the accordion item is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether this accordion item should be opened by default.
    #[props(default)]
    pub default_open: bool,

    /// Callback for when the accordion's open/closed state changes.
    ///
    /// The new value is provided.
    #[props(default)]
    pub on_change: Callback<bool, ()>,

    /// Callback for when the trigger is clicked.
    #[props(default)]
    pub on_trigger_click: Callback,

    /// Additional attributes to extend the item element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the accordion item.
    pub children: Element,
}

/// # Accordion Item
///
/// The accordion item component represents a single item within an accordion, which can be expanded or collapsed to show or hide its content.
///
/// The [`AccordionItem`] component must be used underneath the [`Accordion`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::accordion::{
///     Accordion, AccordionContent, AccordionItem, AccordionTrigger,
/// };
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Accordion {
///             AccordionItem {
///                 AccordionTrigger {
///                     "the quick brown fox"
///                 }
///                 AccordionContent {
///                     "Jumped over the lazy dog."
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`AccordionItem`] component defines the following data attributes you can use to control styling:
/// - `data-open`: Indicates if the accordion item is open. values are `true` or `false`.
/// - `data-disabled`: Indicates if the accordion is disabled. values are `true` or `false`.
#[component]
pub fn AccordionItem(props: AccordionItemProps) -> Element {
    let mut ctx: AccordionContext = use_context();
    let aria_id = use_unique_id();

    let item = use_context_provider(|| Item {
        id: ctx.register_item(),
        aria_id,
        disabled: props.disabled,
        on_trigger_click: props.on_trigger_click,
    });

    // Open this item if we're set as default.
    use_hook(move || {
        if props.default_open {
            ctx.set_open(item.id);
        }
    });

    // Handle calling `on_change` callback.
    use_effect(move || {
        let open = ctx.is_open(item.id);
        props.on_change.call(open)
    });

    let owned = attributes!(div {
        "data-open": ctx.is_open(item.id),
        "data-disabled": ctx.is_disabled() || item.is_disabled(),
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        div {
            ..merged,

            {props.children}
        }
    }
}

/// The props for the [`AccordionContent`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AccordionContentProps {
    /// The id of the accordion content element.
    pub id: ReadSignal<Option<String>>,
    /// Additional attributes to extend the content element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the accordion content element.
    pub children: Element,
}

/// # Accordion Content
///
/// The accordion content component represents the content of an accordion item that can be
/// expanded or collapsed. The contents will only be displayed when the [`AccordionItem`] is open.
/// With [`AccordionProps::hidden_until_found`] they stay mounted while closed, as `hidden="until-found"`.
///
/// This must be used underneath the [`AccordionItem`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::accordion::{
///     Accordion, AccordionContent, AccordionItem, AccordionTrigger,
/// };
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Accordion {
///             AccordionItem {
///                 AccordionTrigger {
///                     "the quick brown fox"
///                 }
///                 AccordionContent {
///                     "Jumped over the lazy dog."
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`AccordionContent`] component defines the following data attributes you can use to control styling:
/// - `data-open`: Indicates if the accordion item is open. values are `true` or `false`.
/// - `data-closing`: Present (`true`) only while the close animation runs: the item is closed but
///   the content has not been removed (or re-hidden) yet. Style the close animation on this, not on
///   `data-open="false"`, which is also what a closed `hidden="until-found"` panel looks like.
/// - `data-revealed`: Present (`true`) while the item is open because the browser revealed it for a
///   find-in-page or `#fragment` match. It is already at full size and scrolled to, so it should
///   not play the open animation.
#[component]
pub fn AccordionContent(props: AccordionContentProps) -> Element {
    let item: Item = use_context();
    let id = use_id_or(item.aria_id, props.id);
    let mut ctx: AccordionContext = use_context();
    let open = use_memo(move || ctx.is_open(item.id));
    let until_found = ctx.hidden_until_found;

    let render_element = use_animated_open(id, open);

    // `render_element` is the content's presence in the DOM: it turns on a few frames after the
    // item opens and off once the close animation has settled. So "closed but still present" is
    // exactly the close animation.
    let closing = !open() && render_element();

    // With `hidden_until_found` the content never unmounts. It is shown while the item is open and
    // while it animates closed, and `hidden="until-found"` once that has settled. Deriving it from
    // `open()` as well (not `render_element()` alone) renders a default-open item shown in the
    // server HTML, where `render_element` is still false.
    let hidden = if open() || render_element() || !until_found() {
        PanelHidden::Shown
    } else {
        PanelHidden::UntilFound
    };

    // The browser un-hides the content itself; open the item to match. `set_open` closes the
    // other items unless multiple may be open, same as the trigger.
    let mut revealed = use_signal(|| false);
    use_beforematch(id, until_found, move || {
        if !ctx.is_open(item.id) {
            revealed.set(true);
            ctx.set_open(item.id);
        }
    });
    // A later close or re-open is an ordinary, animated one.
    use_effect(move || {
        if !open() {
            revealed.set(false);
        }
    });

    let mut owned = attributes!(div {
        "data-open": open,
        "data-closing": closing.then_some("true"),
        "data-revealed": (open() && revealed()).then_some("true"),
    });
    owned.push(hidden_attribute(hidden));
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    // `open()` is part of the test, not just `render_element()` (which turns on in an effect, so it is
    // always false in the server HTML and in a client's first render): a default-open item must render
    // its content there too, or the server would paint it closed (39px) and hydration would open it (145px) --
    // the same rule the `hidden` computation above applies for `hidden_until_found`.
    rsx! {
        if open() || render_element() || until_found() {
            div {
                id: id,
                ..merged,

                {props.children}
            }
        }
    }
}

/// The props for the [`AccordionTrigger`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AccordionTriggerProps {
    /// THe id of the accordion trigger element.
    pub id: Option<String>,
    /// Additional attributes to extend the trigger element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the accordion trigger element.
    pub children: Element,
}

/// # Accordion Trigger
///
/// The accordion trigger component is a button that toggles the open/closed state of an [`AccordionItem`].
///
/// The [`AccordionTrigger`] component must be used underneath the [`AccordionItem`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::accordion::{
///     Accordion, AccordionContent, AccordionItem, AccordionTrigger,
/// };
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Accordion {
///             AccordionItem {
///                 AccordionTrigger {
///                     "the quick brown fox"
///                 }
///                 AccordionContent {
///                     "Jumped over the lazy dog."
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn AccordionTrigger(props: AccordionTriggerProps) -> Element {
    let mut ctx: AccordionContext = use_context();
    let item: Item = use_context();
    let direction = use_direction(None);

    let disabled = move || ctx.is_disabled() || item.is_disabled();
    // The trigger is the focusable element, so it registers this accordion
    // item, in tree order (`item.id` is the open state's key, not the order).
    let order = use_tree_item_index(ctx.focus);
    let index = order.index;
    let onmounted = use_item(collection_item(ctx.focus, index).disabled(disabled)).onmounted();

    // `type` is an overridable default; the rest is this trigger's own
    // functional state (managed `disabled`/`tabindex`, and aria-controls/
    // aria-expanded reflecting its real relationship/state).
    let defaults = attributes!(button { r#type: "button" });
    let mut owned = attributes!(button {
        disabled: disabled(),
        tabindex: "0",
        aria_controls: item.aria_id(),
        aria_expanded: ctx.is_open(item.id),
        "data-chevron": (ctx.chevron)().as_str(),
    });
    owned.extend(order.marker_attribute());
    let merged = merge_attributes(vec![defaults, props.attributes.clone(), owned]);

    rsx! {
        button {
            id: props.id,

            onmounted,
            onfocus: move |_| {
                ctx.focus.set_focus(Some(index()));
            },
            onkeydown: move |event| {
                // RTL mirrors the horizontal pair (`Direction::ltr_equivalent`).
                let key = direction.ltr_equivalent(&event.key());
                let horizontal = ctx.is_horizontal();
                let mut prevent_default = true;

                match key {
                    Key::ArrowUp if !horizontal => ctx.focus.focus_prev(),
                    Key::ArrowDown if !horizontal => ctx.focus.focus_next(),
                    Key::ArrowLeft if horizontal => ctx.focus.focus_prev(),
                    Key::ArrowRight if horizontal => ctx.focus.focus_next(),
                    Key::Home => ctx.focus.focus_first(),
                    Key::End => ctx.focus.focus_last(),
                    _ => prevent_default = false,
                };

                if prevent_default {
                    event.prevent_default();
                }
            },

            onclick: move |_| {
                if disabled() {
                    return;
                }
                item.on_trigger_click.call(());

                // If the item is not controlled, handle state.
                match ctx.is_open(item.id) {
                    true => ctx.set_closed(item.id),
                    false => ctx.set_open(item.id),
                }
            },

            ..merged,

            {props.children}
        }
    }
}

/// Internal accordion-item context.
#[derive(Clone, Copy, PartialEq)]
struct Item {
    id: usize,
    aria_id: Signal<String>,
    disabled: ReadSignal<bool>,
    on_trigger_click: Callback,
}

impl Item {
    pub fn is_disabled(&self) -> bool {
        (self.disabled)()
    }

    pub fn aria_id(&self) -> String {
        (self.aria_id)()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::find_elements;

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    /// `(data-open, HIDDEN)` of every `AccordionContent` div: the elements with an `id` and a
    /// `data-open` but no `data-disabled` (the item and root divs carry `data-disabled`).
    fn contents(html: &str) -> Vec<(String, Option<String>)> {
        find_elements(html, |a| {
            a.contains_key("id") && a.contains_key("data-open") && !a.contains_key("data-disabled")
        })
        .into_iter()
        .map(|el| {
            (
                el.attrs["data-open"].clone(),
                el.attrs.get("HIDDEN").cloned(),
            )
        })
        .collect()
    }

    #[component]
    fn Plain() -> Element {
        rsx! {
            Accordion {
                AccordionItem {
                    AccordionTrigger { "one" }
                    AccordionContent { "first text" }
                }
            }
        }
    }

    #[component]
    fn UntilFound() -> Element {
        rsx! {
            Accordion { hidden_until_found: true,
                AccordionItem { default_open: true,
                    AccordionTrigger { "one" }
                    AccordionContent { "first text" }
                }
                AccordionItem {
                    AccordionTrigger { "two" }
                    AccordionContent { "second text" }
                }
            }
        }
    }

    #[test]
    fn closed_content_is_unmounted_by_default() {
        let html = render(Plain);
        assert!(!html.contains("first text"), "{html}");
        assert!(contents(&html).is_empty(), "{html}");
    }

    #[test]
    fn hidden_until_found_renders_closed_items_searchable_and_open_items_shown() {
        // The same markup the client hydrates: the open item is shown (not waiting for the
        // client-only `use_animated_open` effect), the closed one is `until-found`.
        let html = render(UntilFound);
        assert!(
            html.contains("first text") && html.contains("second text"),
            "{html}"
        );
        assert_eq!(
            contents(&html),
            vec![
                ("true".to_string(), None),
                ("false".to_string(), Some("until-found".to_string())),
            ],
            "{html}"
        );
    }
}
