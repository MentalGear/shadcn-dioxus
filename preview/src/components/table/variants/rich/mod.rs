use super::super::component::*;
use crate::components::badge::{Badge, BadgeVariant};
use crate::components::checkbox::Checkbox;
use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;

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
/// column and right-aligned amounts with a footer total. Selection is controlled, so a selected
/// row renders `data-state="selected"` and the header checkbox mirrors "all / some / none".
#[component]
pub fn Demo() -> Element {
    let mut selected = use_signal(|| [false, true, false, false]);
    let count = selected().iter().filter(|s| **s).count();
    let header_state = if count == 0 {
        CheckboxState::Unchecked
    } else if count == ORDERS.len() {
        CheckboxState::Checked
    } else {
        CheckboxState::Indeterminate
    };
    rsx! {
        Table {
            TableCaption { "Recent orders. {count} of {ORDERS.len()} selected." }
            TableHeader {
                TableRow {
                    TableHead {
                        Checkbox {
                            checked: header_state,
                            aria_label: "Select every order",
                            on_checked_change: move |state: CheckboxState| {
                                let all = bool::from(state);
                                selected.set([all; ORDERS.len()]);
                            },
                        }
                    }
                    TableHead { "Order" }
                    TableHead { "Status" }
                    TableHead { style: "text-align: end;", "Total" }
                }
            }
            TableBody {
                for (i , order) in ORDERS.iter().enumerate() {
                    TableRow { key: "{order.id}", selected: selected()[i],
                        TableCell {
                            Checkbox {
                                checked: if selected()[i] { CheckboxState::Checked } else { CheckboxState::Unchecked },
                                aria_label: "Choose order {order.id}",
                                on_checked_change: move |state: CheckboxState| {
                                    selected.with_mut(|rows| rows[i] = bool::from(state));
                                },
                            }
                        }
                        TableCell { "{order.id}" }
                        TableCell {
                            Badge { variant: order.variant, "{order.status}" }
                        }
                        TableCell { style: "text-align: end;", "{order.amount}" }
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
