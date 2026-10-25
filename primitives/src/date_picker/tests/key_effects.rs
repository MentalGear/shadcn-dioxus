use super::*;

// -----------------------------------------------------------------
// `date_segment_key_effects` and friends (dev-docs/reference/mutants-baseline.md fix #7)
//
// These exercise the pure decision logic extracted from `DateSegment`'s
// `onkeydown` handler (the same shape as `move_interaction.rs`'s
// `MoveEvent::from_keyboard`).
// -----------------------------------------------------------------

/// Test-only convenience wrapper taking the flat argument
/// list, so each case below stays a single readable call.
#[allow(clippy::too_many_arguments)]
fn key_effects(
    key: &Key,
    ctrl: bool,
    meta: bool,
    alt: bool,
    current_text: &str,
    max_length: usize,
    reset: bool,
    current_value: Option<i32>,
    default: i32,
    min: i32,
    max: i32,
) -> Vec<DateSegmentEffect<i32>> {
    // `wrap: true` for every call site below: they exercise the
    // month/day-shaped ("cyclical") behavior of the wrap-vs-clamp split
    // (backlog row 84 finding 4) -- see `key_effects_clamped` for the
    // year-shaped (`wrap: false`) cases.
    date_segment_key_effects(
        key,
        SegmentKeyModifiers { ctrl, meta, alt },
        SegmentTextState {
            current_text,
            max_length,
            reset,
        },
        SegmentValueBounds {
            current_value,
            default,
            min,
            max,
            wrap: true,
        },
    )
}

/// Like [`key_effects`], but with `wrap: false` -- the year segment's
/// clamp-at-the-bound policy (backlog row 84 finding 4).
#[allow(clippy::too_many_arguments)]
fn key_effects_clamped(
    key: &Key,
    ctrl: bool,
    meta: bool,
    alt: bool,
    current_text: &str,
    max_length: usize,
    reset: bool,
    current_value: Option<i32>,
    default: i32,
    min: i32,
    max: i32,
) -> Vec<DateSegmentEffect<i32>> {
    date_segment_key_effects(
        key,
        SegmentKeyModifiers { ctrl, meta, alt },
        SegmentTextState {
            current_text,
            max_length,
            reset,
        },
        SegmentValueBounds {
            current_value,
            default,
            min,
            max,
            wrap: false,
        },
    )
}

fn no_modifiers() -> (bool, bool, bool) {
    (false, false, false)
}

#[test]
fn roll_value_wraps_at_bounds() {
    assert_eq!(roll_value(5, 0, 10), 5, "in range is unchanged");
    assert_eq!(roll_value(-1, 0, 10), 10, "below min wraps to max");
    assert_eq!(roll_value(11, 0, 10), 0, "above max wraps to min");
    assert_eq!(roll_value(0, 0, 10), 0, "exactly at min stays");
    assert_eq!(roll_value(10, 0, 10), 10, "exactly at max stays");
}

#[test]
fn key_effects_digit_within_bounds_just_emits_the_new_value() {
    let (ctrl, meta, alt) = no_modifiers();
    let effects = key_effects(
        &Key::Character("1".to_string()),
        ctrl,
        meta,
        alt,
        "0", // current text "0" -> becomes "01"; a 3rd digit ("010"=10) would stay in bounds
        2,
        false,
        Some(0),
        0,
        0,
        31,
    );

    assert_eq!(
        effects,
        vec![
            DateSegmentEffect::EmitValue(Some(1)),
            DateSegmentEffect::PreventDefault,
            DateSegmentEffect::StopPropagation,
        ]
    );
}

#[test]
fn key_effects_digit_that_fills_max_length_advances_focus_before_emitting() {
    // Typing "5" onto day-segment text "3" with max 31: "35" clamps to
    // 31, and since (35+"0"="350" also clamps/parses to > max) focus
    // advances to the next segment. Order matters: focus before emit.
    let (ctrl, meta, alt) = no_modifiers();
    let effects = key_effects(
        &Key::Character("5".to_string()),
        ctrl,
        meta,
        alt,
        "3",
        2,
        false,
        Some(3),
        1,
        1,
        31,
    );

    assert_eq!(
        effects,
        vec![
            DateSegmentEffect::FocusNext,
            DateSegmentEffect::EmitValue(Some(31)),
            DateSegmentEffect::PreventDefault,
            DateSegmentEffect::StopPropagation,
        ]
    );
}

