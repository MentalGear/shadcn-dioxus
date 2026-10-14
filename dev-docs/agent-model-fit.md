# Which agent model fits which lane or task (running log)

Terms (owner, 2026-10-14): a **lane** is a work stream with its own file set and worktree ("the overlays lane owns
`top_layer.rs`"); an **agent** is the subagent (Haiku or Sonnet) that executes a lane or a task ("the theme lane, run by a
Sonnet agent"). Never "a Haiku lane".

Owner request 2026-10-14: track where Haiku agents are good to use and where they should not be. Sonnet stays
the default for delegated work (CLAUDE.md); this file records evidence per lane so the choice is made from
measurements, not habit. Add a row for every delegated lane; revise the rules when the evidence moves.

## Rules so far

**Haiku is a good fit for:**
- Read-only inventories with a crisp, checkable output: grep sweeps, counts per file, file:line lists, "which of these
  files already do X". Fast, accurate, cheap.
- Read-only audits against an external reference when the comparison is textual (demo text, labels, prop names), as long
  as the brief names the exact files and output format.
- Small, mechanical edits with a closed list of target files and a known idiom to copy (demo variants, text changes),
  where the main loop reviews the diff afterwards.

**Haiku should not be used for (use Sonnet, or stronger after repeated failure):**
- Debugging and root-causing (CLAUDE.md already routes these to Sonnet), anything needing a bisect or a new hypothesis.
- Primitives and shared code (`primitives/src/**`, the theme, `top_layer.rs`, focus/placement logic) where a wrong
  construction affects every consumer.
- Designing gates or specs whose job is to remove a failure class (the construction needs judgment).
- Long-running lanes that must wait on shared resources (build lock, other lanes): a Haiku agent tended to hand
  back early with work still queued instead of waiting it out, so it needed a resume (the row 212 demo lane).
- Lanes whose report has to separate "measured" from "inferred" carefully (e.g. pass/fail tallies across reruns).

**Always, whatever the model:** the main loop reviews the diff, and lane results that ran on a shared cargo target are
re-verified on the merged tree (the mtime hazard below makes any model's "clippy clean" untrustworthy there).

## Evidence log

| Date | Lane / task | Agent model | Kind | Outcome | Verdict |
|---|---|---|---|---|---|
| 2026-10-14 | `:hover` inventory for row 205 | Haiku | read-only grep inventory | 116 unwrapped tokens, 36 mixed rules, negated-hover edge cases, stylelint notes, a gate design; ~3.5 min. The theme lane used it directly and its counts held | good fit |
| 2026-10-14 | small-control demo audit vs shadcn (row 212) | Haiku | read-only audit + web fetch | Correct per-variant gap table with exact shadcn text, found rows 187/188 prior work; worked around the blocked GitHub tree API by guessing raw paths (some shadcn variants may be missed, which it stated) | good fit |
| 2026-10-14 | row 212 demo edits (8 demos + a new spec) | Haiku | small mechanical implementation | Code correct, compiled, clippy clean, specs green; diff reviewed OK. But it handed back before its SSG/spec run finished (queued on the build lock) and needed a resume; final tally was assembled from a first run plus reruns, not one clean run; one small deviation (strikethrough icon) flagged honestly. ~60 min wall, mostly lock waits | usable with review; brief it to wait in the foreground for queued work |
| 2026-10-14 | shadcn date picker + combobox research (rows 209/210) | Sonnet | read-only research across shadcn, Base UI and Floating UI source plus our code | Overturned an assumption (shadcn's NL picker has an input; no shadcn demo has Apply/Cancel) and pinpointed the likely overlap path in `top_layer.rs` with line numbers | Sonnet justified (multi-source reasoning) |
| 2026-10-14 | theme lane (rows 203, 205: tokens, hover gate + 95-rule sweep, press feedback) | Sonnet | cross-cutting implementation + new gate | Three clean commits, full gates green, honest deviations | Sonnet justified |
| 2026-10-14 | edge-panel lane (rows 207/208) | Sonnet | primitives + theme recipe + nested drawers | 46/46 drawer + 52/52 sheet/modal specs | Sonnet justified |
| 2026-10-14 | charts lane (row 211) | Sonnet | primitive logic + spec | Fixed in the right shared place, and found the shared-target mtime hazard on its own | Sonnet justified |
| 2026-10-14 | chat-kit demos vs shadcn gap research | Haiku | read-only audit + web fetch | Found the authoritative `.mdx` preview lists, fetched all 44 shadcn examples, produced a 43-row gap table and an ordered demo-only vs needs-API split, and corrected a wrong claim in an older audit; ~7 min | good fit |
| 2026-10-14 | chat-kit demo implementation (11+ interactive demos) | Sonnet (chosen over Haiku) | multi-demo implementation on the scroller API, Dialog, Toast, timers under motion gating | (pending) | chosen by the rules above: interactive state + gate-sensitive timers |
| 2026-10-14 | layout debug lane (row 204) | Sonnet | reproduce + root-cause with a frame recorder, then fix + sweep spec | Could not reproduce the owner's oscillation but found and fixed two real re-layouts, said plainly what stays unproven, and wrote a red-on-main/green-on-fix spec; one literal it introduced was caught by a gate | Sonnet justified (debugging) |
| 2026-10-14 | overlays lane (rows 210, 193) | Sonnet | shared placement engine + close-on-scroll helper | Verified the research's claims before editing, reproduced the overlap first, 59/59 + 26/26 new specs, existing overlay specs no worse than baseline | Sonnet justified |
| 2026-10-14 | date picker lane (row 209) | Sonnet | primitive staging state + demos + parser | Clean construction (one staging hook, drop-on-open-change), 35 new tests, honest about what shadcn parts it could not port | Sonnet justified |
| 2026-10-14 | focus lane (row 206) | Sonnet | shared dialog driver | Worked and was tested (31/31), but the first construction had an install-order dependency (a page-level listener installed by some roots only) that the main loop rejected for the `:focus-visible` signal: Sonnet needed a design review on a cross-cutting primitive | Sonnet, with main-loop design review |
| 2026-10-14 | wrap the 7 deferred `:hover` sheets | Haiku | mechanical CSS edit with a closed file list, an existing script and a gate that verifies it | 11 wrapped + 5 split, read its own diff, fixed a stylelint duplicate-selector by folding with a correct cascade argument, kept indentation convention, all four checks + gate green, exactly 8 files; ~10 min | good fit (closed list + verifying gate) |

## Environment hazard every lane must know (found 2026-10-14 by the charts lane; backlog row 216)

Several worktrees sharing one `CARGO_TARGET_DIR` (or a copied warm template) is not safe as-is: cargo gives a workspace
crate the same unit hash in every worktree and decides freshness by mtime, so a worktree whose files are older than
another tree's artifact silently builds or lints that other tree's code. Touch the worktree's sources before every
cargo/dx run on a shared or copied target, and verify that your change is in the built output. Tracked, with the by-construction fix, as
[`backlog.md` row 216](./backlog.md) (it re-confirms row 68).
