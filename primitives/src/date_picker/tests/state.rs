use super::*;
use crate::test_support::with_runtime;

// -----------------------------------------------------------------
// Context accessors
// -----------------------------------------------------------------

#[test]
fn date_picker_context_set_date_only_emits_on_change() {
    with_runtime(|| {
        let selected = use_signal(|| Some(date!(2024 - 01 - 01)));
        let emitted = use_signal(Vec::<Option<Date>>::new);
        let mut ctx = DatePickerContext {
            selected_date: selected.into(),
            on_value_change: Callback::new(move |d| {
                let mut emitted = emitted;
                emitted.write().push(d);
            }),
            staging: Staging {
                staged: use_signal(|| None),
                displayed: use_memo(move || selected.cloned()),
            },
        };

        // Same value: no-op.
        ctx.set_date(Some(date!(2024 - 01 - 01)));
        assert_eq!(emitted(), Vec::new());

        // Different value: emits.
        ctx.set_date(Some(date!(2024 - 06 - 15)));
        assert_eq!(emitted(), vec![Some(date!(2024 - 06 - 15))]);
    });
}

#[test]
fn date_range_picker_context_set_range_only_emits_on_change() {
    with_runtime(|| {
        let initial = Some(DateRange::new(date!(2024 - 01 - 01), date!(2024 - 01 - 05)));
        let date_range = use_signal(move || initial);
        let emitted = use_signal(Vec::<Option<DateRange>>::new);
        let mut ctx = DateRangePickerContext {
            date_range: date_range.into(),
            set_selected_range: Callback::new(move |r| {
                let mut emitted = emitted;
                emitted.write().push(r);
            }),
            staging: Staging {
                staged: use_signal(|| None),
                displayed: use_memo(move || date_range.cloned()),
            },
        };

        ctx.set_range(initial);
        assert_eq!(emitted(), Vec::new(), "identical range must not re-emit");

        let changed = Some(DateRange::new(date!(2024 - 02 - 01), date!(2024 - 02 - 05)));
        ctx.set_range(changed);
        assert_eq!(emitted(), vec![changed]);
    });
}

// ---- the close dwell (`close_on_select` / `close_delay`) -----------------------------

use std::{cell::Cell, rc::Rc};

const DWELL: Duration = Duration::from_millis(300);

#[test]
fn selection_close_policy() {
    assert_eq!(selection_close(false, DWELL), SelectionClose::Stay);
    assert_eq!(selection_close(false, Duration::ZERO), SelectionClose::Stay);
    assert_eq!(
        selection_close(true, Duration::ZERO),
        SelectionClose::Immediately
    );
    assert_eq!(selection_close(true, DWELL), SelectionClose::After(DWELL));
    assert_eq!(
        selection_close(true, DEFAULT_CLOSE_DELAY),
        SelectionClose::After(Duration::from_millis(300))
    );
}

/// Shared with the test through a root context: the picker's dwell handle and `open` signal
/// (captured by the probe on each render) and whether the probe is mounted.
#[derive(Clone, Default)]
struct DwellHarness {
    dwell: Rc<Cell<Option<CloseDwell>>>,
    open: Rc<Cell<Option<Signal<bool>>>>,
    mounted: Rc<Cell<bool>>,
}

#[component]
fn DwellHost() -> Element {
    let harness: DwellHarness = consume_context();
    rsx! {
        if harness.mounted.get() {
            DwellProbe {}
        }
    }
}

/// Stands in for the picker root: it owns `open` and the `CloseDwell`, exactly as `DatePicker`
/// does.
#[component]
fn DwellProbe() -> Element {
    let harness: DwellHarness = consume_context();
    let open = use_signal(|| true);
    harness.open.set(Some(open));
    harness.dwell.set(Some(use_close_dwell(open)));
    rsx! { "probe" }
}

fn dwell_dom() -> (VirtualDom, DwellHarness) {
    let harness = DwellHarness::default();
    harness.mounted.set(true);
    let mut dom = VirtualDom::new(DwellHost).with_root_context(harness.clone());
    dom.rebuild_in_place();
    dom.process_events();
    (dom, harness)
}

/// What `selection_complete` does from the calendar (a child scope): start the wait.
fn schedule_close(dom: &mut VirtualDom, harness: &DwellHarness, delay: Duration) {
    let dwell = harness.dwell.get().expect("probe rendered");
    let mut open = harness.open.get().expect("probe rendered");
    dom.in_scope(ScopeId::APP, || {
        dwell.schedule(delay, move || open.set(false))
    });
    // Poll the spawned task once so its `sleep` is registered at the current virtual time.
    dom.process_events();
}

fn commit_render(dom: &mut VirtualDom) {
    dom.render_immediate_to_vec();
    dom.process_events();
}

