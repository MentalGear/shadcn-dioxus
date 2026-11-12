# Claude Code notes for this repository

Short on purpose: each rule has one home. Agent procedure lives in [`agent-rules/`](./agent-rules/README.md), everything
else in `dev-docs/` ([map](./dev-docs/README.md)). A new gate never needs an edit to this file.
Path-scoped stubs in `.claude/rules/` load when you touch matching files; the table below covers the rest.

## Always

- **Read [`dev-docs/STATUS.md`](./dev-docs/STATUS.md) first**, then [`dev-docs/README.md`](./dev-docs/README.md).
- **Delegate by default**: subagents in ONE batch on disjoint file sets; the main loop scopes, reviews and decides.
- Every `CARGO_TARGET_DIR` is an absolute, isolated path; never `cargo install dioxus-cli`.
- Never write feedback about the default "commit nag" stop hook in the chat. When it flags changes that belong to an
  in-flight agent or are staged for a later commit, continue silently; act only on finished work to commit and push.
- Whenever a preview deploy is ready (Pages publish succeeded), always post the site link in the chat:
  https://mentalgear.github.io/shadcn-dioxus/ (with the main commit it was built from).

## If you are about to ... then (read the file first)

- **Delegate a task, brief a lane agent, choose a model** -> assess complexity; the lowest tier that fits takes it
  (Haiku, then Sonnet, then Opus); state the owner's goal. Read
  [delegation-and-models](./agent-rules/delegation-and-models.md).
- **Write interactive behaviour or any JS/TS** -> Rust first, browser APIs only via the bridge. Read
  [js-interop](./dev-docs/js-interop.md) and [code-rules](./agent-rules/code-rules.md).
- **Write a comment, test title, demo text or name** -> present tense, current state, no history. Read
  [code-rules](./agent-rules/code-rules.md).
- **See the same problem a second time** -> fix the class by construction. Read
  [code-rules](./agent-rules/code-rules.md).
- **Bring a decision to the owner** -> first take it apart: evidence, an existing rule or a recorded decision may
  settle it, and then it is not a decision. For what is left, put the one best construction forward to approve, with
  its residue, not a menu. Read [session-and-backlog](./agent-rules/session-and-backlog.md).
- **Commit, or call a change finished** -> run every gate first (`scripts/run-gates.sh`). Read
  [gates-before-commit](./agent-rules/gates-before-commit.md).
- **Add a gate** -> header, register in `run-gates.sh`, `gates-doc.py --write`. Read
  [gates-before-commit](./agent-rules/gates-before-commit.md).
- **Write a commit message** -> Conventional Commits plus trailers. Read [commits](./agent-rules/commits.md).
- **Add, rename or change a component** (`component.json`, `component.rs`, `mod.rs`, `style.css`) -> run the install
  smoke. Read [merge-and-install](./agent-rules/merge-and-install.md).
- **Run cargo or dx, or build the SSG** -> absolute isolated target, SSG via `scripts/build-ssg.sh`. Read
  [build-and-fast-loop](./agent-rules/build-and-fast-loop.md).
- **Iterate with `dx serve` or run Playwright** -> fast loop, `test:changed`, timing specs on release only. Read
  [build-and-fast-loop](./agent-rules/build-and-fast-loop.md).
- **Merge lane branches** -> generated files merge through drivers. Read
  [merge-and-install](./agent-rules/merge-and-install.md).
- **Edit a backlog row** -> edit the row file, run `scripts/backlog-index.sh`, commit both. Read
  [session-and-backlog](./agent-rules/session-and-backlog.md).
- **Start in a fresh container without `dx`** -> run `scripts/session-setup.sh` by hand. Read
  [session-and-backlog](./agent-rules/session-and-backlog.md).
