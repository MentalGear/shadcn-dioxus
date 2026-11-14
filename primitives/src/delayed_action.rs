//! The one cancellable delayed task: [`DelayedAction`].
//!
//! "Run this once, later, unless it is cancelled or replaced first" is the shape of every hover
//! intent, close dwell, typeahead reset, long press and open gap in this crate. A handle holds at
//! most one pending task: scheduling replaces it, the task clears the handle before its action
//! runs, and the task is a Dioxus task, so it is dropped with the scope that owns it and there is no
//! JS `setTimeout` for anything to forget to clear (`crate::sleep::sleep`). Writing the pattern
//! inline is a class of its own, see `tests::no_site_keeps_its_own_delayed_task`.

use std::time::Duration;

use dioxus::prelude::*;
use dioxus_core::{Runtime, ScopeId, Task};

use crate::sleep::sleep;

/// A single cancellable delayed action.
///
/// Scheduling a new action cancels whatever this handle was already waiting on, so a trigger can
/// call [`Self::schedule`] on every `onmouseenter` without tracking whether a timer is pending.
#[derive(Clone, Copy)]
pub(crate) struct DelayedAction {
    task: Signal<Option<Task>>,
    /// The scope the task is spawned in; `None` spawns it in the scope that calls
    /// [`Self::schedule`].
    owner: Option<ScopeId>,
}

/// A fresh, unscheduled [`DelayedAction`] whose task lives in the scope that schedules it. A hook:
/// call it unconditionally, once per render.
pub(crate) fn use_delayed_action() -> DelayedAction {
    DelayedAction {
        task: use_signal(|| None),
        owner: None,
    }
}

/// A fresh, unscheduled [`DelayedAction`] whose task lives in the scope that called this hook, so a
/// schedule made from a child that unmounts before the delay is up still runs. A hook: call it
/// unconditionally, once per render.
#[cfg(feature = "calendar")]
pub(crate) fn use_owned_delayed_action() -> DelayedAction {
    DelayedAction {
        task: use_signal(|| None),
        owner: Some(dioxus_core::current_scope_id()),
    }
}

impl DelayedAction {
    /// Cancels a pending action, if any.
    pub(crate) fn cancel(&mut self) {
        if let Some(task) = self.task.write().take() {
            task.cancel();
        }
    }

    /// Whether an action is scheduled and has not yet run. Reading it subscribes the caller.
    pub(crate) fn is_pending(&self) -> bool {
        self.task.read().is_some()
    }

    /// [`Self::is_pending`] without subscribing, for an event handler or a task.
    pub(crate) fn is_pending_untracked(&self) -> bool {
        self.task.peek().is_some()
    }

    /// Runs `action` after `delay`, replacing (cancelling) whatever this handle was waiting on.
    /// The handle reads as not pending from the moment `action` starts.
    pub(crate) fn schedule(&mut self, delay: Duration, action: impl FnOnce() + 'static) {
        self.cancel();
        let mut slot = self.task;
        let wait = async move {
            sleep(delay).await;
            slot.set(None);
            action();
        };
        let task = match self.owner {
            Some(scope) => Runtime::current().spawn(scope, wait),
            None => spawn(wait),
        };
        self.task.set(Some(task));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::path::Path;
    use std::rc::Rc;

    use crate::test_support::with_runtime;

    #[test]
    fn a_scheduled_action_is_pending_until_it_is_cancelled() {
        with_runtime(|| {
            let mut action = use_delayed_action();
            assert!(!action.is_pending_untracked());
            action.schedule(Duration::from_secs(60), || {});
            assert!(action.is_pending_untracked());
            action.cancel();
            assert!(!action.is_pending_untracked());
        });
    }

    #[test]
    fn a_new_schedule_replaces_the_pending_one() {
        with_runtime(|| {
            let ran = Rc::new(Cell::new(0));
            let mut action = use_delayed_action();
            let first = ran.clone();
            action.schedule(Duration::from_secs(60), move || first.set(first.get() + 1));
            let second = ran.clone();
            action.schedule(Duration::from_secs(60), move || {
                second.set(second.get() + 10)
            });
            assert!(
                action.is_pending_untracked(),
                "one task is pending, the second"
            );
            action.cancel();
            assert_eq!(ran.get(), 0);
        });
    }

    fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("source dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                rust_files(&path, out);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                out.push(path);
            }
        }
    }

    /// A signal holding a `Task` is a delayed task written out by hand; [`DelayedAction`] is the one
    /// copy. The carousel's autoplay loop repeats instead of running once, so it keeps its own.
    #[test]
    fn no_site_keeps_its_own_delayed_task() {
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
            if rel == "delayed_action.rs" || rel == "carousel/autoplay.rs" {
                continue;
            }
            let text = std::fs::read_to_string(&file).expect("read source");
            for (number, line) in text.lines().enumerate() {
                let code = line.trim_start();
                if !code.starts_with("//")
                    && code.contains("Signal<Option<")
                    && code.contains("Task>>")
                {
                    offenders.push(format!("{rel}:{}", number + 1));
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "use DelayedAction instead of a Signal<Option<Task>>: {offenders:?}"
        );
    }
}
