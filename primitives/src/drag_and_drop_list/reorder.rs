//! The reorder gesture, apart from what is reordered (backlog row 362,
//! `dev-docs/research/collection-model-2026-10-22.md` piece 7).
//!
//! A pure state machine plus the one key map and the one set of
//! announcements: a pointer drag and the keyboard (Space or Enter lifts, the
//! arrows move, Space or Enter drops, Escape cancels) drive the same
//! [`ReorderState`], and a drop resolves to `(from, to)` slots that the owner
//! applies to a [`KeyOrder`](crate::data_view::KeyOrder). Nothing here renders
//! or owns a signal, so a list item, a table row handle or a column menu can
//! share it.

use crate::direction::{Direction, HorizontalNav};
use dioxus::prelude::Key;

/// Where in the hovered item the pointer sits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DropPosition {
    /// Before the hovered item (the drop slot is its own slot).
    Before,
    /// No pointer position yet (a keyboard move).
    Undefined,
    /// After the hovered item.
    After,
}

impl From<std::cmp::Ordering> for DropPosition {
    fn from(ord: std::cmp::Ordering) -> Self {
        match ord {
            std::cmp::Ordering::Less => Self::Before,
            std::cmp::Ordering::Equal => Self::Undefined,
            std::cmp::Ordering::Greater => Self::After,
        }
    }
}

/// The axis a list is laid out on: which arrow keys move the lifted item and
/// which pointer coordinate picks the slot.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ReorderAxis {
    /// Items stack top to bottom; Up and Down move (never mirrored by RTL).
    #[default]
    Vertical,
    /// Items sit side by side along the inline axis; Left and Right move,
    /// mirrored under right-to-left.
    Horizontal,
}

impl ReorderAxis {
    /// The `data-orientation` value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
        }
    }
}

/// Resolves the final insertion index from a hovered item and pointer position.
pub fn resolve_drop_index(from: usize, hovered: usize, position: DropPosition) -> usize {
    let slot = match position {
        DropPosition::Before | DropPosition::Undefined => hovered,
        DropPosition::After => hovered + 1,
    };
    if from < slot {
        slot - 1
    } else {
        slot
    }
}

/// Whether the final insertion index is before or after the source item.
pub fn resolve_drop_position(from: usize, to: usize) -> DropPosition {
    to.cmp(&from).into()
}

/// Before or after the hovered item for a pointer at `pointer` over an item
/// that starts at `origin` and is `size` long on the list's axis. On the
/// inline axis under RTL the item's start is on the right, so the halves swap.
pub fn pointer_position(
    axis: ReorderAxis,
    direction: Direction,
    pointer: f64,
    origin: f64,
    size: f64,
) -> DropPosition {
    let before_midpoint = pointer < origin + size / 2.0;
    let before = match (axis, direction) {
        (ReorderAxis::Horizontal, Direction::Rtl) => !before_midpoint,
        _ => before_midpoint,
    };
    if before {
        DropPosition::Before
    } else {
        DropPosition::After
    }
}

/// The gesture's state.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ReorderState {
    /// Nothing lifted.
    Idle,
    /// An item is lifted from slot `from`; `to` is the slot it would land in.
    Lifted {
        /// Slot the item was lifted from.
        from: usize,
        /// Slot it would land in, once one is chosen.
        to: Option<usize>,
        /// Which side of the hovered item the pointer is on.
        position: DropPosition,
    },
    /// The item was dropped from `from` to `to` (until the gesture ends).
    Dropped {
        /// Slot it was lifted from.
        from: usize,
        /// Slot it landed in.
        to: usize,
    },
}

impl ReorderState {
    /// The slot the gesture started from.
    pub fn from(self) -> Option<usize> {
        match self {
            Self::Idle => None,
            Self::Lifted { from, .. } | Self::Dropped { from, .. } => Some(from),
        }
    }

    /// The slot the item would land in or landed in.
    pub fn to(self) -> Option<usize> {
        match self {
            Self::Idle => None,
            Self::Lifted { to, .. } => to,
            Self::Dropped { to, .. } => Some(to),
        }
    }

    /// The pointer side of the drop slot (`Undefined` outside a lift).
    pub fn position(self) -> DropPosition {
        match self {
            Self::Lifted { position, .. } => position,
            _ => DropPosition::Undefined,
        }
    }

    /// Whether a gesture is in progress (lifted, or dropped but not ended).
    pub fn is_active(self) -> bool {
        !matches!(self, Self::Idle)
    }

    /// Whether an item is lifted and not yet dropped.
    pub fn is_lifted(self) -> bool {
        matches!(self, Self::Lifted { .. })
    }

