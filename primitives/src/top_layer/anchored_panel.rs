//! The one recipe of an anchored panel: promoted to the top layer, placed next to its anchor and
//! carrying the attributes the engine's stylesheet selects on.

use super::*;
use crate::{merge_attributes, ContentAlign, ContentSide};

/// How the panel's top-layer state follows its `open` signal.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PanelSync {
    /// `use_popover_shown_while_mounted`: the panel is shown while it is mounted and is never hidden from
    /// the closing path, so a CSS exit animation plays on an element that is still in the top layer. For a
    /// panel mounted through `use_animated_open`.
    WhileMounted,
    /// `use_popover_sync`: `showPopover()` / `hidePopover()` follow `open` at once. For a panel that is
    /// always mounted.
    Immediate,
}

/// Where the panel sits.
pub(crate) enum AnchoredPlacement {
    /// Next to the element `anchor_id` names (the id its `anchor_name_style` was built from), `gap` px
    /// away on `side`.
    Anchored {
        anchor_id: String,
        side: ContentSide,
        align: ContentAlign,
        gap: u32,
        /// The `data-dx-anchored` value. [`AnchorKind::SelectItemAligned`] carries none: the panel
        /// binds `position-anchor` but the engine sits out, because the panel places itself.
        kind: AnchorKind,
    },
    /// At a document point, with no anchor element (the context menu).
    Point { x: i32, y: i32 },
}

/// Everything a site says about its panel to [`use_anchored_panel`].
pub(crate) struct AnchoredPanel {
    /// The panel element's id.
    pub(crate) id: String,
    /// Whether the panel is open.
    pub(crate) open: ReadSignal<bool>,
    /// The echo of a browser-side close or open (light dismiss, Escape, `hidePopover()`): called with the
    /// new state so the site can keep its own state in step. A site with nothing to keep in step passes a
    /// no-op.
    pub(crate) set_open: Callback<bool>,
    /// The popover mode.
    pub(crate) popover: PopoverKind,
    /// How the top-layer state follows `open`.
    pub(crate) sync: PanelSync,
    /// The placement.
    pub(crate) placement: AnchoredPlacement,
    /// Attributes the site owns on its panel (`role`, the open state, `data-side`, ...); they win over the
    /// caller's.
    pub(crate) owned: Vec<Attribute>,
}

/// The wiring and attributes of an anchored panel: top-layer promotion (`sync`), placement (the measured
/// fallback of `use_anchor_position_fallback`, or a point), and the merged attribute list.
///
/// The returned list is `attributes` (the caller's, plus anything the site merges under them, such as an
/// `aria-labelledby` default) with `owned` and the `popover` attribute merged over it, so a caller cannot
/// override them (`dev-docs/backlog.md` row 93's duplicate-attribute hazard). An anchored placement also
/// folds the caller's `style` with the anchor binding into one `style` and writes `data-dx-anchored`
/// ([`anchored_content_attributes`]).
///
/// A hook: call it unconditionally, once per render, from the site's panel component, and spread the
/// result on the panel element.
pub(crate) fn use_anchored_panel(
    config: AnchoredPanel,
    attributes: Vec<Attribute>,
) -> Vec<Attribute> {
    let AnchoredPanel {
        id,
        open,
        set_open,
        popover,
        sync,
        placement,
        owned,
    } = config;

    match sync {
        PanelSync::WhileMounted => use_popover_shown_while_mounted(id.clone(), open, set_open),
        PanelSync::Immediate => use_popover_sync(id.clone(), open, set_open),
    }
    // An engine-less kind places itself, so the measured placement sits out.
    let engine_placed =
        matches!(&placement, AnchoredPlacement::Anchored { kind, .. } if kind.as_str().is_some());
    let placed = use_memo(use_reactive!(|engine_placed| open() && engine_placed));
    match &placement {
        AnchoredPlacement::Anchored {
            anchor_id,
            side,
            align,
            gap,
            ..
        } => {
            use_anchor_position_fallback(id.clone(), anchor_id.clone(), placed, *side, *align, *gap)
        }
        AnchoredPlacement::Point { x, y } => use_point_anchor_placement(id.clone(), *x, *y, open),
    }

    let promoted = dioxus_attributes::attributes!(div {
        popover: popover.as_str()
    });
    let attributes = merge_attributes(vec![attributes, owned, promoted]);
    match placement {
        AnchoredPlacement::Anchored {
            anchor_id, kind, ..
        } => anchored_content_attributes(kind, &anchor_id, attributes),
        AnchoredPlacement::Point { .. } => attributes,
    }
}
