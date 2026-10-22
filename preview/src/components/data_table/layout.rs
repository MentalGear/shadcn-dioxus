//! `ColumnLayout`: which columns a data table shows and in what order (backlog
//! row 363). Pure data with pure transitions, like `ViewState` (the row half of
//! the same model): the order is the shared `KeyOrder`, so a column move is the
//! same permutation rule a row move is, and visibility is a set of hidden ids.
//! Same `#[path]` sibling wiring as `state.rs`.

use dioxus_primitives::data_view::KeyOrder;
use std::collections::HashSet;

/// The column order and the hidden columns of one table. The order may list
/// ids that no longer exist and miss ids that are new: every read reconciles it
/// against the ids the table has, so it never depends on the caller keeping it
/// in step.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ColumnLayout {
    /// The manual column order (`None` until a column is moved: source order).
    pub order: Option<KeyOrder>,
    /// Ids of the hidden columns.
    pub hidden: HashSet<String>,
}

impl ColumnLayout {
    /// A layout in source order with `hidden` columns hidden.
    pub fn with_hidden<'a>(hidden: impl IntoIterator<Item = &'a str>) -> Self {
        Self { order: None, hidden: hidden.into_iter().map(str::to_string).collect() }
    }

    /// Whether the column `id` is hidden.
    pub fn is_hidden(&self, id: &str) -> bool {
        self.hidden.contains(id)
    }

    /// Shows or hides the column `id`.
    pub fn set_hidden(&mut self, id: &str, hidden: bool) {
        if hidden {
            self.hidden.insert(id.to_string());
        } else {
            self.hidden.remove(id);
        }
    }

    /// Every id of `ids` (source order) arranged by the layout's order, hidden
    /// ones included: the order the Columns menu lists them in.
    pub fn arranged<'a>(&self, ids: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
        match &self.order {
            Some(order) => order.reconcile(ids),
            None => ids.into_iter().collect(),
        }
    }

    /// The ids of `ids` that are shown, in layout order: the table's columns.
    pub fn visible<'a>(&self, ids: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
        let mut arranged = self.arranged(ids);
        arranged.retain(|id| !self.is_hidden(id));
        arranged
    }

    /// Moves the column `id` one place `delta` (negative = earlier) among
    /// `listed`, the ids the Columns menu offers (a subset of `all`, the ids of
    /// every column in source order): it swaps with the listed neighbour, so a
    /// column the menu does not list keeps its place and a hidden one is moved
    /// past like any other. Stops at the ends. Returns the 0-based position the
    /// column now has among `listed` (in layout order), or `None` when nothing
    /// moved (an end, or an id that is not listed).
    pub fn move_listed(&mut self, all: &[&str], listed: &[&str], id: &str, delta: isize) -> Option<usize> {
        let arranged_all = self.arranged(all.iter().copied());
        let arranged: Vec<&str> = arranged_all.iter().copied().filter(|c| listed.contains(c)).collect();
        let at = arranged.iter().position(|c| *c == id)?;
        let neighbour = *arranged.get(at.checked_add_signed(delta)?)?;
        let mut order = KeyOrder::new(arranged_all.iter().map(|c| c.to_string()));
        let moved = if delta < 0 { order.move_before(id, neighbour) } else { order.move_after(id, neighbour) };
        if !moved {
            return None;
        }
        self.order = Some(order);
        at.checked_add_signed(delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDS: [&str; 4] = ["id", "status", "email", "amount"];

    #[test]
    fn source_order_with_nothing_hidden_shows_every_column() {
        let layout = ColumnLayout::default();
        assert_eq!(layout.visible(IDS), IDS);
        assert_eq!(layout.arranged(IDS), IDS);
    }

    #[test]
    fn hidden_columns_leave_visible_but_stay_arranged() {
        let mut layout = ColumnLayout::with_hidden(["status"]);
        assert_eq!(layout.visible(IDS), ["id", "email", "amount"]);
        assert_eq!(layout.arranged(IDS), IDS);
        layout.set_hidden("status", false);
        layout.set_hidden("amount", true);
        assert_eq!(layout.visible(IDS), ["id", "status", "email"]);
    }

    #[test]
    fn a_move_swaps_with_the_listed_neighbour_and_stops_at_the_ends() {
        let mut layout = ColumnLayout::default();
        assert_eq!(layout.move_listed(&IDS, &IDS, "email", -1), Some(1));
        assert_eq!(layout.arranged(IDS), ["id", "email", "status", "amount"]);
        assert_eq!(layout.move_listed(&IDS, &IDS, "email", 1), Some(2));
        assert_eq!(layout.arranged(IDS), IDS);
        assert_eq!(layout.move_listed(&IDS, &IDS, "id", -1), None);
        assert_eq!(layout.move_listed(&IDS, &IDS, "amount", 1), None);
        assert_eq!(layout.move_listed(&IDS, &IDS, "nope", 1), None);
    }

    #[test]
    fn an_unlisted_column_keeps_its_place_and_a_hidden_one_is_moved_past() {
        // `id` is not offered by the menu: `email` moving up swaps with `status`, not `id`.
        let listed = ["status", "email", "amount"];
        let mut layout = ColumnLayout::with_hidden(["status"]);
        assert_eq!(layout.move_listed(&IDS, &listed, "email", -1), Some(0));
        assert_eq!(layout.arranged(IDS), ["id", "email", "status", "amount"]);
        assert_eq!(layout.visible(IDS), ["id", "email", "amount"]);
    }

    #[test]
    fn a_stale_order_is_reconciled_when_columns_change() {
        let mut layout = ColumnLayout::default();
        layout.move_listed(&IDS, &IDS, "amount", -2);
        assert_eq!(layout.arranged(IDS), ["id", "amount", "status", "email"]);
        // `status` removed, `notes` added later: listed ones keep their order, new ones follow.
        let now = ["id", "email", "amount", "notes"];
        assert_eq!(layout.arranged(now), ["id", "amount", "email", "notes"]);
    }
}
