use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use dioxus::prelude::*;

const MANY: usize = 100_000;
/// The row counts the buttons switch between: `Auto` windows the first and keeps every row in
/// the DOM for the second (800 is under its threshold of 1,000).
const COUNTS: [usize; 2] = [MANY, 800];
const ALIGNS: [(ScrollAlign, &str); 4] = [
    (ScrollAlign::Start, "Start"),
    (ScrollAlign::Center, "Center"),
    (ScrollAlign::End, "End"),
    (ScrollAlign::Nearest, "Nearest"),
];
/// Every row is this tall, so a jump lands where its alignment says, to the pixel.
const ROW: u32 = 48;

#[component]
pub fn Demo() -> Element {
    let scroller = use_virtual_scroller();
    let mut rows = use_signal(|| MANY);
    let mut align = use_signal(|| ScrollAlign::Start);
    let mut typed = use_signal(|| "5000".to_string());
    let mut last_jump = use_signal(|| None::<(usize, ScrollAlign)>);

    let mut jump = move |row: usize| {
        let index = row.saturating_sub(1).min(rows.peek().saturating_sub(1));
        scroller.scroll_to_index(index, align());
        last_jump.set(Some((index + 1, align())));
    };

    rsx! {
        div { class: "dx-virtual-list-demo", "data-vl-demo": "scroll-to",
            p { class: "dx-virtual-list-subtitle",
                "Scroll to a row that is not mounted: the list works out where it is from its row sizes, moves the window there and puts the row at the chosen alignment. Nearest does nothing while the row is already fully in view. With 800 rows every row is in the DOM and the browser scrolls instead."
            }
            style { r#".dx-virtual-list-container {{
  position: relative;
  height: 24rem;
  contain: layout paint;
  overflow-y: auto;
}}

.dx-virtual-list-demo {{
  display: flex;
  flex-direction: column;
  margin: 0 auto;
  gap: 0.75rem;
}}

.dx-virtual-list-demo .dx-virtual-list-subtitle,
.dx-virtual-list-demo .dx-virtual-list-readout {{
  margin: 0;
  color: var(--dx-muted-foreground);
  font-size: 0.9rem;
}}

.dx-virtual-list-readout strong {{
  color: var(--dx-foreground);
  font-variant-numeric: tabular-nums;
}}

.dx-virtual-list-controls {{
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--dx-space-2);
}}

.dx-virtual-list-row {{
  display: flex;
  align-items: center;
  box-sizing: border-box;
  height: {ROW}px;
  padding: 0 0.9rem;
  border-bottom: 1px solid var(--dx-border);
  background: var(--dx-card);
  color: var(--dx-foreground);
  font-variant-numeric: tabular-nums;
}}"# }
            div { class: "dx-virtual-list-controls",
                Input {
                    r#type: "number",
                    min: "1",
                    "aria-label": "Row number",
                    "data-vl-row-input": "",
                    value: "{typed}",
                    style: "width: 8rem;",
                    oninput: move |event: FormEvent| typed.set(event.value()),
                }
                Button {
                    "data-vl-go": "",
                    onclick: move |_| {
                        if let Ok(row) = typed.peek().trim().parse::<usize>() {
                            jump(row);
                        }
                    },
                    "Scroll to row"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    "data-vl-jump-5000": "",
                    onclick: move |_| jump(5_000),
                    "Jump to 5,000"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    "data-vl-jump-last": "",
                    onclick: move |_| {
                        let last = *rows.peek();
                        jump(last);
                    },
                    "Last row"
                }
            }
            div { class: "dx-virtual-list-controls", role: "group", "aria-label": "Alignment",
                for (value, name) in ALIGNS {
                    Button {
                        key: "{name}",
                        variant: if align() == value { ButtonVariant::Secondary } else { ButtonVariant::Outline },
                        "aria-pressed": (align() == value).to_string(),
                        onclick: move |_| align.set(value),
                        "{name}"
                    }
                }
            }
            div { class: "dx-virtual-list-controls", role: "group", "aria-label": "Row count",
                for count in COUNTS {
                    Button {
                        key: "{count}",
                        variant: if rows() == count { ButtonVariant::Secondary } else { ButtonVariant::Outline },
                        "aria-pressed": (rows() == count).to_string(),
                        onclick: move |_| rows.set(count),
                        "{count} rows"
                    }
                }
            }
            p { class: "dx-virtual-list-readout",
                "Last jump: "
                strong { "data-vl-last-jump": "",
                    if let Some((row, align)) = last_jump() {
                        "row {row}, {align:?}"
                    } else {
                        "none"
                    }
                }
            }
            VirtualList {
                count: rows,
                scroller,
                buffer: 8usize,
                estimate_size: |_idx: usize| ROW,
                "aria-label": "Rows to scroll to",
                render_item: move |idx: usize| rsx! {
                    div { class: "dx-virtual-list-row", "Row {idx + 1}" }
                },
            }
        }
    }
}
