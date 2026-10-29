//! State attributes: the one place a component gets its `data-*` state vocabulary (backlog row 481).
//!
//! A component never writes `"data-state"`, `"data-checked"`, `"data-open"`, `"data-disabled"` or any other
//! name of the table below by hand; it spreads the list one function here returns:
//!
//! ```rust,ignore
//! let owned = attributes!(button {
//!     ..state_attrs::checked(checked()),
//!     ..state_attrs::disabled(props.disabled),
//!     ..state_attrs::readonly(props.read_only),
//! });
//! ```
//!
//! # Two vocabularies, one helper
//!
//! * Radix's `data-state="checked|unchecked|open|closed|on|off|active|inactive"` (the vocabulary the
//!   shipped stylesheets and the `tier3-radix` oracle select on). Every function that has a state value
//!   emits it.
//! * Base UI's presence attributes (the vocabulary shadcn's `base-nova` classes and Tailwind's
//!   `data-open:` / `data-checked:` variants select on): the attribute is present with an empty value
//!   while the state holds and **absent** while it does not, so a presence selector (`[data-open]`)
//!   never matches the false case.
//!
//! Both come from the same call, so they cannot disagree.
//!
//! # The vocabulary
//!
//! | function | `data-state` | presence attributes | Base UI part it follows |
//! |---|---|---|---|
//! | [`open`] | `open` / `closed` | `data-open` / `data-closed` | Dialog, Popover, Tooltip, HoverCard, Menu, Select, Combobox, NavigationMenu popups and panels |
//! | [`trigger_open`] | `open` / `closed` | `data-open` / `data-closed`, `data-popup-open` while open | the triggers of the same parts |
//! | [`checked`] | `checked` / `unchecked` / `indeterminate` | `data-checked` / `data-unchecked` / `data-indeterminate` | Checkbox, Switch, Radio, CheckboxItem, RadioItem |
//! | [`pressed`] | `on` / `off` | `data-pressed` while pressed | Toggle, ToggleGroup items |
//! | [`active`] | `active` / `inactive` | `data-active` while active | Tabs.Tab |
//! | [`panel_hidden`] | `active` / `inactive` | `data-hidden` while inactive | Tabs.Panel |
//! | [`popup_open`] | none | `data-popup-open` while open | Popover, Tooltip, HoverCard, Select triggers |
//! | [`panel_open`] | none | `data-panel-open` while open | Accordion and Collapsible triggers |
//! | [`selected`] | `selected` while selected | `data-selected` while selected | rows and items selected by a parent |
//! | [`progress`] | `loading` / `indeterminate` | `data-indeterminate` / `data-progressing` / `data-complete` | Progress.Root |
//! | [`presence`] | none | one [`Flag`], present-as-empty or absent | `data-readonly`, `data-highlighted`, `data-invalid`, `data-selected`, ... |
//! | [`state`] | the given value | none | a `data-state` with no Base UI counterpart (Avatar, Progress status, ...) |
//!
//! # Valued booleans are legacy
//!
//! Many stylesheets and specs select a boolean as `[data-disabled="true"]` / `[data-open="false"]`, so
//! those attributes are still emitted as `data-x=true|false` through [`valued`] (and [`disabled`], which is
//! `valued(Flag::Disabled, ..)`). A valued boolean matches a presence selector in its false case, so it is
//! a convention that cannot be pasted from upstream Tailwind classes: each call is counted per file by
//! `scripts/check-state-attributes.sh` against a shrink-only baseline, and moving a flag to presence is a
//! one-line change here plus its CSS and specs. A new component uses [`presence`].
//!
//! # The gate
//!
//! `scripts/check-state-attributes.sh` fails a component file that writes any name of the vocabulary as a
//! literal (`"data-state"`, `"data-open"`, `"data-disabled"`, ...) outside this file, and a presence
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
    /// `data-state="unchecked"`, `data-unchecked`.
    Unchecked,
    /// `data-state="checked"`, `data-checked`.
    Checked,
    /// `data-state="indeterminate"`, `data-indeterminate`.
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