#[test]
fn key_effects_digit_below_min_never_advances_focus_even_if_probe_overflows() {
    // value=3 is below min=5 (in_range must be false), even though the "add
    // one more digit" probe ("30") does exceed max=9. This distinguishes
    // `in_range && new_value > max` from a mistaken `||`.
    let (ctrl, meta, alt) = no_modifiers();
    let effects = key_effects(
        &Key::Character("3".to_string()),
        ctrl,
        meta,
        alt,
        "",
        2,
        false,
        None,
        5,
        5,
        9,
    );
    assert_eq!(
        effects,
        vec![
            DateSegmentEffect::EmitValue(Some(3)),
            DateSegmentEffect::PreventDefault,
            DateSegmentEffect::StopPropagation,
        ],
        "below-min values must never trigger the focus-advance heuristic"
    );
}

#[test]
fn key_effects_digit_probe_exactly_at_max_does_not_advance_focus() {
    // value=1 is in range [0,10], and the "add one more digit" probe ("10")
    // equals max exactly (not greater than it). This distinguishes
    // `new_value > max` from a mistaken `>=`.
    let (ctrl, meta, alt) = no_modifiers();
    let effects = key_effects(
        &Key::Character("1".to_string()),
        ctrl,
        meta,
        alt,
        "",
        2,
        false,
        None,
        0,
        0,
        10,
    );
    assert_eq!(
        effects,
        vec![
            DateSegmentEffect::EmitValue(Some(1)),
            DateSegmentEffect::PreventDefault,
            DateSegmentEffect::StopPropagation,
        ],
        "a probe that lands exactly on max must not advance focus"
    );
}

#[test]
fn key_effects_digit_resets_buffer_when_at_max_length_or_reset_pending() {
    let (ctrl, meta, alt) = no_modifiers();

    // Text is already at max_length: starts a fresh buffer instead of appending.
    let effects = key_effects(
        &Key::Character("9".to_string()),
        ctrl,
        meta,
        alt,
        "12",
        2,
        false,
        Some(12),
        1,
        1,
        31,
    );
    assert_eq!(
            effects,
            vec![
                // Restarting from "9" (not "129") means the "add one more digit"
                // overflow probe ("90") also exceeds max, so focus advances too.
                DateSegmentEffect::ConsumeReset,
                DateSegmentEffect::FocusNext,
                DateSegmentEffect::EmitValue(Some(9)),
                DateSegmentEffect::PreventDefault,
                DateSegmentEffect::StopPropagation,
            ],
            "a full buffer restarts rather than appending (would otherwise fail to parse or grow unbounded)"
        );

    // `reset` pending (e.g. segment was just focused): also starts fresh, and
    // reports that the reset flag should be consumed.
    let effects = key_effects(
        &Key::Character("9".to_string()),
        ctrl,
        meta,
        alt,
        "1",
        2,
        true,
        Some(1),
        1,
        1,
        31,
    );
    assert_eq!(
        effects,
        vec![
            DateSegmentEffect::ConsumeReset,
            DateSegmentEffect::FocusNext,
            DateSegmentEffect::EmitValue(Some(9)),
            DateSegmentEffect::PreventDefault,
            DateSegmentEffect::StopPropagation,
        ]
    );
}

#[test]
fn key_effects_digit_ignores_keyboard_shortcuts() {
    for (ctrl, meta, alt) in [
        (true, false, false),
        (false, true, false),
        (false, false, true),
    ] {
        let effects = key_effects(
            &Key::Character("5".to_string()),
            ctrl,
            meta,
            alt,
            "1",
            2,
            false,
            Some(1),
            0,
            0,
            31,
        );
        assert_eq!(
            effects,
            Vec::new(),
            "ctrl/meta/alt+digit must pass through as a shortcut, untouched"
        );
    }
}

