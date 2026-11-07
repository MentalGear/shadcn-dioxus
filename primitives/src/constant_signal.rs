//! A constant `ReadSignal`, built once per component instance.
//!
//! The hooks that take a `ReadSignal` and never change it (a collection's `loop` flag, a constant
//! `required`, a slider's `min_steps`) need a signal, not a new one per render.
//! `ReadSignal::new(Signal::new(v))` in a render body allocates a signal every render, and as a prop
//! it compares unequal each time, which defeats the child's memoisation. `use_constant_signal` keeps
//! the first one. `scripts/check-constant-signal.sh` bans the raw form in render code.

use dioxus::prelude::*;

/// A `ReadSignal` holding `value`, created on the first render and reused after it. A hook.
pub(crate) fn use_constant_signal<T: 'static>(value: T) -> ReadSignal<T> {
    use_hook(move || ReadSignal::new(Signal::new(value)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::with_runtime;

    #[test]
    fn the_signal_holds_the_value() {
        assert_eq!(with_runtime(|| use_constant_signal(7u32)()), 7);
    }

    #[test]
    fn a_rerender_returns_the_same_signal() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let seen: Rc<RefCell<Vec<ReadSignal<u32>>>> = Rc::default();
        let sink = seen.clone();
        let mut dom = VirtualDom::new_with_props(
            |sink: Rc<RefCell<Vec<ReadSignal<u32>>>>| {
                sink.borrow_mut().push(use_constant_signal(3));
                rsx! { div {} }
            },
            sink,
        );
        dom.rebuild_in_place();
        dom.mark_dirty(ScopeId::APP);
        dom.render_immediate(&mut dioxus_core::NoOpMutations);
        let seen = seen.borrow();
        assert_eq!(seen.len(), 2, "the component rendered twice");
        assert!(seen[0] == seen[1], "a second render built a new signal");
    }
}
