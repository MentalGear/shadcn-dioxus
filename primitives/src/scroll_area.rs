//! Defines the [`ScrollArea`] component for creating scrollable areas with customizable scrollbars.

use crate::direction::{use_direction, Direction};
use crate::merge_attributes;
use dioxus::prelude::*;
use dioxus_attributes::attributes;

/// The props for the [`ScrollArea`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ScrollAreaProps {
    /// The scroll direction (which axis is scrollable) -- not to be
    /// confused with [`Self::dir`] (text/layout direction).
    #[props(default)]
    pub direction: ReadSignal<ScrollDirection>,

    /// The text direction. This crate wraps the browser's own native
    /// scrollbar (no custom-drawn thumb, unlike Radix's `ScrollArea`), so
    /// the CSS Overflow spec's own `dir`-relative behavior repositions the
    /// scrollbar to the correct physical side for free -- see this lane's
    /// own `$S/batch3/rtl-rust/reference.md`'s ScrollArea row. Defaults to
    /// the nearest [`crate::direction::DirectionProvider`], or LTR if
    /// there is none.
    #[props(default)]
    pub dir: Option<Direction>,

    /// Whether the scrollbars should be always visible.
    #[props(default)]
    pub always_show_scrollbars: ReadSignal<bool>,

    /// The scroll type.
    #[props(default)]
    pub scroll_type: ReadSignal<ScrollType>,

    /// A scroll-edge fade on the scrolling element: the content dissolves toward an edge
    /// that has more to scroll to. Defaults to [`ScrollFade::None`]. See [`ScrollFade`].
    #[props(default)]
    pub fade: ScrollFade,

    /// Additional attributes to apply to the scroll area element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the scroll area component.
    pub children: Element,
}

/// Which edges of a scroller fade out while there is more content past them.
///
/// Each variant is one class of the `dx-scroll-fade` utility in `dx-effects.css` (a pure-CSS
/// scroll-driven `mask-image`, no JavaScript), put on the element that actually scrolls: the
/// [`ScrollArea`], the `VirtualList` container and the `MessageScrollerViewport`. At rest the
/// end edge fades to hint at more content; scrolling fades the start edge in, and at the end
/// the end edge sharpens. No overflow, no fade. A browser without scroll-driven animations
/// shows a static end-edge fade.
///
/// The class does nothing unless `dx-effects.css` is linked once at the app root, next to the
/// theme (the `scroll_area`, `virtual_list` and `message_scroller` registry entries list it in
/// `globalAssets`). Because the fade is a mask on the scroller, a border or background on the
/// same element fades with the content: put those on a wrapper and let the scroller be the
/// inner element.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ScrollFade {
    /// No fade. The default (the class is not added at all).
    #[default]
    None,
    /// Both block edges of a vertical scroller (`dx-scroll-fade-y`).
    Y,
    /// Both inline edges of a horizontal scroller, mirrored in RTL (`dx-scroll-fade-x`).
    X,
    /// The top edge only (`dx-scroll-fade-t`): a transcript that opens at the bottom and
    /// loads history upward.
    Top,
    /// The bottom edge only (`dx-scroll-fade-b`).
    Bottom,
    /// The inline-start edge only (`dx-scroll-fade-s`; the right edge in RTL).
    Start,
    /// The inline-end edge only (`dx-scroll-fade-e`; the left edge in RTL).
    End,
}

impl ScrollFade {
    /// The `dx-effects.css` class this variant puts on the scroller, or `None` for
    /// [`ScrollFade::None`].
    pub fn class(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Y => Some("dx-scroll-fade-y"),
            Self::X => Some("dx-scroll-fade-x"),
            Self::Top => Some("dx-scroll-fade-t"),
            Self::Bottom => Some("dx-scroll-fade-b"),
            Self::Start => Some("dx-scroll-fade-s"),
            Self::End => Some("dx-scroll-fade-e"),
        }
    }

    /// The `data-scroll-fade` value (`y`, `x`, `top`, `bottom`, `start`, `end`), or `None` for
    /// [`ScrollFade::None`]; a styling and test hook, the same spelling as the variant.
    pub fn as_str(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Y => Some("y"),
            Self::X => Some("x"),
            Self::Top => Some("top"),
            Self::Bottom => Some("bottom"),
            Self::Start => Some("start"),
            Self::End => Some("end"),
        }
    }

    /// The attributes a scroller carries for this fade: the class and `data-scroll-fade`
    /// (nothing for [`ScrollFade::None`]). Merge them with the scroller's other attributes
    /// (`class` concatenates).
    pub fn attributes(self) -> Vec<Attribute> {
        // Empty for `None`: a `class` of `AttributeValue::None` would replace the earlier
        // lists' class in `merge_attributes` instead of concatenating with it.
        let (Some(class), Some(name)) = (self.class(), self.as_str()) else {
            return Vec::new();
        };
        attributes!(div {
            class: "{class}",
            "data-scroll-fade": "{name}",
        })
    }
}

/// The direction in which scrolling is allowed.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum ScrollDirection {
    /// Allow vertical scrolling only.
    Vertical,
    /// Allow horizontal scrolling only.
    Horizontal,
    /// Allow scrolling in both directions.
    #[default]
    Both,
}

