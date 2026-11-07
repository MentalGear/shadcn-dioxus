// The names every `rust` block of a component `docs.md` may use without declaring them (backlog row 392).
// `build.rs` (`write_docs_samples`) pastes the part above the marker line into each generated sample module
// and the `let` lines below it, for the names a sample actually mentions, at the top of the sample's body.
// Only what a reader would take from the page's own prose belongs here (the imports, the `children` of a
// wrapper, a chart's `config` and `data`); a block that needs more is made self-contained in its docs.md.
use dioxus_icons::lucide::{
    ArrowUpRight, BluetoothConnected, CircleAlert, FileText, Footprints, Inbox, Play, Terminal,
};
use dioxus_primitives::avatar::AvatarState;
use dioxus_primitives::checkbox::CheckboxState;
use dioxus_primitives::message_scroller::{ScrollBehavior, ScrollToMessageOptions};
use dioxus_primitives::modality::{FocusTarget, OpenModality};
use dioxus_primitives::scroll_area::ScrollDirection;
use dioxus_primitives::toast::{ToastOptions, ToastType, ToastUpdate};
use dioxus_primitives::{AsChild, ContentAlign, ContentSide};
use palette::{encoding, Hsva};
use crate::components::tag_group::RemoveButton;
use dioxus_primitives::toast::consume_toast;
#[component]
fn Icon() -> Element {
    rsx! {}
}
use std::collections::HashSet;
use std::time::Duration;
fn reply() {}
fn label_of(value: &str) -> String {
    value.to_string()
}
fn save<T>(_: T) {}
fn partial_text() -> String {
    String::new()
}
fn is_generating() -> bool {
    false
}
// --- ambient locals ---
let children: Element = rsx! {};
let config = ChartConfig::new();
let data: Vec<ChartDatum> = Vec::new();
let CODE = "fn main() {}";
let highlighted: Vec<Element> = Vec::new();
let color = use_signal(|| Hsva::<encoding::Srgb, f64>::new(0.0, 1.0, 1.0, 1.0));
let mut text = use_signal(String::new);
let count = 0usize;
let message_text = String::new();
let reply_text = String::new();
let my_items = use_signal(Vec::<String>::new);
let toast_api = consume_toast();
