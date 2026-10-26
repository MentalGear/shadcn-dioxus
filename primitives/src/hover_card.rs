//! Defines the [`HoverCard`] component and its subcomponents.

use crate::menu_sub::{use_delayed_action, DelayedAction};
use crate::{
    merge_attributes, use_animated_open, use_controlled, use_id_or, use_unique_id, ContentAlign,
    ContentSide,
};
use dioxus::prelude::*;
use dioxus_attributes::attributes;
use std::time::Duration;

/// Default [`HoverCardProps::delay`], in milliseconds: how long the pointer
/// must rest on the trigger before the card opens. Base UI's
/// `PreviewCard.Trigger` `delay` default (`OPEN_DELAY = 600`,
/// `packages/react/src/preview-card/utils/constants.ts`).
pub const DEFAULT_HOVER_CARD_DELAY: u64 = 600;

/// Default [`HoverCardProps::close_delay`], in milliseconds: how long the card
/// stays open after the pointer leaves the trigger (or the card), so it can
/// travel across the gap. Base UI's `PreviewCard.Trigger` `closeDelay` default
/// (`CLOSE_DELAY = 300`, same file).
pub const DEFAULT_HOVER_CARD_CLOSE_DELAY: u64 = 300;

/// The one place a hover-opened, non-modal floating hint (this card, and
/// [`crate::tooltip`]) turns pointer and focus events into open/close writes.
///
/// - **Pointer enter** on the trigger opens after `delay` ms (at once when
///   `delay` is 0, or when the surface is already open -- which only cancels a
///   pending close). Entering the *content* calls [`Self::hold`], which cancels a pending
///   close, so a pointer travelling from the trigger to the content across a
///   gap does not close it on the way.
/// - **Pointer leave** (trigger or content) closes after `close_delay` ms (at
///   once when 0).
/// - **Focus / blur / Escape** are keyboard-driven and never wait: they
///   cancel both timers and write directly (Radix `HoverCard` opens on focus
///   at once; Base UI `Tooltip` has no focus delay).
///
/// Both timers are [`DelayedAction`]s: tasks spawned in the scope that
/// schedules them, so they are cancelled when that scope drops (nothing
/// outlives an unmount), and scheduling replaces whatever the handle was
/// waiting on. Every timer re-checks `disabled` when it fires.
#[derive(Clone, Copy)]
pub(crate) struct HoverTimers {
    open: Memo<bool>,
    set_open: Callback<bool>,
    disabled: ReadSignal<bool>,
    delay: ReadSignal<u64>,
    close_delay: ReadSignal<u64>,
    open_timer: DelayedAction,
    close_timer: DelayedAction,
}

/// Create the [`HoverTimers`] of one root. A hook: call it unconditionally,
/// once per render, in the root's body.
pub(crate) fn use_hover_timers(
    open: Memo<bool>,
    set_open: Callback<bool>,
    disabled: ReadSignal<bool>,
    delay: ReadSignal<u64>,
    close_delay: ReadSignal<u64>,
) -> HoverTimers {
    HoverTimers {
        open,
        set_open,
        disabled,
        delay,
        close_delay,
        open_timer: use_delayed_action(),
        close_timer: use_delayed_action(),
    }
}

impl HoverTimers {
    /// The pointer entered the trigger.
    pub(crate) fn pointer_enter(self) {
        if (self.disabled)() {
            return;
        }
        let mut close_timer = self.close_timer;
        let mut open_timer = self.open_timer;
        close_timer.cancel();
        if *self.open.peek() {
            open_timer.cancel();
            return;
        }
        let delay = *self.delay.peek();
        if delay == 0 {
            open_timer.cancel();
            self.set_open.call(true);
        } else {
            let (disabled, set_open) = (self.disabled, self.set_open);
            open_timer.schedule(Duration::from_millis(delay), move || {
                if !disabled() {
                    set_open.call(true);
                }
            });
        }
    }

    /// The pointer left the trigger or the content.
    pub(crate) fn pointer_leave(self) {
        if (self.disabled)() {
            return;
        }
        let mut open_timer = self.open_timer;
        let mut close_timer = self.close_timer;
        open_timer.cancel();
        let close_delay = *self.close_delay.peek();
        if close_delay == 0 {
            close_timer.cancel();
            self.set_open.call(false);
        } else if *self.open.peek() {
            let (disabled, set_open) = (self.disabled, self.set_open);
            close_timer.schedule(Duration::from_millis(close_delay), move || {
                if !disabled() {
                    set_open.call(false);
                }
            });
        } else {
            close_timer.cancel();
        }
    }

