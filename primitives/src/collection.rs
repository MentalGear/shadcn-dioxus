//! Ordered interactive collection state shared by roving-focus components.

use std::collections::HashMap;
use std::rc::Rc;

use dioxus::core::{current_scope_id, use_drop, Runtime, ScopeId};
use dioxus::prelude::*;

use crate::{use_effect_cleanup, use_unique_id};

/// The attribute that marks a derived-index item's element for the DOM-order
/// pass: `"{collection id}:{slot}"`.
const ORDER_MARKER: &str = "data-collection-item";

/// Lists the slots of one collection's marked elements in document order
/// (`querySelectorAll` returns document order).
const DOM_ORDER_JS: &str = r#"
const prefix = await dioxus.recv();
const slots = [];
for (const el of document.querySelectorAll('[data-collection-item^="' + prefix + '"]')) {
  slots.push(Number(el.getAttribute('data-collection-item').slice(prefix.length)));
}
dioxus.send(slots);
"#;

#[derive(Clone, PartialEq)]
struct CollectionItemState {
    index: usize,
    key: Option<String>,
    disabled: bool,
    hidden: bool,
    selected: bool,
    // Optional label text, read back by `CollectionState::text_entries`;
    // this struct holds every per-item field, so there is no second item
    // registry. See that method's own doc.
    text_value: Option<String>,
}

impl CollectionItemState {
    fn available(&self) -> bool {
        !self.disabled && !self.hidden
    }

    fn same_identity(&self, other: &Self) -> bool {
        match (&self.key, &other.key) {
            (Some(a), Some(b)) => a == b,
            (None, None) => self.index == other.index,
            _ => false,
        }
    }
}

/// A requested initial focus placement.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum CollectionPlacement {
    First,
    Last,
}

/// Group-level options for a collection.
#[derive(Clone, Copy, Default)]
pub(crate) struct CollectionOptions {
    /// When no item is selected and none is focused, make every available item
    /// a tab stop instead of only the first. Native HTML radio-group semantics.
    pub(crate) tabbable_when_empty: bool,
}

/// A cloneable handle for ordered item registration and roving-focus navigation.
#[derive(Clone, Copy)]
pub(crate) struct CollectionState {
    roving_loop: ReadSignal<bool>,
    tabbable_when_empty: bool,
    recent: Signal<Option<usize>>,
    focused: Signal<Option<usize>>,
    focus_key: Signal<Option<String>>,
    items: Signal<Vec<CollectionItemState>>,
    order: CollectionOrder,
}

/// How a collection numbers the items that take no explicit `index`
/// ([`use_item_index`], row 396).
///
/// An item is numbered when it is created, from `next_index`. Creation order
/// is not tree order: Dioxus creates the component siblings nested inside one
/// element of a template in reverse (`dioxus-core`'s `load_placeholders`
/// walks them backwards to keep its template paths valid), while template
/// roots and `for` loops are created forwards. So on the client, once the
/// items are in the document, a pass reads their elements' document order
/// and renumbers them when it disagrees. Indices are only compared, never
/// counted: the pass numbers into a fresh range above every index handed out
/// so far, so no two items ever share an index on the way.
#[derive(Clone, Copy)]
struct CollectionOrder {
    /// The next index handed out. Not reactive: read and bumped once per item.
    next_index: CopyValue<usize>,
    /// The prefix of every derived item's [`ORDER_MARKER`] value.
    marker_prefix: Signal<String>,
    /// Each derived item's index signal, by its slot (the index it was
    /// created with, which its marker carries).
    derived: CopyValue<HashMap<usize, Signal<usize>>>,
    /// Set once any item passes an explicit `index`: the caller then owns
    /// the order, and the pass never runs.
    explicit: CopyValue<bool>,
    /// Whether a pass is scheduled and not finished. Initial focus placement
    /// waits for it ([`CollectionState::try_focus_placement`]).
    pending: Signal<bool>,
    /// The provider's scope, which owns the pass's task: an item that
    /// unmounts mid-pass must not cancel it.
    scope: ScopeId,
}

