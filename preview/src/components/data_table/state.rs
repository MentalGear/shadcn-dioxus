//! Pure state helpers for the Data Table composition pattern
//! (dev-docs/component-backlog.md row 69: "a composition pattern, not a
//! new primitive").
//!
//! Deliberately plain, dioxus-free generic functions over slices and
//! closures, not signals/hooks -- the caller (`component.rs`'s themed
//! composition pieces, or a consumer's own demo like
//! `variants/main/mod.rs`) owns the reactive state and calls these as
//! ordinary Rust to derive what to render. Since row 360 the helpers themselves live in
//! `dioxus_primitives::data_view` (one model for every view) and this file re-exports them under the
//! names this component always had; kept as a sibling file of
//! `component.rs` rather than folded into it so this logic can be unit
//! tested in isolation (`#[cfg(test)] mod tests` below) -- the
//! `dev-docs/plan.md` definition of done's "a conformance/regression test
//! that ... would fail without the change" for this lane's pure-logic
//! half (the composition components themselves are covered by
//! `playwright/data_table.spec.ts` and the tier-1 APG oracle instead).
//!
//! Module wiring note: `component.rs` is itself a *file* module
//! (`preview/src/components/mod.rs`'s `examples!` macro declares
//! `pub(crate) mod component;` inline inside an already-inline
//! `mod data_table { ... }` block), so a plain `mod state;` written inside
//! `component.rs` would resolve to `data_table/component/state.rs` (a
//! nested `component/` subdirectory), not this file. `component.rs`
//! instead writes `#[path = "state.rs"] mod state;`, which is relative to
//! `component.rs`'s own directory and so resolves to this file, a sibling
//! of `component.rs` -- the same directory `shadcn-dioxus add data_table`
//! copies verbatim, so the copied module keeps compiling with zero extra
//! wiring in a consumer's project.

pub use dioxus_primitives::data_view::{
    filter_rows, paginate, sort_rows, sort_rows_by, tri_state, SortDirection,
};

/// Whether `needle` appears anywhere in `haystack`, case-insensitively,
/// after trimming `needle`'s surrounding whitespace. This is the Data
/// Table demo's exact filter-matching semantics (see `variants/main/
/// mod.rs`'s `Demo`, which drives [`filter_rows`] with
/// `includes_string(p.email, &query)` on the `email` column, and
/// `docs.md`) -- named and unit-tested here so the demo can't silently
/// drift from the documented behavior.
///
/// This mirrors shadcn/ui's own Data Table example, which filters through
/// TanStack Table's default `includesString` filter function --
/// `String(value).toLowerCase().includes(filterValue.toLowerCase())`, a
/// case-insensitive substring match anywhere in the cell value, *not*
/// restricted to the start of the string. (Answers the question this was
/// written for: no, matching only the start of the string was never the
/// intent here or in shadcn -- a mid-string fragment is expected to narrow
/// the rows exactly like a prefix would.)
///
/// One deliberate addition beyond TanStack's own filter fn: `needle` is
/// trimmed before matching, so incidental leading/trailing whitespace (a
/// stray space from a mobile keyboard, a pasted value) doesn't turn a
/// query that would otherwise match into one that matches nothing just
/// because email addresses never contain a literal space. An empty (or
/// all-whitespace, since trimming collapses it to empty too) `needle`
/// matches everything -- `str::contains("")` is already `true` for any
/// haystack, so "empty query = no filter" falls out of this one-line
/// check rather than needing its own branch.
pub fn includes_string(haystack: &str, needle: &str) -> bool {
    dioxus_primitives::data_view::contains_query(haystack, needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn includes_string_matches_a_prefix() {
        assert!(includes_string("alexandra@example.com", "al"));
    }

    #[test]
    fn includes_string_matches_an_uppercase_prefix() {
        assert!(includes_string("alexandra@example.com", "AL"));
    }

    #[test]
    fn includes_string_matches_a_mid_string_fragment_not_just_a_prefix() {
        // "rek" is the tail of "derek", not a prefix of anything -- a
        // filter that only matched string starts would reject this.
        assert!(includes_string("derek@example.com", "rek"));
    }

    #[test]
    fn includes_string_is_case_insensitive_on_both_sides() {
        assert!(includes_string("CARMELLA@EXAMPLE.COM", "rmella"));
    }

    #[test]
    fn includes_string_trims_surrounding_whitespace_from_the_query() {
        assert!(includes_string("derek@example.com", "  derek  "));
    }

    #[test]
    fn includes_string_empty_query_matches_everything() {
        assert!(includes_string("anything@example.com", ""));
    }

    #[test]
    fn includes_string_whitespace_only_query_matches_everything() {
        assert!(includes_string("anything@example.com", "   "));
    }

    #[test]
    fn includes_string_no_match_returns_false() {
        assert!(!includes_string("derek@example.com", "zzz"));
    }
}
