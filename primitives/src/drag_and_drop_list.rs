//! Defines the [`DragAndDropList`] component and its sub-components.
//!
//! The gesture itself (state machine, key map, announcements) is
//! [`reorder`]; the order it edits is a [`KeyOrder`]. The list is
//! *uncontrolled* when it only gets `items` (it remembers the order itself)
//! and *controlled* when it also gets `order`: then it reports every move
//! through `on_reorder` and shows exactly the order it is given.
use crate::collection::{collection_item, use_collection_provider, use_item, CollectionState};
use crate::data_view::KeyOrder;
use crate::direction::use_direction;
use crate::{fold_style_attributes, merge_attributes, use_unique_id};
use dioxus::prelude::*;
use dioxus_attributes::attributes;
use std::collections::HashSet;

pub mod reorder;

pub use reorder::ReorderAxis;
use reorder::{
    announce_cancel, announce_drop, announce_lift, announce_move, announce_remove, command_for_key,
    pointer_position, DropPosition, ReorderCommand, ReorderState,
};

/// The stable identity of a sortable item: the `key:` its root element
/// carries, else its position in the `items` it was given. The one derivation
/// both the primitive and a themed wrapper use, so the order's keys, the DOM
/// keys and the focus keys cannot disagree.
pub fn sortable_item_key(children: &Element, index: usize) -> String {
    children
        .as_ref()
        .ok()
        .and_then(|vnode| vnode.key.clone())
        .unwrap_or_else(|| index.to_string())
}

#[derive(Clone, PartialEq)]
struct SortableListItem {
    key: String,
    children: Element,
}

impl SortableListItem {
    fn new(children: Element, index: usize) -> Self {
        Self {
            key: sortable_item_key(&children, index),
            children,
        }
    }
}

/// Context provided by [`DragAndDropListItem`] to its children.
/// Use `use_context::<DragAndDropItemContext>()` to access the current item's index.
#[derive(Clone, Copy)]
pub struct DragAndDropItemContext {
    index: Signal<usize>,
}

impl DragAndDropItemContext {
    /// Returns the index of the current item in the list.
    pub fn index(&self) -> usize {
        (self.index)()
    }
}

/// What the list was given this render, mirrored for event handlers.
#[derive(Clone, Copy, PartialEq)]
struct ListConfig {
    controlled: bool,
    axis: ReorderAxis,
    on_reorder: Callback<Vec<String>>,
    on_remove: Callback<String>,
}

/// Context provided by [`DragAndDropList`] to its descendants.
/// Use `use_context::<DragAndDropContext>()` to access list-level operations.
#[derive(Clone, Copy)]
pub struct DragAndDropContext {
    drag: Signal<ReorderState>,
    /// The items as rendered: the given items arranged by the order.
    list_items: Signal<Vec<SortableListItem>>,
    /// The order an uncontrolled list remembers.
    order: Signal<KeyOrder>,
    /// Keys an uncontrolled list has removed.
    removed: Signal<HashSet<String>>,
    /// Key of the lifted item: focus returns to it by identity, because in a
    /// controlled list the rendered order only changes once the owner has
    /// applied the reported one.
    lifted_key: Signal<Option<String>>,
    config: Signal<ListConfig>,
    focus: CollectionState,
    announcement: Signal<String>,
    /// This list's own id for its [`DragAndDropInstructions`] element, which
    /// the list's `aria-describedby` points at. Generated per instance
    /// (`use_unique_id`) so two lists on one page never share an id (backlog
    /// row 120); both ends read it from here, so they cannot disagree.
    instructions_id: Signal<String>,
}

impl DragAndDropContext {
    fn drag_from(&self) -> Option<usize> {
        (self.drag)().from()
    }

    fn drop_to(&self) -> Option<usize> {
        (self.drag)().to()
    }

    fn drop_position(&self) -> DropPosition {
        (self.drag)().position()
    }

    fn is_dragging(&self) -> bool {
        (self.drag)().is_active()
    }

    fn is_lifted(&self) -> bool {
        (self.drag)().is_lifted()
    }

    fn axis(&self) -> ReorderAxis {
        (self.config)().axis
    }

    fn keys(&self) -> Vec<String> {
        (self.list_items)()
            .into_iter()
            .map(|item| item.key)
            .collect()
    }

    fn start_drag(&mut self, index: usize) {
        let key = (self.list_items)().get(index).map(|item| item.key.clone());
        self.lifted_key.set(key);
        self.drag.set(ReorderState::lift(index));
    }

    fn focus_lifted(&mut self) {
        self.focus.set_focus_key((self.lifted_key)());
    }

    fn end_drag(&mut self) {
        self.focus_lifted();
        self.drag.set(ReorderState::Idle);
    }

