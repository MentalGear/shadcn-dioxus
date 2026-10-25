//! Menu item parts -- checkable items (`menuitemcheckbox`, `menuitemradio`),
//! the `variant`/`inset` item styling hooks, and the presentational
//! `Separator`, `Group` and `Shortcut` -- built once for the whole menu
//! family: [`crate::dropdown_menu`], [`crate::context_menu`] and
//! [`crate::menubar`].
//!
//! # Item parts: `variant`, `inset`, separator, group, shortcut
//!
//! shadcn's three menus share one set of parts, and so do these hosts:
//!
//! - **`variant`** -- [`MenuItemVariant`]: `Default` or `Destructive`. A
//!   destructive item is a plain `menuitem` (the role does not change); the
//!   primitive emits `data-variant="default" | "destructive"` through
//!   `item_style_attributes`, the one place that attribute is written, and
//!   the themed sheets paint it.
//! - **`inset`** -- reserves the leading indicator slot on an item that draws
//!   no indicator, so a menu that mixes plain and checkable items keeps its
//!   labels on one edge. The primitive emits `data-inset="true"` only while
//!   it is set (nothing otherwise), again through `item_style_attributes`.
//! - **Separator / Group / Shortcut** -- `render_separator`
//!   (`role="separator"`, `aria-orientation="horizontal"`), `render_group`
//!   (`role="group"`, name it with `aria-label`) and `render_shortcut` (a
//!   `span`). None of them registers in a roving-focus collection, so arrow
//!   keys, Home/End and typeahead walk straight past them; they take no
//!   `index`.
//!
//! # Checkable items
//!
//! shadcn's `DropdownMenu`, `ContextMenu` and `Menubar` all have a
//! `CheckboxItem` and a `RadioGroup` + `RadioItem`. Writing them three times
//! would be the
//! same trap `crate::menu_semantics` and `crate::menu_root` exist to close:
//! three hosts hand-writing one contract, free to drift apart. So everything
//! that is *the same* for every host lives here, and each host adds only
//! what genuinely differs (which roving-focus collection the item registers
//! in, how a pointer press is committed, what "close the menu" means):
//!
//! - **Roles and ARIA** -- `CheckableState::owned_attributes`: `role`
//!   (`menuitemcheckbox`/`menuitemradio`, from `crate::menu_semantics`),
//!   `aria-checked` (always present, `"true"` or `"false"`), `data-state`
//!   (`checked`/`unchecked`), `aria-disabled`, `data-disabled`, `tabindex`.
//!   Owned by the primitive, so they win over a caller's own attributes.
//! - **State** -- `use_checkbox_state` (controlled or uncontrolled
//!   `checked`) and `use_radio_group` + `use_radio_state` (controlled or
//!   uncontrolled group `value`; an item is checked when its `value` equals
//!   the group's).
//! - **Props** -- [`MenuCheckboxItemProps`], [`MenuRadioItemProps`] and
//!   [`MenuRadioGroupProps`], re-exported by each host under its own name
//!   (`DropdownMenuCheckboxItemProps`, ...), so the three APIs cannot drift.
//! - **The radio group** -- `render_radio_group`, a `role="group"`
//!   element, identical for every host.
//! - **Activation** -- `CheckableState::activate` flips a checkbox or
//!   chooses a radio's value; `is_activation_key` is the one definition of
//!   "Enter or Space".
//!
//! # Close semantics
//!
//! Radix's `onSelect` default, which shadcn inherits: selecting a checkable
//! item **closes the menu**. Every checkable item has a
//! `close_on_select` prop (default `true`) to opt out and keep the menu open
//! after a toggle -- the behaviour APG describes as optional for Space ("When
//! focus is on a menuitemcheckbox, changes the state without closing the
//! menu", `content/patterns/menubar/menu-and-menubar-pattern.html`, "Keyboard
//! Interaction", pinned commit `7e4034b262bc0d25332e330d8a582aaf34113829`),
//! and the default of Base UI (the shadcn Nova registry's primitive).
//!
//! # Controlled or uncontrolled
//!
//! Both are supported, as everywhere in this crate -- but note what
//! "uncontrolled" means in a menu: the state lives in the item, and a menu's
//! content unmounts when it closes, so an uncontrolled `default_checked` /
//! `default_value` item **starts over every time the menu opens** (the same
//! as Radix). Hold the state in the caller and pass `checked` / `value` for
//! anything that must survive a close, which is what shadcn's own demos do.
//!
//! # Typeahead
//!
//! A checkable item registers in the same roving-focus collection as a plain
//! item, with the same `text_value`, so it is a typeahead target
//! (`crate::typeahead`) and is skipped by arrow keys, Home/End and typeahead
//! when disabled, exactly like a plain item. A radio item falls back to its
//! `value` as its label; a checkbox item has no `value`, so it is
//! searchable only when `text_value` is set.

