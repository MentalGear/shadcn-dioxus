use super::super::component::{Column, ColumnAlign, DataTable, DataTableLayout, RowFn, SortDirection};
use dioxus::prelude::*;

const ORDERS: usize = 50_000;

const FIRST: [&str; 8] = ["ada", "ben", "chloe", "dev", "elif", "finn", "gita", "hugo"];
const LAST: [&str; 9] = ["adams", "baker", "chen", "diaz", "evans", "fox", "gupta", "hall", "ito"];
const STATUS: [&str; 4] = ["Paid", "Pending", "Failed", "Refunded"];

#[derive(Debug, Clone, PartialEq)]
struct Order {
    id: String,
    customer: String,
    status: &'static str,
    /// Whole US cents, so the sort key is an integer.
    total_cents: i64,
}

/// A deterministic data set, so the demo needs no fixture file.
fn orders() -> Vec<Order> {
    (0..ORDERS)
        .map(|i| Order {
            id: format!("ORD-{:05}", i + 1),
            customer: format!("{}.{}{}@example.com", FIRST[i % FIRST.len()], LAST[(i / 8) % LAST.len()], i % 97),
            status: STATUS[(i * 31) % STATUS.len()],
            total_cents: ((i as u64 + 1).wrapping_mul(2_654_435_761) % 100_000) as i64,
        })
        .collect()
}

fn format_amount(cents: i64) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

#[component]
pub fn Demo() -> Element {
    let rows = use_memo(orders);
    let columns = vec![
        Column::new("id", "Order", |o: &Order| rsx! { "{o.id}" })
            .sortable(|o: &Order| o.id.clone())
            .grid_width(120),
        Column::new("customer", "Customer", |o: &Order| rsx! { "{o.customer}" })
            .sortable(|o: &Order| o.customer.clone())
            .filterable(|o: &Order| o.customer.clone())
            .grid_width(260),
        Column::new("status", "Status", |o: &Order| rsx! { "{o.status}" })
            .sortable(|o: &Order| o.status)
            .filterable(|o: &Order| o.status.to_string())
            .grid_width(140),
        Column::new("total", "Total", |o: &Order| rsx! { "{format_amount(o.total_cents)}" })
            .sortable(|o: &Order| o.total_cents)
            .align(ColumnAlign::End)
            .grid_width(140),
    ];

    rsx! {
        DataTable::<Order> {
            rows,
            columns,
            row_key: RowFn::new(|o: &Order| o.id.clone()),
            layout: DataTableLayout::Grid,
            selectable: true,
            filter_placeholder: "Filter customers or status...".to_string(),
            filter_label: "Filter orders".to_string(),
            default_sort: Some(("id".to_string(), SortDirection::Ascending)),
            grid_height: "28rem".to_string(),
            grid_label: "Orders, 50,000 rows, sortable, filterable and selectable".to_string(),
        }
    }
}
