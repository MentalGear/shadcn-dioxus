use super::super::component::{Column, ColumnAlign, DataTable, RowFn};
use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Task {
    id: &'static str,
    title: &'static str,
    owner: &'static str,
}

const TASKS: &[Task] = &[
    Task { id: "T-1", title: "Cover page", owner: "Eddie Lake" },
    Task { id: "T-2", title: "Table of contents", owner: "Eddie Lake" },
    Task { id: "T-3", title: "Executive summary", owner: "Jamik Tashpulatov" },
    Task { id: "T-4", title: "Technical approach", owner: "Jamik Tashpulatov" },
    Task { id: "T-5", title: "Design", owner: "Eddie Lake" },
];

/// Row reordering next to a sortable column: an active sort wins over the manual order, so the
/// drag handles are inert while a column is sorted and work again once the sort is cleared.
#[component]
pub fn Demo() -> Element {
    let columns = vec![
        Column::new("title", "Header", |t: &Task| rsx! { "{t.title}" }).sortable(|t: &Task| t.title),
        Column::new("owner", "Reviewer", |t: &Task| rsx! { "{t.owner}" }),
        Column::new("id", "ID", |t: &Task| rsx! { "{t.id}" }).align(ColumnAlign::End),
    ];
    rsx! {
        DataTable::<Task> {
            rows: TASKS.to_vec(),
            columns,
            row_key: RowFn::new(|t: &Task| t.id.to_string()),
            reorderable: true,
        }
    }
}