    fn cancel_drag(&mut self) {
        self.focus_lifted();
        self.drag.set(ReorderState::Idle);
    }

    fn drag_over(&mut self, hovered: usize, position: DropPosition) {
        self.drag.set((self.drag)().hover(hovered, position));
    }

    /// Drops the lifted item onto its chosen slot: the new order is committed
    /// (remembered when uncontrolled) and reported through `on_reorder`.
    fn drop(&mut self) {
        let (state, moved) = (self.drag)().drop_item();
        let Some((from, to)) = moved else {
            return;
        };
        self.drag.set(state);
        if from != to {
            let keys = self.keys();
            if let Some(key) = keys.get(from).cloned() {
                let mut order = KeyOrder::new(keys);
                order.move_key(&key, to);
                self.commit_order(order.into_keys());
            }
        }
    }

    fn commit_order(&mut self, keys: Vec<String>) {
        let config = (self.config)();
        if !config.controlled {
            self.order.set(KeyOrder::new(keys.clone()));
        }
        config.on_reorder.call(keys);
    }

    /// Remove the item at the given index from the list. An uncontrolled list
    /// drops it; a controlled one reports its key through `on_remove` and the
    /// owner decides.
    pub fn remove(&mut self, index: usize) {
        let items = (self.list_items)();
        let Some(removed) = items.get(index).map(|item| item.key.clone()) else {
            return;
        };
        let remaining: Vec<&SortableListItem> =
            items.iter().filter(|item| item.key != removed).collect();
        let new_len = remaining.len();
        let focus_id = new_len
            .checked_sub(1)
            .and_then(|last| remaining.get(index.min(last)).map(|item| item.key.clone()));
        let config = (self.config)();
        if config.controlled {
            config.on_remove.call(removed);
        } else {
            let mut order = (self.order)();
            order.remove(&removed);
            self.order.set(order);
            let mut gone = (self.removed)();
            gone.insert(removed.clone());
            self.removed.set(gone);
            config.on_remove.call(removed);
        }
        self.focus.set_focus_key(focus_id);
        self.announcement.set(announce_remove(index, new_len));
    }

    fn announce(&mut self, msg: String) {
        self.announcement.set(msg);
    }

    fn item_count(&self) -> usize {
        (self.list_items)().len()
    }

    fn is_focused(&self, index: usize) -> bool {
        self.focus.is_focused(index)
    }

    fn set_focus(&mut self, index: Option<usize>) {
        // Every sortable item carries a key, so resolve focus by key to stay
        // stable across reordering. An out-of-bounds (or `None`) index yields no
        // key, which clears focus rather than pointing at an invalid slot.
        let id =
            index.and_then(|index| (self.list_items)().get(index).map(|item| item.key.clone()));
        self.focus.set_focus_key(id);
    }

    fn focus_next(&mut self) {
        self.focus.focus_next();
    }

    fn focus_prev(&mut self) {
        self.focus.focus_prev();
    }

    /// A keyboard step of the lifted item, wrapping at the ends.
    fn step(&mut self, index: usize, delta: isize) {
        let len = self.item_count();
        let state = (self.drag)().step(index, delta, len);
        self.drag.set(state);
        let to = state.to().unwrap_or(index);
        self.announce(announce_move(to, len));
    }

    fn toggle_drag(&mut self, index: usize) {
        if self.is_lifted() {
            let from = self.drag_from().unwrap_or(index);
            let to = self.drop_to().unwrap_or(index);
            self.drop();
            self.end_drag();
            self.announce(announce_drop(from, to));
        } else {
            let count = self.item_count();
            self.start_drag(index);
            self.drag_over(index, DropPosition::Undefined);
            self.announce(announce_lift(index, count));
        }
    }
}

