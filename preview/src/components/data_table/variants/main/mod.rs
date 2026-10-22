use super::super::component::{Column, DataTable, RowFn, SortDirection};
use dioxus::prelude::*;

// Fixture shaped like shadcn/ui's own Data Table demo (id/status/email/
// amount, ~12 rows) per dev-docs/component-backlog.md row 69's demo brief.
// Values are this crate's own, chosen so the sort orders below are easy to
// verify by inspection -- `playwright/data_table.spec.ts` and
// `playwright/oracle/tier1-apg/sortable-table.spec.ts` assert the exact
// row transitions these produce, so this fixture is these specs' fixture
// too (single source of truth, not duplicated).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Payment {
    id: &'static str,
    status: &'static str,
    email: &'static str,
    /// Whole US cents, not a float -- avoids float-equality/ordering
    /// pitfalls in `sort_rows`'s `Ord` key.
    amount_cents: i64,
}

const PAYMENTS: &[Payment] = &[
    Payment { id: "PMT-01", status: "Success", email: "ken99@example.com", amount_cents: 31_600 },
    Payment { id: "PMT-02", status: "Processing", email: "carmella@example.com", amount_cents: 24_200 },
    Payment { id: "PMT-03", status: "Success", email: "alexandra@example.com", amount_cents: 83_700 },
    Payment { id: "PMT-04", status: "Failed", email: "michael@example.com", amount_cents: 72_100 },
    Payment { id: "PMT-05", status: "Pending", email: "jenny@example.com", amount_cents: 50_000 },
    Payment { id: "PMT-06", status: "Success", email: "derek@example.com", amount_cents: 12_000 },
    Payment { id: "PMT-07", status: "Pending", email: "olivia@example.com", amount_cents: 61_000 },
    Payment { id: "PMT-08", status: "Processing", email: "hannah@example.com", amount_cents: 7_500 },
    Payment { id: "PMT-09", status: "Success", email: "isaac@example.com", amount_cents: 43_000 },
    Payment { id: "PMT-10", status: "Failed", email: "monroe@example.com", amount_cents: 99_900 },
    Payment { id: "PMT-11", status: "Pending", email: "zara@example.com", amount_cents: 5_500 },
    Payment { id: "PMT-12", status: "Success", email: "felix@example.com", amount_cents: 28_900 },
];

fn format_amount(cents: i64) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

#[component]
pub fn Demo() -> Element {
    // The whole table is described once: columns (typed sort keys, the one
    // filtered column), a row key, and which features are on. The component
    // owns sort, filter, page and selection (`use_data_table`).
    //
    // Exactly one column is the "active" sort at all times (default: email
    // ascending), matching the vendored APG sortable-table reference, which
    // likewise always has exactly one column carrying `aria-sort` (see
    // `DataTableColumnHeader`'s doc and
    // `playwright/oracle/tier1-apg/sortable-table.spec.ts`'s R1).
    let columns = vec![
        Column::new("id", "ID", |p: &Payment| rsx! { "{p.id}" }),
        Column::new("status", "Status", |p: &Payment| rsx! { "{p.status}" }),
        // Case-insensitive substring match anywhere in the email, not just a
        // prefix -- shadcn/ui parity, see `includes_string`'s doc (`state.rs`).
        Column::new("email", "Email", |p: &Payment| rsx! { "{p.email}" })
            .sortable(|p: &Payment| p.email)
            .filterable(|p: &Payment| p.email.to_string()),
        Column::new("amount", "Amount", |p: &Payment| rsx! { "{format_amount(p.amount_cents)}" })
            .sortable(|p: &Payment| p.amount_cents),
    ];

    rsx! {
        DataTable::<Payment> {
            rows: PAYMENTS.to_vec(),
            columns,
            row_key: RowFn::new(|p: &Payment| p.id.to_string()),
            selectable: true,
            filter_placeholder: "Filter emails...".to_string(),
            filter_label: "Filter by email".to_string(),
            page_sizes: vec![5, 10, 20],
            default_sort: Some(("email".to_string(), SortDirection::Ascending)),
        }
    }
}
