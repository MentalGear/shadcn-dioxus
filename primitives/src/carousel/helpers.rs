use super::*;

/// Whether stepping from data index `old` to `new` (out of `count` total)
/// is a wraparound -- the last slide to the first, or the first to the
/// last -- rather than an ordinary adjacent step. This is what
/// [`Carousel`]'s own mount-time settle effect uses to decide whether a
/// [`LoopMode::Rewind`] transition should jump instantly rather than
/// animate smoothly (see that effect's own doc).
///
/// Ambiguous by construction for `count == 2`: with only two slides,
/// "step to the other one" and "wrap to the other one" are the identical
/// transition (`old = 0, new = 1` is simultaneously an ordinary Next step
/// and what a Previous-wrap would also produce), so this reports `true`
/// for every step at `count == 2` -- accepted rather than resolved, since
/// an instant vs. animated single-slide hop is not a meaningfully
/// different experience, and no caller-observable index pair exists that
/// could distinguish the two cases anyway.
pub(super) fn is_rewind_wrap(old: usize, new: usize, count: usize) -> bool {
    if count == 0 || old == new {
        return false;
    }
    let last = count - 1;
    (old == last && new == 0) || (old == 0 && new == last)
}

/// # Gap model
///
/// shadcn's own carousel puts the inter-slide gap INSIDE each slide's own
/// border-box (a leading-edge `padding-inline`/`padding-block`), with
/// `CarouselContent`'s own scroller carrying the exactly-compensating
/// negative margin on that same edge/axis -- the standard Tailwind
/// `-ml-4`/`pl-4` idiom -- rather than a real flex `gap` between items.
/// This crate ports that construction (backlog row 91's shadcn-parity
/// research) via [`item_gap_padding`]/[`content_gap_margin`] below, exposed
/// as one custom property, `--dx-carousel-gap` (raw-primitive default
/// `0px` -- a themed stylesheet is what gives it an actual token-based
/// value, e.g. `--dx-space-4`, matching this module's own
/// "structural-only, zero-theme-CSS" posture for every other layout-critical
/// declaration in this module).
///
/// Why this matters, not just "matches shadcn": a real flex `gap` is
/// ADDITIVE to a percentage `flex-basis` -- `N` items at `flex-basis:
/// calc(100%/N)` plus `(N-1)` real gaps demand more main-axis size than the
/// container has, by exactly `(N-1) * gap`, so the track overflows and the
/// visible slide count is no longer a whole number (see
/// `dev-docs/backlog.md` row 91's shadcn-carousel-parity research for the
/// "~2.3 slides visible" failure this construction prevents, with no
/// compensating arithmetic needed at the call site). With the gap living
/// INSIDE each item's own border-box instead, `N` items' basis fractions
/// still sum to exactly 100% of the (`-ml`-widened) content box regardless
/// of `N`, so [`CarouselItem`]/[`CarouselVirtualContent`]'s own
/// `--dx-carousel-per-view`/`--dx-carousel-peek` basis `calc()` (see their
/// own "Sizes" doc) never needs to subtract a gap term at all.
pub(super) fn item_gap_padding(orientation: CarouselOrientation) -> &'static str {
    match orientation {
        CarouselOrientation::Horizontal => "padding-inline-start:var(--dx-carousel-gap, 0px);",
        CarouselOrientation::Vertical => "padding-block-start:var(--dx-carousel-gap, 0px);",
    }
}