    /// Lifts the item in slot `index`.
    pub fn lift(index: usize) -> Self {
        Self::Lifted {
            from: index,
            to: None,
            position: DropPosition::Undefined,
        }
    }

    /// The pointer is over slot `hovered` on side `position`.
    #[must_use]
    pub fn hover(self, hovered: usize, position: DropPosition) -> Self {
        let Self::Lifted { from, .. } = self else {
            return self;
        };
        let to = resolve_drop_index(from, hovered, position);
        Self::Lifted {
            from,
            to: Some(to),
            position: resolve_drop_position(from, to),
        }
    }

    /// A keyboard step of `delta` slots from the current target (or from
    /// `index`, the focused slot), wrapping at the ends of a list of `len`.
    #[must_use]
    pub fn step(self, index: usize, delta: isize, len: usize) -> Self {
        let Self::Lifted { from, to, .. } = self else {
            return self;
        };
        if len == 0 {
            return self;
        }
        let current = to.unwrap_or(index) as isize;
        let next = (current + delta).rem_euclid(len as isize) as usize;
        Self::Lifted {
            from,
            to: Some(next),
            position: resolve_drop_position(from, next),
        }
    }

    /// Drops a lifted item onto its chosen slot: the new state and the
    /// `(from, to)` move to apply, or `None` (state unchanged) when nothing is
    /// lifted or no slot was chosen. A second drop of the same gesture is
    /// `None`, so a list-level and a document-level drop cannot apply twice.
    pub fn drop_item(self) -> (Self, Option<(usize, usize)>) {
        match self {
            Self::Lifted {
                from, to: Some(to), ..
            } => (Self::Dropped { from, to }, Some((from, to))),
            other => (other, None),
        }
    }
}

/// What a key press means to the gesture.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReorderCommand {
    /// Lift the focused item, or drop the lifted one (Space or Enter).
    Toggle,
    /// Move the lifted item one slot (`-1` earlier, `1` later), or, with
    /// nothing lifted, move focus the same way.
    Step(isize),
    /// Put the lifted item back (Escape).
    Cancel,
    /// Remove the focused item (Delete or Backspace) when nothing is lifted.
    Remove,
    /// Focus the first item (Home) when nothing is lifted.
    First,
    /// Focus the last item (End) when nothing is lifted.
    Last,
}

/// The one key map: `key` for a list on `axis` laid out in `direction`.
/// Horizontal arrows resolve through [`Direction::resolve_horizontal`], so a
/// horizontal list mirrors under RTL; Up and Down never do.
pub fn command_for_key(
    key: &Key,
    axis: ReorderAxis,
    direction: Direction,
) -> Option<ReorderCommand> {
    let step = match axis {
        ReorderAxis::Vertical => match key {
            Key::ArrowUp => Some(-1),
            Key::ArrowDown => Some(1),
            _ => None,
        },
        ReorderAxis::Horizontal => direction.resolve_horizontal(key).map(|nav| match nav {
            HorizontalNav::Prev => -1,
            HorizontalNav::Next => 1,
        }),
    };
    if let Some(step) = step {
        return Some(ReorderCommand::Step(step));
    }
    match key {
        Key::Enter => Some(ReorderCommand::Toggle),
        Key::Character(c) if c == " " => Some(ReorderCommand::Toggle),
        Key::Escape => Some(ReorderCommand::Cancel),
        Key::Delete | Key::Backspace => Some(ReorderCommand::Remove),
        Key::Home => Some(ReorderCommand::First),
        Key::End => Some(ReorderCommand::Last),
        _ => None,
    }
}

/// Announcement for a lift (`index` is the 0-based slot).
pub fn announce_lift(index: usize, count: usize) -> String {
    format!(
        "You have lifted an item in position {} of {count}",
        index + 1
    )
}

/// Announcement for a keyboard move to slot `to` (0-based).
pub fn announce_move(to: usize, count: usize) -> String {
    format!("You have moved the item to position {} of {count}", to + 1)
}

/// Announcement for a drop (0-based slots).
pub fn announce_drop(from: usize, to: usize) -> String {
    format!(
        "You have dropped the item. It has moved from position {} to position {}",
        from + 1,
        to + 1
    )
}

/// Announcement for a cancel (`from` is the 0-based starting slot).
pub fn announce_cancel(from: usize) -> String {
    format!(
        "Movement cancelled. The item has returned to its starting position of {}",
        from + 1
    )
}

