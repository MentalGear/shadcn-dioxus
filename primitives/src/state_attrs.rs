//! State attributes: the one place a component gets its `data-*` state vocabulary (backlog row 481).
//!
//! A component never writes `"data-checked"`, `"data-open"`, `"data-disabled"` or any other
//! name of the table below by hand; it spreads the list one function here returns:
//!
//! ```rust,ignore
//! let owned = attributes!(button {
//!     ..state_attrs::checked(checked()),
//!     ..state_attrs::presence(state_attrs::Flag::Disabled, props.disabled),
//!     ..state_attrs::readonly(props.read_only),
//! });
//! ```
//!
//! # One vocabulary: Base UI's presence attributes
//!
//! Every state is a presence attribute, the vocabulary shadcn's `base-nova` classes and Tailwind's
//! `data-open:` / `data-checked:` variants select on: the attribute is present with an empty value while the
//! state holds and **absent** while it does not, so a presence selector (`[data-open]`) never matches the
//! false case. ARIA carries the rest (`aria-pressed`, `aria-invalid`, `aria-expanded`, ...), and a part whose
//! state has no Base UI counterpart takes a `data-<adjective>` name by Base UI's convention (`data-loading`,
//! `data-uploading`, `data-done`). There is no `data-state`.
//!
//! # The vocabulary
//!
//! | function | attributes | Base UI part it follows |
//! |---|---|---|
//! | [`open`] | `data-open` / `data-closed` | Dialog, Popover, Tooltip, HoverCard, Menu, Select, Combobox, NavigationMenu popups and panels, Accordion and Collapsible parts, the Sidebar |
//! | [`trigger_open`] | `data-open` / `data-closed`, `data-popup-open` while open | the triggers of the same parts |
//! | [`checked`] | `data-checked` / `data-unchecked` / `data-indeterminate` | Checkbox, Switch, Radio, CheckboxItem, RadioItem |
//! | [`pressed`] | `data-pressed` while pressed | Toggle, ToggleGroup items |
//! | [`active`] | `data-active` while active | Tabs.Tab |
//! | [`panel_hidden`] | `data-hidden` while inactive | Tabs.Panel |
//! | [`popup_open`] | `data-popup-open` while open | Popover, Tooltip, HoverCard, Select triggers |
//! | [`panel_open`] | `data-panel-open` while open | Accordion and Collapsible triggers |
//! | [`selected`] | `data-selected` while selected | rows and items selected by a parent |
//! | [`progress`] | `data-indeterminate` / `data-progressing` / `data-complete` | Progress.Root |
//! | [`presence`] | one [`Flag`], present-as-empty or absent | `data-readonly`, `data-highlighted`, `data-invalid`, `data-dragging`, `data-loading`, ... |
//!
//! # No valued booleans
//!
//! A boolean state is a presence attribute: present (and empty) while it holds, absent otherwise, never
//! `data-x=true|false`. A presence selector (`[data-disabled]`, Tailwind's `data-disabled:`) therefore matches exactly
//! when the state holds, and a stylesheet or spec asks for the false case with `:not([data-disabled])` or
//! `not.toHaveAttribute`. [`presence`] writes one; there is no valued form.
//!
//! # The gate
//!
//! `scripts/check-state-attributes.sh` fails a component file that writes any name of the vocabulary as a
//! literal (`"data-open"`, `"data-disabled"`, ...) outside this file, and a presence
//! attribute given a boolean or `"false"` value.

use dioxus::core::{Attribute, AttributeValue};
use dioxus::prelude::*;

use crate::checkbox::CheckboxState;

/// A boolean read at render time: a plain `bool` or a signal of one (the call reads the signal, which
/// subscribes the calling component to it exactly as passing the signal as an attribute value does).
pub trait IntoFlag {
    /// The current value.
    fn flag(&self) -> bool;
}

impl IntoFlag for bool {
    fn flag(&self) -> bool {
        *self
    }
}

impl IntoFlag for ReadSignal<bool> {
    fn flag(&self) -> bool {
        self.cloned()
    }
}

impl IntoFlag for Signal<bool> {
    fn flag(&self) -> bool {
        self.cloned()
    }
}

impl IntoFlag for Memo<bool> {
    fn flag(&self) -> bool {
        self.cloned()
    }
}