impl CollectionState {
    /// `id` is a hydration-stable unique id ([`use_unique_id`]): derived
    /// items render it in their order marker on the server too.
    pub(crate) fn new(
        roving_loop: ReadSignal<bool>,
        options: CollectionOptions,
        id: Signal<String>,
    ) -> Self {
        Self {
            roving_loop,
            tabbable_when_empty: options.tabbable_when_empty,
            recent: Signal::new(None),
            focused: Signal::new(None),
            focus_key: Signal::new(None),
            items: Signal::new(Vec::new()),
            order: CollectionOrder {
                next_index: CopyValue::new(0),
                marker_prefix: Signal::new(format!("{}:", id.peek())),
                derived: CopyValue::new(HashMap::new()),
                explicit: CopyValue::new(false),
                pending: Signal::new(false),
                scope: current_scope_id(),
            },
        }
    }

    /// Whether a DOM-order pass is in flight, after which the items' indices
    /// may differ. Tracked.
    pub(crate) fn order_pending(&self) -> bool {
        (self.order.pending)()
    }

    /// Hand out the index of an item that is being created: the next one in
    /// creation order, or `explicit` when the caller set one. An explicit
    /// index also moves the counter past itself.
    fn allocate_index(&mut self, explicit: Option<usize>) -> usize {
        let next_index = *self.order.next_index.peek();
        let index = match explicit {
            Some(index) => {
                self.order.explicit.set(true);
                index
            }
            None => next_index,
        };
        self.order
            .next_index
            .set(index.saturating_add(1).max(next_index));
        index
    }

    /// Schedule the DOM-order pass (see [`CollectionOrder`]) unless one is
    /// already scheduled, there is nothing to order, or the caller owns the
    /// order. Call it whenever derived items may have entered the document.
    pub(crate) fn request_order_pass(&mut self) {
        let order = self.order;
        if *order.explicit.peek() || order.derived.peek().is_empty() || *order.pending.peek() {
            return;
        }
        let mut pending = order.pending;
        pending.set(true);
        let mut collection = *self;
        Runtime::current().spawn(order.scope, async move {
            let mut eval = document::eval(DOM_ORDER_JS);
            let slots = match eval.send(order.marker_prefix.peek().clone()) {
                Ok(()) => eval.recv::<Vec<usize>>().await.ok(),
                Err(_) => None,
            };
            if let Some(slots) = slots {
                collection.apply_dom_order(&slots);
            }
            pending.set(false);
        });
    }

    /// Renumber the derived items whose slots `slots` lists (document order)
    /// when their indices are not already ascending in that order.
    ///
    /// The registered items and the focus are renumbered here, in one step:
    /// each item's own registration effect only catches up afterwards, and a
    /// reader in between (an initial focus placement, which waits for this
    /// pass) must already see the new order.
    fn apply_dom_order(&mut self, slots: &[usize]) {
        let derived = self.order.derived.peek();
        let signals: Vec<Signal<usize>> = slots
            .iter()
            .filter_map(|slot| derived.get(slot).copied())
            .collect();
        drop(derived);
        let ascending = signals
            .windows(2)
            .all(|pair| *pair[0].peek() < *pair[1].peek());
        if ascending {
            return;
        }
        let base = *self.order.next_index.peek();
        self.order.next_index.set(base + signals.len());
        let renumbered: HashMap<usize, usize> = signals
            .iter()
            .enumerate()
            .map(|(offset, index)| (*index.peek(), base + offset))
            .collect();
        let renumber = |index: usize| renumbered.get(&index).copied().unwrap_or(index);
        {
            let mut items = self.items.write();
            for item in items.iter_mut() {
                item.index = renumber(item.index);
            }
            items.sort_by_key(|item| item.index);
        }
        let focused = (*self.focused.peek()).map(renumber);
        if *self.focused.peek() != focused {
            self.focused.set(focused);
        }
        let recent = (*self.recent.peek()).map(renumber);
        if *self.recent.peek() != recent {
            self.recent.set(recent);
        }
        for (offset, mut index) in signals.into_iter().enumerate() {
            index.set(base + offset);
        }
    }

    /// Whether arrow navigation wraps around the ends of the collection.
    pub(crate) fn loops(&self) -> bool {
        (self.roving_loop)()
    }

    /// The backing loop signal, for nested collections that inherit the parent's
    /// looping behavior.
    pub(crate) fn loop_signal(&self) -> ReadSignal<bool> {
        self.roving_loop
    }

    fn register_item(&mut self, item: CollectionItemState) {
        let index = item.index;
        let key = item.key.clone();
        let available = item.available();
        if self.items.peek().iter().any(|existing| existing == &item) {
            return;
        }
        let previous_index = self
            .items
            .peek()
            .iter()
            .find(|existing| existing.same_identity(&item))
            .map(|item| item.index);
        sync_item(&mut self.items.write(), item);
        if let Some(previous_index) = previous_index {
            self.move_focus_for_reindexed_key(previous_index, index, key.as_deref());
        } else {
            self.adopt_focus_key_for_index(index, key.clone());
        }
        self.clear_focus_if_unavailable(index, key.as_deref(), available);
    }