/// Announcement for a removal (`index` 0-based, `remaining` items left).
pub fn announce_remove(index: usize, remaining: usize) -> String {
    format!(
        "Removed item from position {}. {remaining} items remaining",
        index + 1
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_index_accounts_for_the_lifted_slot() {
        assert_eq!(resolve_drop_index(0, 2, DropPosition::After), 2);
        assert_eq!(resolve_drop_index(0, 2, DropPosition::Before), 1);
        assert_eq!(resolve_drop_index(3, 1, DropPosition::Before), 1);
        assert_eq!(resolve_drop_index(3, 1, DropPosition::After), 2);
        assert_eq!(resolve_drop_index(2, 2, DropPosition::Undefined), 2);
    }

    #[test]
    fn keyboard_steps_wrap_and_drop_reports_the_move() {
        let s = ReorderState::lift(0).hover(0, DropPosition::Undefined);
        assert_eq!(s.to(), Some(0));
        let s = s.step(0, -1, 3);
        assert_eq!(s.to(), Some(2), "up from the first wraps to the last");
        let s = s.step(0, 1, 3);
        assert_eq!(s.to(), Some(0), "down from the last wraps to the first");
        let s = s.step(0, 1, 3).step(0, 1, 3);
        let (after, moved) = s.drop_item();
        assert_eq!(moved, Some((0, 2)));
        assert_eq!(after, ReorderState::Dropped { from: 0, to: 2 });
        // The second drop of one gesture applies nothing.
        assert_eq!(after.drop_item(), (after, None));
    }

    #[test]
    fn nothing_moves_unless_lifted_with_a_target() {
        assert_eq!(ReorderState::Idle.step(1, 1, 3), ReorderState::Idle);
        assert_eq!(ReorderState::Idle.drop_item().1, None);
        assert_eq!(ReorderState::lift(1).drop_item().1, None);
        assert_eq!(
            ReorderState::Idle.hover(2, DropPosition::After),
            ReorderState::Idle
        );
    }

    #[test]
    fn keys_vertical_ignore_left_right_and_never_mirror() {
        for dir in [Direction::Ltr, Direction::Rtl] {
            let v = ReorderAxis::Vertical;
            assert_eq!(
                command_for_key(&Key::ArrowUp, v, dir),
                Some(ReorderCommand::Step(-1))
            );
            assert_eq!(
                command_for_key(&Key::ArrowDown, v, dir),
                Some(ReorderCommand::Step(1))
            );
            assert_eq!(command_for_key(&Key::ArrowLeft, v, dir), None);
            assert_eq!(command_for_key(&Key::ArrowRight, v, dir), None);
        }
    }

    #[test]
    fn keys_horizontal_mirror_under_rtl() {
        let h = ReorderAxis::Horizontal;
        assert_eq!(
            command_for_key(&Key::ArrowRight, h, Direction::Ltr),
            Some(ReorderCommand::Step(1))
        );
        assert_eq!(
            command_for_key(&Key::ArrowRight, h, Direction::Rtl),
            Some(ReorderCommand::Step(-1))
        );
        assert_eq!(
            command_for_key(&Key::ArrowLeft, h, Direction::Rtl),
            Some(ReorderCommand::Step(1))
        );
        assert_eq!(command_for_key(&Key::ArrowDown, h, Direction::Ltr), None);
    }

    #[test]
    fn gesture_keys() {
        let (v, d) = (ReorderAxis::Vertical, Direction::Ltr);
        assert_eq!(
            command_for_key(&Key::Enter, v, d),
            Some(ReorderCommand::Toggle)
        );
        assert_eq!(
            command_for_key(&Key::Character(" ".into()), v, d),
            Some(ReorderCommand::Toggle)
        );
        assert_eq!(
            command_for_key(&Key::Escape, v, d),
            Some(ReorderCommand::Cancel)
        );
        assert_eq!(
            command_for_key(&Key::Delete, v, d),
            Some(ReorderCommand::Remove)
        );
        assert_eq!(command_for_key(&Key::Character("a".into()), v, d), None);
    }

    #[test]
    fn pointer_halves_swap_on_the_inline_axis_under_rtl() {
        use DropPosition::*;
        let p = |a, d, x| pointer_position(a, d, x, 100.0, 40.0);
        assert_eq!(p(ReorderAxis::Vertical, Direction::Ltr, 110.0), Before);
        assert_eq!(p(ReorderAxis::Vertical, Direction::Rtl, 110.0), Before);
        assert_eq!(p(ReorderAxis::Horizontal, Direction::Ltr, 110.0), Before);
        assert_eq!(p(ReorderAxis::Horizontal, Direction::Rtl, 110.0), After);
        assert_eq!(p(ReorderAxis::Horizontal, Direction::Rtl, 130.0), Before);
    }
}
