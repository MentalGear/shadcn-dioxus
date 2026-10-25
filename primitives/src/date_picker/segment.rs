use super::*;

/// The integer types a [`DateSegment`] edits (`i32` years, `u8` months and days): ordered,
/// copyable, with `+` and `-` for the arrow-key step. A local trait, so the module needs no
/// `num-integer` crate for its bound alone.
pub(super) trait SegmentInt:
    Copy + Ord + std::ops::Add<Output = Self> + std::ops::Sub<Output = Self>
{
    const ONE: Self;

    /// Step up by one.
    fn inc(&mut self) {
        *self = *self + Self::ONE;
    }

    /// Step down by one.
    fn dec(&mut self) {
        *self = *self - Self::ONE;
    }
}

macro_rules! impl_segment_int {
    ($($t:ty),*) => {$(
        impl SegmentInt for $t {
            const ONE: Self = 1;
        }
    )*};
}
impl_segment_int!(i32, u8);

// The props for the [`DateSegment`] component
#[derive(Props, Clone, PartialEq)]
pub(super) struct DateSegmentProps<T: Clone + SegmentInt + 'static> {
    // The index of the segment
    pub index: ReadSignal<usize>,

    // The controlled value of the date picker
    pub value: ReadSignal<Option<T>>,

    // Default value
    pub default: T,

    // Callback when value changes
    #[props(default)]
    pub on_value_change: Callback<Option<T>>,

    // The minimum value
    pub min: ReadSignal<T>,

    // The maximum value
    pub max: ReadSignal<T>,

    // Max field length
    pub max_length: usize,

    /// Whether Up/Down past `min`/`max` wraps to the opposite bound (the
    /// default) or clamps and stays at the bound -- see
    /// `date_segment_key_effects`'s own doc for the calibration (backlog
    /// row 84 finding 4).
    #[props(default = true)]
    pub wrap: bool,

    // Callback when display placeholder
    pub on_format_placeholder: Callback<(), String>,

    /// An `aria-valuetext` alternative to the raw number, for a value whose
    /// number alone doesn't convey its meaning (e.g. a month name) -- per
    /// the APG spinbutton pattern's Roles/States/Properties section. `None`
    /// (the default) renders no `aria-valuetext` at all. Always `None` while the segment has
    /// no value (see [`DateSegment`]'s own doc), matching `aria-valuenow`'s
    /// own absent-while-untouched rule (backlog row 84 findings 2 and 3).
    #[props(default)]
    pub value_text: Option<String>,

    // Additional attributes for the value element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// A single side effect produced by interpreting a `DateSegment` keydown
/// event, in the exact order `DateSegment`'s `onkeydown` handler should apply
/// them. Keeping this as an ordered list (rather than, say, a handful of
/// independent booleans/options) keeps the effect
/// ordering exact -- which differs between branches (e.g. the empty-text
/// case emits the value before moving focus, while the overflow case moves
/// focus before emitting).
#[derive(Debug, Clone, PartialEq)]
pub(super) enum DateSegmentEffect<T> {
    /// Call `on_value_change` with this value.
    EmitValue(Option<T>),
    /// Move focus to the previous segment.
    FocusPrevious,
    /// Move focus to the next segment.
    FocusNext,
    /// Clear the pending `reset_value` flag.
    ConsumeReset,
    /// Call `prevent_default` on the originating event.
    PreventDefault,
    /// Call `stop_propagation` on the originating event.
    StopPropagation,
}

/// Roll a value that has gone out of `[min, max]` around to the opposite
/// bound (used by the arrow-key handlers so incrementing past `max` wraps to
/// `min`, and vice versa). Used for the month and day segments -- see
/// [`date_segment_key_effects`]'s own doc for the calibration behind which
/// segments wrap and which (the year, [`clamp_value`]) don't.
pub(super) fn roll_value<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        max
    } else if value > max {
        min
    } else {
        value
    }
}

/// Clamp a value that has gone out of `[min, max]` back to the nearest
/// bound, instead of [`roll_value`]'s wrap-to-the-opposite-end behavior.
/// Used by the year segment (backlog row 84 finding 4) -- see
/// [`date_segment_key_effects`]'s own doc for why.
pub(super) fn clamp_value<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Compute the effects of committing a new raw text buffer to a `DateSegment`
/// (shared by the Character/Backspace/Delete handlers, which all end by
/// re-deriving the segment's value from its updated text buffer).
pub(super) fn push_commit_text_effects<T: SegmentInt + Copy + FromStr>(
    effects: &mut Vec<DateSegmentEffect<T>>,
    text: &str,
    min: T,
    max: T,
) {
    if text.is_empty() {
        effects.push(DateSegmentEffect::EmitValue(None));
        effects.push(DateSegmentEffect::FocusPrevious);
        return;
    }

    let value = text.parse::<T>().map(|v| v.min(max)).ok();
    if let Some(value) = value {
        let in_range = value >= min && value <= max;
        // If adding a new digit would exceed max, move to the next segment.
        let new_value = format!("{text}0").parse::<T>().unwrap_or(value);
        if in_range && new_value > max {
            effects.push(DateSegmentEffect::FocusNext);
        }
    }
    effects.push(DateSegmentEffect::EmitValue(value));
}