    fn unregister_item(&mut self, item: &CollectionItemState) {
        let removing_focused = self.is_focused_item(item);
        let removed_focused = {
            let mut items = self.items.write();
            let removed = items.iter().any(|existing| existing.same_identity(item));
            items.retain(|existing| !existing.same_identity(item));
            removed && removing_focused
        };

        if removed_focused {
            self.clear_focus();
        }
    }

    pub(crate) fn set_focus(&mut self, index: Option<usize>) {
        let target = match index {
            Some(idx) if self.is_known_unavailable(idx) => None,
            other => other,
        };
        let key = target.and_then(|idx| self.key_for_index(idx));
        if let Some(idx) = target {
            self.recent.set(Some(idx));
        }
        // Only notify subscribers when the value actually changes. A redundant
        // clear (clearing focus when nothing is focused) must not wake effects
        // reading focus state, or it re-triggers the context-menu auto-close
        // (guarded on `focused` below).
        if *self.focus_key.peek() != key {
            self.focus_key.set(key);
        }
        if *self.focused.peek() != target {
            self.focused.set(target);
        }
    }

    pub(crate) fn set_focus_key(&mut self, key: Option<String>) {
        let index = key.as_deref().and_then(|key| self.index_for_key(key));
        if let Some(index) = index {
            self.recent.set(Some(index));
        }
        if *self.focused.peek() != index {
            self.focused.set(index);
        }
        if *self.focus_key.peek() != key {
            self.focus_key.set(key);
        }
    }

    pub(crate) fn clear_focus(&mut self) {
        self.set_focus(None);
    }

    pub(crate) fn focused_index(&self) -> Option<usize> {
        (self.focused)()
    }

    pub(crate) fn recent_focus(&self) -> Option<usize> {
        (self.recent)()
    }

    pub(crate) fn recent_focus_or_default(&self) -> usize {
        self.recent_focus()
            .filter(|&index| self.is_available(index))
            .or_else(|| self.selected_available_index())
            .or_else(|| self.first_available_index())
            .unwrap_or_default()
    }

    pub(crate) fn focused_key(&self) -> Option<String> {
        let focused = self.focused_index()?;
        let key = (self.focus_key)();
        let items = self.items.read();
        match key {
            Some(key) => items
                .iter()
                .find(|item| item.available() && item.key.as_deref() == Some(key.as_str()))
                .and_then(|item| item.key.clone()),
            None => items
                .iter()
                .find(|item| item.index == focused && item.available())
                .and_then(|item| item.key.clone()),
        }
    }

    pub(crate) fn any_focused(&self) -> bool {
        self.focused.read().is_some()
    }

    pub(crate) fn is_focused(&self, index: usize) -> bool {
        self.focused_index() == Some(index)
    }

    pub(crate) fn is_available(&self, index: usize) -> bool {
        self.items
            .read()
            .iter()
            .any(|item| item.index == index && item.available())
    }

    /// Every currently registered item's `(index, text value or empty
    /// string, disabled)`, in index order -- the read side of the per-item
    /// `text_value` registered via [`CollectionItemBuilder::text_value`].
    /// Part of the one per-item registration [`use_item`] threads through
    /// for `key`/`disabled`/`hidden`/`selected`: a feature
    /// that needs to search item text (currently only
    /// `crate::typeahead::find_next_match`) reads it here instead of
    /// keeping its own second, parallel item registry that would
    /// inevitably drift from this one. `disabled` here means "not
    /// available" (disabled *or* hidden), matching every other read on this
    /// type (e.g. [`Self::is_available`]).
    pub(crate) fn text_entries(&self) -> Vec<(usize, String, bool)> {
        self.items
            .read()
            .iter()
            .map(|item| {
                (
                    item.index,
                    item.text_value.clone().unwrap_or_default(),
                    !item.available(),
                )
            })
            .collect()
    }

    pub(crate) fn first_available_index(&self) -> Option<usize> {
        self.items
            .read()
            .iter()
            .find(|item| item.available())
            .map(|item| item.index)
    }

    pub(crate) fn last_available_index(&self) -> Option<usize> {
        self.items
            .read()
            .iter()
            .rev()
            .find(|item| item.available())
            .map(|item| item.index)
    }

