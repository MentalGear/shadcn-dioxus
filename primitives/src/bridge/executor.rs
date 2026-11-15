//! The Rust half of the page executor (row 449): an owner decides the open level of its top-layer element and the
//! page's one executor (`ts/lib/executor.ts`) applies it.
//!
//! Each level is a [`Command::Executor`](super::Command::Executor) stamped with a nonce, the owner's command count.
//! The page keeps the newest nonce it applied per element and drops an older command; when the page changes the level on its own (a light
//! dismiss, Escape) it reports `{ n, open }` with the nonce it had applied, and the owner drops a report older than
//! its newest command. That one rule removes the races between a report and a command:
//!
//! * the echo of the executor's own show or hide never reaches Rust (the executor reports only what the page
//!   caused, read inside the write's own synchronous `beforetoggle`), so a show followed at once by a close cannot
//!   be turned back into an open by a late echo (backlog row 115's class, on every panel);
//! * a close then a reopen that both happen between two reports: the report of the first close carries the nonce
//!   before the reopen and is dropped;
//! * a light dismiss whose report reaches Rust after the owner decided again (the trigger's own `onclick`) is
//!   dropped the same way, so the level the owner decided last is the level the page shows. A report that arrives
//!   before that decision is current and the owner decides from it, as from any other state.
//!
//! The element is addressed by its `id`, which the server renders, so the identity is the same before and after
//! hydration. Decisions stay here; the page side is plumbing.

use serde::Serialize;

#[cfg(not(feature = "native"))]
use super::behaviours::Site;

/// What an executor level opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutorKind {
    /// A `[popover]` element: `showPopover()` / `hidePopover()`.
    Popover,
}

/// How a closed level is applied.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShowMode {
    /// A closed level hides the element at once.
    Immediate,
    /// A closed level hides nothing: the element stays shown until its owner unmounts it, so an exit animation
    /// plays on an element that is still in the top layer.
    Mounted,
}

/// A report of the page executor: the level the page reached on its own, stamped with the nonce it had applied.
#[cfg(not(feature = "native"))]
#[derive(serde::Deserialize)]
struct Report {
    n: u64,
    open: bool,
}

/// Whether a report stamped `reported` still speaks for the owner whose newest command is `newest`: only a report of
/// the newest command does. Anything older was decided over by a later command.
#[cfg(any(not(feature = "native"), test))]
fn report_is_current(reported: u64, newest: u64) -> bool {
    reported == newest
}

/// Drives the open level of the `[popover]` element `id` from `open` through the page executor, and hands a level
/// the page reached on its own (a light dismiss, Escape) to `on_page`, unless the owner has decided again since.
///
/// One session per mounted owner; each change of `open` sends the next command on it. A hook: call it
/// unconditionally, once per render.
#[cfg(not(feature = "native"))]
pub(crate) fn use_popover_level(
    id: String,
    open: impl dioxus::prelude::Readable<Target = bool> + Copy + 'static,
    mode: ShowMode,
    on_page: dioxus::prelude::Callback<bool>,
    site: Site,
) {
    use dioxus::prelude::*;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use super::session::{Command, Session, Subscription};

    let nonce = use_hook(|| Rc::new(Cell::new(0u64)));
    let session = use_hook(|| Rc::new(RefCell::new(None::<Subscription>)));
    {
        let (nonce, session) = (nonce.clone(), session.clone());
        use_effect(move || {
            let level = open.cloned();
            let n = nonce.get() + 1;
            nonce.set(n);
            let command = Command::Executor {
                target: id.clone(),
                kind: ExecutorKind::Popover,
                mode,
                level,
                nonce: n,
                site,
            };
            let mut slot = session.borrow_mut();
            match slot.as_ref() {
                Some(running) => running.send(&[command]),
                None => {
                    let newest = nonce.clone();
                    *slot = Some(Session::new().command(command).open(move |report: Report| {
                        if report_is_current(report.n, newest.get()) {
                            on_page.call(report.open);
                        }
                    }));
                }
            }
        });
    }
    use_drop(move || drop(session.borrow_mut().take()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_report_of_the_newest_command_is_current() {
        assert!(report_is_current(3, 3));
        assert!(
            !report_is_current(2, 3),
            "a report the owner decided over is dropped"
        );
    }

    #[test]
    fn the_command_carries_the_nonce_and_the_site() {
        let command = super::super::session::Command::Executor {
            target: "panel".into(),
            kind: ExecutorKind::Popover,
            mode: ShowMode::Mounted,
            level: true,
            nonce: 4,
            site: crate::site!("executor.popover"),
        };
        assert_eq!(
            serde_json::to_value(&command).unwrap(),
            serde_json::json!({
                "cmd": "executor", "target": "panel", "kind": "popover", "mode": "mounted",
                "level": true, "nonce": 4, "site": "executor.popover"
            })
        );
    }
}