/// Keyboard modifiers relevant to a `DateSegment` keydown (shortcut passthrough
/// and "clear all" detection).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct SegmentKeyModifiers {
    pub(super) ctrl: bool,
    pub(super) meta: bool,
    pub(super) alt: bool,
}

/// The segment's raw text-buffer state at the time of the keydown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SegmentTextState<'a> {
    pub(super) current_text: &'a str,
    pub(super) max_length: usize,
    pub(super) reset: bool,
}

/// The segment's current numeric value and its bounds/default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SegmentValueBounds<T> {
    pub(super) current_value: Option<T>,
    pub(super) default: T,
    pub(super) min: T,
    pub(super) max: T,
    /// Whether Up/Down past `min`/`max` wraps to the opposite bound
    /// ([`roll_value`]) or clamps at it ([`clamp_value`]) -- see
    /// [`date_segment_key_effects`]'s own doc for the calibration.
    pub(super) wrap: bool,
}

/// Compute what a `DateSegment` should do in response to a keydown, as a pure
/// function of the key, the relevant modifier flags, and the segment's
/// current text/value state. Side-effect free and directly testable --
/// mirrors `move_interaction.rs`'s `MoveEvent::from_keyboard`. The caller
/// (`DateSegment`'s `onkeydown` handler) applies the returned effects in
/// order.
///
/// ## Up/Down wrap-vs-clamp calibration (backlog row 84 finding 4)
///
/// `bounds.wrap` decides what Up/Down do at `min`/`max`: wrap to the
/// opposite bound ([`roll_value`]) or clamp and stay there ([`clamp_value`]).
/// Calibrated against two sources per segment, cited in each caller
/// (`DatePickerYearSegment`/`DatePickerMonthSegment`/`DatePickerDaySegment`):
/// native `<input type="date">` in Chromium (its month/day sub-fields wrap
/// on Up/Down at their own bounds; its year sub-field does not -- it just
/// stops incrementing/decrementing) and the APG spinbutton pattern (silent
/// on wrap-vs-clamp for a bounded value, but its own Roles/States/
/// Properties section frames `aria-valuemin`/`aria-valuemax` as real limits
/// the value stays within, which reads more naturally as "stop at the
/// bound" than "silently jump to the opposite end" for a non-cyclical unit
/// like a year). Month and day are cyclical (day 31 of one frame of
/// reference is adjacent to day 1 of the next), so wrapping matches both
/// user expectation and the native platform control; a year is not
/// cyclical, so clamping is the correct default for it specifically.
pub(super) fn date_segment_key_effects<T: SegmentInt + Copy + FromStr>(
    key: &Key,
    modifiers: SegmentKeyModifiers,
    text_state: SegmentTextState,
    bounds: SegmentValueBounds<T>,
) -> Vec<DateSegmentEffect<T>> {
    let SegmentKeyModifiers { ctrl, meta, alt } = modifiers;
    let SegmentTextState {
        current_text,
        max_length,
        reset,
    } = text_state;
    let SegmentValueBounds {
        current_value,
        default,
        min,
        max,
        wrap,
    } = bounds;

    let mut effects = Vec::new();
    match key {
        Key::Character(actual_char) => {
            // Don't block keyboard shortcuts
            if ctrl || meta || alt {
                return effects;
            }
            if actual_char.parse::<T>().is_ok() {
                let mut text = if current_text.len() == max_length || reset {
                    effects.push(DateSegmentEffect::ConsumeReset);
                    String::new()
                } else {
                    current_text.to_string()
                };
                text.push_str(actual_char);
                push_commit_text_effects(&mut effects, &text, min, max);
            }
            effects.push(DateSegmentEffect::PreventDefault);
            effects.push(DateSegmentEffect::StopPropagation);
        }
        Key::Backspace => {
            let mut text = current_text.to_string();
            if ctrl || meta {
                text.clear();
            } else {
                text.pop();
            }
            push_commit_text_effects(&mut effects, &text, min, max);
        }
        Key::Delete => {
            let mut text = current_text.to_string();
            text.remove(0);
            push_commit_text_effects(&mut effects, &text, min, max);
        }
        Key::ArrowLeft => {
            effects.push(DateSegmentEffect::FocusPrevious);
        }
        Key::ArrowRight => {
            effects.push(DateSegmentEffect::FocusNext);
        }
        Key::Enter => {
            effects.push(DateSegmentEffect::FocusNext);
            effects.push(DateSegmentEffect::PreventDefault);
            effects.push(DateSegmentEffect::StopPropagation);
        }
        Key::ArrowUp => {
            let value = match current_value {
                Some(mut value) => {
                    value.inc();
                    if wrap {
                        roll_value(value, min, max)
                    } else {
                        clamp_value(value, min, max)
                    }
                }
                None => default,
            };
            effects.push(DateSegmentEffect::EmitValue(Some(value)));
        }
        Key::ArrowDown => {
            let value = match current_value {
                Some(mut value) => {
                    value.dec();
                    if wrap {
                        roll_value(value, min, max)
                    } else {
                        clamp_value(value, min, max)
                    }
                }
                None => default,
            };
            effects.push(DateSegmentEffect::EmitValue(Some(value)));
        }
        // `aria-valuemin`/`aria-valuemax` are unconditionally present on every
        // `DateSegment` (see its `rsx!` below), so per the APG Spin Button
        // pattern's Keyboard Interaction section (non-optional, unlike Page
        // Up/Down which that same list marks "(Optional)"):
        // "Home: If the spinbutton has a minimum value, sets the value to
        // its minimum." / "End: ... sets the value to its maximum."
        // (w3c/aria-practices `content/patterns/spinbutton/spinbutton-
        // pattern.html#keyboard_interaction` -- the current,
        // non-deprecated pattern doc, not the deprecated `datepicker-
        // spinbuttons.html` example, which predates this rule and has no
        // Home/End support of its own to conflict with it). Without these
        // arms Home/End fall through to the wildcard arm below and leave
        // `aria-valuenow` unchanged.
        Key::Home => {
            effects.push(DateSegmentEffect::EmitValue(Some(min)));
        }
        Key::End => {
            effects.push(DateSegmentEffect::EmitValue(Some(max)));
        }
        _ => (),
    }
    effects
}

