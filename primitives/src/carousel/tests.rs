use super::*;

#[test]
fn clamp_selected_within_bounds_is_unchanged() {
    assert_eq!(clamp_selected(1, 3), 1);
}

#[test]
fn clamp_selected_clamps_to_last_index() {
    assert_eq!(clamp_selected(99, 3), 2);
}

#[test]
fn clamp_selected_with_zero_count_is_zero() {
    assert_eq!(clamp_selected(0, 0), 0);
    assert_eq!(clamp_selected(5, 0), 0);
}

#[test]
fn clamp_visible_range_within_bounds_is_unchanged() {
    assert_eq!(clamp_visible_range((0, 2), 5), (0, 2));
}

#[test]
fn clamp_visible_range_clamps_end_past_last_index() {
    assert_eq!(clamp_visible_range((1, 99), 3), (1, 2));
}

#[test]
fn clamp_visible_range_clamps_start_past_last_index_too() {
    assert_eq!(clamp_visible_range((50, 60), 3), (2, 2));
}

#[test]
fn clamp_visible_range_with_zero_count_is_zero() {
    assert_eq!(clamp_visible_range((0, 0), 0), (0, 0));
    assert_eq!(clamp_visible_range((4, 9), 0), (0, 0));
}

#[test]
fn can_scroll_prev_false_at_first_slide() {
    assert!(!can_scroll_prev(0));
}

#[test]
fn can_scroll_prev_true_after_first_slide() {
    assert!(can_scroll_prev(1));
}

#[test]
fn can_scroll_next_true_before_last_slide() {
    assert!(can_scroll_next(0, 3));
    assert!(can_scroll_next(1, 3));
}

#[test]
fn can_scroll_next_false_at_last_slide() {
    assert!(!can_scroll_next(2, 3));
}

#[test]
fn can_scroll_next_false_with_zero_slides() {
    assert!(!can_scroll_next(0, 0));
}

#[test]
fn next_selected_advances_by_one() {
    assert_eq!(next_selected(0, 3), 1);
    assert_eq!(next_selected(1, 3), 2);
}

#[test]
fn next_selected_stops_at_last_slide_without_wrapping() {
    assert_eq!(next_selected(2, 3), 2);
}

#[test]
fn next_selected_with_zero_slides_is_zero() {
    assert_eq!(next_selected(0, 0), 0);
}

#[test]
fn prev_selected_retreats_by_one() {
    assert_eq!(prev_selected(2), 1);
    assert_eq!(prev_selected(1), 0);
}

#[test]
fn prev_selected_stops_at_first_slide_without_wrapping() {
    assert_eq!(prev_selected(0), 0);
}

#[test]
fn step_prev_without_loop_matches_prev_selected() {
    assert_eq!(step_prev(0, 3, false), prev_selected(0));
    assert_eq!(step_prev(2, 3, false), prev_selected(2));
}

#[test]
fn step_prev_with_loop_wraps_from_first_to_last() {
    assert_eq!(step_prev(0, 3, true), 2);
}

#[test]
fn step_prev_with_loop_mid_range_steps_back_by_one() {
    assert_eq!(step_prev(2, 3, true), 1);
}

#[test]
fn step_prev_with_loop_and_zero_slides_stays_zero() {
    assert_eq!(step_prev(0, 0, true), 0);
}

#[test]
fn step_next_without_loop_matches_next_selected() {
    assert_eq!(step_next(0, 3, false), next_selected(0, 3));
    assert_eq!(step_next(2, 3, false), next_selected(2, 3));
}

#[test]
fn step_next_with_loop_wraps_from_last_to_first() {
    assert_eq!(step_next(2, 3, true), 0);
}

#[test]
fn step_next_with_loop_mid_range_steps_forward_by_one() {
    assert_eq!(step_next(0, 3, true), 1);
}

#[test]
fn step_next_with_loop_and_zero_slides_stays_zero() {
    assert_eq!(step_next(0, 0, true), 0);
}

