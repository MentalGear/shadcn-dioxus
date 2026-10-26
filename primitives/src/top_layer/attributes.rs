//! Anchor-name / position-anchor style and attribute helpers.

use super::*;

/// CSS Anchor Positioning wiring — *not* part of the Popover API itself, but
/// required for it to be usable at all here.
///
/// Moving an element into the top layer changes its containing block to the
/// initial containing block (viewport) regardless of its own `position`
/// value: a `Tooltip` given `popover="manual"`
/// with no other change keeps `position: absolute` but its rendered box
/// jumps hundreds of pixels from its trigger, because "absolute, relative
/// to the nearest positioned ancestor" stops applying once the element
/// is promoted. The `[data-side]` rules in the preview
/// stylesheets (`tooltip`/`hover_card`/`popover` `style.css`) assume a
/// DOM-relative containing block, so without anchoring every non-modal
/// overlay promoted to the top layer would render detached from its trigger.
///
/// CSS Anchor Positioning is the platform's own, still-CSS answer (no
/// `floating-ui`; the JS collision fallback for engines without it is
/// [`use_anchor_position_fallback`] below): an
/// `anchor-name` declared on the trigger and referenced from the content
/// via `position-anchor` lets `top`/`left`/etc. in the *stylesheet* use
/// `anchor()` to mean "relative to that named anchor" even from the top
/// layer. Works in this repo's Chromium
/// (`CSS.supports('anchor-name: --x')` true) — see the preview
/// stylesheets for the `anchor()`-based rules this pairs with, `@supports`-
/// gated so an engine without Anchor Positioning falls back to the plain
/// (non-anchored) rules rather than an invalid declaration.
///
/// Each overlay instance gets its own anchor name, built from its own
/// unique id (already threaded through every one of these components for
/// ARIA), so unrelated instances on the same page never collide. Only
/// meaningful alongside `popover`, so it degrades to an inert empty `style`
/// value off the web arm rather than being `#[cfg]`-gated at every call
/// site.
#[cfg(feature = "web")]
pub(crate) fn anchor_name_style(id: &str) -> String {
    format!("anchor-name: --dxa-{id};")
}

/// See [`anchor_name_style`]. Set on the content element, referencing the
/// same id the trigger's `anchor_name_style` was built from. Only called
/// from the web-arm leaf render functions (the native arm never promotes
/// content to the top layer, so it has no containing-block problem to
/// solve), so -- like [`PopoverKind`] -- this has no native counterpart.
#[cfg(feature = "web")]
pub(crate) fn position_anchor_style(id: &str) -> String {
    format!("position-anchor: --dxa-{id};")
}

/// Which anchored overlay a panel is: the value of the one attribute the engine's stylesheet selects
/// on, `data-dx-anchored` (backlog row 432).
///
/// The engine's rules (`styles.rs`) select `[data-dx-anchored][popover]` and never name a kind, except
/// where kinds genuinely differ (`HoverCard`'s wider gap). So a new anchored overlay is positioned by
/// calling [`anchored_content_attributes`], which it must already call for its `position-anchor`; there is
/// no list in the engine to add it to and no marker class to remember. `scripts/check-anchor-marker.sh`
/// keeps it that way.
#[cfg(feature = "web")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnchorKind {
    Tooltip,
    HoverCard,
    Popover,
    DropdownMenu,
    Menubar,
    Navbar,
    NavigationMenu,
    /// A `Select` list placed by the engine (`SelectPosition::Popper`).
    Select,
    /// A `Select` list placed by its own script (`use_item_aligned`, item-aligned position): it still
    /// binds `position-anchor`, but the engine's `anchor()` rules must not apply on top of its own
    /// viewport-pixel placement, so it carries no attribute.
    SelectItemAligned,
    Combobox,
}

#[cfg(feature = "web")]
impl AnchorKind {
    /// The `data-dx-anchored` value, or `None` for a panel the engine must leave to its own script.
    pub(crate) fn as_str(self) -> Option<&'static str> {
        match self {
            Self::Tooltip => Some("tooltip"),
            Self::HoverCard => Some("hover-card"),
            Self::Popover => Some("popover"),
            Self::DropdownMenu => Some("dropdown-menu"),
            Self::Menubar => Some("menubar"),
            Self::Navbar => Some("navbar"),
            Self::NavigationMenu => Some("navigation-menu"),
            Self::Select => Some("select"),
            Self::SelectItemAligned => None,
            Self::Combobox => Some("combobox"),
        }
    }
}