/// A boolean state with its own attribute and no `data-state` value.
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
    /// `data-open`, on parts that have no `data-state` (Accordion, Collapsible).
    Open,
    /// `data-checked`, on parts that have no `data-state`.
    Checked,
    /// `data-unchecked`, on parts that have no `data-state`.
    Unchecked,
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

/// `data-state` plus the presence attribute of the matching Base UI state.
fn pair(state: &'static str, presence_attr: Option<&'static str>) -> Vec<Attribute> {
    let mut attrs = vec![text("data-state", state)];
    attrs.extend(presence_attr.map(present));
    attrs
}

/// A popup or panel: `data-state="open|closed"`, and `data-open` or `data-closed`.
pub fn open(is_open: impl IntoFlag) -> Vec<Attribute> {
    if is_open.flag() {
        pair("open", Some("data-open"))
    } else {
        pair("closed", Some("data-closed"))
    }
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

/// The trigger of a popup that carries no `data-state` of its own: `data-popup-open` while the popup is open.
pub fn popup_open(is_open: impl IntoFlag) -> Vec<Attribute> {
    if is_open.flag() {
        vec![present("data-popup-open")]
    } else {
        Vec::new()
    }
}

/// The trigger of a panel that has no `data-state` (Accordion, Collapsible): `data-panel-open` while open.
pub fn panel_open(is_open: impl IntoFlag) -> Vec<Attribute> {
    if is_open.flag() {
        vec![present("data-panel-open")]
    } else {
        Vec::new()
    }
}

/// A part that has no `data-state` and is open or closed (Accordion item and panel, Collapsible):
/// the legacy valued `data-open=true|false` (see [`valued`]) and the presence attribute `data-closed`.
pub fn open_valued(is_open: impl IntoFlag) -> Vec<Attribute> {
    let is_open = is_open.flag();
    let mut attrs = valued(Flag::Open, is_open);
    if !is_open {
        attrs.push(present("data-closed"));
    }
    attrs
}

/// A checkbox-like control: `data-state="checked|unchecked|indeterminate"` and the matching
/// `data-checked`, `data-unchecked` or `data-indeterminate`.
pub fn checked(state: impl Into<Checked>) -> Vec<Attribute> {
    match state.into() {
        Checked::Checked => pair("checked", Some("data-checked")),
        Checked::Unchecked => pair("unchecked", Some("data-unchecked")),
        Checked::Indeterminate => pair("indeterminate", Some("data-indeterminate")),
    }
}

/// A toggle: `data-state="on|off"`, and `data-pressed` while it is pressed.
pub fn pressed(is_pressed: impl IntoFlag) -> Vec<Attribute> {
    if is_pressed.flag() {
        pair("on", Some("data-pressed"))
    } else {
        pair("off", None)
    }
}

/// A tab: `data-state="active|inactive"`, and `data-active` while it is the active one.
pub fn active(is_active: impl IntoFlag) -> Vec<Attribute> {
    if is_active.flag() {
        pair("active", Some("data-active"))
    } else {
        pair("inactive", None)
    }
}

/// A tab panel: `data-state="active|inactive"`, and `data-hidden` while it is not the active one.
pub fn panel_hidden(is_active: impl IntoFlag) -> Vec<Attribute> {
    if is_active.flag() {
        pair("active", None)
    } else {
        pair("inactive", Some("data-hidden"))
    }
}

/// A selected row or item that is otherwise unstyled by `data-state`: `data-state="selected"` and
/// `data-selected` while it is selected, nothing while it is not.
pub fn selected(is_selected: impl IntoFlag) -> Vec<Attribute> {
    if is_selected.flag() {
        pair("selected", Some("data-selected"))
    } else {
        Vec::new()
    }
}

/// A progress bar: `data-state="loading"` (a value is set) or `"indeterminate"`, and Base UI's
/// `data-indeterminate`, `data-progressing` (a value below the maximum) or `data-complete`.
pub fn progress(value: Option<f64>, max: f64) -> Vec<Attribute> {
    match value {
        None => pair("indeterminate", Some("data-indeterminate")),
        Some(v) if v >= max => pair("loading", Some("data-complete")),
        Some(_) => pair("loading", Some("data-progressing")),
    }
}

/// A bare `data-state` with a value no Base UI part has a presence attribute for (an Avatar's loading
/// status, a Progress bar's, a visibility state).
pub fn state(value: &'static str) -> Vec<Attribute> {
    pair(value, None)
}

/// A boolean state as a presence attribute: present (empty) while it holds, absent otherwise.
pub fn presence(flag: Flag, on: impl IntoFlag) -> Vec<Attribute> {
    if on.flag() {
        vec![present(flag.name())]
    } else {
        Vec::new()
    }
}

/// A boolean state as a presence attribute whose value is `true`: `data-x="true"` while it holds, absent
/// otherwise, so the presence selector (`[data-x]`) and the legacy `[data-x="true"]` selector both match
/// exactly the true case.
pub fn presence_true(flag: Flag, on: impl IntoFlag) -> Vec<Attribute> {
    if on.flag() {
        vec![text(flag.name(), "true")]
    } else {
        Vec::new()
    }
}

/// `data-readonly` as a presence attribute.
pub fn readonly(on: impl IntoFlag) -> Vec<Attribute> {
    presence(Flag::Readonly, on)
}

/// A boolean state as the legacy valued attribute `data-x=true|false`, which is present in both cases.
/// It exists for the attributes whose stylesheets and specs select `="true"` / `="false"`; the gate
/// counts each call per file against a shrink-only baseline.
pub fn valued(flag: Flag, on: impl IntoFlag) -> Vec<Attribute> {
    vec![Attribute::new(
        flag.name(),
        AttributeValue::Bool(on.flag()),
        None,
        false,
    )]
}

/// `data-disabled=true|false`, the valued form every stylesheet selects (`[data-disabled="true"]`).
pub fn disabled(on: impl IntoFlag) -> Vec<Attribute> {
    valued(Flag::Disabled, on)
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
    fn open_emits_state_and_one_presence_attribute() {
        assert_eq!(
            names(&render(|| open(true))),
            vec![pair_of("data-open", ""), pair_of("data-state", "open")]
        );
        assert_eq!(
            names(&render(|| open(false))),
            vec![pair_of("data-closed", ""), pair_of("data-state", "closed")]
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
            vec![
                pair_of("data-checked", ""),
                pair_of("data-state", "checked")
            ]
        );
        assert_eq!(
            names(&render(|| checked(false))),
            vec![
                pair_of("data-state", "unchecked"),
                pair_of("data-unchecked", "")
            ]
        );
        assert_eq!(
            names(&render(|| checked(CheckboxState::Indeterminate))),
            vec![
                pair_of("data-indeterminate", ""),
                pair_of("data-state", "indeterminate")
            ]
        );
    }

    #[test]
    fn pressed_and_active_omit_the_presence_attribute_when_false() {
        assert_eq!(
            names(&render(|| pressed(true))),
            vec![pair_of("data-pressed", ""), pair_of("data-state", "on")]
        );
        assert_eq!(
            names(&render(|| pressed(false))),
            vec![pair_of("data-state", "off")]
        );
        assert_eq!(
            names(&render(|| active(true))),
            vec![pair_of("data-active", ""), pair_of("data-state", "active")]
        );
        assert_eq!(
            names(&render(|| active(false))),
            vec![pair_of("data-state", "inactive")]
        );
        assert_eq!(
            names(&render(|| panel_hidden(false))),
            vec![
                pair_of("data-hidden", ""),
                pair_of("data-state", "inactive")
            ]
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
    fn valued_booleans_stay_valued_until_their_css_moves() {
        assert_eq!(
            names(&render(|| disabled(true))),
            vec![pair_of("data-disabled", "true")]
        );
        assert_eq!(
            names(&render(|| disabled(false))),
            vec![pair_of("data-disabled", "false")]
        );
        assert_eq!(
            names(&render(|| open_valued(false))),
            vec![pair_of("data-closed", ""), pair_of("data-open", "false")]
        );
        assert_eq!(
            names(&render(|| open_valued(true))),
            vec![pair_of("data-open", "true")]
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
        assert_eq!(value(&boxes[0], "data-state"), Some("checked"));
        assert_eq!(value(&boxes[0], "data-checked"), Some(""));
        assert!(!has(&boxes[0], "data-unchecked") && !has(&boxes[0], "data-indeterminate"));
        assert_eq!(value(&boxes[1], "data-state"), Some("indeterminate"));
        assert_eq!(value(&boxes[1], "data-indeterminate"), Some(""));
        assert!(!has(&boxes[1], "data-checked") && !has(&boxes[1], "data-unchecked"));
        assert_eq!(value(&boxes[2], "data-state"), Some("unchecked"));
        assert_eq!(value(&boxes[2], "data-unchecked"), Some(""));
        assert!(!has(&boxes[2], "data-checked") && !has(&boxes[2], "data-indeterminate"));
        // The valued boolean stays valued; read-only is a presence attribute and absent here.
        assert_eq!(value(&boxes[2], "data-disabled"), Some("false"));
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
            vec![
                ("data-checked".into(), String::new()),
                ("data-disabled".into(), "false".into()),
                ("data-state".into(), "checked".into()),
            ]
        );
        assert_eq!(
            data(&switches[1]),
            vec![
                ("data-disabled".into(), "true".into()),
                ("data-state".into(), "unchecked".into()),
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
        assert_eq!(value(&toggles[0], "data-state"), Some("on"));
        assert!(!has(&toggles[1], "data-pressed"));
        assert_eq!(value(&toggles[1], "data-state"), Some("off"));
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
        assert_eq!(value(&tabs[0], "data-state"), Some("active"));
        assert_eq!(value(&tabs[0], "data-active"), Some(""));
        assert_eq!(value(&tabs[1], "data-state"), Some("inactive"));
        assert!(!has(&tabs[1], "data-active"));
        let panels = by_role(&html, "tabpanel");
        assert_eq!(panels.len(), 2, "{html}");
        assert!(!has(&panels[0], "data-hidden"));
        assert_eq!(value(&panels[1], "data-hidden"), Some(""));
        assert_eq!(value(&panels[1], "data-state"), Some("inactive"));
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
    fn accordion_items_render_open_valued_closed_present_and_panel_open_on_the_trigger() {
        let html = render(Accordions);
        let items: Vec<Tag> = by_attr(&html, "data-open")
            .into_iter()
            .filter(|t| t.attrs.contains_key("data-disabled"))
            .collect();
        assert_eq!(items.len(), 2, "{html}");
        // `data-open` stays `true|false` (stylesheets select `="true"`); `data-closed` is presence.
        assert_eq!(value(&items[0], "data-open"), Some("true"));
        assert!(!has(&items[0], "data-closed"));
        assert_eq!(value(&items[1], "data-open"), Some("false"));
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
        assert_eq!(value(&triggers[0], "data-open"), Some("true"));
        assert_eq!(value(&triggers[0], "data-panel-open"), Some(""));
        assert!(!has(&triggers[0], "data-closed"));
        assert_eq!(value(&triggers[1], "data-open"), Some("false"));
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
    fn dialog_renders_open_as_presence_beside_data_state() {
        let html = render(Dialogs);
        let roots = by_attr(&html, "data-state");
        assert_eq!(roots.len(), 1, "{html}");
        assert_eq!(value(&roots[0], "data-state"), Some("open"));
        assert_eq!(value(&roots[0], "data-open"), Some(""), "{html}");
        assert!(!has(&roots[0], "data-closed"), "{html}");
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
        let roots = by_attr(&html, "data-state");
        assert!(
            roots.iter().any(
                |t| value(t, "data-state") == Some("open") && value(t, "data-open") == Some("")
            ),
            "{html}"
        );
        assert!(
            roots
                .iter()
                .any(|t| value(t, "data-state") == Some("closed")
                    && value(t, "data-closed") == Some("")),
            "{html}"
        );
    }
}
