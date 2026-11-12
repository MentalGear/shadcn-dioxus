# Code rules: comments, JavaScript interop, repeated problems

Read this when you write a code comment, test title, demo text or name, write interactive behaviour or any JS/TS, or
see the same problem a second time.

- **When the same problem shows up more than once, stop patching instances.** Two or more occurrences is a class, not a
  coincidence: deconstruct down to the root cause, then propose the best-fit fix *by construction*, one that makes the
  whole class unable to recur, rather than fixing the instances one at a time. Say plainly which instances the
  construction subsumes and which (if any) it does not.
- **Code describes only its current state** ([`dev-docs/history-in-comments.md`](../dev-docs/history-in-comments.md)):
  comments, test titles, demo text and names say what the code is and why, in present tense, plus the backlog row.
  The story (dates, commits, "used to", which bug or report found it, which lane or round) goes straight into the
  backlog row's `## Status`/`## Detail`, or `dev-docs/archive/` for a long note, never into the code. The history gates
  enforce it; write it this way the first time.
- **JavaScript interop:** Rust first; new code reaches browser APIs only through the bridge (`primitives/src/bridge/`);
  no new JS in Rust strings (the remaining legacy eval sites are row 434's to port); JS volume can only shrink
  (`check-js-volume.sh`); the plan's other gates are planned, and `js-interop.md` says which have landed. Read
  [`dev-docs/js-interop.md`](../dev-docs/js-interop.md) before writing any interactive behaviour.
