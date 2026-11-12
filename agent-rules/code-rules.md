# Code rules: comments, JavaScript interop, repeated problems

Read this when you write a code comment, test title, demo text or name, write interactive behaviour or any JS/TS,
see the same problem a second time, or reach one of the local-minimum checkpoints below.

- **When the same problem shows up more than once, stop patching instances.** Two or more occurrences is a class, not a
  coincidence: deconstruct down to the root cause, then propose the best-fit fix *by construction*, one that makes the
  whole class unable to recur, rather than fixing the instances one at a time. Say plainly which instances the
  construction subsumes and which (if any) it does not.
- **Check for a local minimum, then solve at the highest root** (owner 2026-11-12). Steady progress on the current
  path proves nothing about the path. Stop and ask "is this a local minimum?" at these points: before a batch is
  dispatched and again when it is merged; when a lane hands back blocked or a brief turns out to rest on a wrong
  premise; at the third patch, rerun or review pass in the same area; when a workaround needs its own workaround; when
  a plan, row or doc contradicts the source; when cost keeps rising while the result stays the same. If the answer is
  yes or unclear: zoom out to what the owner wants from the whole thing, list the separate issues and trace each to the
  highest cause they share (the missing contract, the unchecked claim, the wrong default), and fix that cause by
  construction, so the issues below it cannot recur. Say which issues the construction removes and which it leaves. A
  root that needs a whole design weighed goes to an Opus agent; a root that changes a recorded decision goes to the
  owner as one construction to approve ([session-and-backlog](./session-and-backlog.md)).
- **Code describes only its current state** ([`dev-docs/history-in-comments.md`](../dev-docs/history-in-comments.md)):
  comments, test titles, demo text and names say what the code is and why, in present tense, plus the backlog row.
  The story (dates, commits, "used to", which bug or report found it, which lane or round) goes straight into the
  backlog row's `## Status`/`## Detail`, or `dev-docs/archive/` for a long note, never into the code. The history gates
  enforce it; write it this way the first time.
- **JavaScript interop:** Rust first; new code reaches browser APIs only through the bridge (`primitives/src/bridge/`);
  no new JS in Rust strings (the remaining legacy eval sites are row 434's to port); JS volume can only shrink
  (`check-js-volume.sh`); the plan's other gates are planned, and `js-interop.md` says which have landed. Read
  [`dev-docs/js-interop.md`](../dev-docs/js-interop.md) before writing any interactive behaviour.