/// Fold [`position_anchor_style`] and a caller's own `style` into exactly
/// one `style` attribute, then merge the rest of `attributes` on top.
///
/// Every anchored-content leaf (`TooltipContentRendered`,
/// `HoverCardContentRendered`, `DropdownMenuContentRendered`,
/// `MenubarContentRendered`, `PopoverModalContent`/`PopoverNonModalContent`,
/// `SelectListRendered`, `ComboboxListRendered`) must not set
/// `style: position_anchor_style(&id)` as a bare literal on the element
/// while *also* spreading `..attributes` -- a caller passing its own
/// `style` (shorthand or plain, e.g. the `top_layer` preview fixture's
/// `MenubarContent { style: "min-height: 100px;", .. }`) would then produce two
/// `style` attributes on one tag: the exact duplicate-attribute hazard
/// `dev-docs/conformance-harness.md` hydration-parity Rule 4 documents.
/// `dioxus-ssr` keeps the *first* `style` it renders (`position-anchor` for
/// these components), while the WASM client's `set_attribute` overwrites
/// with the *last* one it applies (the caller's) -- so SSR and CSR silently
/// disagree about which `style` wins, and on the client the anchor binding
/// this whole module exists to set up is lost, breaking CSS Anchor
/// Positioning without any error. `merge_attributes` alone cannot fix this:
/// it dedupes by `(name, namespace)`, and a bare `style: "..."` literal (as
/// written here) is a different attribute from the `attributes` list's own
/// `style`/style-shorthand entries from its point of view, so plain
/// `merge_attributes` never sees them as the same key to begin with.
///
/// So instead: run `attributes` through [`crate::fold_style_attributes`]
/// first (same construction as `ContextMenuTrigger`'s touch-suppression
/// style, `context_menu.rs`) to pull out whatever `style` the caller
/// supplied -- shorthand props, a plain `style` literal, or both -- as one
/// string, prepend `position_anchor_style(id)` to it (anchor binding first
/// so the caller's own declarations, listed after, can still override any
/// property they want without ever being able to drop the anchor binding
/// itself), and merge that single resulting `style` attribute back in with
/// everything else `fold_style_attributes` left untouched. Exactly one
/// `style` attribute reaches the `rsx!` call either way, so SSR and CSR
/// agree by construction and there is nothing left for Rule 4 to catch.
///
/// Call this as the last step before an anchored leaf's `rsx!` block, on
/// whatever `attributes` variable would otherwise be spread with
/// `..attributes` -- including one a call site already ran through its own
/// `merge_attributes` for a marker class or a default `aria-labelledby`, so
/// this never needs a second, separate merge pass for those. A call site
/// carries no `style: position_anchor_style(&id)` literal of its own; the
/// returned list already carries that `style`.
///
/// This is also the ONE writer of `data-dx-anchored` (`kind`, see [`AnchorKind`]): the engine's stylesheet
/// scopes every anchoring rule to that attribute, so an anchored panel is anchored by calling this, and a
/// caller's own `data-dx-anchored` cannot unscope it (it is merged last, after `attributes`).
#[cfg(feature = "web")]
pub(crate) fn anchored_content_attributes(
    kind: AnchorKind,
    id: &str,
    attributes: Vec<Attribute>,
) -> Vec<Attribute> {
    let (caller_style, rest) = crate::fold_style_attributes(attributes);
    let style = match caller_style {
        Some(caller_style) => format!("{} {caller_style}", position_anchor_style(id)),
        None => position_anchor_style(id),
    };
    crate::merge_attributes(vec![
        dioxus_attributes::attributes!(div { style: "{style}" }),
        rest,
        dioxus_attributes::attributes!(div { "data-dx-anchored": kind.as_str() }),
    ])
}