    /// The pointer entered the *content*: it only cancels a pending close. Unlike
    /// [`Self::pointer_enter`] it never opens anything, so the pointer touching
    /// a surface that is already fading out cannot re-open it.
    pub(crate) fn hold(self) {
        let mut close_timer = self.close_timer;
        close_timer.cancel();
    }

    /// Keyboard focus reached the trigger: open now, no delay.
    pub(crate) fn focus(self) {
        self.cancel_pending();
        if !(self.disabled)() {
            self.set_open.call(true);
        }
    }

    /// Keyboard focus left the trigger: close now, no delay.
    pub(crate) fn blur(self) {
        self.cancel_pending();
        if !(self.disabled)() {
            self.set_open.call(false);
        }
    }

    /// Escape: close now, whatever `disabled` says.
    pub(crate) fn dismiss(self) {
        self.cancel_pending();
        self.set_open.call(false);
    }

    fn cancel_pending(self) {
        let mut open_timer = self.open_timer;
        let mut close_timer = self.close_timer;
        open_timer.cancel();
        close_timer.cancel();
    }
}

#[derive(Clone, Copy)]
struct HoverCardCtx {
    // State
    open: Memo<bool>,
    set_open: Callback<bool>,
    timers: HoverTimers,

    // ARIA attributes
    content_id: Signal<String>,
}

/// The props for the [`HoverCard`] component
#[derive(Props, Clone, PartialEq)]
pub struct HoverCardProps {
    /// Whether the hover card is open
    pub open: ReadSignal<Option<bool>>,

    /// Default open state
    #[props(default)]
    pub default_open: bool,

    /// Callback when open state changes
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// Whether the hover card is disabled
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Milliseconds the pointer must rest on the trigger before the card
    /// opens (Radix `openDelay`, Base UI `PreviewCard.Trigger` `delay`).
    /// Defaults to 600, Base UI's value. Keyboard focus opens it at once.
    #[props(default = ReadSignal::new(Signal::new(DEFAULT_HOVER_CARD_DELAY)))]
    pub delay: ReadSignal<u64>,

    /// Milliseconds the card stays open after the pointer leaves the trigger
    /// or the card (Radix `closeDelay`, Base UI `closeDelay`). Entering the
    /// card again within that time cancels the close. Defaults to 300, Base
    /// UI's value. Blur and Escape close it at once.
    #[props(default = ReadSignal::new(Signal::new(DEFAULT_HOVER_CARD_CLOSE_DELAY)))]
    pub close_delay: ReadSignal<u64>,

    /// Additional attributes for the hover card
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the hover card
    pub children: Element,
}