#[component]
pub(super) fn DateSegment<T: Clone + Copy + SegmentInt + FromStr + Display + 'static>(
    props: DateSegmentProps<T>,
) -> Element {
    let mut text_value = use_signal(|| "".to_string());
    use_effect(move || {
        let text = match (props.value)() {
            Some(value) => value.to_string(),
            None => String::default(),
        };
        text_value.set(text);
    });

    let mut reset_value = use_signal(|| false);

    // The formatted text for the segment
    let display_value = use_memo(move || {
        let value = (props.value)();
        match value {
            Some(value) => format!("{:0>width$}", value, width = props.max_length),
            None => props
                .on_format_placeholder
                .call(())
                .repeat(props.max_length),
        }
    });

    // backlog row 84 findings 2/3/6 (the "untouched segment" state machine):
    // `aria-valuenow` must mirror `props.value` faithfully -- present once
    // the segment actually holds a value, absent while it doesn't -- rather
    // than substituting `props.default` for the "no value yet" case
    // (`(props.value)().unwrap_or(props.default)`).
    // That substitution would silently claim a value (typically today's date)
    // the user never set (finding 3), AND make the first Up/Down keypress
    // on a never-touched segment invisible from an aria/assistive-tech
    // standpoint (finding 6): `date_segment_key_effects`'s `None => default`
    // branch (see its own doc) commits exactly `default` on that first
    // press, so if `aria-valuenow` were ALREADY reporting `default`
    // beforehand, the announced value would be identical before and after
    // -- indistinguishable from a dropped keystroke. With "no value" not
    // reported as "= default", that same first press is a real, detectable
    // transition (the attribute goes from absent to present). `value_text`
    // (below, in the rendered `span`) is gated on this same `Option`, so
    // `aria-valuetext` (finding 2) tracks `aria-valuenow` in lockstep
    // instead of drifting into its own "untouched" state.
    let now_value = use_memo(move || (props.value)());

    let mut ctx = use_context::<BaseDatePickerContext>();

    use_effect(move || {
        // If this item is not focused, always keep the value clamped
        if !ctx.focus.is_focused(props.index.cloned()) {
            if let Some(value) = (props.value)() {
                let clamped_value = value.clamp(props.min.cloned(), props.max.cloned());
                if clamped_value != value {
                    props.on_value_change.call(Some(clamped_value));
                }
            }
        }
    });

    let direction = use_direction(None);
    let handle_keydown = move |event: Event<KeyboardData>| {
        // RTL mirrors the horizontal pair (`Direction::ltr_equivalent`): the
        // segments sit in a row that mirrors with `dir`.
        let key = direction.ltr_equivalent(&event.key());
        let modifiers = event.modifiers();
        let effects = date_segment_key_effects(
            &key,
            SegmentKeyModifiers {
                ctrl: modifiers.ctrl(),
                meta: modifiers.meta(),
                alt: modifiers.alt(),
            },
            SegmentTextState {
                current_text: &text_value(),
                max_length: props.max_length,
                reset: reset_value(),
            },
            SegmentValueBounds {
                current_value: (props.value)(),
                default: props.default,
                min: props.min.cloned(),
                max: props.max.cloned(),
                wrap: props.wrap,
            },
        );

        for effect in effects {
            match effect {
                DateSegmentEffect::EmitValue(value) => props.on_value_change.call(value),
                DateSegmentEffect::FocusPrevious => ctx.focus.focus_prev(),
                DateSegmentEffect::FocusNext => ctx.focus.focus_next(),
                DateSegmentEffect::ConsumeReset => reset_value.set(false),
                DateSegmentEffect::PreventDefault => event.prevent_default(),
                DateSegmentEffect::StopPropagation => event.stop_propagation(),
            }
        }
    };

    let disabled = move || (ctx.disabled)();
    let onmounted =
        use_item(collection_item(ctx.focus, props.index).disabled(disabled)).onmounted();

    let span_id = use_unique_id();
    let id = use_memo(move || format!("span-{span_id}"));

    // `id`/`spellcheck`/`enterkeyhint` are overridable defaults (this `id`
    // is a generated fallback, not referenced by any aria wiring -- see the
    // "No `aria-labelledby`" note above); the rest is this segment's own
    // functional state (role/aria-value*/inputmode/contenteditable define
    // the widget, `tabindex`/the data attributes are managed) and is owned.
    let defaults = attributes!(span {
        id,
        spellcheck: false,
        enterkeyhint: "next",
    });
    // `Option<String>` (rather than always-a-string): Dioxus omits the
    // attribute entirely when `None`, the same convention this crate
    // already uses for other optional ARIA state (e.g. `navigation_menu.rs`'s
    // `aria_current: props.active.then_some("page")`) -- see `now_value`'s
    // own doc above for why this must be absent, not a substituted default,
    // while untouched.
    //
    // No `aria-labelledby` here: per the APG Spin Button pattern's own
    // Roles/States/Properties section ("If the spinbutton has a visible
    // label, it is referenced by aria-labelledby ... . Otherwise, the
    // spinbutton element has a label provided by aria-label" --
    // w3c/aria-practices `content/patterns/spinbutton/
    // spinbutton-pattern.html#roles_states_properties`), and
    // every caller here (`DatePickerYearSegment` etc., below) always passes
    // a plain `aria_label: "year"`/`"month"`/`"day"` -- there never is a
    // separate *visible* label element for `aria-labelledby` to reference.
    // An `aria-labelledby="{id}-label"` would be a dangling reference:
    // nothing in this module (or its preview consumers) renders an
    // element with that id (`document.getElementById(idref)` === `null`).
    // Chromium's own accessible-name computation falls back to `aria-label`
    // when `aria-labelledby`'s IDREFs don't resolve (Playwright's
    // `ariaSnapshot()` shows it, and axe-core's ruleset does not flag it
    // either), so it would not be user-visible in this repo's own harness --
    // but it is dead, misleading markup that contradicts the cited rule
    // outright (an `aria-labelledby` with nothing to reference is not
    // "referencing a visible label"), and relying on a fallback another
    // AT/browser might implement differently is exactly the kind of
    // fragility the cited either/or rule exists to avoid. `aria-label`
    // alone (already present via `props.attributes`, merged below) is the
    // correct, sufficient source per that rule.
    let owned = attributes!(span {
        role: "spinbutton",
        aria_valuemin: props.min.to_string(),
        aria_valuemax: props.max.to_string(),
        aria_valuenow: now_value().map(|v| v.to_string()),
        aria_valuetext: props.value_text.clone(),
        inputmode: "numeric",
        contenteditable: !(ctx.read_only)(),
        tabindex: "0",
        "no-date": (props.value)().is_none(),
        "data-disabled": (ctx.disabled)(),
    });
    let merged = merge_attributes(vec![defaults, props.attributes.clone(), owned]);

    rsx! {
        span {
            onkeydown: handle_keydown,
            onmounted,
            onfocus: move |_| {
                reset_value.set(true);
                ctx.focus.set_focus(Some(props.index.cloned()));
                if (ctx.open)() {
                    ctx.open.set(false);
                }
            },
            ..merged,
            {display_value}
        }
    }
}