    pub(crate) fn selected_available_index(&self) -> Option<usize> {
        self.items
            .read()
            .iter()
            .find(|item| item.selected && item.available())
            .map(|item| item.index)
    }

    pub(crate) fn roving_tabindex(&self, index: usize) -> &'static str {
        if !self.is_available(index) {
            return "-1";
        }
        if !self.loops() {
            return "0";
        }
        // The roving anchor: the most recently focused item, else the selected
        // item, else the first available one. Mirrors React Aria's
        // `focusedKey ?? firstSelectedKey ?? firstKey`.
        let anchor = self
            .recent_focus()
            .filter(|&index| self.is_available(index))
            .or_else(|| self.selected_available_index());
        match anchor {
            Some(anchor) => {
                if anchor == index {
                    "0"
                } else {
                    "-1"
                }
            }
            // No focus and no selection: either every item is a tab stop
            // (native radio-group semantics) or just the first one.
            None if self.tabbable_when_empty => "0",
            None if self.first_available_index() == Some(index) => "0",
            None => "-1",
        }
    }

    pub(crate) fn focus_first(&mut self) {
        self.set_focus(self.first_available_index());
    }

    pub(crate) fn focus_last(&mut self) {
        self.set_focus(self.last_available_index());
    }

    pub(crate) fn focus_next(&mut self) {
        let indices = self.available_indices();
        self.set_focus(next_index_after(
            &indices,
            self.recent_focus(),
            self.loops(),
        ));
    }

    pub(crate) fn focus_prev(&mut self) {
        let indices = self.available_indices();
        self.set_focus(prev_index_before(
            &indices,
            self.recent_focus(),
            self.loops(),
        ));
    }

    /// Move focus to the next available item matching `predicate`, starting from
    /// the currently focused item. The collection owns the ordering; callers only
    /// describe which items qualify.
    pub(crate) fn focus_next_matching(&mut self, predicate: impl Fn(usize) -> bool) {
        let indices = self.available_indices_matching(predicate);
        self.set_focus(next_index_after(
            &indices,
            self.focused_index(),
            self.loops(),
        ));
    }

    /// Move focus to the previous available item matching `predicate`, starting
    /// from the currently focused item.
    pub(crate) fn focus_prev_matching(&mut self, predicate: impl Fn(usize) -> bool) {
        let indices = self.available_indices_matching(predicate);
        self.set_focus(prev_index_before(
            &indices,
            self.focused_index(),
            self.loops(),
        ));
    }

    pub(crate) fn try_focus_placement(&mut self, placement: CollectionPlacement) -> bool {
        // An order pass in flight may still renumber the items: "first" is
        // only known once it is done. Tracked, so the caller's effect
        // retries when it finishes.
        if (self.order.pending)() {
            return false;
        }
        let index = match placement {
            CollectionPlacement::First => self.first_available_index(),
            CollectionPlacement::Last => self.last_available_index(),
        };
        if let Some(index) = index {
            self.set_focus(Some(index));
            true
        } else {
            false
        }
    }

    fn control_mount_focus(&self, index: usize, controlled_ref: Signal<Option<Rc<MountedData>>>) {
        let controlled_ref = controlled_ref();
        if self.is_focused(index) && self.is_available(index) {
            if let Some(md) = controlled_ref {
                spawn(async move {
                    // focus-ok: roving focus moves the DOM focus with the collection's focused index
                    let _ = md.set_focus(true).await;
                });
            }
        }
    }

    fn available_indices(&self) -> Vec<usize> {
        self.available_indices_matching(|_| true)
    }

    fn available_indices_matching(&self, predicate: impl Fn(usize) -> bool) -> Vec<usize> {
        let mut indices: Vec<_> = self
            .items
            .read()
            .iter()
            .filter(|item| item.available() && predicate(item.index))
            .map(|item| item.index)
            .collect();
        indices.sort_unstable();
        indices.dedup();
        indices
    }

    fn is_known_unavailable(&self, index: usize) -> bool {
        let items = self.items.peek();
        let mut known = false;
        for item in items.iter().filter(|item| item.index == index) {
            known = true;
            if item.available() {
                return false;
            }
        }
        known
    }

    fn key_for_index(&self, index: usize) -> Option<String> {
        let items = self.items.peek();
        items
            .iter()
            .find(|item| item.index == index && item.available())
            .or_else(|| items.iter().find(|item| item.index == index))
            .and_then(|item| item.key.clone())
    }

    fn index_for_key(&self, key: &str) -> Option<usize> {
        self.items
            .peek()
            .iter()
            .find(|item| item.key.as_deref() == Some(key) && item.available())
            .map(|item| item.index)
    }

    fn clear_focus_if_unavailable(&mut self, index: usize, key: Option<&str>, available: bool) {
        if available {
            return;
        }
        let is_focused = match key {
            Some(key) => self.focus_key.peek().as_deref() == Some(key),
            None => self.focus_key.peek().is_none() && *self.focused.peek() == Some(index),
        };
        if is_focused {
            self.clear_focus();
        }
    }

    fn move_focus_for_reindexed_key(
        &mut self,
        previous_index: usize,
        index: usize,
        key: Option<&str>,
    ) {
        if previous_index == index {
            return;
        }

        let moving_focused_item = match key {
            Some(key) => self.focus_key.peek().as_deref() == Some(key),
            None => self.focus_key.peek().is_none() && *self.focused.peek() == Some(previous_index),
        };

        if moving_focused_item {
            self.focused.set(Some(index));
            self.recent.set(Some(index));
        }
    }

    fn adopt_focus_key_for_index(&mut self, index: usize, key: Option<String>) {
        if key.is_none() {
            return;
        }
        if *self.focused.peek() == Some(index) && self.focus_key.peek().is_none() {
            self.focus_key.set(key);
        }
    }

    fn is_focused_item(&self, item: &CollectionItemState) -> bool {
        match item.key.as_deref() {
            Some(key) => self.focus_key.peek().as_deref() == Some(key),
            None => self.focus_key.peek().is_none() && *self.focused.peek() == Some(item.index),
        }
    }
}