/// # HoverCard
///
/// The `HoverCard` component wraps a [`HoverCardTrigger`] and a [`HoverCardContent`]. It provides a way to show additional information when hovering over an element.
///
/// ## No `open_on_hover` option
///
/// Unlike the menus' `open_on_hover` switch (`NavigationMenu`, `Navbar`,
/// `Menubar`, the `DropdownMenu`/`ContextMenu` submenus), a hover card has no
/// click-activated mode: hover (and focus) *is* its definition -- the content
/// is supplementary and must never be the only way to reach anything. For a
/// panel opened by a click, use [`crate::popover`].
///
/// ## Delays
///
/// [`HoverCardProps::delay`] and [`HoverCardProps::close_delay`] (milliseconds)
/// are Radix `HoverCard`'s `openDelay`/`closeDelay` and Base UI
/// `PreviewCard.Trigger`'s `delay`/`closeDelay`, with Base UI's defaults: 600
/// to open, 300 to close.
///
/// - Nothing opens until the pointer has rested on the trigger for `delay`;
///   leaving before then cancels the open, so sweeping across a row of
///   triggers flashes nothing.
/// - Leaving the trigger (or the card) closes after `close_delay`; entering
///   the card, or the trigger again, in that time cancels the close, so the
///   pointer can cross the gap between them.
/// - Keyboard focus opens the card at once and blur or Escape close it at
///   once: the delays are a pointer's, not a keyboard's.
/// - The timers belong to the component's scope, so unmounting cancels them,
///   and `disabled` is re-checked when one fires.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::{
///     ContentAlign, ContentSide,
///     hover_card::{
///         HoverCard, HoverCardContent, HoverCardTrigger,
///     }
/// };
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         HoverCard {
///             HoverCardTrigger {
///                 i { "Dioxus" }
///             }
///             HoverCardContent {
///                 side: ContentSide::Bottom,
///                 div {
///                     padding: "1rem",
///                     "Dioxus is"
///                     i { " the " }
///                     "Rust framework for building fullstack web, desktop, and mobile apps. Iterate with live hotreloading, add server functions, and deploy in record time."
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`HoverCard`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates the current state of the hover card. Values are `open` or `closed`.
/// - `data-disabled`: Indicates whether the item is disabled. Values are `true` or `false`.
#[component]
pub fn HoverCard(props: HoverCardProps) -> Element {
    // See `Tooltip`'s identical call for why this must be at the root,
    // installed well before the first open, rather than only from
    // `use_anchor_position_fallback` (which only runs once `HoverCardContent`
    // first mounts).
    #[cfg(feature = "web")]
    use_effect(crate::top_layer::ensure_anchor_positioning_styles);

    let (open, set_open) = use_controlled(props.open, props.default_open, props.on_open_change);
    // Generate a unique ID for the hover card content
    let content_id = use_unique_id();
    let timers = use_hover_timers(
        open,
        set_open,
        props.disabled,
        props.delay,
        props.close_delay,
    );

    use_context_provider(|| HoverCardCtx {
        open,
        set_open,
        timers,
        content_id,
    });

    // Owned by this component -- open/disabled state must win over a
    // caller's own attributes (`dev-docs/backlog.md` row 93's
    // duplicate-attribute hazard).
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(div {
            "data-state": if open() { "open" } else { "closed" },
            "data-disabled": (props.disabled)(),
        }),
    ]);

    rsx! {
        div {
            ..attributes,

            {props.children}
        }
    }
}

/// The props for the [`HoverCardTrigger`] component
#[derive(Props, Clone, PartialEq)]
pub struct HoverCardTriggerProps {
    /// Optional ID for the trigger element
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes for the hover card trigger
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the hover card trigger
    pub children: Element,
}