#[test]
fn key_effects_non_digit_character_blocks_default_but_does_not_edit() {
    let (ctrl, meta, alt) = no_modifiers();
    let effects = key_effects(
        &Key::Character("x".to_string()),
        ctrl,
        meta,
        alt,
        "1",
        2,
        false,
        Some(1),
        0,
        0,
        31,
    );
    assert_eq!(
        effects,
        vec![
            DateSegmentEffect::PreventDefault,
            DateSegmentEffect::StopPropagation,
        ]
    );
}

#[test]
fn key_effects_backspace_pops_last_character() {
    let (ctrl, meta, alt) = no_modifiers();
    let effects = key_effects(
        &Key::Backspace,
        ctrl,
        meta,
        alt,
        "12",
        2,
        false,
        Some(12),
        0,
        0,
        31,
    );
    assert_eq!(effects, vec![DateSegmentEffect::EmitValue(Some(1))]);
}

#[test]
fn key_effects_backspace_with_ctrl_or_meta_clears_all() {
    let effects = key_effects(
        &Key::Backspace,
        true,
        false,
        false,
        "12",
        2,
        false,
        Some(12),
        0,
        0,
        31,
    );
    assert_eq!(
        effects,
        vec![
            DateSegmentEffect::EmitValue(None),
            DateSegmentEffect::FocusPrevious,
        ],
        "clearing to empty emits None then moves focus back"
    );
}

#[test]
fn key_effects_delete_drops_first_character() {
    let (ctrl, meta, alt) = no_modifiers();
    let effects = key_effects(
        &Key::Delete,
        ctrl,
        meta,
        alt,
        "12",
        2,
        false,
        Some(12),
        0,
        0,
        31,
    );
    assert_eq!(effects, vec![DateSegmentEffect::EmitValue(Some(2))]);
}

#[test]
fn key_effects_arrow_left_and_right_move_focus_only() {
    let (ctrl, meta, alt) = no_modifiers();
    let left = key_effects(
        &Key::ArrowLeft,
        ctrl,
        meta,
        alt,
        "1",
        2,
        false,
        Some(1),
        0,
        0,
        31,
    );
    assert_eq!(left, vec![DateSegmentEffect::FocusPrevious]);

    let right = key_effects(
        &Key::ArrowRight,
        ctrl,
        meta,
        alt,
        "1",
        2,
        false,
        Some(1),
        0,
        0,
        31,
    );
    assert_eq!(right, vec![DateSegmentEffect::FocusNext]);
}

#[test]
fn key_effects_enter_moves_focus_next_and_blocks_default() {
    let (ctrl, meta, alt) = no_modifiers();
    let effects = key_effects(
        &Key::Enter,
        ctrl,
        meta,
        alt,
        "1",
        2,
        false,
        Some(1),
        0,
        0,
        31,
    );
    assert_eq!(
        effects,
        vec![
            DateSegmentEffect::FocusNext,
            DateSegmentEffect::PreventDefault,
            DateSegmentEffect::StopPropagation,
        ]
    );
}

#[test]
fn key_effects_arrow_up_increments_and_wraps_from_max_to_min() {
    let (ctrl, meta, alt) = no_modifiers();
    let effects = key_effects(
        &Key::ArrowUp,
        ctrl,
        meta,
        alt,
        "",
        2,
        false,
        Some(5),
        0,
        0,
        10,
    );
    assert_eq!(effects, vec![DateSegmentEffect::EmitValue(Some(6))]);

    let wrapped = key_effects(
        &Key::ArrowUp,
        ctrl,
        meta,
        alt,
        "",
        2,
        false,
        Some(10),
        0,
        0,
        10,
    );
    assert_eq!(
        wrapped,
        vec![DateSegmentEffect::EmitValue(Some(0))],
        "incrementing past max wraps to min"
    );
}