use dioxus::prelude::*;
use dioxus_attributes::attributes;

use crate::{merge_attributes, use_controlled};

/// Which look a menu item has: shadcn's `variant="default" | "destructive"`
/// on `DropdownMenuItem`, `ContextMenuItem` and `MenubarItem`. The role is a
/// plain `menuitem` either way; a destructive item only changes how it is
/// painted (the destructive ink and a destructive-tinted highlight).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuItemVariant {
    /// An ordinary item.
    #[default]
    Default,
    /// An item whose action destroys something ("Delete"): drawn in the
    /// destructive ink.
    Destructive,
}

impl MenuItemVariant {
    /// The value of the `data-variant` attribute the item carries.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Destructive => "destructive",
        }
    }
}

/// The `data-variant` and `data-inset` attributes of a menu item -- the one
/// place either is written, for every item of every host (plain items,
/// submenu triggers, checkbox and radio items), so the three sheets key
/// their rules off one vocabulary: `[data-variant="destructive"]` and
/// `[data-inset="true"]`.
///
/// `data-variant` is always present (shadcn's `data-variant={variant}`);
/// `data-inset` only while `inset` is set, so an item that does not ask for
/// the slot is not touched (a caller's own `"data-inset": "true"` keeps
/// working). A host merges this *after* the caller's attributes so the
/// primitive's value wins.
pub(crate) fn item_style_attributes(variant: MenuItemVariant, inset: bool) -> Vec<Attribute> {
    let mut attributes = attributes!(div {
        "data-variant": variant.as_str(),
    });
    if inset {
        attributes.extend(attributes!(div {
            "data-inset": "true",
        }));
    }
    attributes
}

/// The props for a separator in a menu: [`DropdownMenuSeparator`]
/// (`crate::dropdown_menu`), [`ContextMenuSeparator`] (`crate::context_menu`)
/// and [`MenubarSeparator`] (`crate::menubar`) all take exactly these.
///
/// [`DropdownMenuSeparator`]: crate::dropdown_menu::DropdownMenuSeparator
/// [`ContextMenuSeparator`]: crate::context_menu::ContextMenuSeparator
/// [`MenubarSeparator`]: crate::menubar::MenubarSeparator
#[derive(Props, Clone, PartialEq)]
pub struct MenuSeparatorProps {
    /// Additional attributes to apply to the separator element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The props for a group of items in a menu: [`DropdownMenuGroup`]
/// (`crate::dropdown_menu`), [`ContextMenuGroup`] (`crate::context_menu`) and
/// [`MenubarGroup`] (`crate::menubar`) all take exactly these.
///
/// [`DropdownMenuGroup`]: crate::dropdown_menu::DropdownMenuGroup
/// [`ContextMenuGroup`]: crate::context_menu::ContextMenuGroup
/// [`MenubarGroup`]: crate::menubar::MenubarGroup
#[derive(Props, Clone, PartialEq)]
pub struct MenuGroupProps {
    /// Additional attributes to apply to the group element. Give the group
    /// an accessible name with `aria-label` or `aria-labelledby` when it has
    /// a visible heading.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The items of the group.
    pub children: Element,
}

/// The props for the shortcut hint of an item: [`DropdownMenuShortcut`]
/// (`crate::dropdown_menu`), [`ContextMenuShortcut`] (`crate::context_menu`)
/// and [`MenubarShortcut`] (`crate::menubar`) all take exactly these.
///
/// [`DropdownMenuShortcut`]: crate::dropdown_menu::DropdownMenuShortcut
/// [`ContextMenuShortcut`]: crate::context_menu::ContextMenuShortcut
/// [`MenubarShortcut`]: crate::menubar::MenubarShortcut
#[derive(Props, Clone, PartialEq)]
pub struct MenuShortcutProps {
    /// Additional attributes to apply to the shortcut element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The hint's text, for example `⌘T`.
    pub children: Element,
}

/// The props for a checkbox item in a menu: [`DropdownMenuCheckboxItem`]
/// (`crate::dropdown_menu`), [`ContextMenuCheckboxItem`]
/// (`crate::context_menu`) and [`MenubarCheckboxItem`] (`crate::menubar`)
/// all take exactly these.
///
/// [`DropdownMenuCheckboxItem`]: crate::dropdown_menu::DropdownMenuCheckboxItem
/// [`ContextMenuCheckboxItem`]: crate::context_menu::ContextMenuCheckboxItem
/// [`MenubarCheckboxItem`]: crate::menubar::MenubarCheckboxItem
#[derive(Props, Clone, PartialEq)]
pub struct MenuCheckboxItemProps {
    /// The item's place in keyboard order. Leave it unset: items are ordered as they are
    /// created, which is tree order. Set it on every item only when one is inserted between
    /// items that are already mounted, or a keyed list reorders them.
    #[props(default)]
    pub index: ReadSignal<Option<usize>>,

