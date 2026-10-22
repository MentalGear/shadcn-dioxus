//! [`VirtualizeMode`]: the one "how are off-screen rows kept cheap" type of every virtual
//! consumer (backlog row 343). `VirtualList`, the data body of `Table` ([`crate::table_rows`])
//! and, from row 341, `VirtualGrid` all take the same `virtualize` and `auto_threshold` props, so
//! no consumer has its own threshold logic, its own default, or no `Auto`.
//!
//! # The three modes
//!
//! | | `ContentVisibility` | `Windowed` |
//! |---|---|---|
//! | In the DOM | every row | the rows in view plus an overscan |
//! | Server-rendered HTML | every row | none |
//! | Find-in-page, selection | reach every row | only the mounted window |
//! | Accessibility tree | every row | only the window (rows announce `n of N`) |
//! | Cost grows with the row count | yes (DOM and memory are linear) | no |
//!
//! [`VirtualizeMode::Auto`] (the default) is `ContentVisibility` while `count <= auto_threshold`
//! and `Windowed` above it.

/// The row count up to which [`VirtualizeMode::Auto`] keeps every row in the DOM.
///
/// 1,000 because the cost of keeping every row grows with the count: mounting them is one
/// main-thread task proportional to the row count (about 90 ms for 1,000 rows and 205 ms, one
/// 147 ms task, for 5,000 in the `VirtualList` demo, release build), while a window mounts about
/// 40 rows whatever the count. Raise it for lists of cheap rows, lower it for heavy ones.
pub const DEFAULT_AUTO_THRESHOLD: usize = 1_000;

/// How a virtual component keeps off-screen rows cheap. See the module doc's table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VirtualizeMode {
    /// [`ContentVisibility`](Self::ContentVisibility) while `count <= auto_threshold`,
    /// [`Windowed`](Self::Windowed) above it. The default.
    #[default]
    Auto,
    /// Every row stays in the DOM (and in the server's HTML). A list lets the browser skip the
    /// ones that are off-screen (`content-visibility: auto`); a `Table` cannot, because
    /// browsers do not apply `content-visibility` to table rows or row groups, so there it is
    /// every row in the DOM and nothing more. Native find-in-page, selection, focus and the
    /// accessibility tree reach every row (find-in-page: not Safari 18-25 for skipped content).
    /// DOM and memory grow with the row count.
    ContentVisibility,
    /// Only the rows in view plus an overscan are mounted. Flat cost for any row count; rows
    /// outside the window are unreachable by find-in-page and the server renders none.
    Windowed,
}

impl VirtualizeMode {
    /// The `virtualize` prop's wire spelling: `"auto"`, `"content-visibility"` or `"windowed"`.
    /// A rendered component's `data-virtualize` is always one of the last two.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::ContentVisibility => "content-visibility",
            Self::Windowed => "windowed",
        }
    }

    /// The concrete mode for `count` rows: `self` unless it is `Auto`, which becomes
    /// `ContentVisibility` up to `auto_threshold` rows and `Windowed` above. Never returns
    /// `Auto`.
    pub fn resolve(self, count: usize, auto_threshold: usize) -> Self {
        match self {
            Self::Auto if count <= auto_threshold => Self::ContentVisibility,
            Self::Auto => Self::Windowed,
            concrete => concrete,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_picks_by_count_and_threshold() {
        let auto = VirtualizeMode::Auto;
        assert_eq!(auto.resolve(0, 1_000), VirtualizeMode::ContentVisibility);
        assert_eq!(
            auto.resolve(1_000, 1_000),
            VirtualizeMode::ContentVisibility
        );
        assert_eq!(auto.resolve(1_001, 1_000), VirtualizeMode::Windowed);
        assert_eq!(auto.resolve(100, 10), VirtualizeMode::Windowed);
    }

    #[test]
    fn an_explicit_mode_ignores_the_count() {
        for count in [0, 1, 1_000, 1_001, 1_000_000] {
            assert_eq!(
                VirtualizeMode::ContentVisibility.resolve(count, 1_000),
                VirtualizeMode::ContentVisibility
            );
            assert_eq!(
                VirtualizeMode::Windowed.resolve(count, 1_000),
                VirtualizeMode::Windowed
            );
        }
    }

    #[test]
    fn the_default_is_auto_and_resolve_never_returns_it() {
        assert_eq!(VirtualizeMode::default(), VirtualizeMode::Auto);
        for count in [0, 1, 999, 1_000, 1_001, usize::MAX] {
            for mode in [
                VirtualizeMode::Auto,
                VirtualizeMode::ContentVisibility,
                VirtualizeMode::Windowed,
            ] {
                assert_ne!(
                    mode.resolve(count, DEFAULT_AUTO_THRESHOLD),
                    VirtualizeMode::Auto
                );
            }
        }
    }

    #[test]
    fn wire_spellings_match_message_scroller() {
        assert_eq!(
            VirtualizeMode::ContentVisibility.as_str(),
            "content-visibility"
        );
        assert_eq!(
            VirtualizeMode::ContentVisibility.as_str(),
            crate::message_scroller::Virtualization::ContentVisibility.as_str()
        );
        assert_eq!(VirtualizeMode::Windowed.as_str(), "windowed");
        assert_eq!(VirtualizeMode::Auto.as_str(), "auto");
    }
}