fn is_open(dom: &mut VirtualDom, harness: &DwellHarness) -> bool {
    let open = harness.open.get().expect("probe rendered");
    dom.in_scope(ScopeId::APP, || *open.peek())
}

fn is_pending(dom: &mut VirtualDom, harness: &DwellHarness) -> bool {
    let dwell = harness.dwell.get().expect("probe rendered");
    dom.in_scope(ScopeId::APP, || dwell.is_pending())
}

fn cancel(dom: &mut VirtualDom, harness: &DwellHarness) {
    let dwell = harness.dwell.get().expect("probe rendered");
    dom.in_scope(ScopeId::APP, || dwell.cancel());
}

async fn advance(dom: &mut VirtualDom, by: Duration) {
    tokio::time::advance(by).await;
    dom.process_events();
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn the_picker_closes_once_after_the_delay_and_not_before() {
    let (mut dom, harness) = dwell_dom();
    assert!(
        !is_pending(&mut dom, &harness),
        "nothing pending before a selection"
    );
    schedule_close(&mut dom, &harness, DWELL);
    assert!(
        is_pending(&mut dom, &harness),
        "a selection leaves a close pending"
    );

    advance(&mut dom, DWELL - Duration::from_millis(1)).await;
    assert!(
        is_open(&mut dom, &harness),
        "must still be open inside the dwell"
    );
    assert!(is_pending(&mut dom, &harness));
    advance(&mut dom, Duration::from_millis(1)).await;
    assert!(
        !is_open(&mut dom, &harness),
        "closes once the dwell has elapsed"
    );
    assert!(
        !is_pending(&mut dom, &harness),
        "and nothing is pending once it has"
    );
    advance(&mut dom, DWELL * 4).await;
    assert!(!is_open(&mut dom, &harness));
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn scheduling_again_restarts_the_wait_instead_of_stacking_a_second_close() {
    let (mut dom, harness) = dwell_dom();
    schedule_close(&mut dom, &harness, DWELL);
    advance(&mut dom, Duration::from_millis(200)).await;
    schedule_close(&mut dom, &harness, DWELL); // the user picked another date

    advance(&mut dom, Duration::from_millis(200)).await;
    assert!(
        is_open(&mut dom, &harness),
        "the first schedule's deadline has passed, but it was replaced"
    );
    advance(&mut dom, Duration::from_millis(100)).await;
    assert!(
        !is_open(&mut dom, &harness),
        "the restarted wait elapses once"
    );
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn cancel_drops_the_pending_close() {
    let (mut dom, harness) = dwell_dom();
    schedule_close(&mut dom, &harness, DWELL);
    advance(&mut dom, Duration::from_millis(100)).await;
    cancel(&mut dom, &harness);
    assert!(
        !is_pending(&mut dom, &harness),
        "cancelled means nothing is pending"
    );
    advance(&mut dom, DWELL * 4).await;
    assert!(
        is_open(&mut dom, &harness),
        "an interaction during the dwell cancels the close"
    );

    // Cancelling when nothing is pending is a no-op, and the dwell is reusable.
    cancel(&mut dom, &harness);
    schedule_close(&mut dom, &harness, DWELL);
    advance(&mut dom, DWELL).await;
    assert!(!is_open(&mut dom, &harness));
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn closing_or_reopening_the_popover_cancels_the_pending_close() {
    let (mut dom, harness) = dwell_dom();
    commit_render(&mut dom); // the mount-time effect run
    schedule_close(&mut dom, &harness, DWELL);
    advance(&mut dom, Duration::from_millis(100)).await;

    // The user closes the popover (Escape / outside click / trigger) and reopens it inside the dwell.
    let mut open = harness.open.get().unwrap();
    dom.in_scope(ScopeId::APP, || open.set(false));
    commit_render(&mut dom);
    dom.in_scope(ScopeId::APP, || open.set(true));
    commit_render(&mut dom);

    advance(&mut dom, DWELL * 4).await;
    assert!(
        is_open(&mut dom, &harness),
        "the close scheduled for the previous open session must not land on this one"
    );
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn a_started_wait_never_fires_after_the_picker_unmounts() {
    let (mut dom, harness) = dwell_dom();
    let fired = Rc::new(Cell::new(0usize));
    let dwell = harness.dwell.get().unwrap();
    let counter = fired.clone();
    // Scheduled from OUTSIDE the probe's scope, as the calendar (a child scope) would.
    dom.in_scope(ScopeId::APP, || {
        dwell.schedule(DWELL, move || counter.set(counter.get() + 1))
    });
    dom.process_events();
    advance(&mut dom, Duration::from_millis(100)).await;

    // Unmount the picker mid-dwell.
    harness.mounted.set(false);
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec();
    for _ in 0..10 {
        advance(&mut dom, DWELL).await;
    }
    assert_eq!(
        fired.get(),
        0,
        "a timer owned by the picker's scope is gone with it: nothing may fire after unmount"
    );
}

// ---- a controlled range change is never answered (a second range must not hang the page) -----------------

/// A controlled `DateRangePicker` the way the demo uses it: the parent owns the range and
/// applies whatever `on_range_change` reports. Everything the picker reports is recorded.
#[derive(Clone, Default)]
struct RangeHarness {
    selected: Rc<Cell<Option<Signal<Option<DateRange>>>>>,
    reported: Rc<std::cell::RefCell<Vec<Option<DateRange>>>>,
}

#[component]
fn ControlledRangeHost() -> Element {
    let harness: RangeHarness = consume_context();
    let mut selected =
        use_signal(|| Some(DateRange::new(date!(2026 - 02 - 01), date!(2026 - 02 - 05))));
    harness.selected.set(Some(selected));
    rsx! {
        DateRangePicker {
            selected_range: selected(),
            on_range_change: move |range: Option<DateRange>| {
                harness.reported.borrow_mut().push(range);
                selected.set(range);
            },
            DateRangePickerInput {}
        }
    }
}

/// Run the VirtualDom until it is quiet -- bounded, so an unbounded loop is a failed assertion
/// and not a hung test.
fn settle(dom: &mut VirtualDom) {
    for _ in 0..60 {
        dom.process_events();
        dom.render_immediate_to_vec();
    }
}

/// A range the PARENT sets (a calendar click it has just applied, a "this week" button) must
/// never be reported back through `on_range_change`: the picker already holds it. A sync-up
/// effect subscribed to the parent's value (through `DateRangePickerContext::set_range`'s
/// tracked read) could wake ahead of the sync-down effect on a parent change, read the
/// PREVIOUS range from the segments and report that, and the two ranges would alternate
/// forever (row 83). Which of the two effects wakes first is hash-ordered, hence the many
/// changes: a subscribed effect wakes them the wrong way round for at least one of them.
#[test]
fn a_range_the_parent_sets_is_never_reported_back() {
    let harness = RangeHarness::default();
    let mut dom = VirtualDom::new(ControlledRangeHost).with_root_context(harness.clone());
    dom.rebuild_in_place();
    settle(&mut dom);
    assert!(
        harness.reported.borrow().is_empty(),
        "mounting a controlled range must not report anything: {:?}",
        harness.reported.borrow()
    );

    let mut selected = harness.selected.get().expect("host rendered");
    for step in 0..16i64 {
        let start = date!(2026 - 03 - 01) + time::Duration::days(step * 3);
        let next = Some(DateRange::new(start, start + time::Duration::days(2)));
        dom.in_scope(ScopeId::APP, || selected.set(next));
        settle(&mut dom);
        assert!(
            harness.reported.borrow().is_empty(),
            "step {step}: the picker answered the parent's own range {next:?} with {:?}",
            harness.reported.borrow()
        );
        assert_eq!(
            dom.in_scope(ScopeId::APP, || *selected.peek()),
            next,
            "step {step}: the parent's range was overwritten"
        );
    }
}

// ---- Apply / Cancel staging (`confirm`) ---------------------------------------------------

/// What the probe hands back: the picker's staging, its confirm handle, `open`, the committed
/// value is the parent's, and every value reported through `on_value_change` is recorded.
#[derive(Clone, Default)]
struct StagingHarness {
    parts: Rc<Cell<Option<StagingParts>>>,
    reported: Rc<std::cell::RefCell<Vec<Option<Date>>>>,
}

#[derive(Clone, Copy)]
struct StagingParts {
    staging: Staging<Date>,
    confirm: DatePickerConfirm,
    open: Signal<bool>,
    enabled: Signal<bool>,
}

#[component]
fn StagingHost() -> Element {
    let harness: StagingHarness = consume_context();
    let open = use_signal(|| true);
    let mut committed = use_signal(|| Some(date!(2026 - 02 - 01)));
    let enabled = use_signal(|| true);
    let dwell = use_close_dwell(open);
    let reported = harness.reported.clone();
    let commit = use_callback(move |value: Option<Date>| {
        reported.borrow_mut().push(value);
        committed.set(value);
    });
    let (staging, confirm) = use_staging(enabled.into(), open, dwell, committed.into(), commit);
    harness.parts.set(Some(StagingParts {
        staging,
        confirm,
        open,
        enabled,
    }));
    rsx! { "probe" }
}

fn staging_dom() -> (VirtualDom, StagingHarness, StagingParts) {
    let harness = StagingHarness::default();
    let mut dom = VirtualDom::new(StagingHost).with_root_context(harness.clone());
    dom.rebuild_in_place();
    settle(&mut dom);
    let parts = harness.parts.get().expect("probe rendered");
    (dom, harness, parts)
}

fn stage(dom: &mut VirtualDom, parts: StagingParts, value: Option<Date>) -> bool {
    let staged = dom.in_scope(ScopeId::APP, || parts.staging.stage(parts.confirm, value));
    settle(dom);
    staged
}

fn displayed(dom: &mut VirtualDom, parts: StagingParts) -> Option<Date> {
    dom.in_scope(ScopeId::APP, || *parts.staging.displayed.peek())
}

fn can_apply(dom: &mut VirtualDom, parts: StagingParts) -> bool {
    dom.in_scope(ScopeId::APP, || *parts.confirm.can_apply.peek())
}

#[test]
fn a_staged_selection_is_shown_but_not_committed_and_apply_commits_it_once() {
    let (mut dom, harness, parts) = staging_dom();
    assert!(
        !can_apply(&mut dom, parts),
        "nothing staged, nothing to apply"
    );
    assert_eq!(displayed(&mut dom, parts), Some(date!(2026 - 02 - 01)));

    assert!(stage(&mut dom, parts, Some(date!(2026 - 02 - 09))));
    assert_eq!(displayed(&mut dom, parts), Some(date!(2026 - 02 - 09)));
    assert!(can_apply(&mut dom, parts));
    assert!(
        harness.reported.borrow().is_empty(),
        "staging must not report: {:?}",
        harness.reported.borrow()
    );

    dom.in_scope(ScopeId::APP, || parts.confirm.apply());
    settle(&mut dom);
    assert_eq!(
        *harness.reported.borrow(),
        vec![Some(date!(2026 - 02 - 09))]
    );
    assert!(!is_open_of(&mut dom, parts), "Apply closes the popover");
    assert!(
        !can_apply(&mut dom, parts),
        "applied: nothing left to apply"
    );
    assert_eq!(displayed(&mut dom, parts), Some(date!(2026 - 02 - 09)));
}

fn is_open_of(dom: &mut VirtualDom, parts: StagingParts) -> bool {
    dom.in_scope(ScopeId::APP, || *parts.open.peek())
}

#[test]
fn cancel_and_every_other_close_drop_the_staged_selection() {
    let (mut dom, harness, parts) = staging_dom();
    let committed = Some(date!(2026 - 02 - 01));

    // Cancel.
    stage(&mut dom, parts, Some(date!(2026 - 02 - 09)));
    dom.in_scope(ScopeId::APP, || parts.confirm.cancel());
    settle(&mut dom);
    assert!(!is_open_of(&mut dom, parts));
    assert_eq!(displayed(&mut dom, parts), committed);
    assert!(!can_apply(&mut dom, parts));

    // Escape / an outside click / the trigger all just set `open` to false.
    let mut open = parts.open;
    dom.in_scope(ScopeId::APP, || open.set(true));
    settle(&mut dom);
    stage(&mut dom, parts, Some(date!(2026 - 02 - 10)));
    assert_eq!(displayed(&mut dom, parts), Some(date!(2026 - 02 - 10)));
    dom.in_scope(ScopeId::APP, || open.set(false));
    settle(&mut dom);
    assert_eq!(
        displayed(&mut dom, parts),
        committed,
        "a light dismiss restores the value"
    );
    assert!(
        harness.reported.borrow().is_empty(),
        "nothing was ever committed"
    );
}

#[test]
fn staging_the_committed_value_again_leaves_nothing_to_apply() {
    let (mut dom, _harness, parts) = staging_dom();
    stage(&mut dom, parts, Some(date!(2026 - 02 - 09)));
    assert!(can_apply(&mut dom, parts));
    stage(&mut dom, parts, Some(date!(2026 - 02 - 01)));
    assert!(!can_apply(&mut dom, parts));
    // Clearing the selection is a change like any other.
    stage(&mut dom, parts, None);
    assert!(can_apply(&mut dom, parts));
    assert_eq!(displayed(&mut dom, parts), None);
}

#[test]
fn without_confirm_nothing_is_staged() {
    let (mut dom, harness, parts) = staging_dom();
    let mut enabled = parts.enabled;
    dom.in_scope(ScopeId::APP, || enabled.set(false));
    settle(&mut dom);
    assert!(!stage(&mut dom, parts, Some(date!(2026 - 02 - 09))));
    assert!(!can_apply(&mut dom, parts));
    assert_eq!(displayed(&mut dom, parts), Some(date!(2026 - 02 - 01)));
    assert!(harness.reported.borrow().is_empty());
}
