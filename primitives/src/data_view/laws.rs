//! Law tests: after any mix of data edits and transitions the view obeys its
//! invariants. Deterministic pseudo-random sequences (a fixed-seed LCG), so a
//! failure reproduces from the seed in its message and no dependency is added.

use super::*;
use std::collections::HashSet;

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

#[derive(Debug, Clone)]
struct Row {
    id: String,
    name: String,
    n: i32,
}

fn random_rows(rng: &mut Lcg) -> Vec<Row> {
    let count = rng.below(30);
    let mut ids: Vec<usize> = (0..40).collect();
    // A random subset in a random order: ids are unique, source order is arbitrary.
    for i in (1..ids.len()).rev() {
        ids.swap(i, rng.below(i + 1));
    }
    ids.truncate(count);
    ids.into_iter()
        .map(|i| Row {
            id: format!("k{i}"),
            name: format!("{}{}", ["al", "bo", "ca", "da"][rng.below(4)], i),
            n: rng.below(6) as i32,
        })
        .collect()
}

fn fields() -> Vec<Field<Row>> {
    vec![
        Field::new("name")
            .sortable(|r: &Row| r.name.clone())
            .filterable(|r: &Row| r.name.clone()),
        Field::new("n").sortable(|r: &Row| r.n),
    ]
}

fn step(rng: &mut Lcg, state: &mut ViewState, rows: &[Row], view: &View) {
    match rng.below(9) {
        0 => state.toggle_sort(["name", "n"][rng.below(2)]),
        1 => state.set_filter(["", "a", "AL", " b ", "9", "zz", "da1"][rng.below(7)]),
        2 => state.set_page_size(rng.below(8)),
        3 => state.set_page(rng.below(12), view.page_count),
        4 => state.next_page(view.page_count),
        5 => state.previous_page(),
        6 => {
            let key = if rows.is_empty() || rng.below(4) == 0 {
                format!("gone{}", rng.below(5))
            } else {
                rows[rng.below(rows.len())].id.clone()
            };
            state.set_selected(&key, rng.below(3) != 0);
        }
        7 => {
            let mut keys: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
            keys.push("stale".into());
            for i in (1..keys.len()).rev() {
                keys.swap(i, rng.below(i + 1));
            }
            keys.truncate(rng.below(keys.len() + 1));
            state.order = if rng.below(4) == 0 {
                None
            } else {
                Some(KeyOrder::new(keys))
            };
        }
        _ => state.page = rng.below(50), // a page past the end, as after the data shrinks
    }
}

#[test]
fn view_laws_hold_over_random_sequences() {
    let fields = fields();
    for seed in 1..=40u64 {
        let mut rng = Lcg(seed);
        let mut rows = random_rows(&mut rng);
        let mut state = ViewState::new(1 + rng.below(6), None);
        for stepno in 0..200 {
            if rng.below(10) == 0 {
                rows = random_rows(&mut rng);
            }
            let ctx = format!("seed {seed} step {stepno}");
            let view = derive(&rows, &state, &fields, |r| r.id.clone());

            // Everything on one page: the whole filtered, ordered list.
            let mut all_state = state.clone();
            all_state.page = 0;
            all_state.page_size = usize::MAX;
            let all = derive(&rows, &all_state, &fields, |r| r.id.clone());

            // The filtered keys, by brute force.
            let query = state.filter.trim().to_lowercase();
            let expected: HashSet<String> = rows
                .iter()
                .filter(|r| query.is_empty() || r.name.to_lowercase().contains(&query))
                .map(|r| r.id.clone())
                .collect();

            // Duplicate-free permutation of the filtered keys.
            let unique: HashSet<&String> = all.keys.iter().collect();
            assert_eq!(unique.len(), all.keys.len(), "{ctx}: duplicate keys");
            assert_eq!(
                unique,
                expected.iter().collect::<HashSet<_>>(),
                "{ctx}: not a permutation of the filtered keys"
            );
            assert_eq!(all.filtered_count, expected.len(), "{ctx}");
            assert_eq!(view.filtered_count, expected.len(), "{ctx}");

            // The page is the clamped window of that list.
            assert!(
                view.page < view.page_count,
                "{ctx}: page {} of {}",
                view.page,
                view.page_count
            );
            assert_eq!(
                view.page_count,
                expected.len().div_ceil(state.page_size).max(1),
                "{ctx}"
            );
            let start = (view.page * state.page_size).min(all.keys.len());
            let end = (start + state.page_size).min(all.keys.len());
            assert_eq!(
                view.keys,
                all.keys[start..end],
                "{ctx}: page is not a window of the whole"
            );
            assert_eq!(view.indices.len(), view.keys.len(), "{ctx}");
            for (i, k) in view.indices.iter().zip(&view.keys) {
                assert_eq!(&rows[*i].id, k, "{ctx}: index and key disagree");
            }

            // Counts.
            assert!(
                view.selected_in_view <= view.filtered_count,
                "{ctx}: {} of {}",
                view.selected_in_view,
                view.filtered_count
            );
            assert!(view.selected_in_view <= view.selected_total, "{ctx}");
            assert_eq!(view.selected_total, state.selected.len(), "{ctx}");
            assert_eq!(
                view.selected_in_view,
                expected
                    .iter()
                    .filter(|k| state.selected.contains(*k))
                    .count(),
                "{ctx}"
            );

            // Reconciling an order is idempotent and a permutation of the current keys.
            if let Some(order) = &state.order {
                let current: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
                let once = order.reconcile(current.iter().copied());
                let again = KeyOrder::new(once.iter().map(|k| k.to_string()))
                    .reconcile(current.iter().copied());
                assert_eq!(once, again, "{ctx}: reconcile is not idempotent");
                let mut sorted_once = once.clone();
                sorted_once.sort_unstable();
                let mut sorted_current = current.clone();
                sorted_current.sort_unstable();
                assert_eq!(
                    sorted_once, sorted_current,
                    "{ctx}: reconcile is not a permutation"
                );
            }

            step(&mut rng, &mut state, &rows, &view);
        }
    }
}