/// The checked state of a checkbox-like control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Checked {
    /// `data-unchecked`.
    Unchecked,
    /// `data-checked`.
    Checked,
    /// `data-indeterminate`.
    Indeterminate,
}

impl From<bool> for Checked {
    fn from(on: bool) -> Self {
        if on {
            Checked::Checked
        } else {
            Checked::Unchecked
        }
    }
}

impl From<CheckboxState> for Checked {
    fn from(state: CheckboxState) -> Self {
        match state {
            CheckboxState::Checked => Checked::Checked,
            CheckboxState::Unchecked => Checked::Unchecked,
            CheckboxState::Indeterminate => Checked::Indeterminate,
        }
    }
}

/// A boolean state with its own presence attribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flag {
    /// `data-disabled`.
    Disabled,
    /// `data-readonly`.
    Readonly,
    /// `data-highlighted`: the item the pointer or the keyboard is on.
    Highlighted,
    /// `data-selected`.
    Selected,
    /// `data-invalid`.
    Invalid,
    /// `data-active`.
    Active,
    /// `data-dragging`.
    Dragging,
    /// `data-open`.
    Open,
    /// `data-checked`.
    Checked,
    /// `data-unchecked`.
    Unchecked,
    /// `data-popup-open`: the trigger of an open popup.
    PopupOpen,
    /// `data-panel-open`: the trigger of an open panel.
    PanelOpen,
    /// `data-pressed`.
    Pressed,
    /// `data-hidden`.
    Hidden,
    /// `data-loading`: an Avatar whose image is still loading, an Attachment busy uploading.
    Loading,
    /// `data-loaded`: an Avatar whose image has loaded.
    Loaded,
    /// `data-error`.
    Error,
    /// `data-empty`: an Avatar given no image.
    Empty,
    /// `data-idle`.
    Idle,
    /// `data-uploading`.
    Uploading,
    /// `data-processing`.
    Processing,
    /// `data-done`.
    Done,
    /// `data-scrollbar-hidden`: a scroller whose native scrollbar is not drawn while it still scrolls.
    ScrollbarHidden,
}

impl Flag {
    /// The attribute name.
    pub const fn name(self) -> &'static str {
        match self {
            Flag::Disabled => "data-disabled",
            Flag::Readonly => "data-readonly",
            Flag::Highlighted => "data-highlighted",
            Flag::Selected => "data-selected",
            Flag::Invalid => "data-invalid",
            Flag::Active => "data-active",
            Flag::Dragging => "data-dragging",
            Flag::Open => "data-open",
            Flag::Checked => "data-checked",
            Flag::Unchecked => "data-unchecked",
            Flag::PopupOpen => "data-popup-open",
            Flag::PanelOpen => "data-panel-open",
            Flag::Pressed => "data-pressed",
            Flag::Hidden => "data-hidden",
            Flag::Loading => "data-loading",
            Flag::Loaded => "data-loaded",
            Flag::Error => "data-error",
            Flag::Empty => "data-empty",
            Flag::Idle => "data-idle",
            Flag::Uploading => "data-uploading",
            Flag::Processing => "data-processing",
            Flag::Done => "data-done",
            Flag::ScrollbarHidden => "data-scrollbar-hidden",
        }
    }
}

fn text(name: &'static str, value: &'static str) -> Attribute {
    Attribute::new(name, AttributeValue::Text(value.to_string()), None, false)
}

/// A presence attribute: empty while it holds. Callers leave it out while it does not.
fn present(name: &'static str) -> Attribute {
    text(name, "")
}

/// A popup or panel: `data-open` while it is open, `data-closed` while it is closed.
pub fn open(is_open: impl IntoFlag) -> Vec<Attribute> {
    vec![present(if is_open.flag() {
        "data-open"
    } else {
        "data-closed"
    })]
}

/// The trigger of a popup: [`open`] plus `data-popup-open` while it is open.
pub fn trigger_open(is_open: impl IntoFlag) -> Vec<Attribute> {
    let is_open = is_open.flag();
    let mut attrs = open(is_open);
    if is_open {
        attrs.push(present("data-popup-open"));
    }
    attrs
}

/// The trigger of a popup that carries no open/closed pair of its own: `data-popup-open` while the popup is open.
pub fn popup_open(is_open: impl IntoFlag) -> Vec<Attribute> {
    presence(Flag::PopupOpen, is_open)
}

