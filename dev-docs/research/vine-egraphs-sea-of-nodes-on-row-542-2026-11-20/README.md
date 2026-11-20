# Evidence for the Vine, e-graph and sea-of-nodes evaluation (2026-11-20)

The note is [`../vine-egraphs-sea-of-nodes-on-row-542-2026-11-20.md`](../vine-egraphs-sea-of-nodes-on-row-542-2026-11-20.md).
This directory holds what it rests on.

- [`vine-facts.md`](./vine-facts.md) and [`graph-facts.md`](./graph-facts.md): the facts two research agents gathered
  on 2026-11-20, each statement with its source URL and fetch date.
- `ledgers/`: the receipts. `ledger-vine.jsonl` (49 claims: 43 verified, 6 unverifiable) and `ledger-graph.jsonl` (67
  claims: 65 verified, 2 unverifiable) quote pages saved during the session; the saved pages are third-party text and
  are not in the repository, so those quote receipts resolve only where the pages are fetched again to the paths the
  ledgers name. `ledger-eval.jsonl` (73 claims: 49 verified, 6 logged, 14 opinion, 4 unverifiable) quotes this
  repository at commit 642f304 and the same saved pages.
- `measure/`: the scripts and logs behind the note's two measurements, the count of gates that read Rust source and
  the token-clone scan of `primitives/src` (about 2 s per mode).

Check a ledger with `python3 -I scripts/report-receipts.py <ledger>`
([`agent-rules/delegation-and-models.md`](../../../agent-rules/delegation-and-models.md)).