#[test]
fn slide_label_is_one_based_n_of_m() {
    assert_eq!(slide_label(0, 6), "1 of 6");
    assert_eq!(slide_label(5, 6), "6 of 6");
}

// `is_mount_settle` -- guards against "slide 1 -> 2 jumps instead of
// animating" (see the function's own doc, and `Carousel`'s
// mount-time effect, for the construction). The failure it prevents is
// exactly the case `mount_settle_is_false_before_any_items_have_registered`
// below combined with `mount_settle_is_true_the_instant_items_exist`
// *not* being reachable before a real navigation -- gating on
// `has_items` rather than "did this particular id lookup succeed" is
// what makes `mount_settle_is_false_on_a_real_navigation_after_mount`
// hold unconditionally.
#[test]
fn mount_settle_is_false_before_any_items_have_registered() {
    // The mount effect's own early runs, before `CarouselItem`
    // registration completes -- `is_first` is still `true`, but there
    // is nothing to scroll to yet, so this must not consume it.
    assert!(!is_mount_settle(true, false));
}

#[test]
fn mount_settle_is_true_the_instant_items_exist() {
    // The first run where `has_items` flips `true` -- whether that is
    // this effect's very first execution (items already registered
    // by the time it runs) or a later retry once registration
    // completes (the common `default_value == 0` case, where
    // `selected()`'s own value never changes) -- this is always the
    // mount-time settle, and must be instant.
    assert!(is_mount_settle(true, true));
}

#[test]
fn mount_settle_is_false_on_a_real_navigation_after_mount() {
    // Once the caller has set `is_first` to `false` (which it does on
    // every run where `has_items` is `true`, per this function's own
    // doc), no later run -- including every genuine user navigation --
    // can ever read `true` again, regardless of `has_items`.
    assert!(!is_mount_settle(false, true));
    assert!(!is_mount_settle(false, false));
}

#[test]
fn carousel_key_intent_horizontal_ltr_matches_physical_keys() {
    assert_eq!(
        carousel_key_intent(
            &Key::ArrowLeft,
            CarouselOrientation::Horizontal,
            Direction::Ltr
        ),
        Some(HorizontalNav::Prev)
    );
    assert_eq!(
        carousel_key_intent(
            &Key::ArrowRight,
            CarouselOrientation::Horizontal,
            Direction::Ltr
        ),
        Some(HorizontalNav::Next)
    );
}

#[test]
fn carousel_key_intent_horizontal_rtl_swaps_physical_keys() {
    assert_eq!(
        carousel_key_intent(
            &Key::ArrowLeft,
            CarouselOrientation::Horizontal,
            Direction::Rtl
        ),
        Some(HorizontalNav::Next)
    );
    assert_eq!(
        carousel_key_intent(
            &Key::ArrowRight,
            CarouselOrientation::Horizontal,
            Direction::Rtl
        ),
        Some(HorizontalNav::Prev)
    );
}

#[test]
fn carousel_key_intent_vertical_ignores_direction() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        assert_eq!(
            carousel_key_intent(&Key::ArrowUp, CarouselOrientation::Vertical, direction),
            Some(HorizontalNav::Prev)
        );
        assert_eq!(
            carousel_key_intent(&Key::ArrowDown, CarouselOrientation::Vertical, direction),
            Some(HorizontalNav::Next)
        );
    }
}

#[test]
fn carousel_key_intent_vertical_ignores_horizontal_arrow_keys() {
    assert_eq!(
        carousel_key_intent(
            &Key::ArrowLeft,
            CarouselOrientation::Vertical,
            Direction::Ltr
        ),
        None
    );
    assert_eq!(
        carousel_key_intent(
            &Key::ArrowRight,
            CarouselOrientation::Vertical,
            Direction::Ltr
        ),
        None
    );
}

#[test]
fn carousel_key_intent_ignores_unrelated_keys() {
    for orientation in [
        CarouselOrientation::Horizontal,
        CarouselOrientation::Vertical,
    ] {
        assert_eq!(
            carousel_key_intent(&Key::Home, orientation, Direction::Ltr),
            None
        );
        assert_eq!(
            carousel_key_intent(&Key::Enter, orientation, Direction::Ltr),
            None
        );
    }
}

