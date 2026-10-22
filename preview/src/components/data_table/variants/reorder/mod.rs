use super::super::component::{Column, ColumnAlign, DataTable, RowFn};
use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Task {
    id: &'static str,
    title: &'static str,
    status: &'static str,
    owner: &'static str,
}

const TASKS: &[Task] = &[
    Task { id: "T-1", title: "Cover page", status: "In process", owner: "Eddie Lake" },
    Task { id: "T-2", title: "Table of contents", status: "Done", owner: "Eddie Lake" },
    Task { id: "T-3", title: "Executive summary", status: "Done", owner: "Eddie Lake" },
    Task { id: "T-4", title: "Technical approach", status: "Done", owner: "Jamik Tashpulatov" },
    Task { id: "T-5", title: "Design", status: "In process", owner: "Jamik Tashpulatov" },
];

/// `dashboard-01`'s data table: a drag-handle column that reorders the rows (pointer, or Space and
/// the arrow keys on the handle), and a Columns menu that hides columns and moves them (Alt+Up and
/// Alt+Down on a menu item).
#[component]
pub fn Demo() -> Element {
    let columns = vec![
        Column::new("title", "Header", |t: &Task| rsx! { "{t.title}" }).hideable(false),
        Column::new("status", "Status", |t: &Task| rsx! { "{t.status}" }),
        Column::new("owner", "Reviewer", |t: &Task| rsx! { "{t.owner}" }),
        Column::new("id", "ID", |t: &Task| rsx! { "{t.id}" }).align(ColumnAlign::End),
    ];
    rsx! {
        DataTable::<Task> {
            rows: TASKS.to_vec(),
            columns,
            row_key: RowFn::new(|t: &Task| t.id.to_string()),
            reorderable: true,
            column_menu: true,
        }
    }
}
