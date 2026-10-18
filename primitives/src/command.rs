//! Command palette: a filterable, always-visible list of actions, typically
//! shown inside a modal dialog (`CommandDialog`).
//!
//! This module is **original composition against this repo's own house
//! infrastructure**, not a port of either reference fork below. It builds
//! on exactly the pieces `docs/component-backlog.md`'s "Command palette --
//! plan (2026-09-03)" section names:
//!
//! - **`Dialog`** (`dialog.rs`) -- [`CommandDialog`] is a thin wrapper over
//!   [`crate::dialog::DialogRoot`] + [`crate::dialog::DialogContent`],
//!   unchanged, reusing its native `<dialog>`/`showModal()` modal machinery
//!   (focus trap, restore, inertness, top layer) as-is.
//! - **`Combobox`** (`combobox/`) -- not the `Combobox`/`ComboboxList`
//!   *components* (see "Why a sibling context" below), but the same
//!   underlying, already-shared infrastructure `Combobox` itself is built
//!   on: `crate::selectable` (`use_selectable_root`, `use_selectable_option`,
//!   `SelectableContext`, the pointer-select helpers), `crate::listbox`
//!   (`use_listbox_id`), and `crate::collection` (`collection_item`,
//!   `use_item`). [`Command`]/[`CommandItem`] compose these hooks directly,
//!   the same way `combobox/components/combobox.rs`/`option.rs` do, and
//!   [`CommandInput`]'s keyboard handling (Arrow/Home/End/Enter) mirrors
//!   `ComboboxInput`'s, minus the parts that only make sense for a popover
//!   (see below). The default filter is literally
//!   [`crate::combobox::default_combobox_filter`], reused, not
//!   reimplemented.
//! - **`SelectGroup`/`SelectGroupLabel`** (`select/components/group.rs`) --
//!   [`CommandGroup`]/[`CommandGroupLabel`] port that exact shape: a
//!   `role="group"` element plus a mount-time `use_effect` in the label that
//!   sets a context signal the group reads into `aria-labelledby`.
//!
//! ## Why a sibling context, not `combobox::ComboboxContext`
//!
//! The plan's open design question recommended a sibling module over a new
//! `Combobox`/`ComboboxList` display mode, and left one detail to confirm by
//! execution: whether `Command` could reuse `ComboboxContext` itself.
//! Reading `combobox/context.rs` settles it -- `ComboboxContext` is
//! `pub(super)`, private to the `combobox` module tree, so no sibling module
//! can name it at all. What *is* reachable from anywhere in this crate is
//! the lower layer `ComboboxContext` is itself built from --
//! `crate::selectable`/`crate::listbox`/`crate::collection` are private
//! modules whose items are `pub(crate)`, i.e. visible crate-wide. `Command`
//! therefore defines its own small `CommandContext`, built on that same
//! `pub(crate)` layer, rather than either vendoring `ComboboxContext`'s
//! private shape or modifying `combobox/` to export it -- consistent with
//! this module's other files being off limits to edit.
//!
//! `CommandContext` ends up smaller than `ComboboxContext`, not just
//! differently-scoped: the plan predicted "`Command` mostly needs *less* of
//! what `Combobox` does, not more," and reading `ComboboxContext` end to end
//! confirms exactly which pieces drop out --
//!
//! - No `input_id`: that field exists solely so `ComboboxList`'s web arm can
//!   anchor-position itself against `ComboboxInput` once promoted to the top
//!   layer (`top_layer::anchor_name_style`). `CommandList` is never promoted
//!   anywhere -- it is always an in-flow child of `Command`, the same shape
//!   `ComboboxListRendered`'s `#[cfg(not(feature = "web"))]` arm already
//!   renders unconditionally today -- so there is no anchor to bind.
//! - No `set_open`/`open_with_empty_query_and_focus_first`/`_last`: a
//!   command palette's list has no open/closed state of its own to toggle --
//!   the surrounding `CommandDialog` (or whatever the caller composes
//!   `Command` into) owns visibility. `CommandContext` still carries a
//!   `SelectableContext` (so it can reuse `use_selectable_root`/
//!   `use_selectable_option`/the collection's roving focus unmodified), but
//!   wires its `open` to a `Controlled<bool>` whose *value* signal is a
//!   constant `Some(true)` -- `use_controlled` then always resolves `open()`
//!   to `true` regardless of any `set_open` call `SelectableContext`'s own
//!   methods (e.g. `select_value`'s closes-on-select path) might still make;
//!   there is simply nothing downstream that ever reads `open` as a gate on
//!   rendering, the way `ComboboxList`'s `use_listbox_container` does.
//!
//! ## References -- read-never-vendor
//!
//! Per `docs/lifting-from-forks.md` §1 and the plan doc's own "References"
//! section, both of the below were read for shape only. Nothing from either
//! was copied; no provenance header applies.
//!
//! - `dignifiedquire/dx-components`'s `command.rs` -- entangled with that
//!   fork's own `popper.rs`/`focus_scope.rs`, which this repo doesn't have.
//! - `rust-ui/dioxus-ui`'s `command.rs` -- filtering/`aria-selected` are
//!   driven by injected vanilla-JS strings outside Dioxus's reactive model;
//!   explicitly "behaviour not to be taken" per the plan doc. Its
//!   `Command`/`CommandGroup`/`CommandItem`/shortcut-slot naming convention
//!   is the one thing carried over (naming, not code).
//!
//! ## Scope
//!
//! [`CommandItem`]'s `shortcut` slot is presentation-only: it renders
//! whatever element the caller passes (typically a themed `Kbd`/`KbdGroup`
//! pair) with no keyboard binding of its own. A global keyboard-shortcut
//! dispatcher that would actually fire the action on that key combo is a
//! separate, unscoped feature -- not built here, per the plan doc.

