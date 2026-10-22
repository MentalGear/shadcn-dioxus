use super::super::component::*;
use crate::components::data_table::{use_data_table, Column, DataTableOptions, RowFn};
use crate::components::badge::{Badge, BadgeVariant};
use crate::components::checkbox::Checkbox;
use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;

#[derive(Clone, Copy, PartialEq)]
struct Order {
    id: &'static str,
    status: &'static str,
    variant: BadgeVariant,
    amount: &'static str,
}

const ORDERS: [Order; 4] = [
    Order {
        id: "ORD-3021",
        status: "Paid",
        variant: BadgeVariant::Secondary,
        amount: "$320.00",
    },
    Order {
        id: "ORD-3022",
        status: "Pending",
        variant: BadgeVariant::Outline,
        amount: "$149.50",
    },
    Order {
        id: "ORD-3023",
        status: "Failed",
        variant: BadgeVariant::Destructive,
        amount: "$89.00",
    },
    Order {
        id: "ORD-3024",
        status: "Paid",
        variant: BadgeVariant::Secondary,
        amount: "$210.50",
    },
];

/// A leading selection column (each `Checkbox` named by its own `aria_label`), a `Badge` status
/// column and right-aligned amounts with a footer total, rendered from the Data Table's model
/// (`use_data_table`: the rows, their keys, the selection and the header checkbox's all / some / none
/// state all come from it), so a selected row renders `data-state="selected"`.
#[component]
pub fn Demo() -> Element {
    let rows = use_signal(|| ORDERS.to_vec());
    let columns = vec![
        Column::new("id", "Order", |o: &Order| rsx! { "{o.id}" }),
        Column::new("status", "Status", |o: &Order| rsx! { Badge { variant: o.variant, "{o.status}" } }),
        Column::new("amount", "Total", |o: &Order| rsx! { "{o.amount}" }),
    ];
    let model = use_data_table(
        rows.into(),
        &columns,
        &RowFn::new(|o: &Order| o.id.to_string()),
        DataTableOptions { page_size: usize::MAX, ..Default::default() },
    );
    let state = model.state;
    use_hook(|| state.set_selected("ORD-3022", true));
    let view = &model.view;
    let page_keys = view.keys.clone();
    rsx! {
        Table {
            TableCaption { "Recent orders. {view.selected_count} of {ORDERS.len()} selected." }
            TableHeader {
                TableRow {
                    TableHead {
                        Checkbox {
                            checked: view.select_all,
                            aria_label: "Select every order",
                            on_checked_change: move |next: CheckboxState| state.set_many_selected(&page_keys, bool::from(next)),
                        }
                    }
                    for column in columns.iter() {
                        TableHead { key: "{column.id}", style: if column.id == "amount" { "text-align: end;" } else { "" }, "{column.header}" }
                    }
                }
            }
            TableBody {
                for (row , key) in view.rows.iter().zip(view.keys.iter()) {
                    TableRow { key: "{key}", selected: model.is_selected(key),
                        TableCell {
                            Checkbox {
                                checked: if model.is_selected(key) { CheckboxState::Checked } else { CheckboxState::Unchecked },
                                aria_label: "Choose order {key}",
                                on_checked_change: {
                                    let key = key.clone();
                                    move |next: CheckboxState| state.set_selected(&key, bool::from(next))
                                },
                            }
                        }
                        for column in columns.iter() {
                            TableCell { key: "{column.id}", style: if column.id == "amount" { "text-align: end;" } else { "" }, {column.cell.call(row)} }
                        }
                    }
                }
            }
            TableFooter {
                TableRow {
                    TableCell { colspan: 3, "Total" }
                    TableCell { style: "text-align: end;", "$769.00" }
                }
            }
        }
    }
}
