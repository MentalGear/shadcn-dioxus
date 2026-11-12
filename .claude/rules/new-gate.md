---
paths:
  - "scripts/check-*.sh"
  - "scripts/run-gates.sh"
---
A new gate: write its header, register it in `run-gates.sh`, run `scripts/gates-doc.py --write`; no other doc is
edited. Run every gate before committing. Read `agent-rules/gates-before-commit.md`.