/// The type of scrolling behavior.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum ScrollType {
    /// Browser default scrolling.
    #[default]
    Auto,
    /// Always show scrollbars.
    Always,
    /// Hide scrollbars but enable scrolling.
    Hidden,
}

/// # ScrollArea
///
/// The `ScrollArea` component creates a scrollable area. If you don't
/// have any focusable content within the scroll area, you should make the
/// scroll area focusable by adding a `tabindex` attribute.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::scroll_area::{ScrollArea, ScrollDirection};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         ScrollArea {
///             width: "10em",
///             height: "10em",
///             direction: ScrollDirection::Vertical,
///             tabindex: "0",
///             div { class: "dx-scroll-content",
///                 for i in 1..=20 {
///                     p {
///                         "Scrollable content item {i}"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`ScrollArea`] component defines the following data attributes you can use to control styling:
/// - `data-scroll-direction`: Indicates the scroll direction. Values are `vertical`, `horizontal`, or `both`.
/// - `data-direction`: The resolved text direction. Values are `ltr` or `rtl`.
/// - `data-scroll-fade`: Present with the `fade` prop; `y`, `x`, `top`, `bottom`, `start` or `end`.
///
/// ## Scroll fade
///
/// `fade: ScrollFade::Y` (or `X`, `Top`, `Bottom`, `Start`, `End`) adds the matching
/// `dx-scroll-fade-*` class of `dx-effects.css` to this element, which is the one that scrolls.
/// See [`ScrollFade`] for what it needs.
#[component]
pub fn ScrollArea(props: ScrollAreaProps) -> Element {
    let direction = props.direction;
    let scroll_type = props.scroll_type;
    let always_show = props.always_show_scrollbars;
    let text_direction = use_direction(props.dir);

    let (overflow_x, overflow_y, scrollbar_width) = match scroll_type() {
        ScrollType::Auto => match direction() {
            ScrollDirection::Vertical => (Some("hidden"), Some("auto"), None),
            ScrollDirection::Horizontal => (Some("auto"), Some("hidden"), None),
            ScrollDirection::Both => (Some("auto"), Some("auto"), None),
        },
        ScrollType::Always => match direction() {
            ScrollDirection::Vertical => (Some("hidden"), Some("scroll"), None),
            ScrollDirection::Horizontal => (Some("scroll"), Some("hidden"), None),
            ScrollDirection::Both => (Some("scroll"), Some("scroll"), None),
        },
        ScrollType::Hidden => match direction() {
            ScrollDirection::Vertical => (Some("hidden"), Some("scroll"), Some("none")),
            ScrollDirection::Horizontal => (Some("scroll"), Some("hidden"), Some("none")),
            ScrollDirection::Both => (Some("scroll"), Some("scroll"), Some("none")),
        },
    };

    let visibility_class = use_memo(move || {
        if always_show() {
            "dx-scroll-area-always-show"
        } else {
            "dx-scroll-area-auto-hide"
        }
    });

    // `class` always concatenates regardless of merge order; the rest is
    // this scroll area's own computed state (`scrollbar-width`/the
    // `data-*` pair).
    //
    // The overflow is a `style` attribute of the same list, not separate style fields next to the
    // spread: a spread `style` placed beside explicit style fields replaces them (a group given
    // `style: "width: 100%"` lost its `overflow-x: auto` and spilled out of the page).
    let overflow_style = format!(
        "overflow-x: {}; overflow-y: {};",
        overflow_x.unwrap_or("visible"),
        overflow_y.unwrap_or("visible")
    );
    let defaults = attributes!(div {
        class: "{visibility_class}",
        style: "{overflow_style}",
    });
    let fade = props.fade.attributes();
    let owned = attributes!(div {
        "scrollbar-width": scrollbar_width,
        "data-scroll-direction": match direction() {
            ScrollDirection::Vertical => "vertical",
            ScrollDirection::Horizontal => "horizontal",
            ScrollDirection::Both => "both",
        },
        "data-direction": text_direction.as_str(),
    });
    let merged = merge_attributes(vec![defaults, fade, props.attributes.clone(), owned]);
    // One `style` is built from the plain one and the shorthand-namespaced ones (`width: ".."`)
    // a caller may pass: the DOM renderer would let a plain `style` replace the shorthands.
    // `merged` has had every `style` folded out, so merging the folded one back in has nothing
    // left to collide with (the same idiom as `resizable.rs`).
    let (style, merged) = crate::fold_style_attributes(merged);
    let style = style.unwrap_or_default();
    let merged = merge_attributes(vec![merged, attributes!(div { style })]);

    rsx! {
        div {
            dir: text_direction.as_str(),
            ..merged,

            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[component]
    fn Styled() -> Element {
        rsx! {
            ScrollArea { direction: ScrollDirection::Horizontal, style: "width: 100%;", max_width: "20rem", "x" }
        }
    }

    /// A caller's own `style` is added to the scroll area's overflow, never a replacement for it.
    #[test]
    fn a_callers_style_keeps_the_overflow() {
        let mut dom = VirtualDom::new(Styled);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("overflow-x: auto"), "{html}");
        assert!(html.contains("overflow-y: hidden"), "{html}");
        assert!(html.contains("width: 100%"), "{html}");
        // ... and so are the shorthand-set properties, in the one `style`.
        assert!(html.contains("max-width:20rem"), "{html}");
        assert_eq!(html.matches("style=").count(), 1, "{html}");
    }
}