    /// Whether the item is checked. If not provided, the item is
    /// uncontrolled and uses [`Self::default_checked`].
    pub checked: ReadSignal<Option<bool>>,

    /// The initial checked state when the item is uncontrolled.
    #[props(default)]
    pub default_checked: bool,

    /// Callback fired with the new checked state when the item is toggled.
    #[props(default)]
    pub on_checked_change: Callback<bool>,

    /// Whether the item is disabled. A disabled item cannot be toggled and
    /// is skipped by roving focus and typeahead.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Explicit label text used for typeahead search. A checkbox item has no
    /// `value` to fall back to, so it is only searchable when this is set.
    #[props(default)]
    pub text_value: ReadSignal<Option<String>>,

    /// Whether toggling the item closes the menu. Defaults to `true`, Radix's
    /// `onSelect` default; set `false` to keep the menu open so several
    /// items can be toggled in one visit.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub close_on_select: ReadSignal<bool>,

    /// Reserves the leading indicator slot without drawing anything -- shadcn's
    /// `inset`. See `item_style_attributes`.
    #[props(default)]
    pub inset: bool,

    /// Additional attributes to apply to the item element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the item: its label, and any icon.
    pub children: Element,
}

/// The props for a radio item in a menu: [`DropdownMenuRadioItem`]
/// (`crate::dropdown_menu`), [`ContextMenuRadioItem`]
/// (`crate::context_menu`) and [`MenubarRadioItem`] (`crate::menubar`) all
/// take exactly these. It must be inside the matching `*RadioGroup`.
///
/// [`DropdownMenuRadioItem`]: crate::dropdown_menu::DropdownMenuRadioItem
/// [`ContextMenuRadioItem`]: crate::context_menu::ContextMenuRadioItem
/// [`MenubarRadioItem`]: crate::menubar::MenubarRadioItem
#[derive(Props, Clone, PartialEq)]
pub struct MenuRadioItemProps {
    /// The item's place in keyboard order. Leave it unset: items are ordered as they are
    /// created, which is tree order. Set it on every item only when one is inserted between
    /// items that are already mounted, or a keyed list reorders them.
    #[props(default)]
    pub index: ReadSignal<Option<usize>>,

    /// The value of the item. The item is checked while the group's value
    /// equals this.
    pub value: ReadSignal<String>,

    /// Whether the item is disabled. A disabled item cannot be chosen and is
    /// skipped by roving focus and typeahead.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Explicit label text used for typeahead search. Falls back to
    /// [`Self::value`].
    #[props(default)]
    pub text_value: ReadSignal<Option<String>>,

    /// Whether choosing the item closes the menu. Defaults to `true`,
    /// Radix's `onSelect` default; set `false` to keep the menu open.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub close_on_select: ReadSignal<bool>,

    /// Reserves the leading indicator slot without drawing anything -- shadcn's
    /// `inset`. See `item_style_attributes`.
    #[props(default)]
    pub inset: bool,

