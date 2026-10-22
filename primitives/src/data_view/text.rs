//! The one "the text contains the query" rule.

/// The query as the matcher compares it: trimmed and lowercased.
pub fn normalize_query(query: &str) -> String {
    query.trim().to_lowercase()
}

/// [`contains_query`] against a query already passed through
/// [`normalize_query`], so a loop over many rows normalizes the query once.
pub fn contains_query_normalized(haystack: &str, normalized_query: &str) -> bool {
    normalized_query.is_empty() || haystack.to_lowercase().contains(normalized_query)
}

/// Whether `query` appears anywhere in `haystack`, case-insensitively, after
/// trimming the query's surrounding whitespace. An empty or all-whitespace
/// query matches everything.
///
/// This is shadcn/ui's Data Table `includesString` (TanStack's default filter
/// function, `String(value).toLowerCase().includes(filterValue.toLowerCase())`:
/// a substring anywhere, not a prefix) plus the one deliberate addition of the
/// trim, so a stray space from a mobile keyboard or a paste does not turn a
/// matching query into one that matches nothing. The Data Table filter, the
/// email client's search and the Combobox default filter all call this.
pub fn contains_query(haystack: &str, query: &str) -> bool {
    contains_query_normalized(haystack, &normalize_query(query))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substring_case_insensitive_trimmed_and_empty_matches_all() {
        assert!(contains_query("alexandra@example.com", "al"));
        assert!(contains_query("alexandra@example.com", "AL"));
        assert!(contains_query("derek@example.com", "rek"));
        assert!(contains_query("CARMELLA@EXAMPLE.COM", "rmella"));
        assert!(contains_query("derek@example.com", "  derek  "));
        assert!(contains_query("anything", ""));
        assert!(contains_query("anything", "   "));
        assert!(!contains_query("derek@example.com", "zzz"));
    }
}
