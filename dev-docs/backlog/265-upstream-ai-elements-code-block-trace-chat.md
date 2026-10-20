---
id: 265
title: "Upstream ai-elements (chat, message, code-block, trace, checkpoint, ...) vs our chat kit"
status: "PARTIAL 2026-10-19 (items 1-2 landed: code_block, trace, checkpoint and chat demos; item 3 is an…"
size: "L"
area: chat
updated: 2026-10-20
---

# Row 265: Upstream ai-elements vs our chat kit

## Item

shadcn-ui/ui landed an `ai-elements` docs section and two new base UI components since our 295a1f1 snapshot; none exist here.

## Status / trigger

**PARTIAL 2026-10-19 (items 1-2 landed: code_block, trace, checkpoint and chat demos; item 3 is an owner decision)**. Found by row 213 item 10. Source: `dev-docs/research/message-scroller-upstream-2026-10-19.md`. **2026-10-20 sync: items 1-2 verified merged in PR #112 (7cd70e72 code_block, trace, checkpoint and chat demos; b16d860f merge); item 3 still waits on the owner.**

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

## Update 2026-10-19 (lane chatkit)

Scope chosen (default; owner may widen): (a) three new components, (b) the upstream demo patterns as new variants.

- `code_block` (Shadcn): CodeBlock, Header, Title, Actions, CopyButton, Content (`code` or highlighted `CodeBlockLine` children, `wrap`, `line_numbers`, `start_line`), Line (`Added`/`Removed`/`Highlighted`), Footer. No highlighter is bundled (markdown ships none either); lines take any highlighter's spans. Markdown's `highlight` slot renders inside Markdown's own `<pre>`, which `CodeBlockContent` cannot nest in, so the pairing is "split the reply at its fences" (documented, Code Block Chat demo); a `<code>`-only part would be the follow-up if wanted.
- `trace` (Shadcn): Trace, Line (`Running` shimmer through `use_motion_when`, `Error`, `Pending`), Icon, Title, Description, Collapsible/Trigger/Content over the collapsible primitive (button with aria-expanded/controls; controlled or not). Nova's height transition is not ported (content unmounts when closed).
- `checkpoint` (Extra): decision = a small component, not only a recipe. Upstream documents it as a separator Marker plus a Button; the failure class in the recipe is N identical "Restore" buttons and a hand-added `role="separator"` that hides the label. By construction `label` is a required prop, the button's accessible name is built from it ("Restore checkpoint: <label>", visible word first), and the row is never role=separator. Built from Marker + Button; subsumes every hand-rolled checkpoint row.
- New demo variants: `message/code_block_chat`, `message/message_pattern`, `message_scroller/agent_chat` (upstream `chat-demo` agent turn: reasoning Trace, tool line, CodeBlock, Checkpoint with restore + replay). Our existing `full_chat` already covers the composer half of `chat-demo`.
- Specs: playwright/{code-block,trace,checkpoint,chat-demos}.spec.ts. All props documented (no check-props-docs baseline entries needed).
- Remaining: item 3 (the rest of ai-elements: owner decision); evidence of spec/gate runs is in the lane report.