/// The trigger of a panel (Accordion, Collapsible): `data-panel-open` while the panel is open.
pub fn panel_open(is_open: impl IntoFlag) -> Vec<Attribute> {
    presence(Flag::PanelOpen, is_open)
}

/// A checkbox-like control: `data-checked`, `data-unchecked` or `data-indeterminate`.
pub fn checked(state: impl Into<Checked>) -> Vec<Attribute> {
    vec![present(match state.into() {
        Checked::Checked => "data-checked",
        Checked::Unchecked => "data-unchecked",
        Checked::Indeterminate => "data-indeterminate",
    })]
}

/// A toggle: `data-pressed` while it is pressed.
pub fn pressed(is_pressed: impl IntoFlag) -> Vec<Attribute> {
    presence(Flag::Pressed, is_pressed)
}

/// A tab: `data-active` while it is the active one.
pub fn active(is_active: impl IntoFlag) -> Vec<Attribute> {
    presence(Flag::Active, is_active)
}

/// A tab panel: `data-hidden` while it is not the active one.
pub fn panel_hidden(is_active: impl IntoFlag) -> Vec<Attribute> {
    presence(Flag::Hidden, !is_active.flag())
}

/// A row or item selected by a parent: `data-selected` while it is selected.
pub fn selected(is_selected: impl IntoFlag) -> Vec<Attribute> {
    presence(Flag::Selected, is_selected)
}

/// A progress bar: Base UI's `data-indeterminate` (no value), `data-progressing` (a value below the
/// maximum) or `data-complete`.
pub fn progress(value: Option<f64>, max: f64) -> Vec<Attribute> {
    vec![present(match value {
        None => "data-indeterminate",
        Some(v) if v >= max => "data-complete",
        Some(_) => "data-progressing",
    })]
}

/// A boolean state as a presence attribute: present (empty) while it holds, absent otherwise.
pub fn presence(flag: Flag, on: impl IntoFlag) -> Vec<Attribute> {
    if on.flag() {
        vec![present(flag.name())]
    } else {
        Vec::new()
    }
}