#[test]
fn key_effects_arrow_down_decrements_and_wraps_from_min_to_max() {
    let (ctrl, meta, alt) = no_modifiers();
    let effects = key_effects(
        &Key::ArrowDown,
        ctrl,
        meta,
        alt,
        "",
        2,
        false,
        Some(5),
        0,
        0,
        10,
    );
    assert_eq!(effects, vec![DateSegmentEffect::EmitValue(Some(4))]);

    let wrapped = key_effects(
        &Key::ArrowDown,
        ctrl,
        meta,
        alt,
        "",
        2,
        false,
        Some(0),
        0,
        0,
        10,
    );
    assert_eq!(
        wrapped,
        vec![DateSegmentEffect::EmitValue(Some(10))],
        "decrementing past min wraps to max"
    );
}

#[test]
fn key_effects_arrow_up_down_clamp_at_bounds_instead_of_wrapping_when_wrap_is_false() {
    // backlog row 84 finding 4: the YEAR segment must clamp, not wrap.
    // `roll_value`'s own wrap test above (`key_effects_arrow_up_
    // increments_and_wraps_from_max_to_min`) is `wrap: true`'s
    // (month/day's) behavior; this is `wrap: false`'s (year's).
    let (ctrl, meta, alt) = no_modifiers();
    let up_at_max = key_effects_clamped(
        &Key::ArrowUp,
        ctrl,
        meta,
        alt,
        "",
        4,
        false,
        Some(2050),
        2000,
        1925,
        2050,
    );
    assert_eq!(
        up_at_max,
        vec![DateSegmentEffect::EmitValue(Some(2050))],
        "incrementing past max clamps at max instead of wrapping to min"
    );

    let down_at_min = key_effects_clamped(
        &Key::ArrowDown,
        ctrl,
        meta,
        alt,
        "",
        4,
        false,
        Some(1925),
        2000,
        1925,
        2050,
    );
    assert_eq!(
        down_at_min,
        vec![DateSegmentEffect::EmitValue(Some(1925))],
        "decrementing past min clamps at min instead of wrapping to max"
    );

    // Away from either bound, clamped and wrapped behavior agree --
    // this isolates the *boundary* as what actually differs.
    let up_in_range = key_effects_clamped(
        &Key::ArrowUp,
        ctrl,
        meta,
        alt,
        "",
        4,
        false,
        Some(2000),
        2000,
        1925,
        2050,
    );
    assert_eq!(up_in_range, vec![DateSegmentEffect::EmitValue(Some(2001))]);
}

#[test]
fn key_effects_arrow_up_down_use_default_when_no_current_value() {
    let (ctrl, meta, alt) = no_modifiers();
    let up = key_effects(&Key::ArrowUp, ctrl, meta, alt, "", 2, false, None, 7, 0, 10);
    assert_eq!(up, vec![DateSegmentEffect::EmitValue(Some(7))]);

    let down = key_effects(
        &Key::ArrowDown,
        ctrl,
        meta,
        alt,
        "",
        2,
        false,
        None,
        7,
        0,
        10,
    );
    assert_eq!(down, vec![DateSegmentEffect::EmitValue(Some(7))]);
}

// -----------------------------------------------------------------
// Home/End (APG spinbutton-pattern.html #keyboard_interaction --
// playwright/date-picker.spec.ts's "Home/End" describe block is the
// browser-level guard for the same behavior; these are its pure-
// function-level counterpart, mirroring every other `key_effects_*`
// test above).
// -----------------------------------------------------------------

#[test]
fn key_effects_home_sets_the_minimum_regardless_of_current_value() {
    let (ctrl, meta, alt) = no_modifiers();
    let effects = key_effects(
        &Key::Home,
        ctrl,
        meta,
        alt,
        "15",
        2,
        false,
        Some(15),
        1,
        1,
        31,
    );
    assert_eq!(
        effects,
        vec![DateSegmentEffect::EmitValue(Some(1))],
        "Home sets the value to the minimum, per the cited APG rule"
    );

    // Also true with no current value yet (the field still has a
    // well-defined `min`, so the rule still applies).
    let effects = key_effects(&Key::Home, ctrl, meta, alt, "", 2, false, None, 1, 1, 31);
    assert_eq!(effects, vec![DateSegmentEffect::EmitValue(Some(1))]);
}