use dioxus::prelude::*;
use dioxus_attributes::attributes;

use crate::{
    collection::{collection_item, use_item},
    combobox::default_combobox_filter,
    listbox::use_listbox_id,
    merge_attributes,
    selectable::{
        pointer_select_cancel, pointer_select_commit, pointer_select_start, use_selectable_option,
        use_selectable_root, use_single_selectable_value, OptionState, RcPartialEqValue,
        SelectableContext, SelectableOptionConfig, SelectionMode,
    },
    use_controlled, use_id_or, use_unique_id, Controlled,
};

/// Marker provided by [`CommandDialog`]: its presence tells [`CommandInput`] it is inside a
/// modal dialog and should claim focus by default.
#[derive(Clone, Copy)]
struct CommandDialogContext;

/// Shared state for the `Command` family. See the module doc for why this is
/// a sibling of, not a reuse of, `combobox::ComboboxContext`.
#[derive(Clone, Copy)]
struct CommandContext {
    selectable: SelectableContext,
    query: Memo<String>,
    set_query: Callback<String>,
    filter: Callback<(String, String), bool>,
    /// Flipped to `true` by an effect on the root, i.e. never during SSG/JS-off rendering and not
    /// before the first client render has committed. Items register themselves in
    /// `selectable.options` from effects too, so until this is set "not registered" means "not
    /// registered yet", not "filtered out".
    registered: Signal<bool>,
}

impl CommandContext {
    fn predicate_for(&self, query: String) -> impl Fn(&OptionState) -> bool {
        let filter = self.filter;
        move |option: &OptionState| filter.call((query.clone(), option.text_value.clone()))
    }

    fn predicate(&self) -> impl Fn(&OptionState) -> bool {
        self.predicate_for(self.query.cloned())
    }

    /// An option that has not registered yet counts as visible, so the static (SSG / JS-off)
    /// render and the hydration render both contain every item rather than an empty list. Once
    /// the root's `registered` flag is set, an option that is absent is filtered out as before.
    fn is_visible(&self, tab_index: usize) -> bool {
        let predicate = self.predicate();
        match self
            .selectable
            .options
            .read()
            .iter()
            .find(|option| option.index == tab_index)
        {
            Some(option) => predicate(option),
            None => !(self.registered)(),
        }
    }

    /// Whether [`CommandEmpty`] must stay hidden: either something matches, or registration has
    /// not happened yet so "nothing matches" cannot be known (the static render shows the items,
    /// never the empty message).
    fn has_visible_options(&self) -> bool {
        !(self.registered)() || self.selectable.options.read().iter().any(self.predicate())
    }

    fn focused_option_id(&self) -> Option<String> {
        self.selectable.focused_option_id()
    }

    fn focus_next_visible(&mut self) {
        self.selectable.focus_next_where(self.predicate());
    }

    fn focus_prev_visible(&mut self) {
        self.selectable.focus_prev_where(self.predicate());
    }

    fn focus_first_visible(&mut self) {
        self.selectable.focus_first_where(self.predicate());
    }