pub(crate) fn use_collection_provider(roving_loop: ReadSignal<bool>) -> CollectionState {
    use_collection_provider_with(roving_loop, CollectionOptions::default())
}

pub(crate) fn use_collection_provider_with(
    roving_loop: ReadSignal<bool>,
    options: CollectionOptions,
) -> CollectionState {
    let id = use_unique_id();
    use_context_provider(|| CollectionState::new(roving_loop, options, id))
}

/// An item's index in its collection, from [`use_item_index`].
#[derive(Clone, Copy)]
pub(crate) struct ItemIndex {
    /// The index, for [`collection_item`] and anything else the item keys by
    /// it.
    pub(crate) index: ReadSignal<usize>,
    /// The collection's marker prefix and this item's slot, for its
    /// `data-collection-item` attribute. `None` when the caller set the index.
    marker: Option<(Signal<String>, usize)>,
}

impl ItemIndex {
    /// The order marker attribute the DOM-order pass reads. Render it on the
    /// item's focusable element (the one whose document position is the
    /// item's place in keyboard order); empty when the caller set the index.
    pub(crate) fn marker_attribute(&self) -> Vec<Attribute> {
        self.marker
            .map(|(prefix, slot)| {
                let value = format!("{}{slot}", prefix.peek());
                Attribute::new(ORDER_MARKER, value, None, false)
            })
            .into_iter()
            .collect()
    }
}

/// The index of one item in `collection`, for [`collection_item`] (row 396).
///
/// Without an explicit `index` the item is numbered for its place in the
/// tree: in creation order at first, then, on the client, in document order
/// once its element (carrying [`ItemIndex::marker_attribute`]) is in the
/// document -- see [`CollectionOrder`]. An item mounted later (a menu's items
/// on open, an item appended to a list) is placed by the same pass.
///
/// `index` is the override for a caller that wants to own the order. Once
/// any item of the collection sets it, none is renumbered, so set it on
/// every item or on none.
///
/// A hook: call it unconditionally, once per render, before [`use_item`].
pub(crate) fn use_item_index(
    collection: CollectionState,
    index: ReadSignal<Option<usize>>,
) -> ItemIndex {
    let explicit = *index.peek();
    let slot = use_hook(move || {
        let mut collection = collection;
        collection.allocate_index(explicit)
    });
    // Owned by the provider, whose task the DOM-order pass writes it from;
    // dropped with the item.
    let derived = use_hook(move || Signal::new_in_scope(slot, collection.order.scope));
    let marker = use_hook(move || {
        explicit.is_none().then(|| {
            let mut derived_items = collection.order.derived;
            derived_items.write().insert(slot, derived);
            (collection.order.marker_prefix, slot)
        })
    });
    use_drop(move || {
        let mut derived_items = collection.order.derived;
        if let Ok(mut derived) = derived_items.try_write() {
            derived.remove(&slot);
        };
        derived.manually_drop();
    });
    // Before `use_item`'s own registration effect, so a deferred focus
    // placement sees the pass pending as soon as it sees the item.
    use_effect(move || {
        let mut collection = collection;
        collection.request_order_pass();
    });
    let index = use_memo(move || index().unwrap_or_else(|| *derived.read()));
    let index = use_hook(move || ReadSignal::new(index));
    ItemIndex { index, marker }
}