/// # Sizes (whole slides by default, opt-in peek)
///
/// [`CarouselItem`]/[`CarouselVirtualContent`]'s own default `flex-basis`
/// is `calc((100% - var(--dx-carousel-peek, 0%)) / var(--dx-carousel-per-view, 1))`
/// -- `--dx-carousel-per-view` (an integer, default `1`) is how many WHOLE
/// slides show at once, and `--dx-carousel-peek` (a percentage of one
/// slide's own share of the track, default `0%`) is how much of the
/// *next* slide additionally peeks into view, opt-in only. With both left
/// at their defaults this is exactly `100%` -- the plain basis, so a caller
/// that overrides `flex-basis` directly via an inline `style`, e.g. this
/// package's own `multiple` variant, is unaffected: an
/// inline `style`-supplied `flex-basis` still simply appears later in the
/// same `style` attribute and wins (this
/// module's own established "caller style always wins" construction,
/// [`fold_style_attributes`]'s own call sites throughout this module).
///
/// Because the gap (see the "Gap model" doc above) already lives inside
/// each item's own border-box as padding, this `calc()` never needs to
/// subtract a gap term itself -- `N` whole slides' basis fractions already
/// sum to exactly `100%` of the (gap-widened) content box regardless of
/// `N`.
///
/// **`box-sizing: border-box` is declared here, inline, because that
/// sentence is only true under it.** Under the default `content-box` the gap
/// padding is added *on top of* the basis, so every slide is one gap wider
/// than the snapport. A themed `.dx-carousel-item` class cannot supply it:
/// [`CarouselVirtualContent`]'s own slides (rendered by this
/// primitive, with no class) never carry that class, so they would
/// measure 352px in a 336px snapport. A snap area larger than the snapport makes every
/// position inside it a valid rest position (CSS Scroll Snap: snap areas
/// larger than the snapport), so the browser would not snap those slides
/// at all -- a wheel/trackpad gesture would come to rest up to one gap off a
/// slide boundary, and the virtual window's own re-align would then "snap" the
/// leftover itself, as a visible instant `scrollBy` at scroll end
/// (`playwright/carousel-virtual-wheel.spec.ts`). Declaring it on the one
/// style every slide renderer shares ([`CarouselItem`] and
/// [`CarouselVirtualContent`] alike) makes the invariant hold by
/// construction, with or without any theme stylesheet -- this module's "works
/// with zero theme CSS" rule for layout-critical properties. A caller's own
/// inline `box-sizing` still wins (it comes later in the same `style`
/// attribute).
pub(super) fn item_basis_style() -> &'static str {
    "box-sizing:border-box;flex:0 0 calc((100% - var(--dx-carousel-peek, 0%)) / var(--dx-carousel-per-view, 1));"
}

/// The mirror of [`item_gap_padding`], applied to [`CarouselContent`]'s own
/// scroller element (see that function's own doc for the full construction):
/// a negative margin on the identical edge/axis, so the FIRST item's own
/// leading-edge padding lands flush with the viewport's own clip boundary
/// (`CarouselContent`'s own `carousel-viewport` wrapper) rather than shifting
/// every slide's visible content rightward/downward by one gap's worth.
pub(super) fn content_gap_margin(orientation: CarouselOrientation) -> &'static str {
    match orientation {
        CarouselOrientation::Horizontal => {
            "margin-inline-start:calc(var(--dx-carousel-gap, 0px) * -1);"
        }
        CarouselOrientation::Vertical => {
            "margin-block-start:calc(var(--dx-carousel-gap, 0px) * -1);"
        }
    }
}

/// The `gap` prop ([`CarouselContentProps::gap`],
/// [`CarouselVirtualContentProps::gap`]) as the inline declaration it
/// renders: `--dx-carousel-gap:<value>;`, or nothing for `None`/blank. Kept
/// separate from [`content_gap_margin`]/[`item_gap_padding`] on purpose: it
/// only sets the variable those already read, so the axis handling stays in
/// one place.
pub(super) fn content_gap_var(gap: Option<&str>) -> String {
    match gap.map(str::trim) {
        Some(value) if !value.is_empty() => format!("--dx-carousel-gap:{value};"),
        _ => String::new(),
    }
}