/// The props for the [`DragAndDropList`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DragAndDropListProps {
    /// The items to render. Each takes its identity from the `key:` on its
    /// root element (else its position here); give every item a key whenever
    /// the order is controlled or persisted.
    pub items: Vec<Element>,

    /// Controlled order: the keys of `items`, first to last. When set, the
    /// list shows exactly this order (keys it does not list follow in the
    /// order of `items`, listed keys with no item are skipped) and changes it
    /// only when the owner writes a new one after [`on_reorder`](Self::on_reorder).
    /// Unset, the list remembers the order itself, starting from `items`.
    #[props(default)]
    pub order: Option<ReadSignal<Vec<String>>>,

    /// Called with the complete new order (every key, first to last) after an
    /// item is moved, by pointer or keyboard, controlled or not. Not called
    /// when an item is dropped where it was.
    #[props(default)]
    pub on_reorder: Callback<Vec<String>>,

    /// Called with the key of an item the user removed. An uncontrolled list
    /// has already dropped it; a controlled list drops it only when the owner
    /// stops passing it.
    #[props(default)]
    pub on_remove: Callback<String>,

    /// The axis the items are laid out on. `Vertical` (the default) moves a
    /// lifted item with Up and Down; `Horizontal` uses Left and Right,
    /// mirrored under right-to-left, and the pointer picks the slot by x.
    #[props(default)]
    pub axis: ReorderAxis,

    /// Accessible label for the list
    #[props(default)]
    pub aria_label: Option<String>,

    /// Opacity of the drag "ghost": the item being dragged, which stays in
    /// the list as a placeholder while its slot is chosen. `0.0` (invisible)
    /// to `1.0` (opaque); values outside that range are clamped and `NaN` is
    /// ignored. When set, it is written as `--dx-dnd-ghost-opacity:<value>;`
    /// on the list root's inline style, where the item styles read it.
    /// `None` (the default) writes nothing, so the stylesheet's value
    /// applies -- `0.9` in the shipped stylesheet -- and a CSS rule that sets
    /// `--dx-dnd-ghost-opacity` themes it the same way.
    ///
    /// This is the in-list ghost only. The browser's own drag image (the
    /// translucent copy that follows the pointer) is drawn by the browser and
    /// cannot be styled from here.
    #[props(default)]
    pub ghost_opacity: Option<f32>,

    /// The room the other items make at the drop slot while an item is
    /// dragged over it: every item after the slot moves by this much, so the
    /// space between the two neighbours of the slot grows by exactly this
    /// length. Any non-negative CSS length (`"25px"`, `"1.5rem"`,
    /// `"var(--dx-space-6)"`); a bare number is read as pixels (`"40"` is
    /// `40px`). When set, it is written as `--dx-dnd-drop-gap:<value>;` on
    /// the list root's inline style. `None` (the default) writes nothing, so
    /// the stylesheet's value applies -- `25px` in the shipped stylesheet --
    /// and a CSS rule that sets `--dx-dnd-drop-gap` themes it the same way.
    /// An unusable value (empty, negative, or containing `;`, `{` or `}`) is
    /// ignored rather than written.
    ///
    /// The items move with a `transform`, so the list's own layout box does
    /// not grow: leave this much room below the list.
    #[props(default)]
    pub drop_gap: Option<String>,

    /// Additional attributes to apply to the list element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the list component.
    #[props(default)]
    pub children: Option<Element>,
}

/// The props for the [`DragAndDropListItems`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DragAndDropListItemsProps {
    /// Accessible label for the list.
    pub aria_label: String,

    /// Additional attributes to apply to the inner list element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the inner list element.
    #[props(default)]
    pub children: Option<Element>,
}

/// The props for the [`DragAndDropInstructions`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DragAndDropInstructionsProps {
    /// Additional attributes to apply to the instructions element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The props for the [`DragAndDropLiveRegion`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DragAndDropLiveRegionProps {
    /// Additional attributes to apply to the live region element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The `--dx-dnd-ghost-opacity` declaration for the `ghost_opacity` prop, or
/// an empty string for `None` / `NaN` (nothing is written, so the stylesheet
/// default stays in force). Out-of-range values are clamped to `0.0..=1.0`.
fn ghost_opacity_declaration(ghost_opacity: Option<f32>) -> String {
    match ghost_opacity {
        Some(value) if value.is_finite() => {
            format!("--dx-dnd-ghost-opacity:{};", value.clamp(0.0, 1.0))
        }
        _ => String::new(),
    }
}

/// The `--dx-dnd-drop-gap` declaration for the `drop_gap` prop, or an empty
/// string when the value is absent or unusable. A bare non-negative number
/// is pixels; anything else is passed through as the CSS length it is, except
/// that a value which could close the declaration early (`;`, `{`, `}`) or is
/// a negative bare number is dropped.
fn drop_gap_declaration(drop_gap: Option<&str>) -> String {
    let Some(raw) = drop_gap.map(str::trim).filter(|raw| !raw.is_empty()) else {
        return String::new();
    };
    if raw.contains([';', '{', '}']) {
        return String::new();
    }
    if let Ok(number) = raw.parse::<f64>() {
        // A bare number is only a length as pixels, and only a sane one.
        return if number.is_finite() && number >= 0.0 {
            format!("--dx-dnd-drop-gap:{raw}px;")
        } else {
            String::new()
        };
    }
    format!("--dx-dnd-drop-gap:{raw};")
}