/// [`use_item_index`] for an item that takes no `index` override.
pub(crate) fn use_tree_item_index(collection: CollectionState) -> ItemIndex {
    let none = use_hook(|| ReadSignal::new(Signal::new(None)));
    use_item_index(collection, none)
}

/// Everything an item needs from its collection, returned by the single
/// per-item entry point [`collection_item`]. Mirrors React Aria's
/// `useSelectableItem`, which returns one `itemProps` bundle (tabindex, focus
/// handling, focused state) instead of forcing each component to hand-roll it.
#[derive(Clone, Copy)]
pub(crate) struct CollectionItem {
    /// Roving `tabindex` for the item's focusable element (`"0"` or `"-1"`).
    pub(crate) tabindex: Memo<&'static str>,
    focused: Memo<bool>,
    controlled_ref: Signal<Option<Rc<MountedData>>>,
    collection: CollectionState,
}

impl CollectionItem {
    /// Whether this item is the currently focused one.
    pub(crate) fn focused(&self) -> bool {
        (self.focused)()
    }

    /// A mounted handler that lets the collection drive DOM focus for this item.
    /// Attach it to the focusable element.
    pub(crate) fn onmounted(self) -> impl FnMut(MountedEvent) {
        let mut controlled_ref = self.controlled_ref;
        let mut collection = self.collection;
        move |event: MountedEvent| {
            controlled_ref.set(Some(event.data()));
            // The element may mount after its item (a listbox's options on
            // open): order it now that it is in the document.
            collection.request_order_pass();
        }
    }
}

/// Begin registering one collection item. Chain only the optional inputs the
/// component actually has — `key`, `disabled`, `hidden`, `selected` — then pass
/// the builder to [`use_item`]. Unset inputs default to "none".
///
/// This is a plain constructor and runs no hooks; the hook is [`use_item`].
pub(crate) fn collection_item(
    collection: CollectionState,
    index: impl Readable<Target = usize> + Copy + 'static,
) -> CollectionItemBuilder {
    CollectionItemBuilder {
        collection,
        index: Rc::new(move || index.cloned()),
        key: Rc::new(|| None),
        disabled: Rc::new(|| false),
        hidden: Rc::new(|| false),
        selected: Rc::new(|| false),
        text_value: Rc::new(|| None),
    }
}

/// Builder for a single collection item. Construct it with [`collection_item`]
/// and pass it to [`use_item`] to register.
#[must_use = "pass the builder to use_item() to register the item"]
pub(crate) struct CollectionItemBuilder {
    collection: CollectionState,
    index: Rc<dyn Fn() -> usize>,
    key: Rc<dyn Fn() -> Option<String>>,
    disabled: Rc<dyn Fn() -> bool>,
    hidden: Rc<dyn Fn() -> bool>,
    selected: Rc<dyn Fn() -> bool>,
    text_value: Rc<dyn Fn() -> Option<String>>,
}

impl CollectionItemBuilder {
    /// Whether the item is currently disabled (skipped by roving focus).
    pub(crate) fn disabled(mut self, disabled: impl Fn() -> bool + 'static) -> Self {
        self.disabled = Rc::new(disabled);
        self
    }

    /// The item key, used for focus identity and `focused_key()` lookups.
    pub(crate) fn key(mut self, key: impl Fn() -> Option<String> + 'static) -> Self {
        self.key = Rc::new(key);
        self
    }

    /// Whether the item is currently hidden (e.g. filtered out or removed).
    pub(crate) fn hidden(mut self, hidden: impl Fn() -> bool + 'static) -> Self {
        self.hidden = Rc::new(hidden);
        self
    }

    /// Whether this item is the selected one. When nothing is focused yet, the
    /// selected item becomes the roving tab stop — mirrors React Aria seeding
    /// `focusedKey` from `firstSelectedKey`.
    pub(crate) fn selected(mut self, selected: impl Fn() -> bool + 'static) -> Self {
        self.selected = Rc::new(selected);
        self
    }