    /// Additional attributes to apply to the item element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the item: its label, and any icon.
    pub children: Element,
}

/// The props for a radio group in a menu: [`DropdownMenuRadioGroup`]
/// (`crate::dropdown_menu`), [`ContextMenuRadioGroup`]
/// (`crate::context_menu`) and [`MenubarRadioGroup`] (`crate::menubar`) all
/// take exactly these.
///
/// [`DropdownMenuRadioGroup`]: crate::dropdown_menu::DropdownMenuRadioGroup
/// [`ContextMenuRadioGroup`]: crate::context_menu::ContextMenuRadioGroup
/// [`MenubarRadioGroup`]: crate::menubar::MenubarRadioGroup
#[derive(Props, Clone, PartialEq)]
pub struct MenuRadioGroupProps {
    /// The value of the checked radio item. If not provided, the group is
    /// uncontrolled and uses [`Self::default_value`].
    pub value: ReadSignal<Option<String>>,

    /// The initial value when the group is uncontrolled.
    #[props(default)]
    pub default_value: String,

    /// Callback fired with the item's value when a radio item is chosen.
    #[props(default)]
    pub on_value_change: Callback<String>,

    /// Additional attributes to apply to the group element. Give the group
    /// an accessible name with `aria-label` or `aria-labelledby`.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The radio items of the group.
    pub children: Element,
}

/// Which of the two checkable roles an item has.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckableKind {
    Checkbox,
    Radio,
}

impl CheckableKind {
    fn role(self) -> &'static str {
        match self {
            Self::Checkbox => crate::menu_semantics::MENU_ITEM_CHECKBOX_ROLE,
            Self::Radio => crate::menu_semantics::MENU_ITEM_RADIO_ROLE,
        }
    }
}

/// What a checkable item reports (is it checked) and does (flip, or choose
/// its value) -- the half of a checkable item that is independent of the
/// host menu. `Copy`, so each host's handlers capture it freely.
#[derive(Clone, Copy)]
pub(crate) struct CheckableState {
    kind: CheckableKind,
    checked: Memo<bool>,
    activate: Callback<()>,
    /// A radio item's `value`, the label typeahead falls back to.
    text_fallback: Option<ReadSignal<String>>,
}

impl CheckableState {
    pub(crate) fn checked(&self) -> bool {
        (self.checked)()
    }

    /// Toggle a checkbox / choose a radio's value. Does not close the menu;
    /// that is the host's call, via the item's `close_on_select`.
    pub(crate) fn activate(&self) {
        self.activate.call(());
    }

    /// The item's typeahead label: the explicit `text_value`, else (radio)
    /// its `value`.
    pub(crate) fn text_value(&self, explicit: ReadSignal<Option<String>>) -> Option<String> {
        explicit().or_else(|| self.text_fallback.map(|value| value()))
    }

    /// The attributes every host's checkable item owns, to be merged *after*
    /// the caller's own so they win (`dev-docs/backlog.md` row 93's
    /// duplicate-attribute hazard).
    ///
    /// `aria-checked` is always present: WAI-ARIA 1.2 makes it a required
    /// state on both roles, and an absent value reads as "not a checkbox".
    pub(crate) fn owned_attributes(
        &self,
        disabled: bool,
        inset: bool,
        tabindex: &'static str,
    ) -> Vec<Attribute> {
        let checked = self.checked();
        merge_attributes(vec![
            attributes!(div {
                role: self.kind.role(),
                aria_checked: if checked { "true" } else { "false" },
                "data-state": if checked { "checked" } else { "unchecked" },
                aria_disabled: disabled,
                "data-disabled": disabled,
                tabindex: tabindex,
            }),
            item_style_attributes(MenuItemVariant::Default, inset),
        ])
    }
}

/// The inputs every host's checkable item shares, whichever role it has --
/// what is left of a [`MenuCheckboxItemProps`]/[`MenuRadioItemProps`] once
/// the state half has been turned into a [`CheckableState`].
pub(crate) struct CheckableCommon {
    pub(crate) index: ReadSignal<Option<usize>>,
    pub(crate) disabled: ReadSignal<bool>,
    pub(crate) text_value: ReadSignal<Option<String>>,
    pub(crate) close_on_select: ReadSignal<bool>,
    pub(crate) inset: bool,
    pub(crate) attributes: Vec<Attribute>,
    pub(crate) children: Element,
}