    fn focus_last_visible(&mut self) {
        self.selectable.focus_last_where(self.predicate());
    }

    fn select_focused(&mut self) {
        self.selectable.select_focused();
    }
}

/// Props for [`Command`].
#[derive(Props, Clone, PartialEq)]
pub struct CommandProps<T: Clone + PartialEq + 'static = String> {
    /// The controlled value. If supplied, `Command` is controlled and the
    /// signal's `None` value means no item is selected.
    #[props(default)]
    pub value: Option<ReadSignal<Option<T>>>,

    /// The default uncontrolled value.
    #[props(default)]
    pub default_value: Option<T>,

    /// Callback fired when the value changes (i.e. an item is chosen).
    #[props(default)]
    pub on_value_change: Callback<Option<T>>,

    /// Whether the whole command palette is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// The controlled text query used to filter items.
    #[props(default)]
    pub query: ReadSignal<Option<String>>,

    /// The initial text query when uncontrolled.
    #[props(default)]
    pub default_query: ReadSignal<String>,

    /// Callback fired when the text query changes.
    #[props(default)]
    pub on_query_change: Callback<String>,

    /// Whether arrow-key navigation should wrap.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// Custom filter callback. Receives `(query, item_text_value)`. Defaults
    /// to [`crate::combobox::default_combobox_filter`] -- the same
    /// case-insensitive substring match `Combobox` uses by default, reused
    /// rather than reimplemented.
    #[props(default = Callback::new(|(q, t): (String, String)| default_combobox_filter(&q, &t)))]
    pub filter: Callback<(String, String), bool>,

    /// Additional attributes for the root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Children, typically [`CommandInput`] followed by [`CommandList`].
    pub children: Element,
}

/// Sets up [`CommandContext`]. `open` is wired to a constant `true` -- see
/// the module doc's "Why a sibling context" section for why `Command`'s
/// list has no open/closed state of its own to track.
fn use_command_root<T: Clone + PartialEq + 'static>(
    props: &CommandProps<T>,
) -> Memo<Vec<RcPartialEqValue>> {
    let (selected, set_value, _reset_to_default) = use_single_selectable_value(
        props.value,
        props.default_value.clone(),
        props.on_value_change,
        "command",
    );

    let always_open = Controlled {
        value: ReadSignal::new(Signal::new(Some(true))),
        default: ReadSignal::new(Signal::new(true)),
        on_change: Callback::default(),
    };
    let selectable = use_selectable_root(
        selected,
        set_value,
        SelectionMode::Single,
        props.disabled,
        props.roving_loop,
        always_open,
    );
    let (query, set_query) = use_controlled(
        props.query,
        props.default_query.cloned(),
        props.on_query_change,
    );

    let mut registered = use_signal(|| false);
    // Deferred one task past the effect batch: every `CommandItem` registers from its own effect
    // in the same batch, and flipping this flag before they have would render one frame in which
    // an item is "absent" (hidden) and `CommandEmpty` is shown.
    use_effect(move || {
        spawn(async move {
            registered.set(true);
        });
    });

    use_context_provider(|| CommandContext {
        selectable,
        query,
        set_query,
        filter: props.filter,
        registered,
    });

    selected
}

/// A filterable, always-visible list of actions. Typically placed inside a
/// [`CommandDialog`], but usable on its own (e.g. inline in a sidebar).
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::command::{Command, CommandEmpty, CommandInput, CommandItem, CommandList};
///
/// #[component]
/// fn Demo() -> Element {
///     let mut query = use_signal(String::new);
///
///     rsx! {
///         Command::<String> {
///             query: Some(query()),
///             on_query_change: move |next| query.set(next),
///             CommandInput { aria_label: "Search commands" }
///             CommandList { aria_label: "Commands",
///                 CommandEmpty { "No results found." }
///                 CommandItem::<String> {
///                     index: 0usize,
///                     value: "new-file".to_string(),
///                     text_value: "New File",
///                     "New File"
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn Command<T: Clone + PartialEq + 'static>(props: CommandProps<T>) -> Element {
    use_command_root(&props);

    // Owned by this component -- disabled state must win over a caller's
    // own attributes (`docs/backlog.md` row 93's duplicate-attribute
    // hazard).
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(div {
            "data-disabled": (props.disabled)(),
        }),
    ]);

    rsx! {
        div { ..attributes, {props.children} }
    }
}