    /// Optional label text for this item, read back (in bulk, across every
    /// registered item) by [`CollectionState::text_entries`]. Unset by
    /// default (`None`) — nothing reads this unless a caller opts in, so
    /// a `collection_item(...)` call that never chains this is unaffected.
    pub(crate) fn text_value(mut self, text_value: impl Fn() -> Option<String> + 'static) -> Self {
        self.text_value = Rc::new(text_value);
        self
    }
}

/// Register an item builder and return its handle: a roving `tabindex`,
/// `focused()` state, and an `onmounted()` focus handler. This is the hook — it
/// calls `use_effect`/`use_signal`/`use_memo`, so call it unconditionally, once
/// per render.
pub(crate) fn use_item(builder: CollectionItemBuilder) -> CollectionItem {
    let CollectionItemBuilder {
        mut collection,
        index,
        key,
        disabled,
        hidden,
        selected,
        text_value,
    } = builder;

    let mut previous_item: Signal<Option<CollectionItemState>> = use_signal(|| None);

    use_effect({
        let index = index.clone();
        let disabled = disabled.clone();
        let hidden = hidden.clone();
        move || {
            let item = CollectionItemState {
                index: index(),
                key: key(),
                disabled: disabled(),
                hidden: hidden(),
                selected: selected(),
                text_value: text_value(),
            };
            let stale_item = previous_item.peek().clone();
            if let Some(stale_item) = stale_item {
                if !stale_item.same_identity(&item) {
                    collection.unregister_item(&stale_item);
                }
            }
            collection.register_item(item.clone());
            previous_item.set(Some(item));
        }
    });

    use_effect_cleanup(move || {
        if let Some(item) = previous_item.peek().as_ref() {
            collection.unregister_item(item);
        }
    });

    // Keep DOM focus in sync when this item becomes focused. The handler that
    // populates `controlled_ref` may be attached in a different component.
    let controlled_ref: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    use_effect({
        let index = index.clone();
        move || {
            if disabled() || hidden() {
                return;
            }
            collection.control_mount_focus(index(), controlled_ref);
        }
    });

    let tabindex = use_memo({
        let index = index.clone();
        move || collection.roving_tabindex(index())
    });
    let focused = use_memo(move || collection.is_focused(index()));

    CollectionItem {
        tabindex,
        focused,
        controlled_ref,
        collection,
    }
}

pub(crate) fn use_deferred_collection_focus(
    mut collection: CollectionState,
    mut placement: Signal<Option<CollectionPlacement>>,
    active: impl Fn() -> bool + Copy + 'static,
) {
    use_effect(move || {
        if !active() {
            placement.set(None);
            return;
        }
        // `.peek()`, not `()`: every caller sets `placement` before it
        // flips `active` from false to true (`*Context::open_with_focus`'s
        // "never the other order" doc, e.g. `dropdown_menu.rs`), so
        // `active()` above is already this effect's trigger for a fresh
        // request, and the retry once the collection has items to focus
        // is driven by `try_focus_placement`'s own tracked reads of the
        // collection below, not by re-reading `placement`. Tracking it
        // here would subscribe this effect to the value it writes two
        // lines down (`scripts/check-self-subscribing-effects.sh`,
        // dev-docs/backlog.md row 73's class).
        let Some(placement_value) = *placement.peek() else {
            return;
        };
        if collection.try_focus_placement(placement_value) {
            placement.set(None);
        }
    });
}

fn sync_item(items: &mut Vec<CollectionItemState>, item: CollectionItemState) {
    if let Some(position) = items
        .iter()
        .position(|existing| existing.same_identity(&item))
    {
        items.remove(position);
    }
    let insert_at = items.partition_point(|existing| existing.index <= item.index);
    items.insert(insert_at, item);
}

fn next_index_after(indices: &[usize], current: Option<usize>, roving_loop: bool) -> Option<usize> {
    match current {
        Some(current) => {
            let next_position = indices.partition_point(|&index| index <= current);
            indices
                .get(next_position)
                .copied()
                .or_else(|| roving_loop.then(|| indices.first().copied()).flatten())
        }
        None => indices.first().copied(),
    }
}