impl From<MenuCheckboxItemProps> for CheckableCommon {
    fn from(props: MenuCheckboxItemProps) -> Self {
        Self {
            index: props.index,
            disabled: props.disabled,
            text_value: props.text_value,
            close_on_select: props.close_on_select,
            inset: props.inset,
            attributes: props.attributes,
            children: props.children,
        }
    }
}

impl From<MenuRadioItemProps> for CheckableCommon {
    fn from(props: MenuRadioItemProps) -> Self {
        Self {
            index: props.index,
            disabled: props.disabled,
            text_value: props.text_value,
            close_on_select: props.close_on_select,
            inset: props.inset,
            attributes: props.attributes,
            children: props.children,
        }
    }
}

/// The state of a checkbox item: controlled when `checked` is `Some`, else
/// internal, seeded from `default_checked`. Activating flips it and fires
/// `on_checked_change`. A hook -- call it unconditionally, once per render.
pub(crate) fn use_checkbox_state(
    checked: ReadSignal<Option<bool>>,
    default_checked: bool,
    on_checked_change: Callback<bool>,
) -> CheckableState {
    let (value, set_value) = use_controlled(checked, default_checked, on_checked_change);
    let activate = use_callback(move |()| set_value.call(!value()));
    CheckableState {
        kind: CheckableKind::Checkbox,
        checked: value,
        activate,
        text_fallback: None,
    }
}

#[derive(Clone, Copy)]
struct RadioGroupContext {
    value: Memo<String>,
    set_value: Callback<String>,
}

/// Provide a radio group's value to its [`use_radio_state`] items:
/// controlled when `value` is `Some`, else internal, seeded from
/// `default_value`. A hook -- call it unconditionally, once per render.
pub(crate) fn use_radio_group(
    value: ReadSignal<Option<String>>,
    default_value: String,
    on_value_change: Callback<String>,
) {
    let (value, set_value) = use_controlled(value, default_value, on_value_change);
    use_context_provider(|| RadioGroupContext { value, set_value });
}

/// The state of a radio item: checked while the enclosing group's value
/// equals this item's `value`; activating sets the group's value to it.
/// A hook -- call it unconditionally, once per render.
///
/// # Panics
///
/// Panics if there is no enclosing radio group, as every other context-bound
/// item in this crate does outside its parent.
pub(crate) fn use_radio_state(value: ReadSignal<String>) -> CheckableState {
    let group: RadioGroupContext = use_context();
    let checked = use_memo(move || (group.value)() == value());
    let activate = use_callback(move |()| group.set_value.call(value()));
    CheckableState {
        kind: CheckableKind::Radio,
        checked,
        activate,
        text_fallback: Some(value),
    }
}

/// The radio group element, shared by every host's `*RadioGroup`: a
/// `role="group"` (`crate::menu_semantics::MENU_GROUP_ROLE`) whose radio
/// items read the group's value. Call it from a host's component body; it
/// calls hooks.
pub(crate) fn render_radio_group(props: MenuRadioGroupProps) -> Element {
    use_radio_group(props.value, props.default_value, props.on_value_change);

    // Owned by this component -- see `CheckableState::owned_attributes`.
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(div {
            role: crate::menu_semantics::MENU_GROUP_ROLE,
        }),
    ]);

    rsx! {
        div { ..attributes, {props.children} }
    }
}

/// The separator element, shared by every host's `*Separator`: a
/// `role="separator"` with `aria-orientation="horizontal"` between groups of
/// items. Not focusable and not a roving-focus target.
pub(crate) fn render_separator(props: MenuSeparatorProps) -> Element {
    // Owned by this component -- see `CheckableState::owned_attributes`.
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(div {
            role: crate::menu_semantics::MENU_SEPARATOR_ROLE,
            aria_orientation: "horizontal",
        }),
    ]);

    rsx! {
        div { ..attributes }
    }
}

/// The group element, shared by every host's `*Group`: a `role="group"`
/// around related items. Presentation only -- it adds no behaviour, and its
/// items still register in the enclosing menu's roving-focus collection.
pub(crate) fn render_group(props: MenuGroupProps) -> Element {
    // Owned by this component -- see `CheckableState::owned_attributes`.
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(div {
            role: crate::menu_semantics::MENU_GROUP_ROLE,
        }),
    ]);

    rsx! {
        div { ..attributes, {props.children} }
    }
}