/// Props for [`CommandInput`].
#[derive(Props, Clone, PartialEq)]
pub struct CommandInputProps {
    /// Placeholder shown when the input is empty.
    #[props(default)]
    pub placeholder: ReadSignal<String>,

    /// Optional id for the input element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Accessible name for the input.
    #[props(default)]
    pub aria_label: Option<String>,

    /// Whether the input takes focus when it appears. `None` (the default) is dialog-aware:
    /// `true` inside a [`CommandDialog`], where the open policy gives it focus for a keyboard or
    /// mouse open and leaves the panel focused for touch (see [`CommandInput`]), and `false`
    /// for a `Command` placed inline in a page, which must not steal focus (or scroll) on load.
    /// `Some(_)` overrides that either way.
    #[props(default)]
    pub autofocus: Option<bool>,

    /// Called when the input element is mounted. `CommandInput` also uses
    /// this internally to focus itself once mounted when no dialog driver
    /// does (see [`CommandInput`]). A
    /// caller-supplied callback here still fires, chained after that
    /// internal behavior -- the same forwarding shape `Toggle`'s own
    /// `onmounted` prop already uses (`toggle.rs`) so a caller doesn't lose
    /// either half.
    #[props(default)]
    pub onmounted: Callback<Event<MountedData>>,

    /// Additional attributes.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The text input that filters [`CommandList`]'s items.
///
/// ## Focus on open (backlog rows 206 and 226)
///
/// Inside a [`CommandDialog`] on the web arm the input carries `data-dx-autofocus` (not the
/// `autofocus` attribute), the marker the shared open-focus policy
/// ([`crate::modality::OPEN_FOCUS_JS`]) reads when the dialog opens: opened by keyboard or mouse
/// the search box takes focus (a palette's search box is its point); opened by touch or pen the
/// panel does, so the software keyboard does not rise while the dialog is still being placed and
/// the user taps the field to type. The policy focuses the element itself, so nothing here races
/// `showModal()`'s own focusing step.
///
/// Without a `<dialog>` to open (an inline `Command` with an explicit `autofocus: Some(true)`, or
/// the non-web arms) the input renders the plain `autofocus` attribute and focuses itself once
/// mounted, as before. An inline `Command` that grabbed focus on mount would pull the page's focus
/// (and scroll) to itself on every load, so `None` is `false` there.
#[component]
pub fn CommandInput(props: CommandInputProps) -> Element {
    let mut ctx = use_context::<CommandContext>();

    let id = use_unique_id();
    let id = use_id_or(id, props.id);

    let query = ctx.query;
    let set_query = ctx.set_query;
    let user_onmounted = props.onmounted;
    // Dialog-aware default: only a `CommandDialog` ancestor needs the input to claim focus.
    let in_dialog = try_use_context::<CommandDialogContext>().is_some();
    let autofocus = props.autofocus.unwrap_or(in_dialog);
    // On the web arm a dialog's open driver chooses the initial focus from the opening modality
    // (`data-dx-autofocus`); everywhere else the input focuses itself the old way.
    let driver_autofocus = autofocus && in_dialog && cfg!(feature = "web");
    let self_autofocus = autofocus && !driver_autofocus;

    let active_descendant = use_memo(move || ctx.focused_option_id());

    let onkeydown = move |event: KeyboardEvent| match event.key() {
        Key::ArrowDown => {
            ctx.focus_next_visible();
            event.prevent_default();
            event.stop_propagation();
        }
        Key::ArrowUp => {
            ctx.focus_prev_visible();
            event.prevent_default();
            event.stop_propagation();
        }
        Key::Home => {
            ctx.focus_first_visible();
            event.prevent_default();
            event.stop_propagation();
        }
        Key::End => {
            ctx.focus_last_visible();
            event.prevent_default();
            event.stop_propagation();
        }
        Key::Enter => {
            ctx.select_focused();
            event.prevent_default();
            event.stop_propagation();
        }
        // No Escape handling here, unlike `ComboboxInput`: `Command` has no
        // popover of its own to dismiss. When used inside `CommandDialog`,
        // Escape is left to bubble to the native `<dialog>`'s own
        // `cancel`/`close` handling (`dialog.rs`) instead.
        _ => {}
    };

    // Owned by this component -- input semantics/aria wiring/state must
    // win over a caller's own attributes (`docs/backlog.md` row 93's
    // duplicate-attribute hazard).
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(input {
            r#type: "text",
            value: query.cloned(),
            placeholder: props.placeholder,
            autocomplete: "off",
            spellcheck: "false",
            // A filter, not a name: iOS must not capitalise the first letter of the query.
            autocapitalize: "off",
            disabled: (ctx.selectable.disabled)(),
            // focus-ok: no dialog driver to choose the focus (inline Command, explicit prop)
            autofocus: self_autofocus,
            "data-dx-autofocus": driver_autofocus.then_some("true"),
            role: "combobox",
            aria_autocomplete: "list",
            aria_expanded: "true",
            aria_controls: ctx.selectable.list_id,
            aria_activedescendant: active_descendant(),
        }),
    ]);

    rsx! {
        input {
            id,
            aria_label: props.aria_label.clone(),

            onmounted: move |evt: Event<MountedData>| {
                // Backstop for the `autofocus` attribute when no dialog driver chooses the focus.
                let data = evt.data();
                if self_autofocus {
                    spawn(async move {
                        // focus-ok: explicit `autofocus` on an inline Command or a non-web arm
                        let _ = data.set_focus(true).await;
                    });
                }
                user_onmounted.call(evt);
            },
            oninput: move |event| {
                set_query.call(event.value());
                ctx.selectable.collection.clear_focus();
            },
            onkeydown,

            ..attributes,
        }
    }
}

