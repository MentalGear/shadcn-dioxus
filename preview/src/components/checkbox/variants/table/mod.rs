use super::super::component::*;
use crate::components::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;

struct Member {
    name: &'static str,
    email: &'static str,
    role: &'static str,
}

const MEMBERS: &[Member] = &[
    Member {
        name: "Olivia Martin",
        email: "olivia@example.com",
        role: "Owner",
    },
    Member {
        name: "Jackson Lee",
        email: "jackson@example.com",
        role: "Admin",
    },
    Member {
        name: "Isabella Nguyen",
        email: "isabella@example.com",
        role: "Member",
    },
    Member {
        name: "William Kim",
        email: "will@example.com",
        role: "Member",
    },
];

// The themed `Checkbox` has no indeterminate styling (the indicator draws the
// check icon for it), so the header box is checked only when every row is and
// unchecked otherwise, instead of showing a misleading check for "some".
#[component]
pub fn Demo() -> Element {
    let mut selected = use_signal(|| vec![false, true, false, false]);
    let all = selected().iter().all(|s| *s);
    let to_state = |on: bool| {
        if on {
            CheckboxState::Checked
        } else {
            CheckboxState::Unchecked
        }
    };
    rsx! {
        div { style: "width: 100%; max-width: 28rem;",
            Table {
                TableHeader {
                    TableRow {
                        TableHead { style: "width: 2rem;",
                            Checkbox {
                                aria_label: "Select every member",
                                checked: Some(to_state(all)),
                                on_checked_change: move |next| {
                                    let on = next == CheckboxState::Checked;
                                    selected.write().iter_mut().for_each(|s| *s = on);
                                },
                            }
                        }
                        TableHead { "Name" }
                        TableHead { "Role" }
                    }
                }
                TableBody {
                    for (i , member) in MEMBERS.iter().enumerate() {
                        TableRow { key: "{member.email}", selected: selected()[i],
                            TableCell {
                                Checkbox {
                                    aria_label: "Select member {member.name}",
                                    checked: Some(to_state(selected()[i])),
                                    on_checked_change: move |next| {
                                        selected.write()[i] = next == CheckboxState::Checked;
                                    },
                                }
                            }
                            TableCell { "{member.name}" }
                            TableCell { "{member.role}" }
                        }
                    }
                }
            }
        }
    }
}
