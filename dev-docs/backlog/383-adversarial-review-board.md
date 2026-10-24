---
id: 383
title: "Adversarial review board: independent expert reviewers attack the finished library, last item of the backlog"
status: "BACKLOG. Runs last: after row 365's proposals are answered and every other engineering row has…"
size: "M"
area: process
updated: 2026-10-24
---

# Row 383: Adversarial review board

## Item

The owner asked for the last backlog item to be an adversarial review board of independent, excellent subagents. Each reviewer gets the code and the docs, not the previous reviews' conclusions, and tries to break one dimension of the finished library.

## Status / trigger

BACKLOG. Runs last: after row 365's proposals are answered and every other engineering row has landed, on the deployed main.

## Size

M

## Detail

- **Board:** one reviewer per dimension, each on the strongest available model, briefed independently with no shared conclusions: correctness of the primitives (state machines, focus, keyboard, ARIA against APG), shadcn parity (look and behaviour per component against the shadcn reference), accessibility (axe plus manual screen-reader paths), security and supply chain (the playbook's controls against the tree), performance (main-thread, wasm size, virtualization at scale), API and docs (a first-time user installing components with `dx components add` into a fresh app), and the test suite itself (mutations the suite fails to catch).
- **Per finding:** file and line, a reproduction (command or spec), severity, and a proposed fix by construction. The main loop verifies each finding before it counts, and the owner sees the verified list.
- **Output:** a report under `dev-docs/research/`, one new row per accepted finding, and a decision card per proposal. Nothing is built before the owner's answer.