/// Props for [`CommandList`].
#[derive(Props, Clone, PartialEq)]
pub struct CommandListProps {
    /// Optional id for the list element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Accessible name for the list.
    #[props(default)]
    pub aria_label: Option<String>,

    /// Additional attributes.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Children, typically [`CommandGroup`]s/[`CommandItem`]s and an
    /// optional [`CommandEmpty`].
    pub children: Element,
}

/// The always-visible listbox of [`CommandItem`]s. Unlike
/// `combobox::ComboboxList`, this never promotes itself to the top layer or
/// gates on an open/closed animation -- it is a plain, always in-flow
/// `div[role="listbox"]`, the same shape `ComboboxListRendered`'s
/// `#[cfg(not(feature = "web"))]` arm already renders unconditionally. See
/// the module doc's "Why a sibling context" section.
#[component]
pub fn CommandList(props: CommandListProps) -> Element {
    let ctx = use_context::<CommandContext>();
    let id = use_listbox_id(props.id, ctx.selectable.list_id);

    // Owned by this component -- role must win over a caller's own
    // attributes (`docs/backlog.md` row 93's duplicate-attribute hazard).
    let attributes = merge_attributes(vec![props.attributes, attributes!(div { role: "listbox" })]);

    rsx! {
        div {
            id,
            aria_label: props.aria_label.clone(),
            ..attributes,
            {props.children}
        }
    }
}

/// Props for [`CommandEmpty`].
#[derive(Props, Clone, PartialEq)]
pub struct CommandEmptyProps {
    /// Additional attributes.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Children rendered when no item matches the current query.
    pub children: Element,
}

/// Renders when no [`CommandItem`] matches the current query. Mirrors
/// `combobox::ComboboxEmpty`'s shape, minus the render-gate `ComboboxEmpty`
/// takes from `ListboxContext` -- `CommandList` has no such gate to read.
#[component]
pub fn CommandEmpty(props: CommandEmptyProps) -> Element {
    let ctx = use_context::<CommandContext>();
    let any_visible = use_memo(move || ctx.has_visible_options());

    if any_visible() {
        return rsx! {};
    }

    // Owned by this component -- role must win over a caller's own
    // attributes (`docs/backlog.md` row 93's duplicate-attribute hazard).
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(div {
            role: "presentation"
        }),
    ]);

    rsx! {
        div { ..attributes, {props.children} }
    }
}

/// Context for [`CommandGroupLabel`] to publish its id into, read back by
/// [`CommandGroup`]'s `aria-labelledby`. Ported from
/// `select::SelectGroupContext`'s identical shape.
#[derive(Clone, Copy)]
struct CommandGroupContext {
    labeled_by: Signal<Option<String>>,
}

/// Props for [`CommandGroup`].
#[derive(Props, Clone, PartialEq)]
pub struct CommandGroupProps {
    /// Whether the group is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Optional id for the group element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Children, typically a [`CommandGroupLabel`] followed by
    /// [`CommandItem`]s.
    pub children: Element,
}