/// # HoverCardTrigger
///
/// The [`HoverCardTrigger`] component triggers the [`HoverCardContent`] to appear when hovered or focused.
///
/// This component must be used inside a [`HoverCard`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::{
///     ContentAlign, ContentSide,
///     hover_card::{
///         HoverCard, HoverCardContent, HoverCardTrigger,
///     }
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         HoverCard {
///             HoverCardTrigger {
///                 i { "Dioxus" }
///             }
///             HoverCardContent {
///                 side: ContentSide::Bottom,
///                 div {
///                     padding: "1rem",
///                     "Dioxus is"
///                     i { " the " }
///                     "Rust framework for building fullstack web, desktop, and mobile apps. Iterate with live hotreloading, add server functions, and deploy in record time."
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn HoverCardTrigger(props: HoverCardTriggerProps) -> Element {
    let ctx: HoverCardCtx = use_context();

    // Generate a unique ID for the trigger
    let trigger_id = use_unique_id();

    // Use use_id_or to handle the ID
    let id = use_id_or(trigger_id, props.id);

    // Tooltip pattern: "Escape: Dismisses the Tooltip." HoverCard shares
    // Tooltip's role and show-on-focus/hide-on-blur contract (see
    // `tooltip.rs`'s identical `handle_keydown`), so it needs the same
    // `onkeydown`.
    let handle_keydown = move |event: Event<KeyboardData>| {
        if event.key() == Key::Escape && (ctx.open)() {
            event.prevent_default();
            ctx.timers.dismiss();
        }
    };

    // Ties this trigger to the content's `position-anchor` so its
    // `[data-side]` CSS still resolves relative to this trigger once the
    // content is promoted to the top layer -- see
    // `crate::top_layer::anchor_name_style`/`anchored_trigger_attributes`.
    // Not a bare `style: anchor_name_style(..)` literal directly
    // alongside the `..props.attributes` spread below: a caller supplying
    // their own `style` would then produce two `style` attributes on this
    // same element (`dev-docs/conformance-harness.md` hydration-parity
    // Rule 4; `scripts/check-attr-spread-collision.sh`). Folding it here
    // first, like every other anchored trigger, avoids that and (unlike plain
    // `merge_attributes`, which folds a plain `style` attribute but leaves
    // shorthand style props -- `padding: ".."`, namespace `Some("style")`
    // -- as separate attributes) renders exactly one `style` carrying the
    // anchor binding plus whatever the caller supplied.
    let attributes =
        crate::top_layer::anchored_trigger_attributes(&ctx.content_id.cloned(), props.attributes);
    // Owned by this component -- role/focusability/aria wiring must win
    // over a caller's own attributes (`dev-docs/backlog.md` row 93's
    // duplicate-attribute hazard).
    let attributes = merge_attributes(vec![
        attributes,
        attributes!(div {
            tabindex: "0", // Make the trigger focusable
            role: "button",
            aria_describedby: (ctx.open)().then(|| ctx.content_id.cloned()),
        }),
    ]);

    rsx! {
        div {
            id,
            // Mouse events: delayed by `delay` / `close_delay`
            onmouseenter: move |_| ctx.timers.pointer_enter(),
            onmouseleave: move |_| ctx.timers.pointer_leave(),

            // Focus events: keyboard, so immediate
            onfocus: move |_| ctx.timers.focus(),
            onblur: move |_| ctx.timers.blur(),
            onkeydown: handle_keydown,

            ..attributes,
            {props.children}
        }
    }
}

/// The props for the [`HoverCardContent`] component
#[derive(Props, Clone, PartialEq)]
pub struct HoverCardContentProps {
    /// Optional ID for the hover card content
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Side of the trigger to place the hover card
    #[props(default = ContentSide::Top)]
    pub side: ContentSide,

    /// Alignment of the hover card relative to the trigger
    #[props(default = ContentAlign::Center)]
    pub align: ContentAlign,

    /// Keeps the card in the DOM while it is closed, hidden (`hidden`, `data-state="closed"`), like
    /// Radix's `forceMount`: for an animation library that drives the card itself, or content that
    /// should exist in the markup. By default (`false`) a closed card is not rendered at all; it is
    /// mounted when it opens and removed after its exit animation.
    #[props(default = false)]
    pub force_mount: bool,

    /// Additional attributes for the hover card content
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the hover card content
    pub children: Element,
}

/// # HoverCardContent
///
/// The [`HoverCardContent`] component defines the content of the parent [`HoverCard`]. It is only rendered while the hover card is open (and for its exit animation), unless [`HoverCardContentProps::force_mount`] is set, which keeps it in the DOM, hidden, while it is closed.
///
/// This component must be used inside a [`HoverCard`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::{
///     ContentAlign, ContentSide,
///     hover_card::{
///         HoverCard, HoverCardContent, HoverCardTrigger,
///     }
/// };
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         HoverCard {
///             HoverCardTrigger {
///                 i { "Dioxus" }
///             }
///             HoverCardContent {
///                 side: ContentSide::Bottom,
///                 div {
///                     padding: "1rem",
///                     "Dioxus is"
///                     i { " the " }
///                     "Rust framework for building fullstack web, desktop, and mobile apps. Iterate with live hotreloading, add server functions, and deploy in record time."
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`HoverCardContent`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates the current state of the hover card. Values are `open` or `closed`.
/// - `data-side`: Indicates the side of the trigger where the hover card is placed. Values are `top`, `right`, `bottom`, or `left`.
/// - `data-align`: Indicates the alignment of the hover card relative to the trigger. Values are `start`, `center`, or `end`.
#[component]
pub fn HoverCardContent(props: HoverCardContentProps) -> Element {
    let ctx: HoverCardCtx = use_context();

    let is_open = (ctx.open)();

    // Use use_id_or to handle the ID
    let id = use_id_or(ctx.content_id, props.id);

    let render = use_animated_open(id, ctx.open);

    // `HoverCardContentRendered` is a real component (not a plain fn) so it
    // -- and the hooks it calls internally (`use_popover_shown_while_mounted`
    // on the web arm) -- get a fresh
    // scope each time `render()` mounts/unmounts it, matching this
    // element's actual DOM lifetime. See
    // `tooltip.rs`'s identical `TooltipContentRendered` comment for why a
    // plain fn here would be a conditional-hook-call hazard instead.
    // `force_mount` keeps the card in the DOM while it is closed, as a dormant instance:
    // `use_animated_open` still decides when the card is *shown* (open, and for its exit
    // animation), and `shown` is the instance's key, so the card is remounted at each of those
    // edges -- a card whose exit animation has finished leaves the top layer by being replaced,
    // exactly as an unmounted one does, and never stays a closed card that is still painted.
    let shown = render();
    rsx! {
        if shown || props.force_mount {
            HoverCardContentRendered {
                key: "{shown}",
                dormant: !shown,
                id: id.cloned(),
                open: ctx.open,
                set_open: ctx.set_open,
                is_open,
                side: props.side,
                align: props.align,
                attributes: props.attributes,
                children: props.children,
            }
        }
    }
}

/// Web arm (Phase 4.4, dev-docs/plan.md): promote to the top layer via
/// `popover="manual"` (see `crate::top_layer::PopoverKind::Manual`'s doc for
/// why `manual` and not `auto` — a `HoverCard`'s own mouseenter/mouseleave
/// pair already owns its lifecycle, and MDN's own naming for this pattern
/// ("hover card") does not imply light dismiss the way a click-triggered
/// popover does).
///
/// Uses `crate::top_layer::use_popover_shown_while_mounted`, not
/// `use_popover_sync` (dev-docs/backlog.md row 19) -- same reasoning as
/// `tooltip.rs`'s identical `TooltipContentRendered`: this component is
/// mounted by `render()` (`use_animated_open`) above for the whole CSS
/// close animation, and `use_popover_sync` would call `hidePopover()` the
/// instant `open` goes `false`, hiding the element via the UA's
/// `[popover]:not(:popover-open) { display: none }` rule before that
/// animation ever plays. `use_popover_shown_while_mounted` never calls
/// `hidePopover()` from the closing path; only `render()` dropping to
/// `false` and the real DOM removal that follows takes this element out of
/// the top layer.
#[cfg(feature = "web")]
#[component]
fn HoverCardContentRendered(
    /// Whether the card is only mounted because of `force_mount`: closed, and not in its exit animation. It is hidden.
    dormant: bool,
    /// The DOM id of the card element.
    id: String,
    /// Whether the card is open. On the web, its popover is kept shown while this is true.
    open: Memo<bool>,
    /// Sets the open state. On the web, it is called when the browser or a scroll closes the card.
    set_open: Callback<bool>,
    /// Whether the card is open. Sets its `data-state`.
    is_open: bool,
    /// The side of the trigger the card is placed on.
    side: ContentSide,
    /// The alignment of the card relative to the trigger.
    align: ContentAlign,
    /// Extra HTML attributes for the card element.
    attributes: Vec<Attribute>,
    /// The card's content.
    children: Element,
) -> Element {
    crate::top_layer::use_popover_shown_while_mounted(id.clone(), open, set_open);
    // JS-measured static positioning fallback for Firefox/WebKit (no CSS
    // Anchor Positioning) -- see `top_layer::use_anchor_position_fallback`'s
    // doc. `anchor_id` is this content's own `id`: `HoverCardTrigger`'s
    // `anchor-name` is keyed off `ctx.content_id`, which this component
    // (below) keeps synced to this same id.
    crate::top_layer::use_anchor_position_fallback(id.clone(), id.clone(), open, side, align, 10);
    // Any scroll closes a hover card (`top_layer::use_close_on_scroll`'s doc).
    crate::top_layer::use_close_on_scroll(
        id.clone(),
        id.clone(),
        open,
        crate::top_layer::ScrollClose::AnyScroll,
        set_open,
    );

    // Entering the card holds it open (cancels a pending close); leaving it
    // starts the same `close_delay` as leaving the trigger. Both share the
    // root's timers, so a pointer crossing trigger -> card or card -> trigger
    // always has exactly one close pending, never two.
    let timers = use_context::<HoverCardCtx>().timers;
    let handle_mouse_enter = move |_: Event<MouseData>| timers.hold();
    let handle_mouse_leave = move |_: Event<MouseData>| timers.pointer_leave();

    // `anchored_content_attributes` below writes `data-dx-anchored="hover-card"`, the attribute the
    // engine's anchor stylesheet selects on (see `tooltip.rs`'s `TooltipContentRendered` for why an
    // attribute and not a class).
    let attributes = merge_attributes(vec![
        attributes,
        // Owned by this component -- role/top-layer wiring/positioning
        // state must win over a caller's own attributes
        // (`dev-docs/backlog.md` row 93's duplicate-attribute hazard).
        attributes!(div {
            role: "tooltip",
            popover: crate::top_layer::PopoverKind::Manual.as_str(),
            "data-state": if is_open { "open" } else { "closed" },
            "data-side": side.as_str(),
            "data-align": align.as_str(),
            hidden: dormant,
        }),
    ]);
    // Folds the caller's own `style` together with the anchor binding into
    // one `style` attribute -- see `top_layer::anchored_content_attributes`'s
    // doc for why a bare `style: position_anchor_style(&id)` literal
    // alongside `..attributes` is the duplicate-`style` hazard
    // (`dev-docs/conformance-harness.md` hydration-parity Rule 4).
    let attributes = crate::top_layer::anchored_content_attributes(
        crate::top_layer::AnchorKind::HoverCard,
        &id,
        attributes,
    );

    rsx! {
        div {
            id: id.clone(),
            onmouseenter: handle_mouse_enter,
            onmouseleave: handle_mouse_leave,
            // Escape dismisses from inside the card too, in case it holds
            // focusable content -- same Tooltip-pattern contract as the
            // trigger's own `handle_keydown` (`HoverCardTrigger`, above).
            onkeydown: move |event: Event<KeyboardData>| {
                if event.key() == Key::Escape {
                    event.prevent_default();
                    timers.dismiss();
                }
            },
            ..attributes,
            {children}
        }
    }
}

/// Native (Blitz) arm -- see
/// `dev-docs/reference/platform-rationale.md` Caveat 2.
#[cfg(not(feature = "web"))]
#[component]
fn HoverCardContentRendered(
    /// Whether the card is only mounted because of `force_mount`: closed, and not in its exit animation. It is hidden.
    dormant: bool,
    /// The DOM id of the card element.
    id: String,
    // Unused on this arm -- kept as a same-named field so the shared,
    // cfg-independent call site in `HoverCardContent` compiles unchanged on
    // both arms.
    /// Whether the card is open. On the web, its popover is kept shown while this is true.
    open: Memo<bool>,
    /// Sets the open state. On the web, it is called when the browser or a scroll closes the card.
    set_open: Callback<bool>,
    /// Whether the card is open. Sets its `data-state`.
    is_open: bool,
    /// The side of the trigger the card is placed on.
    side: ContentSide,
    /// The alignment of the card relative to the trigger.
    align: ContentAlign,
    /// Extra HTML attributes for the card element.
    attributes: Vec<Attribute>,
    /// The card's content.
    children: Element,
) -> Element {
    let _ = (open, set_open);
    // Entering the card holds it open (cancels a pending close); leaving it
    // starts the same `close_delay` as leaving the trigger. Both share the
    // root's timers, so a pointer crossing trigger -> card or card -> trigger
    // always has exactly one close pending, never two.
    let timers = use_context::<HoverCardCtx>().timers;
    let handle_mouse_enter = move |_: Event<MouseData>| timers.hold();
    let handle_mouse_leave = move |_: Event<MouseData>| timers.pointer_leave();

    // Owned by this component -- see the web arm's identical construction
    // above (`dev-docs/backlog.md` row 93).
    let attributes = merge_attributes(vec![
        attributes,
        attributes!(div {
            role: "tooltip",
            "data-state": if is_open { "open" } else { "closed" },
            "data-side": side.as_str(),
            "data-align": align.as_str(),
            hidden: dormant,
        }),
    ]);

    rsx! {
        div {
            id,
            onmouseenter: handle_mouse_enter,
            onmouseleave: handle_mouse_leave,
            onkeydown: move |event: Event<KeyboardData>| {
                if event.key() == Key::Escape {
                    event.prevent_default();
                    timers.dismiss();
                }
            },
            ..attributes,
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[component]
    fn Card(force_mount: bool) -> Element {
        rsx! {
            HoverCard { open: None::<bool>,
                HoverCardTrigger { "trigger" }
                HoverCardContent { force_mount, "card body" }
            }
        }
    }

    fn render(force_mount: bool) -> String {
        let mut dom = VirtualDom::new_with_props(Card, CardProps { force_mount });
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn a_closed_card_is_not_rendered_unless_it_is_force_mounted() {
        // backlog row 425: `force_mount` keeps the closed card in the DOM, hidden; the default
        // renders nothing while it is closed.
        let html = render(false);
        assert!(!html.contains("card body"), "{html}");

        let html = render(true);
        assert!(html.contains("card body"), "{html}");
        assert!(html.contains("data-state=\"closed\""), "{html}");
        assert!(html.contains("hidden=true"), "{html}");
    }
}
