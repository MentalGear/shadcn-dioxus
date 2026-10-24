# Message Scroller: upstream re-sync check (row 213 item 10, 2026-10-19)

Provenance: a lane read of a blobless clone of `shadcn-ui/ui` (`/home/user/tgt/shadcn-ui`, outside the repo), HEAD
`e31626978fe8e3804fe2e792e11e32dc560e40a8` (2026-10-19, "feat(registry): serve @ballmac per style (#12367)"), against the
snapshot our port and catalog follow, `295a1f114a138f23b5dfee0e0c6812394dfeb90c` (2026-10-02). 98 commits apart. History is
complete (`git cat-file -t 295a1f1` = commit), which the 2026-10-14 shallow clone could not do.

## Result: the scroller did not change

`git log 295a1f1..HEAD` and `git diff --stat 295a1f1 HEAD` over every message-scroller path (all 3 bases' `ui/message-scroller.tsx`,
`new-york-v4/ui/message-scroller.tsx`, `packages/react/src/message-scroller/**` (components, controller, commands, refs, geometry,
stores, types, utils, tests), the three docs pages, all `apps/v4/examples/*/message-scroller-*.tsx`) is empty. The scroller's last
upstream change is `c257f688` (2026-09-04, registry migration to `cn`), before the snapshot; the last behavioural fix is
`503a3a57` (2026-08-31, hide until the opening position applies), which our port already has (the opening-position flash guard).
Our port therefore has zero upstream drift: nothing to port, no new spec needed.

## Classification of everything that did change near it

| Upstream change (295a1f1..HEAD) | Class |
|---|---|
| `packages/react/package.json`: vitest and @vitest/browser ^3.2.6 -> ^3.2.7 | irrelevant (React test tooling) |
| `style-nova.css` +77 lines: `.cn-trace*`, `.cn-code-block*` | irrelevant to the scroller; belongs to the two new components below |
| New `registry/bases/base/ui/{code-block,trace}.tsx` + examples `chat-demo`, `code-block-chat`, `message-pattern` (#12307 "trace, code block and ai elements", 2026-10-14) | new components, not scroller gaps: recorded as row 265 |
| New docs section `content/docs/ai-elements/{base,radix,aria}/*` (68 files, ~10.7k lines: chat, message, checkpoint, confirmation, context, file-tree, inline-citation, model-selector, plan, prompt-input, queue, reasoning, sources, stack-trace, subagent, suggestion, task, terminal, test-results, tool, chain-of-thought) | new product surface; the chat and message pages compose the scroller, not change it. Row 265 |
| `components/*/marker.mdx` one sentence: a Checkpoint is a separator Marker with a restore button | docs only; our Marker `separator` variant already exists |

## Provenance header

`message_scroller.js` previously named no commit. It now pins `shadcn-ui/ui@295a1f1` (ported snapshot) and records the 2026-10-19
re-check against `e319626` (no scroller change).