/// Groups related [`CommandItem`]s inside a [`CommandList`]. Ported from
/// `select::SelectGroup`'s exact shape (`role="group"` +
/// `aria-labelledby`), minus the `render`/`ListboxContext` gate `SelectGroup`
/// reads -- `CommandList` is always rendered, so there is nothing to gate on.
#[component]
pub fn CommandGroup(props: CommandGroupProps) -> Element {
    let ctx = use_context::<CommandContext>();
    let disabled = ctx.selectable.disabled.cloned() || props.disabled.cloned();

    let labeled_by = use_signal(|| None);
    use_context_provider(|| CommandGroupContext { labeled_by });

    // Owned by this component -- role/disabled/labelledby wiring must win
    // over a caller's own attributes (`docs/backlog.md` row 93's
    // duplicate-attribute hazard).
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(div {
            role: "group",
            aria_disabled: disabled,
            aria_labelledby: labeled_by,
        }),
    ]);

    rsx! {
        div {
            id: props.id,
            ..attributes,
            {props.children}
        }
    }
}

/// Props for [`CommandGroupLabel`].
#[derive(Props, Clone, PartialEq)]
pub struct CommandGroupLabelProps {
    /// Optional id for the label element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Children rendered inside the label.
    pub children: Element,
}

/// Labels a [`CommandGroup`]. Must be used inside one. Ported from
/// `select::SelectGroupLabel`'s exact shape: a mount-time `use_effect` sets
/// the enclosing group's `aria-labelledby` target.
#[component]
pub fn CommandGroupLabel(props: CommandGroupLabelProps) -> Element {
    let mut ctx: CommandGroupContext = use_context();

    let id = use_unique_id();
    let id = use_id_or(id, props.id);

    use_effect(move || {
        ctx.labeled_by.set(Some(id()));
    });

    rsx! {
        div {
            id,
            ..props.attributes,
            {props.children}
        }
    }
}

/// Props for [`CommandItem`].
#[derive(Props, Clone, PartialEq)]
pub struct CommandItemProps<T: Clone + PartialEq + 'static> {
    /// The value carried by this item.
    pub value: ReadSignal<T>,

    /// Display/searchable text. Required for non-string types.
    #[props(default)]
    pub text_value: ReadSignal<Option<String>>,

    /// Whether the item is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Optional id for the item element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Registration order used for keyboard navigation.
    pub index: ReadSignal<usize>,

    /// Optional aria-label.
    #[props(default)]
    pub aria_label: Option<String>,

    /// A presentation-only shortcut hint (e.g. a themed `Kbd`/`KbdGroup`
    /// pair). Carries no keyboard binding of its own -- see the module doc's
    /// "Scope" section. Rendered `aria-hidden` (see [`CommandItem`]'s doc):
    /// it is a sighted-user convenience, not part of the item's accessible
    /// name.
    #[props(default)]
    pub shortcut: Option<Element>,

    /// Additional attributes.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Children rendered inside the item.
    pub children: Element,
}

/// A selectable action inside a [`CommandList`]. Mirrors
/// `combobox::ComboboxOption`'s shape (registration, pointer selection,
/// ARIA/data attributes), minus the `ListboxContext` render-gate `Combobox`
/// options read -- `CommandList` is always rendered -- plus the optional
/// `shortcut` slot.
#[component]
pub fn CommandItem<T: PartialEq + Clone + 'static>(props: CommandItemProps<T>) -> Element {
    let index = props.index;
    let mut ctx: CommandContext = use_context();
    let visible = move || ctx.is_visible(index());

    let option = use_selectable_option(
        ctx.selectable,
        SelectableOptionConfig {
            id: props.id,
            index,
            value: props.value,
            text_value: props.text_value,
            option_disabled: props.disabled,
            component_name: "CommandItem",
        },
    );
    use_item(
        collection_item(ctx.selectable.collection, index)
            .key(move || Some(option.id.cloned()))
            .disabled(move || option.disabled.cloned())
            .hidden(move || !visible())
            .selected(move || (option.selected)()),
    );

    if !visible() {
        return rsx! {};
    }

    // Owned by this component -- role/selection state must win over a
    // caller's own attributes (`docs/backlog.md` row 93's
    // duplicate-attribute hazard).
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(div {
            role: "option",
            aria_selected: (option.selected)(),
            aria_disabled: (option.disabled)(),
            "data-highlighted": (option.focused)(),
            "data-disabled": (option.disabled)(),
            "data-selected": (option.selected)(),
        }),
    ]);

    rsx! {
        div {
            id: option.id,
            aria_label: props.aria_label.clone(),

            onmouseenter: move |_| {
                if !(option.disabled)() {
                    ctx.selectable.collection.set_focus(Some((option.index)()));
                }
            },
            onpointerdown: move |event| {
                pointer_select_start(&event, (option.disabled)(), option.down_pos);
            },
            onpointerup: move |event| {
                if pointer_select_commit(&event, (option.disabled)(), option.down_pos) {
                    ctx.selectable.select_value(RcPartialEqValue::new(option.value.cloned()));
                }
            },
            onpointercancel: move |_| {
                pointer_select_cancel(option.down_pos);
            },

            ..attributes,
            {props.children}
            if let Some(shortcut) = &props.shortcut {
                // A plain `data-*` marker, not a class -- gives a themed
                // layer a selector hook (e.g. `margin-left: auto` to push
                // it to the item's trailing edge) without this headless
                // primitive naming, or depending on, whatever class the
                // caller's own shortcut element happens to carry.
                // `aria-hidden` keeps it out of the item's accessible name
                // (e.g. AT would otherwise hear "New File, Command, N" on
                // every item with a shortcut) -- it is a sighted-user
                // convenience, per `CommandItemProps::shortcut`'s doc.
                span {
                    "data-command-item-shortcut": "true",
                    aria_hidden: "true",
                    {shortcut.clone()}
                }
            }
        }
    }
}