/// The `nav_disabled_opacity` prop ([`CarouselProps::nav_disabled_opacity`])
/// as the inline declarations it renders on the root: the opacity itself as
/// `--dx-carousel-nav-disabled-opacity:<n>;`, plus -- only when the value is
/// `0` -- `--dx-carousel-nav-disabled-visibility:hidden;`. `None` and
/// non-finite values (`NaN`, infinities) render nothing, leaving whatever the
/// stylesheet says in force; anything else is clamped into `0..=1`.
///
/// **Why a second variable.** An arrow faded to nothing should not stay
/// reachable by assistive technology (an `opacity: 0` control is still in the
/// accessibility tree and still hit-testable), and CSS has no way to turn "the
/// opacity variable is `0`" into `visibility: hidden` (a style query on a
/// custom property is not available in every engine, and a numeric custom
/// property cannot be mapped to a keyword). So the prop states the keyword
/// itself, in a variable of its own that the stylesheet reads with a `visible`
/// fallback. A stylesheet-only caller sets the same two variables by hand.
pub(super) fn nav_disabled_vars(opacity: Option<f32>) -> String {
    match opacity.filter(|value| value.is_finite()) {
        Some(value) => {
            let value = value.clamp(0.0, 1.0);
            if value == 0.0 {
                "--dx-carousel-nav-disabled-opacity:0;--dx-carousel-nav-disabled-visibility:hidden;"
                    .to_string()
            } else {
                format!("--dx-carousel-nav-disabled-opacity:{value};")
            }
        }
        None => String::new(),
    }
}

/// `overscroll-behavior` on the scroll axis, from whether BOTH physical ends
/// of the currently rendered slides are true ends of the data (the same rule
/// the rubber-bands follow, `carousel_true_end_js!`): `""` (the platform's
/// default, `auto`) when they are, `none` otherwise.
///
/// **Why.** A quick trackpad flick must not produce an "end of slides"
/// rubber-band while more slides are still to come. A single flick holds the
/// scroller clamped against its rendered slice's physical end mid-list (a
/// virtual list), or against its physical ends (a looping list, which has no
/// true end), for many frames -- with `overscroll-behavior: auto`, which is
/// exactly where macOS Firefox draws its own native elastic bounce. The
/// browser-idle gate deliberately keeps the window from re-centring
/// mid-gesture (THE RULE: never write the scroll position while the browser
/// is scrolling), and a gesture can travel far more than any slice holds
/// (a recorded gesture spans ~16 slides;
/// Chromium moves up to one slide per wheel event even with
/// `scroll-snap-stop: always`), so reaching a false end cannot be prevented
/// by sizing the slice without abandoning virtualisation. What can be
/// prevented is the lie: at a false end the platform must not bounce.
///
/// **The rule, per axis.** `overscroll-behavior` covers both ends of an axis
/// at once, so it is `none` whenever EITHER end of the rendered slides is a
/// false end: a looping carousel (plain or virtual -- no true end at all), or
/// a virtualised window that does not currently hold both the first and the
/// last item. Only when both ends are true (plain non-looping content, or a
/// virtual list rendering its whole data set) does it stay the platform
/// default, so a plain non-looping
/// carousel keeps it. Trade-off, accepted: while one end of a virtual window is true
/// and the other false (e.g. items 0-2 of 200), the true end loses the
/// platform's own bounce (this crate's wheel/drag band still engages there,
/// being a transform) and, for a vertical virtual list, scroll chaining to
/// the page at that end; a false end bouncing is what this prevents, and
/// chaining away from the middle of a list is never right either. `none`
/// also stops scroll chaining for a looping carousel's axis. Deterministic
/// from state, never an effect-only value: a virtual list's server render
/// and hydrating first render both render the whole list (so the same
/// value); only the post-mount windowing changes it.
pub(super) fn content_overscroll_style(
    orientation: CarouselOrientation,
    both_ends_true: bool,
) -> &'static str {
    if both_ends_true {
        return "";
    }
    match orientation {
        CarouselOrientation::Horizontal => "overscroll-behavior-inline:none;",
        CarouselOrientation::Vertical => "overscroll-behavior-block:none;",
    }
}

/// The radius [`CarouselVirtualContent`] actually renders with: the
/// caller's `radius`, except that a looping list's window is capped below
/// its data count. A wrapping window wider than the data repeats items
/// (`virtual::window` keys by position, so it would render, e.g., two
/// "3 of 12" slides) -- and since selection and `inert` follow the DATA
/// index, both copies would read as the current slide. Capping at
/// `(count - 2) / 2` keeps `2 * r + 1 < count`, one rendered slide per item,
/// which also keeps the auto-virtualise rule (`count > 2 * r + 1`) true, so a
/// looping list of 4+ items still gets its seamless window. Non-looping
/// lists, and loops too small to window (under 4 items), use `radius`
/// as given.
pub(super) fn effective_radius(radius: usize, count: usize, looping: bool) -> usize {
    if looping && count >= 4 {
        radius.min((count - 2) / 2).max(1)
    } else {
        radius
    }
}

