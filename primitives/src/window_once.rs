//! State that belongs to one window, not to the thread.
//!
//! A desktop app with two windows runs two VirtualDoms on one thread, each with its own document. A
//! `thread_local!` that says "this page already has its stylesheet" or "this page's scroll is locked"
//! is therefore shared by both documents: the second window is told the work is done and never gets
//! it. State of that kind lives in the VirtualDom's root context instead, as the pointer bridge, the
//! form-reset read queue and the bridge transport keep theirs.

use std::marker::PhantomData;

use dioxus::prelude::*;

/// The marker a window leaves in its root context once `M`'s work has been claimed.
struct Claimed<M>(PhantomData<M>);

impl<M> Clone for Claimed<M> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

/// `true` the first time `M` asks in this VirtualDom, `false` after. Outside a runtime it answers
/// `false`: there is no window to do the work for.
pub(crate) fn first_in_window<M: 'static>() -> bool {
    if dioxus_core::Runtime::try_current().is_none()
        || try_consume_context::<Claimed<M>>().is_some()
    {
        return false;
    }
    crate::dioxus_core::provide_root_context(Claimed::<M>(PhantomData));
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::with_runtime;
    use std::path::Path;

    struct Work;

    #[test]
    fn each_window_is_first_once() {
        with_runtime(|| {
            assert!(first_in_window::<Work>());
            assert!(!first_in_window::<Work>());
        });
        with_runtime(|| {
            assert!(
                first_in_window::<Work>(),
                "the second window has not done the work"
            );
        });
    }

    #[test]
    fn outside_a_runtime_nothing_is_first() {
        assert!(!first_in_window::<Work>());
    }

    fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("read dir").flatten() {
            let path = entry.path();
            if path.is_dir() {
                rust_files(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    /// A `thread_local!` outside test code is state shared by every window on the thread. The one
    /// that stays, `dismiss::NEXT_SEQ`, only counts up and is compared within one window, so a
    /// second window's layers still order correctly.
    #[test]
    fn no_thread_local_holds_window_state() {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        rust_files(&src, &mut files);
        let mut offenders = Vec::new();
        for file in files {
            let rel = file
                .strip_prefix(&src)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if rel == "dismiss.rs" || rel == "window_once.rs" {
                continue;
            }
            let text = std::fs::read_to_string(&file).expect("read source");
            let code_part = text.split("#[cfg(test)]").next().unwrap_or("");
            for (number, line) in code_part.lines().enumerate() {
                let code = line.trim_start();
                if !code.starts_with("//") && code.starts_with("thread_local!") {
                    offenders.push(format!("{rel}:{}", number + 1));
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "keep per-window state in the root context (window_once): {offenders:?}"
        );
    }
}
