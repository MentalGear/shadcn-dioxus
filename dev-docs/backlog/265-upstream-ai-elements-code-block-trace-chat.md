---
id: 265
title: "Upstream ai-elements (chat, message, code-block, trace, checkpoint, ...) vs our chat kit"
status: "BACKLOG (found by row 213 item 10, 2026-10-19). Source…"
size: "L"
area: chat
updated: 2026-10-19
---

# Row 265: Upstream ai-elements vs our chat kit

## Item

shadcn-ui/ui landed an `ai-elements` docs section and two new base UI components since our 295a1f1 snapshot; none exist here.

## Status / trigger

**BACKLOG (found by row 213 item 10, 2026-10-19)**. Source: `dev-docs/research/message-scroller-upstream-2026-10-19.md`.

## Size

L

## Detail

Upstream commit `4233ed1c` (#12307, 2026-10-14) added `registry/bases/base/ui/code-block.tsx` (257 lines) and `trace.tsx`
(173), Nova styles `.cn-code-block*` / `.cn-trace*` (77 lines in `style-nova.css`), examples `chat-demo`, `code-block-chat`,
`message-pattern`, and 68 docs files under `content/docs/ai-elements/{base,radix,aria}` (chat, message, checkpoint,
confirmation, context, file-tree, inline-citation, model-selector, plan, prompt-input, queue, reasoning, sources,
stack-trace, subagent, suggestion, task, terminal, test-results, tool, chain-of-thought). The scroller itself did not
change. Open work: (1) read the new Nova blocks and decide which of Code Block / Trace / Checkpoint (a separator Marker
with a restore button) we want as registry components; Code Block pairs with the `markdown` component's `highlight` slot
(row 213, markdown); (2) compare `chat-demo`/`message-pattern` with our 25 chat demos and add the missing patterns;
(3) decide whether the rest of ai-elements is in scope at all (owner decision), since most are prompt-level compositions.
Each new component follows the usual gates (installed-paths, component-catalog, spec per behaviour).