/// `data-readonly` as a presence attribute.
pub fn readonly(on: impl IntoFlag) -> Vec<Attribute> {
    presence(Flag::Readonly, on)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::find_elements;
    use std::rc::Rc;

    fn render(attrs: impl Fn() -> Vec<Attribute> + Clone + 'static) -> String {
        let mut dom = VirtualDom::new_with_props(
            move |attrs: Rc<dyn Fn() -> Vec<Attribute>>| {
                let attrs = attrs();
                rsx! { div { id: "t", ..attrs } }
            },
            Rc::new(attrs) as Rc<dyn Fn() -> Vec<Attribute>>,
        );
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    fn names(html: &str) -> Vec<(String, String)> {
        let els = find_elements(html, |a| a.contains_key("id"));
        let mut out: Vec<_> = els[0]
            .attrs
            .iter()
            .filter(|(k, _)| k.starts_with("data-"))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        out.sort();
        out
    }

    fn pair_of(name: &str, value: &str) -> (String, String) {
        (name.to_string(), value.to_string())
    }

    #[test]
    fn open_emits_one_presence_attribute() {
        assert_eq!(
            names(&render(|| open(true))),
            vec![pair_of("data-open", "")]
        );
        assert_eq!(
            names(&render(|| open(false))),
            vec![pair_of("data-closed", "")]
        );
    }

    #[test]
    fn trigger_open_adds_popup_open_only_while_open() {
        let on = names(&render(|| trigger_open(true)));
        assert!(on.contains(&pair_of("data-popup-open", "")), "{on:?}");
        let off = names(&render(|| trigger_open(false)));
        assert!(!off.iter().any(|(k, _)| k == "data-popup-open"), "{off:?}");
    }

    #[test]
    fn checked_covers_the_three_states() {
        assert_eq!(
            names(&render(|| checked(true))),
            vec![pair_of("data-checked", "")]
        );
        assert_eq!(
            names(&render(|| checked(false))),
            vec![pair_of("data-unchecked", "")]
        );
        assert_eq!(
            names(&render(|| checked(CheckboxState::Indeterminate))),
            vec![pair_of("data-indeterminate", "")]
        );
    }

    #[test]
    fn pressed_and_active_are_absent_when_false() {
        assert_eq!(
            names(&render(|| pressed(true))),
            vec![pair_of("data-pressed", "")]
        );
        assert!(names(&render(|| pressed(false))).is_empty());
        assert_eq!(
            names(&render(|| active(true))),
            vec![pair_of("data-active", "")]
        );
        assert!(names(&render(|| active(false))).is_empty());
        assert_eq!(
            names(&render(|| panel_hidden(false))),
            vec![pair_of("data-hidden", "")]
        );
        assert!(names(&render(|| panel_hidden(true))).is_empty());
    }

    #[test]
    fn progress_is_indeterminate_progressing_or_complete() {
        assert_eq!(
            names(&render(|| progress(None, 100.0))),
            vec![pair_of("data-indeterminate", "")]
        );
        assert_eq!(
            names(&render(|| progress(Some(40.0), 100.0))),
            vec![pair_of("data-progressing", "")]
        );
        assert_eq!(
            names(&render(|| progress(Some(100.0), 100.0))),
            vec![pair_of("data-complete", "")]
        );
    }

    #[test]
    fn presence_is_empty_when_true_and_absent_when_false() {
        assert_eq!(
            names(&render(|| presence(Flag::Highlighted, true))),
            vec![pair_of("data-highlighted", "")]
        );
        assert!(names(&render(|| presence(Flag::Highlighted, false))).is_empty());
        assert_eq!(
            names(&render(|| readonly(true))),
            vec![pair_of("data-readonly", "")]
        );
        assert!(names(&render(|| readonly(false))).is_empty());
        assert_eq!(
            names(&render(|| panel_open(true))),
            vec![pair_of("data-panel-open", "")]
        );
        assert!(names(&render(|| panel_open(false))).is_empty());
    }

    #[test]
    fn a_boolean_is_present_while_it_holds_and_absent_otherwise() {
        assert_eq!(
            names(&render(|| presence(Flag::Disabled, true))),
            vec![pair_of("data-disabled", "")]
        );
        assert!(names(&render(|| presence(Flag::Disabled, false))).is_empty());
        assert_eq!(
            names(&render(|| open(false))),
            vec![pair_of("data-closed", "")]
        );
        assert_eq!(
            names(&render(|| open(true))),
            vec![pair_of("data-open", "")]
        );
    }

    #[test]
    fn a_signal_is_read_where_the_helper_is_called() {
        crate::test_support::with_runtime(|| {
            let on = Signal::new(true);
            let attrs = open(on);
            assert!(attrs.iter().any(|a| a.name == "data-open"));
        });
    }
}

/// The helper through the real components: what each one renders for a true and a false state.
#[cfg(test)]
mod ssr_tests {
    use super::*;
    use crate::accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};
    use crate::checkbox::Checkbox;
    use crate::collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};
    use crate::dialog::{DialogContent, DialogRoot};
    use crate::popover::{PopoverContent, PopoverRoot, PopoverTrigger};
    use crate::radio_group::{RadioGroup, RadioItem};
    use crate::switch::{Switch, SwitchThumb};
    use crate::tabs::{TabContent, TabList, TabTrigger, Tabs};
    use crate::test_support::{find_elements, Element as Tag};
    use crate::toggle::Toggle;

    fn render(app: fn() -> dioxus::prelude::Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        // The open effect mounts popup content on the second pass.
        dom.process_events();
        dom.render_immediate_to_vec();
        dioxus_ssr::render(&dom)
    }

    fn by_role(html: &str, role: &str) -> Vec<Tag> {
        find_elements(html, |a| a.get("role").map(String::as_str) == Some(role))
    }

    fn by_attr(html: &str, key: &str) -> Vec<Tag> {
        find_elements(html, |a| a.contains_key(key))
    }

    /// The `data-*` names an element carries, sorted, each with its value.
    fn data(tag: &Tag) -> Vec<(String, String)> {
        let mut out: Vec<_> = tag
            .attrs
            .iter()
            .filter(|(k, _)| {
                k.starts_with("data-") && !k.starts_with("data-dx") && *k != "data-slot"
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        out.sort();
        out
    }

    fn has(tag: &Tag, name: &str) -> bool {
        tag.attrs.contains_key(name)
    }

    fn value<'a>(tag: &'a Tag, name: &str) -> Option<&'a str> {
        tag.attrs.get(name).map(String::as_str)
    }

    #[component]
    fn Checkboxes() -> Element {
        rsx! {
            Checkbox { default_checked: crate::checkbox::CheckboxState::Checked, name: "a" }
            Checkbox { default_checked: crate::checkbox::CheckboxState::Indeterminate, name: "b" }
            Checkbox { name: "c" }
        }
    }

    #[test]
    fn checkbox_renders_one_checked_family_attribute_per_state() {
        let html = render(Checkboxes);
        let boxes = by_role(&html, "checkbox");
        assert_eq!(boxes.len(), 3, "{html}");
        assert_eq!(value(&boxes[0], "data-checked"), Some(""));
        assert!(!has(&boxes[0], "data-unchecked") && !has(&boxes[0], "data-indeterminate"));
        assert_eq!(value(&boxes[1], "data-indeterminate"), Some(""));
        assert!(!has(&boxes[1], "data-checked") && !has(&boxes[1], "data-unchecked"));
        assert_eq!(value(&boxes[2], "data-unchecked"), Some(""));
        assert!(!has(&boxes[2], "data-checked") && !has(&boxes[2], "data-indeterminate"));
        // Disabled and read-only are presence attributes, absent here.
        assert!(!has(&boxes[2], "data-disabled"));
        assert!(!has(&boxes[2], "data-readonly"));
    }

    #[component]
    fn Switches() -> Element {
        rsx! {
            Switch { default_checked: true, name: "a", SwitchThumb {} }
            Switch { name: "b", disabled: true, SwitchThumb {} }
        }
    }

    #[test]
    fn switch_renders_checked_or_unchecked() {
        let html = render(Switches);
        let switches = by_role(&html, "switch");
        assert_eq!(switches.len(), 2, "{html}");
        assert_eq!(
            data(&switches[0]),
            vec![("data-checked".into(), String::new()),]
        );
        assert_eq!(
            data(&switches[1]),
            vec![
                ("data-disabled".into(), String::new()),
                ("data-unchecked".into(), String::new()),
            ]
        );
    }

    #[component]
    fn Toggles() -> Element {
        rsx! {
            Toggle { pressed: None, default_pressed: true, "on" }
            Toggle { pressed: None, "off" }
        }
    }

    #[test]
    fn toggle_renders_pressed_only_while_pressed() {
        let html = render(Toggles);
        let toggles = by_attr(&html, "aria-pressed");
        assert_eq!(toggles.len(), 2, "{html}");
        assert_eq!(value(&toggles[0], "data-pressed"), Some(""));
        assert!(!has(&toggles[1], "data-pressed"));
    }

    #[component]
    fn RadioItems() -> Element {
        rsx! {
            RadioGroup { default_value: "a".to_string(), read_only: true,
                RadioItem { value: "a".to_string(), "A" }
                RadioItem { value: "b".to_string(), "B" }
            }
        }
    }

    #[test]
    fn radio_items_render_checked_and_unchecked() {
        let html = render(RadioItems);
        let items = by_role(&html, "radio");
        assert_eq!(items.len(), 2, "{html}");
        assert_eq!(value(&items[0], "data-checked"), Some(""));
        assert!(!has(&items[0], "data-unchecked"));
        assert_eq!(value(&items[1], "data-unchecked"), Some(""));
        assert!(!has(&items[1], "data-checked"));
        assert_eq!(value(&items[1], "data-readonly"), Some(""));
        let group = by_role(&html, "radiogroup");
        assert_eq!(value(&group[0], "data-readonly"), Some(""));
    }

    #[component]
    fn TabSet() -> Element {
        rsx! {
            Tabs { default_value: "a".to_string(),
                TabList {
                    TabTrigger { value: "a".to_string(), "A" }
                    TabTrigger { value: "b".to_string(), "B" }
                }
                TabContent { value: "a".to_string(), "alpha" }
                TabContent { value: "b".to_string(), "beta" }
            }
        }
    }

    #[test]
    fn tabs_render_active_on_the_trigger_and_hidden_on_the_panel() {
        let html = render(TabSet);
        let tabs = by_role(&html, "tab");
        assert_eq!(tabs.len(), 2, "{html}");
        assert_eq!(value(&tabs[0], "data-active"), Some(""));
        assert!(!has(&tabs[1], "data-active"));
        let panels = by_role(&html, "tabpanel");
        assert_eq!(panels.len(), 2, "{html}");
        assert!(!has(&panels[0], "data-hidden"));
        assert_eq!(value(&panels[1], "data-hidden"), Some(""));
    }

    #[component]
    fn Accordions() -> Element {
        rsx! {
            Accordion {
                AccordionItem { default_open: true,
                    AccordionTrigger { "one" }
                    AccordionContent { "first" }
                }
                AccordionItem {
                    AccordionTrigger { "two" }
                    AccordionContent { "second" }
                }
            }
        }
    }

    #[test]
    fn accordion_items_render_open_or_closed_as_presence_and_panel_open_on_the_trigger() {
        let html = render(Accordions);
        let items = find_elements(&html, |a| {
            (a.contains_key("data-open") || a.contains_key("data-closed"))
                && !a.contains_key("id")
                && !a.contains_key("aria-expanded")
        });
        assert_eq!(items.len(), 2, "{html}");
        assert_eq!(value(&items[0], "data-open"), Some(""));
        assert!(!has(&items[0], "data-closed"));
        assert!(!has(&items[1], "data-open"));
        assert_eq!(value(&items[1], "data-closed"), Some(""));
        let triggers = by_attr(&html, "aria-expanded");
        assert_eq!(triggers.len(), 2, "{html}");
        assert_eq!(value(&triggers[0], "data-panel-open"), Some(""));
        assert!(!has(&triggers[1], "data-panel-open"));
    }

    #[component]
    fn Collapsibles() -> Element {
        rsx! {
            Collapsible { default_open: true,
                CollapsibleTrigger { "t" }
                CollapsibleContent { "c" }
            }
            Collapsible {
                CollapsibleTrigger { "t" }
                CollapsibleContent { "c" }
            }
        }
    }

    #[test]
    fn collapsible_renders_closed_as_presence_and_the_trigger_panel_open() {
        let html = render(Collapsibles);
        let triggers = by_attr(&html, "aria-expanded");
        assert_eq!(triggers.len(), 2, "{html}");
        assert_eq!(value(&triggers[0], "data-open"), Some(""));
        assert_eq!(value(&triggers[0], "data-panel-open"), Some(""));
        assert!(!has(&triggers[0], "data-closed"));
        assert!(!has(&triggers[1], "data-open"));
        assert_eq!(value(&triggers[1], "data-closed"), Some(""));
        assert!(!has(&triggers[1], "data-panel-open"));
    }

    #[component]
    fn Dialogs() -> Element {
        rsx! {
            DialogRoot { default_open: true, DialogContent { "body" } }
        }
    }

    #[test]
    fn dialog_renders_open_as_presence() {
        let html = render(Dialogs);
        // The root carries the open state; on the browser arm the `<dialog>` carries its own too (its `[open]`
        // stays set for the whole exit, so a stylesheet keys the exit on the `<dialog>`'s own `[data-closed]`).
        let expected = if crate::NATIVE_ARM { 1 } else { 2 };
        let open = by_attr(&html, "data-open");
        assert_eq!(open.len(), expected, "{html}");
        for tag in &open {
            assert_eq!(value(tag, "data-open"), Some(""), "{html}");
            assert!(!has(tag, "data-closed"), "{html}");
        }
    }

    #[component]
    fn Popovers() -> Element {
        rsx! {
            PopoverRoot { default_open: true,
                PopoverTrigger { "t" }
                PopoverContent { "body" }
            }
            PopoverRoot { default_open: false,
                PopoverTrigger { "t" }
                PopoverContent { "never" }
            }
        }
    }

    #[test]
    fn popover_renders_open_closed_and_popup_open_on_its_trigger() {
        let html = render(Popovers);
        let triggers = by_attr(&html, "aria-haspopup");
        assert_eq!(triggers.len(), 2, "{html}");
        assert_eq!(value(&triggers[0], "data-popup-open"), Some(""));
        assert!(!has(&triggers[1], "data-popup-open"));
        let open_roots = by_attr(&html, "data-open");
        assert!(!open_roots.is_empty(), "{html}");
        assert!(open_roots.iter().all(|t| !has(t, "data-closed")), "{html}");
        assert!(!by_attr(&html, "data-closed").is_empty(), "{html}");
    }
}
