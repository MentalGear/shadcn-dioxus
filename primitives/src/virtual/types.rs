//! Core types for the virtual list implementation.

/// A unique key for identifying items in the virtualizer.
pub(crate) type Key = usize;

/// A single virtualized item with computed position.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VirtualItem {
    key: Key,
    index: usize,
    start: u32,
    size: u32,
}

impl VirtualItem {
    pub(crate) fn new(key: Key, index: usize, start: u32, size: u32) -> Self {
        Self {
            key,
            index,
            start,
            size,
        }
    }

    pub(crate) fn key(&self) -> Key {
        self.key
    }

    pub(crate) fn index(&self) -> usize {
        self.index
    }

    pub(crate) fn start(&self) -> u32 {
        self.start
    }

    #[cfg(test)]
    pub(crate) fn end(&self) -> u32 {
        self.start + self.size
    }

    pub(crate) fn size(&self) -> u32 {
        self.size
    }
}

/// One report of a scroll container's state, from the shared DOM bridge
/// (`super::bridge`): what every windowed consumer's engine reads. The list and the wrapping
/// grid read the block axis (`top`, `height`); the data grid reads both.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScrollReport {
    /// `scrollTop`.
    pub(crate) top: u32,
    /// `|scrollLeft|`: the distance from the inline start, positive in LTR and RTL alike.
    pub(crate) left: u32,
    /// The container's visible width (capped at the window's).
    pub(crate) width: u32,
    /// The container's visible height (capped at the window's).
    pub(crate) height: u32,
    /// Whether a scroll is in progress (the canvas size stays frozen until it ends).
    pub(crate) is_scrolling: bool,
}