#[test]
fn orientation_as_str_matches_the_data_orientation_tokens() {
    assert_eq!(CarouselOrientation::Horizontal.as_str(), "horizontal");
    assert_eq!(CarouselOrientation::Vertical.as_str(), "vertical");
}

#[test]
fn orientation_default_is_horizontal() {
    assert_eq!(
        CarouselOrientation::default(),
        CarouselOrientation::Horizontal
    );
}

#[test]
fn shortest_signed_delta_zero_count_is_zero() {
    assert_eq!(shortest_signed_delta(0, 0, 0), 0);
}

#[test]
fn shortest_signed_delta_same_index_is_zero() {
    assert_eq!(shortest_signed_delta(3, 3, 12), 0);
}

#[test]
fn shortest_signed_delta_forward_step_is_plus_one() {
    assert_eq!(shortest_signed_delta(4, 5, 12), 1);
}

#[test]
fn shortest_signed_delta_backward_step_is_minus_one() {
    assert_eq!(shortest_signed_delta(5, 4, 12), -1);
}

#[test]
fn shortest_signed_delta_wraps_forward_at_the_end() {
    // step_next's own rewind wrap: last -> first is +1 in position
    // space, not -(count - 1).
    assert_eq!(shortest_signed_delta(11, 0, 12), 1);
}

#[test]
fn shortest_signed_delta_wraps_backward_at_the_start() {
    assert_eq!(shortest_signed_delta(0, 11, 12), -1);
}

#[test]
fn shortest_signed_delta_picks_the_nearer_direction_for_a_distant_jump() {
    // 12 slides, jumping from 1 to 7 (6 apart either way) -- either
    // signed direction is equally short; the formula's own tie-break
    // (`wrapped > n/2`, strict) keeps the positive/forward one rather
    // than flipping to `-6`.
    assert_eq!(shortest_signed_delta(1, 7, 12), 6);
    // A jump that is unambiguously nearer forward.
    assert_eq!(shortest_signed_delta(1, 4, 12), 3);
    // ...and unambiguously nearer backward.
    assert_eq!(shortest_signed_delta(1, 10, 12), -3);
}

#[test]
fn shortest_signed_delta_small_count_never_exceeds_half() {
    for count in 2..8 {
        for old in 0..count {
            for new in 0..count {
                if old == new {
                    continue;
                }
                let d = shortest_signed_delta(old, new, count);
                assert!(d != 0);
                assert!(d.unsigned_abs() <= count / 2 + 1);
                // Applying it must land back on `new`, modulo `count`.
                let landed = ((old as isize + d).rem_euclid(count as isize)) as usize;
                assert_eq!(landed, new);
            }
        }
    }
}

#[test]
fn is_rewind_wrap_true_for_last_to_first() {
    assert!(is_rewind_wrap(4, 0, 5));
}

#[test]
fn is_rewind_wrap_true_for_first_to_last() {
    assert!(is_rewind_wrap(0, 4, 5));
}

#[test]
fn is_rewind_wrap_false_for_an_ordinary_adjacent_step() {
    assert!(!is_rewind_wrap(1, 2, 5));
    assert!(!is_rewind_wrap(2, 1, 5));
}

#[test]
fn is_rewind_wrap_false_for_a_same_index_no_op() {
    assert!(!is_rewind_wrap(2, 2, 5));
}

#[test]
fn is_rewind_wrap_false_with_zero_slides() {
    assert!(!is_rewind_wrap(0, 0, 0));
}

#[test]
fn is_rewind_wrap_ambiguous_case_at_count_two_reports_true() {
    // See this function's own doc: at `count == 2` a wrap and an
    // ordinary step are the identical transition either direction.
    assert!(is_rewind_wrap(0, 1, 2));
    assert!(is_rewind_wrap(1, 0, 2));
}

#[test]
fn loop_mode_default_is_seamless() {
    assert_eq!(LoopMode::default(), LoopMode::Seamless);
}