/// # DragAndDropList
///
/// A list can be used to display content related to a single subject.
/// The content can consist of multiple elements of varying type and size.
/// Used when a user wants to change a collection order.
///
/// ## Styling hooks
///
/// The primitive is unstyled; it publishes two CSS custom properties on its
/// root element for a stylesheet to read, set from props or from CSS:
///
/// - `--dx-dnd-ghost-opacity` -- opacity of the dragged item's ghost
///   ([`DragAndDropListProps::ghost_opacity`]).
/// - `--dx-dnd-drop-gap` -- the room the other items make at the drop slot
///   ([`DragAndDropListProps::drop_gap`]).
///
/// ## Example
///
/// ```rust
///use dioxus::prelude::*;
///use dioxus_primitives::drag_and_drop_list::{DragAndDropList, DragAndDropListItem};
///#[component]
///pub fn Demo() -> Element {
///    let items = ["Item1", "Item2", "Item3"]
///        .map(|t| {
///            rsx! { {t} }
///        })
///        .to_vec();
///    rsx! {
///        DragAndDropList { items, ghost_opacity: 0.9, drop_gap: "25px" }
///    }
///}
/// ```
#[component]
pub fn DragAndDropList(props: DragAndDropListProps) -> Element {
    let drag = use_signal(|| ReorderState::Idle);
    let order = use_signal(KeyOrder::default);
    let removed = use_signal(HashSet::<String>::new);
    let lifted_key = use_signal(|| None::<String>);
    let mut list_items = use_signal(Vec::<SortableListItem>::new);
    let mut config = use_signal(|| ListConfig {
        controlled: props.order.is_some(),
        axis: props.axis,
        on_reorder: props.on_reorder,
        on_remove: props.on_remove,
    });
    let announcement = use_signal(String::new);
    let focus = use_collection_provider(ReadSignal::new(Signal::new(true)));
    let instructions_id = use_unique_id();

    // What the handlers see tracks the props: the config, and the items
    // arranged by the one order (the owner's when controlled, else the one
    // remembered), minus the ones an uncontrolled list removed.
    let current = ListConfig {
        controlled: props.order.is_some(),
        axis: props.axis,
        on_reorder: props.on_reorder,
        on_remove: props.on_remove,
    };
    if *config.peek() != current {
        config.set(current);
    }
    let arranged = {
        let source = props
            .items
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, item)| SortableListItem::new(item, index))
            .filter(|item| props.order.is_some() || !removed.peek().contains(&item.key));
        match props.order {
            Some(controlled) => KeyOrder::new(controlled.read().iter().cloned())
                .arrange(source, |item| item.key.as_str()),
            None => order.read().arrange(source, |item| item.key.as_str()),
        }
    };
    if *list_items.peek() != arranged {
        list_items.set(arranged);
    }

    use_context_provider(move || DragAndDropContext {
        drag,
        list_items,
        order,
        removed,
        lifted_key,
        config,
        focus,
        announcement,
        instructions_id,
    });

    let label = props
        .aria_label
        .as_deref()
        .unwrap_or("Sortable list")
        .to_string();

    let children = props.children.unwrap_or_else(|| {
        rsx! {
            DragAndDropInstructions {}
            DragAndDropListItems {
                aria_label: label,
            }
            DragAndDropLiveRegion {}
        }
    });

    // The tuning props become custom properties on the root's inline style,
    // ahead of any caller `style` so a caller's own declaration still wins.
    // `fold_style_attributes` merges the caller's style into this one string,
    // so the root never carries two `style` attributes.
    let (caller_style, rest_attrs) = fold_style_attributes(props.attributes);
    let style = format!(
        "{}{}{}",
        ghost_opacity_declaration(props.ghost_opacity),
        drop_gap_declaration(props.drop_gap.as_deref()),
        caller_style.map(|s| format!(" {s}")).unwrap_or_default(),
    );
    let style = (!style.is_empty()).then_some(style);
    // `data-orientation` goes through the same merge, so a caller's own
    // `data-orientation` cannot become a second attribute on the root.
    let axis = props.axis.as_str();
    let attributes = merge_attributes(vec![
        attributes!(div { "data-orientation": axis }),
        rest_attrs,
        attributes!(div { style: style }),
    ]);

    rsx! {
        div {
            ..attributes,
            {children}
        }
    }
}

/// Return render data for the current sortable items.
pub fn use_drag_and_drop_list_items() -> Vec<DragAndDropListRenderItem> {
    let ctx: DragAndDropContext = use_context();
    (ctx.list_items)()
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            // Propagate any `key:` the caller set on the item's root element
            // through to the keyed sortable item fragment.
            DragAndDropListRenderItem {
                index,
                key: item.key,
                children: item.children,
            }
        })
        .collect()
}