/// Clamp `index` into the valid range for a carousel of `count` slides
/// (`0` when `count` is `0`, otherwise `[0, count - 1]`). The one
/// definition every path that could produce an out-of-range index
/// (a controlled `value` prop, a stale `selected` before the first
/// [`CarouselItem`] has registered, a JS-reported scroll position) is
/// funneled through, so a caller passing e.g. `value: Some(99)` on a
/// 3-slide carousel can never desync [`CarouselApi::can_scroll_next`]
/// from what is actually rendered.
pub(super) fn clamp_selected(index: usize, count: usize) -> usize {
    if count == 0 {
        0
    } else {
        index.min(count - 1)
    }
}

/// Defensive clamp for a `(start, end)` visible-range reported by a JS
/// settle bridge -- guards against a reply that raced a shrinking `count`
/// (an item unregistering between the geometry read and this signal
/// write). Never lets either end point past the last valid index, and
/// never lets `start` exceed `end` once clamped.
pub(super) fn clamp_visible_range(range: (usize, usize), count: usize) -> (usize, usize) {
    if count == 0 {
        return (0, 0);
    }
    let last = count - 1;
    let start = range.0.min(last);
    let end = range.1.min(last).max(start);
    (start, end)
}

/// Whether a "previous" action is currently meaningful, without `loop`
/// (research §8.2; see [`step_prev`] for the looping case): a real,
/// reachable boundary -- `index == 0` -- not a cosmetic one.
pub(super) fn can_scroll_prev(selected: usize) -> bool {
    selected > 0
}

/// Whether a "next" action is currently meaningful -- the mirror of
/// [`can_scroll_prev`]. `count == 0` (no slides registered yet, e.g. the
/// very first render before any [`CarouselItem`] has mounted) is never
/// scrollable either way.
pub(super) fn can_scroll_next(selected: usize, count: usize) -> bool {
    count > 0 && selected + 1 < count
}

/// The index one step forward from `selected`, saturating at the last
/// slide (never wrapping -- see [`can_scroll_next`]'s doc).
pub(super) fn next_selected(selected: usize, count: usize) -> usize {
    if count == 0 {
        0
    } else {
        (selected + 1).min(count - 1)
    }
}

/// The index one step back from `selected`, saturating at `0` (never
/// wrapping -- see [`can_scroll_prev`]'s doc).
pub(super) fn prev_selected(selected: usize) -> usize {
    selected.saturating_sub(1)
}

/// The index one step back from `selected`, honoring `loop_enabled` --
/// the looping model (backlog row 91, `dev-docs/archive/research/carousel-2026-09-19.md`
/// §8.2): **rewind-style** looping. From the first slide, wraps to the
/// last (`count - 1`) rather than saturating; the actual scroll is still a
/// single [`CAROUSEL_SCROLL_TO_JS`] paging call to that far index
/// (a visible "rewind" across every intervening slide), not an
/// embla-style cloned-node illusion -- clones would violate the overscroll
/// port's own invariant 5 (`dev-docs/archive/research/carousel-overscroll-2026-09-23.md`
/// §6): they would enter the snap engine's own candidate list, the "N of
/// M" slide count, and `:nth-child` styling. Falls back to
/// [`prev_selected`]'s ordinary saturating behavior when `loop_enabled` is
/// `false` (the default) or `count == 0` (nothing to wrap to
/// either way).
pub(super) fn step_prev(selected: usize, count: usize, loop_enabled: bool) -> usize {
    if loop_enabled && count > 0 {
        if selected == 0 {
            count - 1
        } else {
            selected - 1
        }
    } else {
        prev_selected(selected)
    }
}

