use super::*;
use dioxus_core::Mutation;

static GENERATION: GlobalSignal<u32> = Signal::global(|| 0);

#[component]
fn Host() -> Element {
    rsx! {
        span { SegmentText { generation: GENERATION(), text: "2026" } }
    }
}

fn count(edits: &[Mutation], is: impl Fn(&Mutation) -> bool) -> usize {
    edits.iter().filter(|edit| is(edit)).count()
}

#[test]
fn a_new_generation_replaces_the_text_node_and_the_same_one_does_not() {
    let mut dom = VirtualDom::new(Host);
    dom.rebuild_in_place();
    assert!(dioxus_ssr::render(&dom).contains("2026"));

    // The same generation: the text did not change, so the vdom writes nothing.
    dom.mark_dirty(ScopeId::APP);
    let unchanged = dom.render_immediate_to_vec();
    assert!(unchanged.edits.is_empty(), "{:?}", unchanged.edits);

    // A new generation: a fresh text node is created and the old one is removed.
    dom.in_runtime(|| *GENERATION.write() += 1);
    let replaced = dom.render_immediate_to_vec();
    let created = count(
        &replaced.edits,
        |e| matches!(e, Mutation::CreateTextNode { value, .. } if value == "2026"),
    );
    let removed = count(&replaced.edits, |e| {
        matches!(
            e,
            Mutation::Remove { .. }
                | Mutation::ReplaceWith { .. }
                | Mutation::ReplacePlaceholder { .. }
        )
    });
    assert_eq!(created, 1, "{:?}", replaced.edits);
    assert!(removed >= 1, "{:?}", replaced.edits);
    assert!(dioxus_ssr::render(&dom).contains("2026"));
}
