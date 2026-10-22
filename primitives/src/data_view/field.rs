//! [`Field`]: how a row sorts and which text the filter matches.

use super::derive::RowCompare;
use std::cmp::Ordering;
use std::rc::Rc;

type Comparator<T> = Rc<dyn Fn(&T, &T) -> Ordering>;
type FilterText<T> = Rc<dyn Fn(&T) -> String>;

/// One queryable property of a row: an id, an optional typed comparator and an
/// optional filter text. A list that only reorders needs no field; a table
/// column (`Column<T>` in the Data Table) is a field plus header, cell, width
/// and alignment.
pub struct Field<T: 'static> {
    /// Stable id: what the sort state refers to. Unique within a view.
    pub id: String,
    compare: Option<Comparator<T>>,
    filter_text: Option<FilterText<T>>,
}

impl<T: 'static> Field<T> {
    /// A field that neither sorts nor filters.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            compare: None,
            filter_text: None,
        }
    }

    /// Makes the field sortable by a typed key, so a sort cannot name a type
    /// that does not order.
    #[must_use]
    pub fn sortable<K: Ord>(mut self, key: impl Fn(&T) -> K + 'static) -> Self {
        self.compare = Some(Rc::new(move |a, b| key(a).cmp(&key(b))));
        self
    }

    /// Makes the view's filter match this field's text (a row matches when any
    /// filterable field does).
    #[must_use]
    pub fn filterable(mut self, text: impl Fn(&T) -> String + 'static) -> Self {
        self.filter_text = Some(Rc::new(text));
        self
    }

    /// Whether the field has a comparator.
    pub fn is_sortable(&self) -> bool {
        self.compare.is_some()
    }

    /// Whether the field has a filter text.
    pub fn is_filterable(&self) -> bool {
        self.filter_text.is_some()
    }

    /// The ascending comparator, if any.
    pub fn comparator(&self) -> Option<&RowCompare<'_, T>> {
        self.compare.as_deref()
    }

    /// The filter text of `row`, if the field is filterable.
    pub fn text_of(&self, row: &T) -> Option<String> {
        self.filter_text.as_ref().map(|f| f(row))
    }
}

impl<T: 'static> Clone for Field<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            compare: self.compare.clone(),
            filter_text: self.filter_text.clone(),
        }
    }
}

impl<T: 'static> PartialEq for Field<T> {
    fn eq(&self, other: &Self) -> bool {
        fn same<X: ?Sized>(a: &Option<Rc<X>>, b: &Option<Rc<X>>) -> bool {
            match (a, b) {
                (Some(a), Some(b)) => Rc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
        }
        self.id == other.id
            && same(&self.compare, &other.compare)
            && same(&self.filter_text, &other.filter_text)
    }
}

impl<T: 'static> std::fmt::Debug for Field<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Field")
            .field("id", &self.id)
            .field("sortable", &self.is_sortable())
            .field("filterable", &self.is_filterable())
            .finish()
    }
}
