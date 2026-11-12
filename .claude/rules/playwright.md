---
paths:
  - "playwright/**"
---
Iterate with `cd playwright && pnpm run test:changed` on one `lane-ssg` build; functional specs may run on debug SSG
builds, timing specs only on release. Read `agent-rules/build-and-fast-loop.md`.