/// The inner list element for sortable items.
#[component]
pub fn DragAndDropListItems(props: DragAndDropListItemsProps) -> Element {
    let mut ctx: DragAndDropContext = use_context();
    let children = props.children.unwrap_or_else(|| {
        rsx! {
            for item in use_drag_and_drop_list_items() {
                Fragment {
                    key: "{item.key}",
                    DragAndDropDropIndicator {
                        index: item.index,
                        position: "before",
                    }
                    DragAndDropListItem {
                        index: item.index,
                        item_key: item.key.clone(),
                        {item.children}
                    }
                    DragAndDropDropIndicator {
                        index: item.index,
                        position: "after",
                    }
                }
            }
        }
    });

    // `aria_roledescription`/`aria_describedby` are owned: the latter is an
    // id reference to `DragAndDropInstructions`'s own owned `id` below, so a
    // caller override on either side would strand the reference. Both read
    // the one per-list id from the context.
    let owned = attributes!(ul {
        aria_roledescription: "sortable list",
        aria_describedby: (ctx.instructions_id)(),
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        ul {
            aria_label: "{props.aria_label}",
            ondragover: move |event: Event<DragData>| {
                // Drops can happen in the visual gaps between items. The
                // nearest item still owns target calculation, but the list
                // must accept the final drop for those gap targets.
                event.prevent_default();
                event.data_transfer().set_drop_effect("move");
            },
            ondrop: move |event: Event<DragData>| {
                event.prevent_default();
                ctx.drop();
            },
            ..merged,
            {children}
        }
    }
}

/// Screen-reader instructions for keyboard sorting.
#[component]
pub fn DragAndDropInstructions(props: DragAndDropInstructionsProps) -> Element {
    let ctx: DragAndDropContext = use_context();
    // `id` is owned: `DragAndDropListItems` above references it via
    // `aria_describedby`, so a caller override would strand that reference
    // (backlog row 93 names this site explicitly). It is the list's own
    // generated id, not a literal: a literal is shared by every list on the
    // page (backlog row 120). `style` is this visually-hidden element's own
    // structural CSS, also owned.
    let owned = attributes!(div {
        id: (ctx.instructions_id)(),
        style: "position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0,0,0,0);",
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        div {
            ..merged,
            "Press Enter to start reordering. Use Arrow keys to change position. Press Enter to confirm or Escape to cancel."
        }
    }
}

/// Live region for drag-and-drop announcements.
#[component]
pub fn DragAndDropLiveRegion(props: DragAndDropLiveRegionProps) -> Element {
    let ctx: DragAndDropContext = use_context();
    let announcement = (ctx.announcement)();

    let owned = attributes!(div {
        role: "status",
        aria_live: "assertive",
        aria_atomic: "true",
        style: "position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0,0,0,0);",
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        div {
            ..merged,
            "{announcement}"
        }
    }
}

/// The props for the [`DragAndDropListItemProps`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DragAndDropListItemProps {
    /// The index of the item in the list
    pub index: usize,

    /// Stable identity for this item. Pass the same value as the item's
    /// `key:` when manually rendering sortable items.
    #[props(default)]
    pub item_key: Option<String>,

    /// Additional attributes to apply to the list item element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the list item component.
    pub children: Element,
}

/// The props for the [`DragAndDropDropIndicator`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DragAndDropDropIndicatorProps {
    /// The index of the item this indicator is adjacent to.
    pub index: usize,

    /// The indicator position relative to the item.
    pub position: &'static str,

    /// Additional attributes to apply to the drop indicator element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// Data for rendering a sortable list item.
#[derive(Clone, PartialEq)]
pub struct DragAndDropListRenderItem {
    /// The current index of this item.
    pub index: usize,

    /// The stable key for this item.
    pub key: String,

    /// The rendered item children.
    pub children: Element,
}

/// # DragAndDropListItem
///
/// This component represents an individual draggable item in the dnd list.
/// This must be used inside a [`DragAndDropList`] component.
///
/// ## Example
///
/// ```rust
///use dioxus::prelude::*;
///use dioxus_primitives::drag_and_drop_list::{DragAndDropList, DragAndDropListItem};
///#[component]
///pub fn Demo() -> Element {
///    let items = ["Item1", "Item2", "Item3"]
///        .map(|t| {
///            rsx! { {t} }
///        })
///        .to_vec();
///    rsx! {
///        DragAndDropList { items }
///    }
///}
/// ```
#[component]
pub fn DragAndDropListItem(props: DragAndDropListItemProps) -> Element {
    let mut ctx: DragAndDropContext = use_context();

    let index = props.index;
    let mut item_ctx = use_context_provider(move || DragAndDropItemContext {
        index: Signal::new(index),
    });
    if *item_ctx.index.peek() != index {
        item_ctx.index.set(index);
    }
    let index_signal = item_ctx.index;

    let item_key = props.item_key.clone();
    let item = use_item(collection_item(ctx.focus, index_signal).key(move || item_key.clone()));
    let mut collection_onmounted = item.onmounted();
    let mut item_ref: Signal<Option<std::rc::Rc<MountedData>>> = use_signal(|| None);

    let direction = use_direction(None);
    let onkeydown = move |event: Event<KeyboardData>| {
        let Some(command) = command_for_key(&event.key(), ctx.axis(), direction) else {
            return;
        };
        event.prevent_default();
        let lifted = ctx.is_dragging();
        match command {
            ReorderCommand::Step(delta) => {
                if lifted {
                    ctx.step(index, delta);
                } else if delta < 0 {
                    ctx.focus_prev();
                } else {
                    ctx.focus_next();
                }
            }
            ReorderCommand::Toggle => ctx.toggle_drag(index),
            ReorderCommand::Cancel => {
                if lifted {
                    let from = ctx.drag_from().unwrap_or(index);
                    ctx.cancel_drag();
                    ctx.announce(announce_cancel(from));
                }
            }
            ReorderCommand::Remove => {
                if !lifted {
                    ctx.remove(index);
                }
            }
            ReorderCommand::First => {
                if !lifted {
                    ctx.set_focus(Some(0));
                }
            }
            ReorderCommand::Last => {
                if !lifted {
                    ctx.set_focus(ctx.item_count().checked_sub(1));
                }
            }
        }
    };

    // All owned: role description/draggable/tabindex define this item's DnD
    // widget semantics, the `data-*` attributes are its own managed
    // drag/focus state. There is no `aria-grabbed` (deprecated since ARIA
    // 1.1): the lift is announced through the live region and shown by
    // `data-is-grabbing`.
    let owned = attributes!(li {
        aria_roledescription: "sortable item",
        draggable: "true",
        tabindex: item.tabindex,
        "data-is-grabbing": if ctx.drag_from().is_some_and(|from| from == index) { "true" },
        // Set when the drop target has returned to this item's starting slot —
        // i.e. dropping now would leave it in place. The primitive suppresses
        // the drop indicator in that case (no gap to point to), so styling
        // hooks off this attribute to surface the "stays here" state.
        "data-drop-at-origin": if ctx.drag_from().is_some_and(|from| from == index) && ctx.drop_to() == Some(index) { "true" },
        "data-focus-visible": if ctx.is_focused(index) { "true" },
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        li {
            onmounted: move |event| {
                item_ref.set(Some(event.data()));
                collection_onmounted(event);
            },
            onfocus: move |_| {
                if !ctx.is_dragging() {
                    ctx.set_focus(Some(index));
                }
            },
            ondragstart: move |event: Event<DragData>| {
                ctx.start_drag(index);
                event.data_transfer().set_effect_allowed("move");
                event.data_transfer().set_drop_effect("move");
                // Note: this is only for Firefox (without it, DnD won't work)
                let _ = event.data_transfer().set_data("text/html", "");
                let mut document_drop_ctx = ctx;
                let mut document_drop = document::eval(
                    r#"
                    function cleanup() {
                        document.removeEventListener("dragover", onDragOver, true);
                        document.removeEventListener("drop", onDrop, true);
                        document.removeEventListener("dragend", onDragEnd, true);
                    }

                    function onDragOver(event) {
                        event.preventDefault();
                        if (event.dataTransfer) {
                            event.dataTransfer.dropEffect = "move";
                        }
                    }

                    function onDrop(event) {
                        event.preventDefault();
                        dioxus.send("drop");
                        cleanup();
                    }

                    function onDragEnd() {
                        dioxus.send("end");
                        cleanup();
                    }

                    document.addEventListener("dragover", onDragOver, true);
                    document.addEventListener("drop", onDrop, true);
                    document.addEventListener("dragend", onDragEnd, true);

                    await dioxus.recv();
                    cleanup();
                    "#,
                );
                spawn(async move {
                    if let Ok(action) = document_drop.recv::<String>().await {
                        if action == "drop" {
                            document_drop_ctx.drop();
                        }
                    }
                    let _ = document_drop.send(true);
                });
            },
            ondragend: move |_| ctx.end_drag(),
            ondragover: move |event: Event<DragData>| {
                event.prevent_default();
                event.data_transfer().set_drop_effect("move");
                async move {
                    if let Some(md) = item_ref() {
                        let pointer = event.client_coordinates();
                        if let Ok(rect) = md.get_client_rect().await {
                            let axis = ctx.axis();
                            let (cursor, origin, size) = match axis {
                                ReorderAxis::Vertical => {
                                    (pointer.y, rect.origin.y, rect.size.height)
                                }
                                ReorderAxis::Horizontal => {
                                    (pointer.x, rect.origin.x, rect.size.width)
                                }
                            };
                            ctx.drag_over(
                                index,
                                pointer_position(axis, direction, cursor, origin, size),
                            );
                        }
                    }
                }
            },
            //ondragleave: move |_| ctx.drop_to.set(None),
            onkeydown,
            ..merged,
            {props.children}
        }
    }
}

/// The drop indicator rendered next to a sortable item.
#[component]
pub fn DragAndDropDropIndicator(props: DragAndDropDropIndicatorProps) -> Element {
    let ctx: DragAndDropContext = use_context();
    let render = ctx.drop_to().is_some_and(|to| to == props.index)
        && match props.position {
            "before" => ctx.drop_position() == DropPosition::Before,
            "after" => ctx.drop_position() == DropPosition::After,
            _ => false,
        };
    if !render {
        return rsx! {};
    }

    let owned = attributes!(div { "data-position": "{props.position}" });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        div { ..merged }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(component: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(component);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    fn items() -> Vec<Element> {
        ["One", "Two", "Three"]
            .map(|label| rsx! { span { key: "{label}", "{label}" } })
            .to_vec()
    }

    /// The opening tag of the first element in `html` carrying `marker`.
    fn tag_containing<'a>(html: &'a str, marker: &str) -> &'a str {
        let at = html
            .find(marker)
            .unwrap_or_else(|| panic!("no {marker} in {html}"));
        let start = html[..at].rfind('<').unwrap();
        let end = html[at..].find('>').unwrap() + at;
        &html[start..=end]
    }

    /// Every value of `attribute="..."` in `html`, in document order.
    fn attribute_values(html: &str, attribute: &str) -> Vec<String> {
        let needle = format!(" {attribute}=\"");
        html.match_indices(&needle)
            .map(|(at, _)| {
                let rest = &html[at + needle.len()..];
                rest[..rest.find('"').unwrap()].to_string()
            })
            .collect()
    }

    #[test]
    fn ghost_opacity_declaration_clamps_and_ignores_nan() {
        assert_eq!(ghost_opacity_declaration(None), "");
        assert_eq!(
            ghost_opacity_declaration(Some(0.6)),
            "--dx-dnd-ghost-opacity:0.6;"
        );
        assert_eq!(
            ghost_opacity_declaration(Some(0.9)),
            "--dx-dnd-ghost-opacity:0.9;"
        );
        assert_eq!(
            ghost_opacity_declaration(Some(1.0)),
            "--dx-dnd-ghost-opacity:1;"
        );
        assert_eq!(
            ghost_opacity_declaration(Some(7.0)),
            "--dx-dnd-ghost-opacity:1;"
        );
        assert_eq!(
            ghost_opacity_declaration(Some(-2.0)),
            "--dx-dnd-ghost-opacity:0;"
        );
        assert_eq!(ghost_opacity_declaration(Some(f32::NAN)), "");
        assert_eq!(ghost_opacity_declaration(Some(f32::INFINITY)), "");
    }

    #[test]
    fn drop_gap_declaration_takes_lengths_and_bare_pixel_numbers() {
        assert_eq!(drop_gap_declaration(None), "");
        assert_eq!(drop_gap_declaration(Some("  ")), "");
        assert_eq!(
            drop_gap_declaration(Some("25px")),
            "--dx-dnd-drop-gap:25px;"
        );
        assert_eq!(
            drop_gap_declaration(Some(" 1.5rem ")),
            "--dx-dnd-drop-gap:1.5rem;"
        );
        assert_eq!(
            drop_gap_declaration(Some("var(--dx-space-6)")),
            "--dx-dnd-drop-gap:var(--dx-space-6);"
        );
        assert_eq!(
            drop_gap_declaration(Some("calc(1rem + 4px)")),
            "--dx-dnd-drop-gap:calc(1rem + 4px);"
        );
        // A bare number is pixels.
        assert_eq!(drop_gap_declaration(Some("40")), "--dx-dnd-drop-gap:40px;");
        assert_eq!(drop_gap_declaration(Some("0")), "--dx-dnd-drop-gap:0px;");
        assert_eq!(
            drop_gap_declaration(Some("12.5")),
            "--dx-dnd-drop-gap:12.5px;"
        );
        // Unusable values write nothing instead of a broken declaration.
        assert_eq!(drop_gap_declaration(Some("-4")), "");
        assert_eq!(drop_gap_declaration(Some("NaN")), "");
        assert_eq!(drop_gap_declaration(Some("inf")), "");
        assert_eq!(drop_gap_declaration(Some("4px; color: red")), "");
        assert_eq!(drop_gap_declaration(Some("4px}")), "");
    }

    #[component]
    fn Untuned() -> Element {
        rsx! { DragAndDropList { items: items() } }
    }

    #[test]
    fn untuned_list_writes_no_custom_properties() {
        let html = render(Untuned);
        assert!(!html.contains("--dx-dnd-"), "{html}");
        assert!(!html.contains(" style=\"--"), "{html}");
    }

    #[component]
    fn Tuned() -> Element {
        rsx! { DragAndDropList { items: items(), ghost_opacity: 0.6, drop_gap: "40px" } }
    }

    #[test]
    fn tuning_props_become_custom_properties_on_the_root() {
        let html = render(Tuned);
        let root = tag_containing(&html, "--dx-dnd-ghost-opacity");
        assert!(root.starts_with("<div"), "{root}");
        assert!(
            root.contains("style=\"--dx-dnd-ghost-opacity:0.6;--dx-dnd-drop-gap:40px;\""),
            "{root}"
        );
    }

    #[component]
    fn TunedWithCallerStyle() -> Element {
        rsx! {
            DragAndDropList {
                items: items(),
                drop_gap: "4px",
                style: "--dx-dnd-drop-gap: 9px; margin: 0;",
            }
        }
    }

    #[test]
    fn caller_style_follows_the_props_in_one_style_attribute() {
        let html = render(TunedWithCallerStyle);
        let root = tag_containing(&html, "--dx-dnd-drop-gap");
        // One merged attribute, prop first so the caller's own declaration wins.
        assert_eq!(root.matches(" style=").count(), 1, "{root}");
        let prop_at = root.find("--dx-dnd-drop-gap:4px;").unwrap();
        let caller_at = root.find("--dx-dnd-drop-gap: 9px;").unwrap();
        assert!(prop_at < caller_at, "{root}");
        assert!(root.contains("margin: 0;"), "{root}");
    }

    #[component]
    fn TwoLists() -> Element {
        rsx! {
            DragAndDropList { items: items(), aria_label: "First" }
            DragAndDropList { items: items(), aria_label: "Second" }
        }
    }

    #[test]
    fn every_list_describes_itself_with_its_own_instructions_id() {
        // Backlog row 120: a hard-coded `id="dnd-instructions"` made every
        // list on a page point at whichever instructions element came first.
        let html = render(TwoLists);
        let described_by = attribute_values(&html, "aria-describedby");
        let ids = attribute_values(&html, "id");
        assert_eq!(described_by.len(), 2, "{html}");
        assert_ne!(described_by[0], described_by[1], "{html}");
        for target in &described_by {
            assert_ne!(target, "dnd-instructions", "{html}");
            assert_eq!(
                ids.iter().filter(|id| *id == target).count(),
                1,
                "`{target}` must name exactly one element: {html}"
            );
        }
        // And each list points at the instructions inside its own root: the
        // instructions precede their list, and the lists do not interleave.
        let id_at = |id: &str| html.find(&format!(" id=\"{id}\"")).unwrap();
        let describedby_at = |id: &str| html.find(&format!("aria-describedby=\"{id}\"")).unwrap();
        assert!(
            id_at(&described_by[0]) < describedby_at(&described_by[0]),
            "{html}"
        );
        assert!(
            describedby_at(&described_by[0]) < id_at(&described_by[1]),
            "{html}"
        );
        assert!(
            id_at(&described_by[1]) < describedby_at(&described_by[1]),
            "{html}"
        );
    }

    /// Text of every sortable item's `span`, in document order.
    fn item_texts(html: &str) -> Vec<String> {
        html.split("<span")
            .skip(1)
            .map(|chunk| {
                let start = chunk.find('>').unwrap() + 1;
                chunk[start..chunk.find("</span>").unwrap()].to_string()
            })
            .collect()
    }

    #[component]
    fn Controlled() -> Element {
        let order = use_signal(|| vec!["Three".to_string(), "One".to_string()]);
        rsx! { DragAndDropList { items: items(), order: ReadSignal::from(order) } }
    }

    #[test]
    fn a_controlled_list_shows_the_given_order_and_appends_unlisted_items() {
        let html = render(Controlled);
        assert_eq!(item_texts(&html), ["Three", "One", "Two"], "{html}");
    }

    #[test]
    fn an_uncontrolled_list_starts_in_item_order_and_has_no_aria_grabbed() {
        let html = render(Untuned);
        assert_eq!(item_texts(&html), ["One", "Two", "Three"], "{html}");
        assert!(!html.contains("aria-grabbed"), "{html}");
    }
}