fn prev_index_before(
    indices: &[usize],
    current: Option<usize>,
    roving_loop: bool,
) -> Option<usize> {
    match current {
        Some(current) => {
            let prev_position = indices.partition_point(|&index| index < current);
            prev_position
                .checked_sub(1)
                .and_then(|position| indices.get(position).copied())
                .or_else(|| roving_loop.then(|| indices.last().copied()).flatten())
        }
        None if roving_loop => indices.last().copied(),
        None => indices.first().copied(),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;
    use crate::test_support::find_elements;

    thread_local! {
        static COLLECTION: Cell<Option<CollectionState>> = const { Cell::new(None) };
    }

    #[component]
    fn Root(children: Element) -> Element {
        let collection = use_collection_provider(ReadSignal::new(Signal::new(true)));
        COLLECTION.set(Some(collection));
        rsx! { {children} }
    }

    #[component]
    fn Probe(label: &'static str, #[props(default)] index: ReadSignal<Option<usize>>) -> Element {
        let collection: CollectionState = use_context();
        let order = use_item_index(collection, index);
        let marker = order.marker_attribute();
        rsx! {
            span { "data-label": label, "data-index": "{order.index}", ..marker }
        }
    }

    fn render(dom: &VirtualDom) -> String {
        dioxus_ssr::render(dom)
    }

    /// `(label, index, marker)` of every probe, in document order.
    fn probes(html: &str) -> Vec<(String, usize, Option<String>)> {
        find_elements(html, |a| a.contains_key("data-label"))
            .into_iter()
            .map(|el| {
                let a = el.attrs;
                (
                    a["data-label"].clone(),
                    a["data-index"].parse().unwrap(),
                    a.get(ORDER_MARKER).cloned(),
                )
            })
            .collect()
    }

    fn labels_by_index(html: &str) -> Vec<String> {
        let mut probes = probes(html);
        probes.sort_by_key(|(_, index, _)| *index);
        probes.into_iter().map(|(label, _, _)| label).collect()
    }

    #[test]
    fn template_roots_and_loops_are_numbered_in_tree_order() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                Root {
                    Probe { label: "a" }
                    for label in ["b", "c"] {
                        Probe { label }
                    }
                    Probe { label: "d" }
                }
            }
        });
        dom.rebuild_in_place();
        assert_eq!(labels_by_index(&render(&dom)), ["a", "b", "c", "d"]);
    }

    /// Why the DOM-order pass exists: siblings nested in one element of a
    /// template are created, and so numbered, in reverse.
    #[test]
    fn siblings_nested_in_one_element_are_created_in_reverse() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                Root {
                    div {
                        Probe { label: "a" }
                        Probe { label: "b" }
                        Probe { label: "c" }
                    }
                }
            }
        });
        dom.rebuild_in_place();
        assert_eq!(labels_by_index(&render(&dom)), ["c", "b", "a"]);
    }

    #[test]
    fn the_dom_order_pass_renumbers_into_a_fresh_ascending_range() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                Root {
                    div {
                        Probe { label: "a" }
                        Probe { label: "b" }
                        Probe { label: "c" }
                    }
                }
            }
        });
        dom.rebuild_in_place();
        let html = render(&dom);
        let before = probes(&html);
        // The slots the markers carry, in document order: what the pass's
        // script returns.
        let slots: Vec<usize> = before
            .iter()
            .map(|(_, _, marker)| {
                let marker = marker
                    .as_deref()
                    .expect("a derived item renders its marker");
                marker.rsplit(':').next().unwrap().parse().unwrap()
            })
            .collect();
        let mut collection = COLLECTION.get().unwrap();
        dom.in_runtime(|| collection.apply_dom_order(&slots));
        dom.render_immediate_to_vec();
        let after = probes(&render(&dom));
        assert_eq!(labels_by_index(&render(&dom)), ["a", "b", "c"]);
        let lowest_after = after.iter().map(|(_, index, _)| *index).min().unwrap();
        let highest_before = before.iter().map(|(_, index, _)| *index).max().unwrap();
        assert!(lowest_after > highest_before, "{before:?} -> {after:?}");
        // Already in order: a second pass changes nothing.
        dom.in_runtime(|| collection.apply_dom_order(&slots));
        dom.render_immediate_to_vec();
        assert_eq!(probes(&render(&dom)), after);
    }

    #[test]
    fn an_explicit_index_is_used_as_given_and_renders_no_marker() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                Root {
                    Probe { label: "a", index: 7usize }
                    Probe { label: "b" }
                }
            }
        });
        dom.rebuild_in_place();
        let probes = probes(&render(&dom));
        assert_eq!(probes[0].1, 7);
        assert_eq!(probes[0].2, None);
        // The counter moved past the explicit index.
        assert_eq!(probes[1].1, 8);
        assert!(*COLLECTION.get().unwrap().order.explicit.peek());
    }
}
