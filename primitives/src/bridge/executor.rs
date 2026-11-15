//! The Rust half of the page executor (row 449): an owner decides the open level of its top-layer element and the
//! page's one executor (`ts/lib/executor.ts`) applies it.
//!
//! The owner renders its level as the element's `data-dx-level` attribute (`<nonce> <open|closed> <mode> <site>`):
//! the nonce is the owner's command count, so the server and the first client render agree on it and the level is in
//! the prerendered HTML. In the effect after each render that changed it, a constant program (`__dxB.execute()`,
//! synchronous on the web) applies every newer level on the page, so a show lands before a write the owner makes
//! after it in the same effect pass (a focus moved into the shown panel). The page keeps the newest nonce it applied
//! per element node (a re-created element starts fresh) and applies a level only once; when the page changes the
//! level on its own (a light dismiss, Escape) it reports `{ n, open }` with the nonce it had applied on the owner's
//! session ([`Command::Executor`](super::Command::Executor)), and the owner drops a report older than its newest
//! level. That one rule removes the races between a report and a command:
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
//! Decisions stay here; the page side is plumbing.

use super::behaviours::Site;

/// How a closed level is applied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShowMode {
    /// A closed level hides the element at once.
    Immediate,
    /// A closed level hides nothing: the element stays shown until its owner unmounts it, so an exit animation
    /// plays on an element that is still in the top layer.
    Mounted,
}

impl ShowMode {
    fn as_str(self) -> &'static str {
        match self {
            ShowMode::Immediate => "immediate",
            ShowMode::Mounted => "mounted",
        }
    }
}

/// The `data-dx-level` value of a level: `<nonce> <open|closed> <mode> <site>`.
fn level_value(nonce: u64, open: bool, mode: ShowMode, site: Site) -> String {
    let level = if open { "open" } else { "closed" };
    format!("{nonce} {level} {} {}", mode.as_str(), site.name())
}

/// A report of the page executor: the level the page reached on its own, stamped with the nonce it had applied.
#[cfg(not(feature = "native"))]
#[derive(serde::Deserialize)]
struct Report {
    n: u64,
    open: bool,
}

/// Whether a report stamped `reported` still speaks for the owner whose newest level is `newest`: only a report of
/// the newest level does. Anything older was decided over by a later level.
fn report_is_current(reported: u64, newest: u64) -> bool {
    reported == newest
}

/// Drives the open level of the `[popover]` element `id` from `open` through the page executor, and hands a level
/// the page reached on its own (a light dismiss, Escape) to `on_page`, unless the owner has decided again since.
///
/// Returns the `data-dx-level` attribute to spread on the element. A hook: call it unconditionally, once per render.
#[cfg(not(feature = "native"))]
pub(crate) fn use_popover_level(
    id: String,
    open: impl dioxus::prelude::Readable<Target = bool> + Copy + 'static,
    mode: ShowMode,
    on_page: dioxus::prelude::Callback<bool>,
    site: Site,
) -> Vec<dioxus::prelude::Attribute> {
    use dioxus::prelude::*;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use super::session::{Command, Session, Subscription};
    use super::transport::{self, Program};

    // the owner's command count: one more for each level `open` takes, the same on the server and the client
    let count = use_hook(|| Rc::new(Cell::new(0u64)));
    let nonce = use_memo(move || {
        open.cloned();
        count.set(count.get() + 1);
        count.get()
    });
    let session = use_hook(|| Rc::new(RefCell::new(None::<Subscription>)));
    {
        let (id, session) = (id.clone(), session.clone());
        use_effect(move || {
            *session.borrow_mut() = Some(
                Session::new()
                    .command(Command::Executor { target: id.clone() })
                    .open(move |report: Report| {
                        if report_is_current(report.n, *nonce.peek()) {
                            on_page.call(report.open);
                        }
                    }),
            );
        });
    }
    use_effect(move || {
        nonce();
        let _ = transport::open(Program::Execute);
    });
    use_drop(move || drop(session.borrow_mut().take()));
    vec![Attribute::new(
        "data-dx-level",
        level_value(nonce(), open.cloned(), mode, site),
        None,
        false,
    )]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_report_of_the_newest_level_is_current() {
        assert!(report_is_current(3, 3));
        assert!(
            !report_is_current(2, 3),
            "a report the owner decided over is dropped"
        );
    }

    #[test]
    fn a_level_value_names_the_nonce_the_level_the_mode_and_the_site() {
        let site = crate::site!("executor.popover");
        assert_eq!(
            level_value(4, true, ShowMode::Mounted, site),
            "4 open mounted executor.popover"
        );
        assert_eq!(
            level_value(1, false, ShowMode::Immediate, site),
            "1 closed immediate executor.popover"
        );
    }
}