/// Props for [`CommandDialog`].
#[derive(Props, Clone, PartialEq)]
pub struct CommandDialogProps {
    /// The ID of the dialog root element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Whether the dialog is modal. Defaults to `true`, matching
    /// [`crate::dialog::DialogRoot`]'s own default.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub is_modal: ReadSignal<bool>,

    /// Whether the palette dims the page behind it with the shared overlay scrim.
    /// Defaults to `true`; forwarded to [`crate::dialog::DialogRoot`]'s `overlay`
    /// (`false` puts `data-dx-overlay="off"` on the `<dialog>`).
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub overlay: ReadSignal<bool>,

    /// The controlled `open` state of the dialog.
    #[props(default)]
    pub open: ReadSignal<Option<bool>>,

    /// The default `open` state of the dialog if it is not controlled.
    #[props(default)]
    pub default_open: bool,

    /// A callback that is called when the open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// Additional attributes to apply to the dialog content element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the dialog, typically a [`Command`] tree.
    pub children: Element,
}

/// `CommandDialog` = [`crate::dialog::DialogRoot`] +
/// [`crate::dialog::DialogContent`] wrapping a [`Command`]. A thin
/// composition, unmodified from `Dialog`'s own modal machinery -- see the
/// module doc. Initial focus lands on [`CommandInput`] via its own
/// `autofocus` attribute (see that component's doc) rather than anything
/// this wrapper does itself.
///
/// Carries two literal default classes (`dx-command-dialog-backdrop` on the
/// root, `dx-command-dialog` on the content) so a themed layer has
/// something to attach styling to without needing this wrapper to expose
/// two separate attribute-forwarding props of its own -- the same
/// hardcoded-default-class shape `DialogContent`'s own `"dx-dialog"`
/// fallback already uses (`dialog.rs`), not a new pattern. A caller's own
/// `attributes` still merge in alongside it (unchanged from `Dialog`'s own
/// `class`-plus-`..attributes` shape).
///
/// ## Example
///
/// Whatever the caller composes inside [`CommandDialog`] should include a
/// [`crate::dialog::DialogTitle`] -- [`crate::dialog::DialogRoot`] always
/// wires the dialog's `aria-labelledby` to a `DialogTitle` id (see
/// `dialog.rs`'s `dialog_labelledby`) whether or not one is rendered, so
/// omitting it leaves a broken `aria-labelledby` reference (an
/// `aria-dialog-name` a11y violation) rather than simply an unnamed
/// dialog. A command palette conventionally has no *visible* heading
/// above its search input, so a themed layer would typically render this
/// visually hidden (e.g. an `sr-only` class), the same way every other
/// `*Dialog`-family consumer in this repo supplies one -- see the
/// `preview` crate's `command`/`dialog`/`sidebar` demos.
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::command::{
///     Command, CommandDialog, CommandEmpty, CommandInput, CommandItem, CommandList,
/// };
/// use dioxus_primitives::dialog::DialogTitle;
///
/// #[component]
/// fn Demo() -> Element {
///     let mut open = use_signal(|| false);
///     let mut query = use_signal(String::new);
///
///     rsx! {
///         button { onclick: move |_| open.set(true), "Open Command Palette" }
///         CommandDialog { open: open(), on_open_change: move |v| open.set(v),
///             DialogTitle { "Command Palette" }
///             Command::<String> {
///                 query: Some(query()),
///                 on_query_change: move |next| query.set(next),
///                 on_value_change: move |_| open.set(false),
///                 CommandInput { aria_label: "Search commands" }
///                 CommandList { aria_label: "Commands",
///                     CommandEmpty { "No results found." }
///                     CommandItem::<String> {
///                         index: 0usize,
///                         value: "new-file".to_string(),
///                         text_value: "New File",
///                         "New File"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn CommandDialog(props: CommandDialogProps) -> Element {
    // Marks everything rendered below as "inside a dialog" so `CommandInput` defaults to
    // claiming focus (see `CommandInputProps::autofocus`).
    use_context_provider(|| CommandDialogContext);
    rsx! {
        crate::dialog::DialogRoot {
            id: props.id,
            is_modal: props.is_modal,
            overlay: props.overlay,
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            class: "dx-command-dialog-backdrop",
            crate::dialog::DialogContent {
                class: "dx-command-dialog",
                attributes: props.attributes,
                {props.children}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    /// Like [`render`], but lets mount-time effects commit first: a dialog only puts its content
    /// in the DOM once an effect has observed `open`.
    async fn render_settled(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        // Several rounds: effects queue state writes that only render on the next pass.
        for _ in 0..4 {
            let _ = tokio::time::timeout(std::time::Duration::from_millis(50), dom.wait_for_work())
                .await;
            dom.render_immediate_to_vec();
        }
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn inline_input_does_not_autofocus_by_default() {
        let html = render(|| {
            rsx! {
                Command::<String> { CommandInput {} }
            }
        });
        assert!(html.contains("<input"), "{html}");
        assert!(!html.contains("autofocus"), "{html}");
    }

    #[test]
    fn explicit_autofocus_overrides_the_inline_default() {
        let html = render(|| {
            rsx! {
                Command::<String> { CommandInput { autofocus: true } }
            }
        });
        assert!(html.contains("autofocus"), "{html}");
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn dialog_input_autofocuses_by_default_and_can_opt_out() {
        let html = render_settled(|| {
            rsx! {
                CommandDialog { default_open: true,
                    Command::<String> { CommandInput {} }
                }
            }
        })
        .await;
        assert!(html.contains("autofocus"), "{html}");

        let html = render_settled(|| {
            rsx! {
                CommandDialog { default_open: true,
                    Command::<String> { CommandInput { autofocus: false } }
                }
            }
        })
        .await;
        assert!(html.contains("<input"), "{html}");
        assert!(!html.contains("autofocus"), "{html}");
    }

    fn list_app(query: Option<String>) -> Element {
        rsx! {
            Command::<String> { query,
                CommandInput {}
                CommandList { aria_label: "Commands",
                    CommandEmpty { "No results found." }
                    CommandItem::<String> {
                        index: 0usize,
                        value: "new-file".to_string(),
                        text_value: "New File",
                        "New File"
                    }
                    CommandItem::<String> {
                        index: 1usize,
                        value: "open".to_string(),
                        text_value: "Open",
                        "Open"
                    }
                }
            }
        }
    }

    #[test]
    fn static_render_lists_every_item_and_no_empty_message() {
        // `rebuild_in_place` + SSR is the SSG / JS-off render: no effect has run, so no item has
        // registered yet. Every item must still be there and the empty message must not be.
        let html = render(|| list_app(None));
        assert!(html.contains("New File") && html.contains("Open"), "{html}");
        assert!(!html.contains("No results found."), "{html}");
        assert_eq!(html.matches("role=\"option\"").count(), 2, "{html}");
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn registered_items_still_filter_and_empty_shows_when_nothing_matches() {
        let html = render_settled(|| list_app(None)).await;
        assert_eq!(html.matches("role=\"option\"").count(), 2, "{html}");
        assert!(!html.contains("No results found."), "{html}");

        let html = render_settled(|| list_app(Some("op".to_string()))).await;
        assert_eq!(html.matches("role=\"option\"").count(), 1, "{html}");
        assert!(!html.contains("No results found."), "{html}");

        let html = render_settled(|| list_app(Some("zzz".to_string()))).await;
        assert_eq!(html.matches("role=\"option\"").count(), 0, "{html}");
        assert!(html.contains("No results found."), "{html}");
    }
}