#[test]
fn key_effects_end_sets_the_maximum_regardless_of_current_value() {
    let (ctrl, meta, alt) = no_modifiers();
    let effects = key_effects(
        &Key::End,
        ctrl,
        meta,
        alt,
        "15",
        2,
        false,
        Some(15),
        1,
        1,
        31,
    );
    assert_eq!(
        effects,
        vec![DateSegmentEffect::EmitValue(Some(31))],
        "End sets the value to the maximum, per the cited APG rule"
    );

    let effects = key_effects(&Key::End, ctrl, meta, alt, "", 2, false, None, 1, 1, 31);
    assert_eq!(effects, vec![DateSegmentEffect::EmitValue(Some(31))]);
}

#[test]
fn key_effects_unhandled_key_produces_no_effects() {
    let (ctrl, meta, alt) = no_modifiers();
    let effects = key_effects(&Key::Tab, ctrl, meta, alt, "1", 2, false, Some(1), 0, 0, 31);
    assert_eq!(effects, Vec::new());
}

// -----------------------------------------------------------------
// Month/day segment min-max guards (dev-docs/reference/mutants-baseline.md fix #7)
// -----------------------------------------------------------------

#[test]
fn month_bounds_clamp_only_at_the_boundary_year() {
    let min_date = date!(2020 - 03 - 01);
    let max_date = date!(2030 - 09 - 01);

    assert_eq!(
        month_bounds_for_year(Some(2020), 2020, 2030, min_date, max_date),
        (3, 12),
        "min-boundary year clamps the min month"
    );
    assert_eq!(
        month_bounds_for_year(Some(2030), 2020, 2030, min_date, max_date),
        (1, 9),
        "max-boundary year clamps the max month"
    );
    assert_eq!(
        month_bounds_for_year(Some(2025), 2020, 2030, min_date, max_date),
        (1, 12),
        "any other year allows the full range"
    );
    assert_eq!(
        month_bounds_for_year(None, 2020, 2030, min_date, max_date),
        (1, 12),
        "no year selected yet allows the full range"
    );
}

#[test]
fn day_bounds_clamp_only_at_the_boundary_year_and_month() {
    let min_date = date!(2020 - 03 - 10);
    let max_date = date!(2030 - 09 - 20);

    assert_eq!(
        day_bounds_for_year_month(Some(2020), Some(3), 2020, 2030, min_date, max_date),
        (10, 31),
        "min-boundary year+month clamps the min day"
    );
    assert_eq!(
        day_bounds_for_year_month(Some(2030), Some(9), 2020, 2030, min_date, max_date),
        (1, 20),
        "max-boundary year+month clamps the max day"
    );
    // Not at a boundary: max day falls back to the target month's actual length.
    assert_eq!(
        day_bounds_for_year_month(Some(2024), Some(2), 2020, 2030, min_date, max_date),
        (1, 29),
        "falls back to the real month length (2024 is a leap year)"
    );
    assert_eq!(
        day_bounds_for_year_month(Some(2020), Some(3), 2020, 2030, min_date, max_date).0,
        10
    );
    assert_eq!(
        day_bounds_for_year_month(None, None, 2020, 2030, min_date, max_date),
        (1, 31),
        "no year/month selected yet allows the full range"
    );

    // Partial boundary matches (year matches but month doesn't, or vice versa)
    // must NOT clamp -- both year AND month must match the boundary.
    assert_eq!(
        day_bounds_for_year_month(Some(2020), Some(5), 2020, 2030, min_date, max_date).0,
        1,
        "min_year matches but month doesn't: falls through to the unclamped default"
    );
    assert_eq!(
        day_bounds_for_year_month(Some(2025), Some(3), 2020, 2030, min_date, max_date).0,
        1,
        "min_date's month matches but year doesn't: falls through to the unclamped default"
    );
    assert_eq!(
        day_bounds_for_year_month(Some(2030), Some(5), 2020, 2030, min_date, max_date).1,
        Month::May.length(2030),
        "max_year matches but month doesn't: falls through to the real month length"
    );
    assert_eq!(
        day_bounds_for_year_month(Some(2025), Some(9), 2020, 2030, min_date, max_date).1,
        Month::September.length(2025),
        "max_date's month matches but year doesn't: falls through to the real month length"
    );
}