/// The shortcut hint, shared by every host's `*Shortcut`: a `span` that sits
/// at the inline end of an item's row. Text only; it does not bind any key.
pub(crate) fn render_shortcut(props: MenuShortcutProps) -> Element {
    rsx! {
        span { ..props.attributes, {props.children} }
    }
}

/// Whether `key` activates a menu item: Enter or Space. The one definition
/// every checkable item's `onkeydown` reads.
pub(crate) fn is_activation_key(key: &Key) -> bool {
    matches!(key, Key::Enter) || matches!(key, Key::Character(c) if c == " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_menu::{
        ContextMenu, ContextMenuCheckboxItem, ContextMenuGroup, ContextMenuItem,
        ContextMenuRadioGroup, ContextMenuRadioItem, ContextMenuSeparator, ContextMenuShortcut,
        ContextMenuTrigger,
    };
    use crate::dropdown_menu::{
        DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuGroup, DropdownMenuItem,
        DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuSeparator, DropdownMenuShortcut,
        DropdownMenuTrigger,
    };
    use crate::menubar::{
        Menubar, MenubarCheckboxItem, MenubarGroup, MenubarItem, MenubarMenu, MenubarRadioGroup,
        MenubarRadioItem, MenubarSeparator, MenubarShortcut, MenubarTrigger,
    };

    fn render(component: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(component);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    /// One menu per host: a plain item, a checked and an unchecked checkbox
    /// item, and a radio group of two with `b` chosen. The items sit directly
    /// under the host root rather than inside its `*Content`, which mounts
    /// only after an effect has flipped it open -- the items themselves only
    /// need the root's context, and that is what is under test.
    #[component]
    fn DropdownMenuWithCheckables() -> Element {
        rsx! {
            DropdownMenu {
                DropdownMenuTrigger { "t" }
                DropdownMenuItem::<String> { value: "plain".to_string(), "Plain" }
                DropdownMenuCheckboxItem { default_checked: true, "On" }
                DropdownMenuCheckboxItem { "Off" }
                DropdownMenuRadioGroup { default_value: "b".to_string(), aria_label: "Pick",
                    DropdownMenuRadioItem { value: "a".to_string(), "A" }
                    DropdownMenuRadioItem { value: "b".to_string(), "B" }
                }
            }
        }
    }

    #[component]
    fn ContextMenuWithCheckables() -> Element {
        rsx! {
            ContextMenu {
                ContextMenuTrigger { "t" }
                ContextMenuItem { value: "plain".to_string(), "Plain" }
                ContextMenuCheckboxItem { default_checked: true, "On" }
                ContextMenuCheckboxItem { "Off" }
                ContextMenuRadioGroup { default_value: "b".to_string(), aria_label: "Pick",
                    ContextMenuRadioItem { value: "a".to_string(), "A" }
                    ContextMenuRadioItem { value: "b".to_string(), "B" }
                }
            }
        }
    }

    #[component]
    fn MenubarWithCheckables() -> Element {
        rsx! {
            Menubar {
                MenubarMenu {
                    MenubarTrigger { "t" }
                    MenubarItem { value: "plain".to_string(), "Plain" }
                    MenubarCheckboxItem { default_checked: true, "On" }
                    MenubarCheckboxItem { "Off" }
                    MenubarRadioGroup { default_value: "b".to_string(), aria_label: "Pick",
                        MenubarRadioItem { value: "a".to_string(), "A" }
                        MenubarRadioItem { value: "b".to_string(), "B" }
                    }
                }
            }
        }
    }

    /// The contract every host must render for the menu above -- asserted
    /// against all of them with this one function, so a host that drifts
    /// (a hand-written role, a missing `aria-checked`) fails here.
    ///
    /// `plain_menuitems` is how many `role="menuitem"` the menu legitimately
    /// has: the one plain item, plus the trigger where the host's trigger is
    /// itself a menuitem (`Menubar`'s is).
    fn assert_checkable_contract(host: &str, html: &str, plain_menuitems: usize) {
        let count = |needle: &str| html.matches(needle).count();
        assert_eq!(count(r#"role="menuitemcheckbox""#), 2, "{host}: {html}");
        assert_eq!(count(r#"role="menuitemradio""#), 2, "{host}: {html}");
        assert_eq!(count(r#"role="group""#), 1, "{host}: {html}");
        // The plain item stays a plain menuitem.
        assert_eq!(
            count(r#"role="menuitem""#),
            plain_menuitems,
            "{host}: {html}"
        );
        // `aria-checked` is on every checkable item and only on those:
        // On + radio `b` are true, Off + radio `a` are false.
        assert_eq!(count(r#"aria-checked="true""#), 2, "{host}: {html}");
        assert_eq!(count(r#"aria-checked="false""#), 2, "{host}: {html}");
        assert_eq!(count("aria-checked="), 4, "{host}: {html}");
        assert_eq!(count(r#"data-state="checked""#), 2, "{host}: {html}");
        assert_eq!(count(r#"data-state="unchecked""#), 2, "{host}: {html}");
    }

    #[test]
    fn dropdown_menu_renders_the_checkable_contract() {
        assert_checkable_contract("DropdownMenu", &render(DropdownMenuWithCheckables), 1);
    }

    #[test]
    fn context_menu_renders_the_checkable_contract() {
        assert_checkable_contract("ContextMenu", &render(ContextMenuWithCheckables), 1);
    }

    #[test]
    fn menubar_renders_the_checkable_contract() {
        assert_checkable_contract("Menubar", &render(MenubarWithCheckables), 2);
    }

    /// A caller cannot override the primitive-owned role or `aria-checked`:
    /// both are merged after the caller's attributes.
    #[component]
    fn CheckboxTryingToOverrideItsRole() -> Element {
        rsx! {
            DropdownMenu {
                DropdownMenuCheckboxItem {
                    role: "menuitem",
                    aria_checked: "mixed",
                    "x"
                }
            }
        }
    }

    #[test]
    fn the_role_and_aria_checked_are_owned_by_the_primitive() {
        let html = render(CheckboxTryingToOverrideItsRole);
        assert_eq!(html.matches("role=").count(), 1, "{html}");
        assert!(html.contains(r#"role="menuitemcheckbox""#), "{html}");
        assert_eq!(html.matches("aria-checked=").count(), 1, "{html}");
        assert!(html.contains(r#"aria-checked="false""#), "{html}");
    }

    #[test]
    fn enter_and_space_activate_and_nothing_else_does() {
        assert!(is_activation_key(&Key::Enter));
        assert!(is_activation_key(&Key::Character(" ".to_string())));
        assert!(!is_activation_key(&Key::Character("a".to_string())));
        assert!(!is_activation_key(&Key::Escape));
        assert!(!is_activation_key(&Key::ArrowDown));
    }

    #[test]
    fn the_two_checkable_kinds_have_distinct_roles() {
        assert_eq!(CheckableKind::Checkbox.role(), "menuitemcheckbox");
        assert_eq!(CheckableKind::Radio.role(), "menuitemradio");
    }

    // ---- item parts: variant, inset, separator, group, shortcut ----

    /// One menu per host holding every part: a plain default item, a
    /// destructive inset item, an inset checkbox item, a separator and a
    /// group around a shortcut. Same direct-under-the-root shape as the
    /// checkable menus above.
    #[component]
    fn DropdownMenuWithParts() -> Element {
        rsx! {
            DropdownMenu {
                DropdownMenuTrigger { "t" }
                DropdownMenuItem::<String> { value: "plain".to_string(), "Plain" }
                DropdownMenuItem::<String> {
                    value: "del".to_string(),
                    variant: MenuItemVariant::Destructive,
                    inset: true,
                    "Delete"
                    DropdownMenuShortcut { "⌫" }
                }
                DropdownMenuCheckboxItem { inset: true, "On" }
                DropdownMenuSeparator {}
                DropdownMenuGroup { aria_label: "G", "x" }
            }
        }
    }

    #[component]
    fn ContextMenuWithParts() -> Element {
        rsx! {
            ContextMenu {
                ContextMenuTrigger { "t" }
                ContextMenuItem { value: "plain".to_string(), "Plain" }
                ContextMenuItem {
                    value: "del".to_string(),
                    variant: MenuItemVariant::Destructive,
                    inset: true,
                    "Delete"
                    ContextMenuShortcut { "⌫" }
                }
                ContextMenuCheckboxItem { inset: true, "On" }
                ContextMenuSeparator {}
                ContextMenuGroup { aria_label: "G", "x" }
            }
        }
    }

    #[component]
    fn MenubarWithParts() -> Element {
        rsx! {
            Menubar {
                MenubarMenu {
                    MenubarTrigger { "t" }
                    MenubarItem { value: "plain".to_string(), "Plain" }
                    MenubarItem {
                        value: "del".to_string(),
                        variant: MenuItemVariant::Destructive,
                        inset: true,
                        "Delete"
                        MenubarShortcut { "⌫" }
                    }
                    MenubarCheckboxItem { inset: true, "On" }
                    MenubarSeparator {}
                    MenubarGroup { aria_label: "G", "x" }
                }
            }
        }
    }

    /// The contract every host must render for the menu above, asserted with
    /// this one function so a host that drifts fails here.
    fn assert_parts_contract(host: &str, html: &str) {
        let count = |needle: &str| html.matches(needle).count();
        // The separator: a labelled-by-role divider, horizontal.
        assert_eq!(count(r#"role="separator""#), 1, "{host}: {html}");
        assert_eq!(
            count(r#"aria-orientation="horizontal""#),
            1,
            "{host}: {html}"
        );
        // The group.
        assert_eq!(count(r#"role="group""#), 1, "{host}: {html}");
        assert_eq!(count(r#"aria-label="G""#), 1, "{host}: {html}");
        // `data-variant` on every item (plain, destructive, checkbox); only
        // the one asked for is destructive.
        assert_eq!(count(r#"data-variant="destructive""#), 1, "{host}: {html}");
        assert_eq!(count(r#"data-variant="default""#), 2, "{host}: {html}");
        // `data-inset` only where asked, on the plain and checkbox items.
        assert_eq!(count(r#"data-inset="true""#), 2, "{host}: {html}");
        // A destructive item is still a plain menuitem.
        assert!(count(r#"role="menuitem""#) >= 2, "{host}: {html}");
        // The shortcut is a span inside the item.
        assert_eq!(count("<span"), 1, "{host}: {html}");
    }

    #[test]
    fn dropdown_menu_renders_the_item_parts() {
        assert_parts_contract("DropdownMenu", &render(DropdownMenuWithParts));
    }

    #[test]
    fn context_menu_renders_the_item_parts() {
        assert_parts_contract("ContextMenu", &render(ContextMenuWithParts));
    }

    #[test]
    fn menubar_renders_the_item_parts() {
        assert_parts_contract("Menubar", &render(MenubarWithParts));
    }

    /// A caller cannot override the primitive-owned `data-variant`, and a
    /// separator's role is owned too.
    #[component]
    fn PartsTryingToOverride() -> Element {
        rsx! {
            DropdownMenu {
                DropdownMenuItem::<String> {
                    value: "x".to_string(),
                    variant: MenuItemVariant::Destructive,
                    "data-variant": "default",
                    "x"
                }
                DropdownMenuSeparator { role: "presentation" }
            }
        }
    }

    #[test]
    fn variant_and_separator_role_are_owned_by_the_primitive() {
        let html = render(PartsTryingToOverride);
        assert_eq!(html.matches("data-variant=").count(), 1, "{html}");
        assert!(html.contains(r#"data-variant="destructive""#), "{html}");
        assert_eq!(html.matches("role=\"separator\"").count(), 1, "{html}");
        assert!(!html.contains("presentation"), "{html}");
    }

    #[test]
    fn item_style_attributes_write_inset_only_when_asked() {
        let names = |attributes: &[Attribute]| -> Vec<&'static str> {
            attributes.iter().map(|a| a.name).collect()
        };
        assert_eq!(
            names(&item_style_attributes(MenuItemVariant::Default, false)),
            ["data-variant"]
        );
        assert_eq!(
            names(&item_style_attributes(MenuItemVariant::Destructive, true)),
            ["data-variant", "data-inset"]
        );
        assert_eq!(MenuItemVariant::default(), MenuItemVariant::Default);
        assert_eq!(MenuItemVariant::Destructive.as_str(), "destructive");
    }
}
