//! The one definition of how a component page groups its docs sections (backlog row 422).
//!
//! `build.rs` and `src/main.rs` include this file with `#[path]`, so the order the generator writes the
//! sections in, the groups `site/component_page.rs` renders and the test that pins both come from the same
//! lists and cannot drift apart. A `docs.md` names no group; the group follows from the section's id.
//!
//! - **Basic**: [`BASIC_ORDER`], the same four sections in the same order on every page. Installation is
//!   written by the page itself (it holds components), the other three are `build.rs` data.
//! - **Component-Specific**: every other section, in the `docs.md` authored order, then the recurring topics
//!   of [`TAIL`] in that order. A topic is matched by its heading's prefix ([`tail_rank`]).
//!
//! `std` only.
#![allow(
    dead_code,
    reason = "build.rs and main.rs include this file and each uses a different subset"
)]

/// The heading of the group every page shares.
pub const BASIC_HEADING: &str = "Basic";
/// The heading of the group of a page's own sections.
pub const SPECIFIC_HEADING: &str = "Component-Specific";

/// A section every component page has, in [`BASIC_ORDER`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Basic {
    /// The text before a `docs.md`'s first `##`. Open on load.
    UsageNotes,
    /// The install commands and the manual files, written by `ComponentHighlight`.
    Installation,
    /// The `docs.md` section named "Component Structure" (the parts and how they nest).
    ComponentStructure,
    /// The generated props tables.
    PropsReference,
}

/// The Basic group, top to bottom.
pub const BASIC_ORDER: [Basic; 4] = [
    Basic::UsageNotes,
    Basic::Installation,
    Basic::ComponentStructure,
    Basic::PropsReference,
];

impl Basic {
    /// The heading the page shows. A `docs.md` writes its "Component Structure" `##` in any case and
    /// `build.rs` retitles it to this text, so the four headings read alike.
    pub const fn heading(self) -> &'static str {
        match self {
            Basic::UsageNotes => "Usage notes",
            Basic::Installation => "Installation",
            Basic::ComponentStructure => "Component structure",
            Basic::PropsReference => "Props reference",
        }
    }

    /// The `h3` id of the section: the anchor target.
    pub const fn id(self) -> &'static str {
        match self {
            Basic::UsageNotes => "docs-usage-notes",
            Basic::Installation => "installation",
            Basic::ComponentStructure => "docs-component-structure",
            Basic::PropsReference => "docs-props-reference",
        }
    }

    /// Whether the section is open when the page loads.
    pub const fn open_by_default(self) -> bool {
        matches!(self, Basic::UsageNotes)
    }
}

/// The Basic section a section id names, `None` for a Component-Specific section.
pub fn basic_of_id(id: &str) -> Option<Basic> {
    BASIC_ORDER.into_iter().find(|basic| basic.id() == id)
}

/// The recurring topics that close the Component-Specific group, in page order. A section whose heading
/// starts with one of these (whole word, any case: "Motion and visibility" is Motion, "Motionless" is not)
/// moves behind the page's own sections.
pub const TAIL: [&str; 5] = [
    "Keyboard",
    "Accessibility",
    "Motion",
    "Direction / RTL",
    "Styling",
];

/// The position of the [`TAIL`] topic a heading starts with, `None` for a page's own section.
pub fn tail_rank(title: &str) -> Option<usize> {
    let title = title.trim().to_ascii_lowercase();
    TAIL.iter().position(|topic| {
        let topic = topic.to_ascii_lowercase();
        title.strip_prefix(&topic).is_some_and(|rest| {
            rest.chars()
                .next()
                .is_none_or(|next| !next.is_ascii_alphanumeric())
        })
    })
}

/// Where a section sits on the page: sorted ascending (the sort is stable), this is the page order. The
/// Basic group first (by [`BASIC_ORDER`]), then the page's own sections (all equal, so authored order
/// holds), then the [`TAIL`] topics. `title` is the heading's text, not its HTML.
pub fn page_position(id: &str, title: &str) -> (u8, usize) {
    if let Some(basic) = basic_of_id(id) {
        let at = BASIC_ORDER.iter().position(|b| *b == basic).unwrap_or(0);
        return (0, at);
    }
    match tail_rank(title) {
        Some(rank) => (1, 1 + rank),
        None => (1, 0),
    }
}

/// A heading's inline HTML as plain text (`<code>Foo</code> bar` -> `Foo bar`), for [`tail_rank`].
pub fn plain_text(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tail_topics_match_by_whole_word_prefix() {
        assert_eq!(tail_rank("Keyboard"), Some(0));
        assert_eq!(tail_rank("Keyboard interaction"), Some(0));
        assert_eq!(tail_rank("Keyboard and focus"), Some(0));
        assert_eq!(tail_rank("accessibility"), Some(1));
        assert_eq!(tail_rank("Motion and visibility"), Some(2));
        assert_eq!(tail_rank("Direction / RTL"), Some(3));
        assert_eq!(tail_rank("Styling (slot variables)"), Some(4));
        assert_eq!(tail_rank("Styling the label"), Some(4));
        assert_eq!(tail_rank("Motionless"), None);
        assert_eq!(tail_rank("Seeing RTL"), None);
        assert_eq!(tail_rank("Layout Notes"), None);
    }

    #[test]
    fn page_position_orders_basic_then_own_then_tail() {
        let mut sections = [
            ("docs-styling-slot-variables", "Styling (slot variables)"),
            ("docs-props-reference", "Props reference"),
            ("docs-accessibility", "Accessibility"),
            ("docs-layout-notes", "Layout Notes"),
            ("docs-component-structure", "Component structure"),
            ("docs-keyboard", "Keyboard"),
            ("docs-size", "Size"),
            ("docs-usage-notes", "Usage notes"),
        ];
        sections.sort_by_key(|(id, title)| page_position(id, title));
        let ids: Vec<&str> = sections.iter().map(|(id, _)| *id).collect();
        assert_eq!(
            ids,
            [
                "docs-usage-notes",
                "docs-component-structure",
                "docs-props-reference",
                "docs-layout-notes",
                "docs-size",
                "docs-keyboard",
                "docs-accessibility",
                "docs-styling-slot-variables",
            ]
        );
    }

    #[test]
    fn plain_text_drops_tags() {
        assert_eq!(plain_text("<code>Foo</code> bar"), "Foo bar");
    }
}