/// No-op whenever this crate's `web` feature is off -- see
/// [`anchor_name_style`]'s doc. Unlike `position_anchor_style`, this one
/// *is* called unconditionally (every trigger sets it, regardless of
/// build), so it needs a real native stub rather than being `#[cfg]`-gated
/// away entirely.
#[cfg(not(feature = "web"))]
pub(crate) fn anchor_name_style(_id: &str) -> String {
    String::new()
}

/// Fold [`anchor_name_style`] and a caller's own `style` into exactly one
/// `style` attribute, then merge the rest of `attributes` on top.
///
/// The trigger-side mirror of `anchored_content_attributes` -- same
/// problem, same fix, the other end of the same anchor pair. Every
/// anchored trigger (`ContextMenuSubTrigger`, `DropdownMenuTrigger`,
/// `DropdownMenuSubTrigger`, `HoverCardTrigger`, `MenubarTrigger`,
/// `NavbarTrigger`, `NavigationMenuTrigger`, `PopoverTrigger`,
/// `TooltipTrigger`) ties itself to its content's `position-anchor` via
/// [`anchor_name_style`]. A bare literal
/// directly alongside a raw `..attributes` spread on the same element
/// would be the exact duplicate-`style` hazard `anchored_content_attributes`'s own
/// doc describes (`dev-docs/conformance-harness.md` hydration-parity Rule 4;
/// `scripts/check-attr-spread-collision.sh` ratchets against it, and
/// `playwright/oracle/attr-synth` synthesizes it directly). A trigger can
/// instead run its own literal through `merge_attributes` before the
/// `rsx!` spread (`TooltipTrigger` does), which *does* avoid that
/// literal HTML duplicate. `merge_attributes` itself folds a plain
/// `style` the way it folds `class` (earlier declarations first), so a
/// caller's `style` does not replace the anchor-name binding through it
/// (an overwrite would silently stop CSS Anchor Positioning the moment any
/// caller set `style` on the trigger; see `lib.rs`'s
/// `style_attribute_is_folded_not_overwritten`). This function exists for
/// what `merge_attributes` cannot do: absorb style set the *shorthand* way (`padding: ".."`, namespace
/// `Some("style")`, a different attribute name to `merge_attributes`), so
/// exactly one `style` reaches the element, and guarantee the anchor binding
/// comes first regardless of which list a call site put it in. So: pull every
/// style-contributing attribute out of `attributes` with
/// [`crate::fold_style_attributes`], prepend the anchor-name binding, and
/// merge the single resulting `style` back in with everything else
/// `fold_style_attributes` left untouched.
///
/// Unlike `anchored_content_attributes`, this carries no `#[cfg(feature =
/// "web")]` guard: every trigger above calls this unconditionally on both
/// arms (see [`anchor_name_style`]'s own doc for why a trigger, unlike a
/// content leaf, is never itself split into separate web/native component
/// fns), so this absorbs the resulting empty anchor-name string itself
/// (contributing nothing to the folded `style` off the web arm) rather than
/// pushing that concern onto every call site.
///
/// Call this as the last step before an anchored trigger's `rsx!` block, on
/// whatever `attributes` variable would otherwise be spread with
/// `..attributes` -- including one a call site already ran through its own
/// `merge_attributes` for an `id`/event handlers/other literals, so this
/// never needs a second, separate merge pass for those. A call site
/// carries no `style: anchor_name_style(&id)` literal of its own; the
/// returned list already carries that `style`.
pub(crate) fn anchored_trigger_attributes(id: &str, attributes: Vec<Attribute>) -> Vec<Attribute> {
    let anchor = anchor_name_style(id);
    let (caller_style, rest) = crate::fold_style_attributes(attributes);
    let style = match (anchor.is_empty(), caller_style) {
        (true, None) => None,
        (true, Some(caller_style)) => Some(caller_style),
        (false, None) => Some(anchor),
        (false, Some(caller_style)) => Some(format!("{anchor} {caller_style}")),
    };
    match style {
        Some(style) => crate::merge_attributes(vec![
            dioxus_attributes::attributes!(div { style: "{style}" }),
            rest,
        ]),
        None => rest,
    }
}