/// The mirror of [`step_prev`]: one step forward, wrapping from the last
/// slide back to the first under `loop_enabled`.
pub(super) fn step_next(selected: usize, count: usize, loop_enabled: bool) -> usize {
    if loop_enabled && count > 0 {
        (selected + 1) % count
    } else {
        next_selected(selected, count)
    }
}

/// The shortest signed step, in `[-(count/2), count/2]`, from data index
/// `old` to data index `new` modulo `count` -- [`CarouselVirtualContent`]'s
/// own seamless-loop paging math: the position nearest the current anchor
/// that carries data index `new` is `anchor + shortest_signed_delta(old,
/// new, count)`, for any `old`/`new` in `[0, count)`. `count == 0` (or
/// `old == new`, which can't otherwise arise from two distinct valid data
/// indices) returns `0` -- a caller-checked precondition
/// (`old != new`, both `< count`) rather than a panic, so a defensive
/// double-check costs nothing.
pub(super) fn shortest_signed_delta(old: usize, new: usize, count: usize) -> isize {
    if count == 0 || old == new {
        return 0;
    }
    let n = count as isize;
    let raw = new as isize - old as isize;
    let mut wrapped = ((raw % n) + n) % n;
    if wrapped > n / 2 {
        wrapped -= n;
    }
    wrapped
}

/// The default accessible name for a slide with no name of its own:
/// `"{one-based index} of {count}"`. APG's own sanctioned exception to
/// "don't encode position/size in an accessible name" -- `role="group"`
/// supports neither `aria-setsize` nor `aria-posinset`
/// (dev-docs/archive/research/carousel-2026-09-19.md §1.2).
pub(super) fn slide_label(index: usize, count: usize) -> String {
    format!("{} of {}", index + 1, count)
}

/// Whether [`Carousel`]'s own mount-time paging effect (below,
/// in its component body) should ask [`CAROUSEL_SCROLL_TO_JS`] for
/// an instant jump
/// (`true`) rather than a smooth transition (`false`) on this run.
///
/// `true` on exactly one run per mount: the first one where `has_items`
/// is `true` (registration has completed), regardless of `is_first`'s
/// *own* history of early-returning runs before that point -- see the
/// effect's own doc for why gating on `has_items` rather than "did this
/// particular id lookup succeed" is what makes it impossible for a real
/// user navigation to ever inherit this `true`.
/// Never `true` again afterward, once the caller has also set `is_first`
/// to `false` on any run where `has_items` was `true` (this function
/// itself is pure and does not do that write -- see the call site).
pub(super) fn is_mount_settle(is_first: bool, has_items: bool) -> bool {
    has_items && is_first
}

/// Resolve a keydown on (or bubbled up to) the [`Carousel`] root to a
/// paging intent, or `None` for every other key.
///
/// [`HorizontalNav`] is [`crate::direction`]'s own "which arrow key means
/// which paging intent" result type; reused here as a plain `Prev`/`Next`
/// result rather than duplicating an identical two-variant enum under a
/// new name -- nothing about its own definition or `resolve_horizontal`'s
/// doc restricts it to a *horizontal* caller; only its name does, and
/// this function's own doc is where that reuse is recorded. For
/// [`CarouselOrientation::Horizontal`] this defers entirely to
/// [`Direction::resolve_horizontal`], so it swaps under `dir="rtl"`
/// exactly the way `tabs.rs`'s roving focus does. For
/// [`CarouselOrientation::Vertical`], `ArrowUp`/`ArrowDown` are used
/// unconditionally -- direction never affects a vertical axis (see
/// `Direction::resolve_horizontal`'s own doc).
pub(super) fn carousel_key_intent(
    key: &Key,
    orientation: CarouselOrientation,
    direction: Direction,
) -> Option<HorizontalNav> {
    match orientation {
        CarouselOrientation::Horizontal => direction.resolve_horizontal(key),
        CarouselOrientation::Vertical => match key {
            Key::ArrowUp => Some(HorizontalNav::Prev),
            Key::ArrowDown => Some(HorizontalNav::Next),
            _ => None,
        },
    }
}
